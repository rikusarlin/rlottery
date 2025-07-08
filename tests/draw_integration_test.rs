use tokio::process::{Command, Child};
use tokio::time::{sleep, Duration, timeout};
use tonic::transport::Channel;
use rlottery::api::draw_service::draw::draw_service_client::DrawServiceClient;
use rlottery::api::draw_service::draw::GetOpenDrawsRequest;
use tokio_postgres::NoTls;

async fn setup_test_environment() -> (DrawServiceClient<Channel>, Child) {
    let database_url = "postgresql://rlottery:password123@localhost/rlottery";
    let admin_database_url = "postgresql://rlottery:password123@localhost/postgres"; // Connect to default postgres DB for cleaning

    // 1. Clean the database
    let mut admin_client_opt = None;
    let max_db_retries = 10;
    let mut db_retries = 0;
    while admin_client_opt.is_none() && db_retries < max_db_retries {
        db_retries += 1;
        sleep(Duration::from_secs(2)).await;
        match tokio_postgres::connect(admin_database_url, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("admin database connection error: {}", e);
                    }
                });
                admin_client_opt = Some(client);
            },
            Err(e) => {
                eprintln!("Attempt {} to connect to admin database failed: {}. Retrying...", db_retries, e);
            }
        }
    }
    let admin_client = admin_client_opt.expect("Failed to connect to admin database after multiple retries");

    // Drop and recreate the test database
    admin_client.execute("DROP DATABASE IF EXISTS rlottery WITH (FORCE);", &[]).await.expect("Failed to drop database");
    admin_client.execute("CREATE DATABASE rlottery;", &[]).await.expect("Failed to create database");
    // The admin_client will be dropped here, closing its connection.

    // 2. Connect to the newly created rlottery database and run migrations
    let mut db_client_opt = None;
    db_retries = 0; // Reset retries for the rlottery database connection
    while db_client_opt.is_none() && db_retries < max_db_retries {
        db_retries += 1;
        sleep(Duration::from_secs(2)).await;
        match tokio_postgres::connect(database_url, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("rlottery database connection error: {}", e);
                    }
                });
                db_client_opt = Some(client);
            },
            Err(e) => {
                eprintln!("Attempt {} to connect to rlottery database failed: {}. Retrying...", db_retries, e);
            }
        }
    }
    let mut db_client = db_client_opt.expect("Failed to connect to rlottery database after multiple retries");

    // Run migrations
    rlottery::db::run_migrations(&mut db_client).await.expect("Failed to run migrations");

    // Build the application
    Command::new("cargo")
        .arg("build")
        .output()
        .await
        .expect("Failed to build rlottery");

    // Start the rlottery application in the background
    let child = Command::new("cargo")
        .arg("run")
        .env("DATABASE_URL", database_url)
        .env("APP_CONFIG_PATH", "./tests/test_config.toml") // Use a specific test config
        .spawn()
        .expect("Failed to start rlottery application");

    // Retry connecting to the gRPC server
    let mut client_opt = None;
    let max_grpc_retries = 10;
    let mut grpc_retries = 0;

    while client_opt.is_none() && grpc_retries < max_grpc_retries {
        grpc_retries += 1;
        sleep(Duration::from_secs(2)).await; // Wait before retrying
        match timeout(Duration::from_secs(1), Channel::from_static("http://[::1]:50053").connect()).await {
            Ok(Ok(channel)) => {
                client_opt = Some(DrawServiceClient::new(channel));
            },
            _ => {
                eprintln!("Attempt {} to connect to gRPC server failed. Retrying...", grpc_retries);
            }
        }
    }

    let client = client_opt.expect("Failed to connect to gRPC server after multiple retries");
    (client, child)
}

#[tokio::test]
async fn test_draw_creation_and_fetch() {
    let (mut client, mut child) = setup_test_environment().await;

    // Give some time for draws to be created by the scheduler
    sleep(Duration::from_secs(15)).await;

    let request = tonic::Request::new(GetOpenDrawsRequest {
        game_id: None,
    });

    let response = client.get_open_draws(request).await.expect("Failed to get open draws");
    let draws = response.into_inner().draws;

    assert!(!draws.is_empty(), "No draws found in the database");
    // Further assertions can be added here to check draw content, status, times, etc.

    // Terminate the spawned application process
    child.kill().await.expect("Failed to kill rlottery process");
}