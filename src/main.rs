use postgres::{Client, NoTls};

pub mod api;
pub mod config;
pub mod core;
pub mod db;

fn main() {
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

    println!("Hello, world!");
}
