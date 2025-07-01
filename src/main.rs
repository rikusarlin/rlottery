use postgres::{Client, NoTls};
use tonic::transport::Server;
use api::wagering_service::{WageringService, wagering::wagering_server::WageringServer};

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
    let mut client = Client::connect("postgresql://postgres:postgres@localhost/rlottery", NoTls)
        .expect("Failed to connect to Postgres");

    match db::run_migrations(&mut client) {
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

    println!("Starting gRPC server on [::1]:50051");
    let addr = "[::1]:50051".parse()?;
    let wagering_service = WageringService;

    Server::builder()
        .add_service(WageringServer::new(wagering_service))
        .serve(addr)
        .await?;

    Ok(())
}
