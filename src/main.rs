use std::env;
use tokio_postgres::NoTls;
use tonic::transport::Server;
use api::wagering_service::{WageringService, wagering::wagering_server::WageringServer};
use api::admin_service::{AdminService, admin::admin_server::AdminServer};
use api::draw_service::{DrawService, draw::draw_service_server::DrawServiceServer};
use crate::core::draw_manager::DrawManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;
use tracing::{info, error};

pub mod api;
pub mod config;
pub mod core;
pub mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_config = config::app_config::Config::from_file("config.toml")
        .expect("Failed to load configuration");
    info!("Loaded configuration: {:?}", app_config);

    // This assumes a local postgresql database `rlottery` exists with user `postgres` and password `postgres`
    let (mut client_raw, connection) = tokio_postgres::connect("postgresql://rlottery:password123@localhost/rlottery", NoTls)
        .await
        .expect("Failed to connect to Postgres");

    tokio::spawn(async move { 
        if let Err(e) = connection.await {
            error!("database connection error: {}", e);
        }
    });

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "run-migrations" {
        match db::run_migrations(&mut client_raw).await {
            Ok(report) => {
                for migration in report.applied_migrations() {
                    info!(
                        "Migration Applied - Name: {}, Version: {}",
                        migration.name(),
                        migration.version()
                    );
                }
                info!("Migrations applied successfully!");
            }
            Err(e) => {
                error!("Error applying migrations: {}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let client = Arc::new(Mutex::new(client_raw));

    // Run migrations on startup if not running as a migration command
    {
        let mut locked_client = client.lock().await;
        match db::run_migrations(&mut *locked_client).await {
            Ok(report) => {
                for migration in report.applied_migrations() {
                    info!(
                        "Migration Applied - Name: {}, Version: {}",
                        migration.name(),
                        migration.version()
                    );
                }
                info!("Migrations applied successfully!");
            }
            Err(e) => {
                error!("Error applying migrations: {}", e);
                std::process::exit(1);
            }
        }

        // Upsert lottery operator
        let operator_name = &app_config.lottery_operator.name;
        db::operator::upsert_lottery_operator(&locked_client, operator_name)
            .await
            .expect("Failed to upsert lottery operator");

        // Upsert game
        let game_id = uuid::Uuid::parse_str(&app_config.game.id).expect("Invalid game ID in config");
        let game_name = &app_config.game.name;
        let operator_id = app_config.game.lottery_operator_id;
        let upsert_game_query = "
            INSERT INTO games (id, lottery_operator_id, name) VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET lottery_operator_id = $2, name = $3
        ";
        locked_client.execute(upsert_game_query, &[&game_id, &operator_id, &game_name]).await.expect("Failed to upsert game");
        info!("Upserted game: {}", game_name);
    }

    // Schedule draw management so we have draws to place wagers in
    let draw_manager_client = client.clone();
    let draw_manager_game_config = app_config.game.clone();
    tokio::spawn(async move {
        DrawManager::schedule_draws(draw_manager_client, draw_manager_game_config)
            .await
            .expect("Failed to schedule draws");
    });

    info!("Starting gRPC servers...");

    let wagering_service = WageringService;
    let admin_service = AdminService;
    let draw_service = DrawService::new(client.clone());

    let wagering_addr = "[::1]:50051".parse()?;
    let admin_addr = "[::1]:50052".parse()?;
    let draw_addr = "[::1]:50053".parse()?;

    let wagering_server = Server::builder()
        .add_service(WageringServer::new(wagering_service))
        .serve(wagering_addr);

    let admin_server = Server::builder()
        .add_service(AdminServer::new(admin_service))
        .serve(admin_addr);

    let draw_server = Server::builder()
        .add_service(DrawServiceServer::new(draw_service))
        .serve(draw_addr);

    tokio::try_join!(wagering_server, admin_server, draw_server)?;

    Ok(())
}
