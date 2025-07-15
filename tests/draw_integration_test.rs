use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rlottery::api::draw_service::draw::draw_service_client::DrawServiceClient;
use rlottery::api::draw_service::draw::GetOpenDrawsRequest;
use rlottery::api::wagering_service::wagering::{PlaceWagerRequest, Board, Selection, Uuid as WageringUuid, GameType};
use rlottery::api::wagering_service::wagering::wagering_client::WageringClient;
use rlottery::api::admin_service::admin::admin_client::AdminClient;
use std::process::Stdio;
use std::time::Duration;
use tempfile::NamedTempFile;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{
    runners::AsyncRunner, ContainerAsync,
};
use tokio::{
    process::{Command, Child},
    time::{sleep, timeout},
    io::{AsyncBufReadExt, BufReader},
};
use tokio_postgres::NoTls;
use tonic::transport::Channel;
use uuid::Uuid;

pub struct TestEnv {
    _container: ContainerAsync<Postgres>,
    pub database_url: String,
}

fn generate_temp_config() -> NamedTempFile {
    let temp_file = tempfile::Builder::new()
        .suffix(".toml")
        .tempfile()
        .expect("Failed to create temp config");

    let config_content = format!(
        r#"
[server]
grpc_address = "127.0.0.1:0"

[database]
url = "" # Will be overridden via DATABASE_URL env

[lottery_operator]
id = 1
name = "TestLotteryOperator"

[game]
id = "a1b2c3d4-e5f6-7890-1234-567890abcdef"
lottery_operator_id = 1
name = "TestAwesomeLotto"
open_draws = 5
allowed_participations = [1, 2, 3]
allowed_system_game_levels = [7, 8]
closed_state_duration_seconds = 500

[[game.draw_levels]]
name = "primary"
selections = 6

[[game.draw_levels]]
name = "secondary"
selections = 1
dependent_on = "primary"

[game.schedule.daily]
time = "18:00"

[env]
RUST_TEST_THREADS = "1"
"#
    );

    std::fs::write(temp_file.path(), config_content).expect("Failed to write config");
    temp_file
}


pub struct RunningApp {
    pub config_path: std::path::PathBuf,
    pub child: Child,
    pub _config_file: NamedTempFile,
    pub _env:TestEnv,
}

impl Drop for RunningApp {
    // Terminate the server when closing the test
    fn drop(&mut self) {
        if let Some(id) = self.child.id() {
            let _ = signal::kill(Pid::from_raw(id as i32), Signal::SIGTERM);
        } else {
            eprintln!("Failed to get process ID to send SIGTERM");
        }
    }
}

#[derive(Default)]
struct GrpcPorts {
    wagering: u16,
    admin: u16,
    draw: u16,
}

pub async fn setup_test_environment() -> (
  DrawServiceClient<Channel>,
  WageringClient<Channel>,
  AdminClient<Channel>,
  RunningApp) {


    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start container");

    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get port");

    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // Wait for DB
    let mut db_client = loop {
        match tokio_postgres::connect(&database_url, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("DB connection error: {}", e);
                    }
                });
                break client;
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    };

    rlottery::db::run_migrations(&mut db_client).await.expect("Migrations failed");

    let temp_config = generate_temp_config();
    let config_path = temp_config.path().to_path_buf();

    let mut child = Command::new("cargo")
        .arg("run")
        .env("DATABASE_URL", &database_url)
        .env("APP_CONFIG_PATH", config_path.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start rlottery application");


    let stdout = child.stdout.take().expect("No stdout");
    let stderr = child.stderr.take().expect("No stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Spawn two async tasks to read both streams concurrently
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("[STDOUT] {}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("[STDERR] {}", line);
        }
    });

    let ports = GrpcPorts {
        wagering: 50051,
        admin: 50052,
        draw: 50053,
    };

    let draw_channel = connect_with_retry(ports.draw, "Draw").await;
    let wagering_channel = connect_with_retry(ports.wagering, "Wagering").await;
    let admin_channel = connect_with_retry(ports.admin, "Admin").await;

    let env = TestEnv {
      _container: container,
      database_url: database_url
    };

    (
        DrawServiceClient::new(draw_channel),
        WageringClient::new(wagering_channel),
        AdminClient::new(admin_channel),
        RunningApp {
            config_path,
            child,
            _config_file: temp_config,
            _env: env,
        },
    )
}

async fn connect_with_retry(port: u16, label: &str) -> Channel {
    let endpoint = format!("http://[::1]:{}", port);
    let mut retries = 0;

    while retries < 10 {
        retries += 1;
        match timeout(Duration::from_secs(2), Channel::from_shared(endpoint.clone()).unwrap().connect()).await {
            Ok(Ok(channel)) => return channel,
            _ => {
                eprintln!("Attempt {} to connect to {} gRPC server at {} failed. Retrying...", retries, label, endpoint);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    panic!("Failed to connect to {} gRPC server after multiple retries", label);
}

#[tokio::test]
async fn test_draw_creation_and_fetch() {
    let (mut draw_client, _wagering_client, _admin_client, _app) = setup_test_environment().await;

    // Give some time for draws to be created by the scheduler
    sleep(Duration::from_secs(15)).await;

    let request = tonic::Request::new(GetOpenDrawsRequest {
        game_id: None,
    });

    let response = draw_client.get_open_draws(request).await.expect("Failed to get open draws");
    let draws = response.into_inner().draws;

    assert!(!draws.is_empty(), "No draws found in the database");
    // Further assertions can be added here to check draw content, status, times, etc.
}

#[tokio::test]
async fn test_place_wager_on_open_draws() {
    let (mut draw_client, mut wagering_client, _admin_client, _app) = setup_test_environment().await;

    // Give some time for draws to be created by the scheduler
    sleep(Duration::from_secs(15)).await;

    let get_draws_request = tonic::Request::new(GetOpenDrawsRequest {
        game_id: None,
    });

    let get_draws_response = draw_client.get_open_draws(get_draws_request).await.expect("Failed to get open draws");
    let draws = get_draws_response.into_inner().draws;

    assert!(draws.len() >= 2, "Not enough open draws to place wager");

    let user_id = Uuid::new_v4();

    let place_wager_request = tonic::Request::new(PlaceWagerRequest {
        user_id: Some(WageringUuid { value: user_id.to_string() }),
        number_of_draws: 2,
        boards: vec![
            Board {
                id: Some(WageringUuid { value: Uuid::new_v4().to_string() }),
                selections: vec![
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 1 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 2 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 3 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 4 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 5 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 6 },
                ],
            },
            Board {
                id: Some(WageringUuid { value: Uuid::new_v4().to_string() }),
                selections: vec![
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 7 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 8 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 9 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 10 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 11 },
                    Selection { id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), draw_level_id: Some(WageringUuid { value: Uuid::new_v4().to_string() }), value: 12 },
                ],
            },
        ],
        quick_pick: false,
        game_type: GameType::Normal.into(),
        system_game_level: 0,
    });

    let place_wager_response = wagering_client.place_wager(place_wager_request).await.expect("Failed to place wager");
    let wagers = place_wager_response.into_inner().wagers;

    assert_eq!(wagers.len(), 2, "Expected 2 wagers to be placed");
    assert_eq!(wagers[0].draw_id, draws[0].id, "First wager should be for the first draw");
    assert_eq!(wagers[1].draw_id, draws[1].id, "Second wager should be for the second draw");
}
