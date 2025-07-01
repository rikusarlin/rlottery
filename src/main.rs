use tokio_postgres::NoTls;
use tonic::transport::Server;
use api::wagering_service::{WageringService, wagering::wagering_server::WageringServer};
use api::admin_service::{AdminService, admin::admin_server::AdminServer};

pub mod api;
pub mod config;
pub mod core;
pub mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = config::app_config::Config::from_file("config.toml")
        .expect("Failed to load configuration");
    println!("Loaded configuration: {:?}", app_config);

    // This assumes a local postgresql database `rlottery` exists with user `postgres` and password `postgres`
    let (mut client, connection) = tokio_postgres::connect("postgresql://rlottery:password123@localhost/rlottery", NoTls)
        .await
        .expect("Failed to connect to Postgres");

    tokio::spawn(async move { 
        if let Err(e) = connection.await {
            eprintln!("database connection error: {}", e);
        }
    });

    match db::run_migrations(&mut client).await {
        Ok(report) => {
            for migration in report.applied_migrations() {
                println!(
                    "Migration Applied - Name: {}, Version: {}",
                    migration.name(),
                    migration.version()
                );
            }
            println!("Migrations applied successfully!");
        }
        Err(e) => {
            eprintln!("Error applying migrations: {}", e);
            std::process::exit(1);
        }
    }

    println!("Starting gRPC servers...");

    let wagering_service = WageringService;
    let admin_service = AdminService;

    let wagering_addr = "[::1]:50051".parse()?;
    let admin_addr = "[::1]:50052".parse()?;

    let wagering_server = Server::builder()
        .add_service(WageringServer::new(wagering_service))
        .serve(wagering_addr);

    let admin_server = Server::builder()
        .add_service(AdminServer::new(admin_service))
        .serve(admin_addr);

    tokio::try_join!(wagering_server, admin_server)?;

    Ok(())
}

