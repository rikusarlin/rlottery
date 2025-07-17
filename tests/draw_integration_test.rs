use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rlottery::api::draw_service::draw::draw_service_client::DrawServiceClient;
use rlottery::api::draw_service::draw::GetOpenDrawsRequest;
use rlottery::api::wagering_service::wagering::{
  PlaceWagerRequest,
  PlaceWagerBoard,
  PlaceWagerSelection,
  Uuid as WageringUuid,
  GameType
};
use rlottery::api::wagering_service::wagering::wagering_client::WageringClient;
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

pub struct RunningApp {
    pub config_path: std::path::PathBuf,
    pub child: Child,
    pub _config_file: NamedTempFile,
    pub _env:TestEnv,
}

impl Drop for RunningApp {
    fn drop(&mut self) {
        if let Some(id) = self.child.id() {
            let pid = Pid::from_raw(id as i32);
            // Attempt graceful shutdown first
            if let Err(e) = signal::kill(pid, Signal::SIGTERM) {
                eprintln!("Failed to send SIGTERM to process {}: {}", id, e);
            }

            // Give it a moment to shut down gracefully
            std::thread::sleep(Duration::from_millis(100));

            // Check if the process is still running and kill it if necessary
            match nix::sys::wait::waitpid(pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
                Ok(nix::sys::wait::WaitStatus::StillAlive) => {
                    eprintln!("Process {} did not terminate gracefully, sending SIGKILL.", id);
                    if let Err(e) = signal::kill(pid, Signal::SIGKILL) {
                        eprintln!("Failed to send SIGKILL to process {}: {}", id, e);
                    }
                }
                Ok(_) => { /* Process already exited */ }
                Err(e) => eprintln!("Error checking process status {}: {}", id, e),
            }

            // Ensure the process is reaped
            match nix::sys::wait::waitpid(pid, None) {
                Ok(_) => println!("Process {} terminated successfully.", id),
                Err(e) => eprintln!("Failed to wait for process {}: {}", id, e),
            }
        } else {
            eprintln!("Failed to get process ID to send SIGTERM/SIGKILL");
            // Fallback for safety if PID is somehow lost
            if let Err(e) = self.child.start_kill() {
                eprintln!("Failed to kill child process without PID: {}", e);
            }
        }
    }
}

#[derive(Default)]
struct GrpcPorts {
    wagering: u16,
    admin: u16,
    draw: u16,
}

pub struct TestContext {
    pub draw_channel: Channel,
    pub wagering_channel: Channel,
    pub admin_channel: Channel,
    pub _app: RunningApp,
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
min_value = 1
max_value = 40

[[game.draw_levels]]
name = "secondary"
selections = 1
dependent_on = "primary"
min_value = 1
max_value = 40

[[game.wager_classes]]
name="normal"
selections=["primary"]
number_of_selections=[6]
stake_min=100
stake_max=100
stake_increment=100

[[game.wager_classes]]
name="system7"
selections=["primary"]
number_of_selections=[7]
stake_min=700
stake_max=700
stake_increment=700

[[game.wager_classes]]
name="system8"
selections=["primary"]
number_of_selections=[8]
stake_min=2800
stake_max=2800
stake_increment=2800

[game.schedule.daily]
time = "18:00"


[env]
RUST_TEST_THREADS = "1"
"#
    );

    std::fs::write(temp_file.path(), config_content).expect("Failed to write config");
    temp_file
}

pub async fn setup_test_environment() -> TestContext {

    // Start Postgres test container
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start container");

    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get port");

    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

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

    // Spawn Rust server and catch its stdout and stderr
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

    // Store gRPC channels
    let draw_channel = connect_with_retry(ports.draw, "Draw").await;
    let wagering_channel = connect_with_retry(ports.wagering, "Wagering").await;
    let admin_channel = connect_with_retry(ports.admin, "Admin").await;

    let env = TestEnv {
        _container: container,
        database_url,
    };

    TestContext {
        draw_channel,
        wagering_channel,
        admin_channel,
        _app: RunningApp {
            config_path,
            child,
            _config_file: temp_config,
            _env: env,
        },
    }
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
    let ctx = setup_test_environment().await;
    let mut draw_client = DrawServiceClient::new(ctx.draw_channel.clone());

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
    let ctx = setup_test_environment().await;

    let mut draw_client = DrawServiceClient::new(ctx.draw_channel.clone());
    let mut wagering_client = WageringClient::new(ctx.wagering_channel.clone());

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
        draws: Vec::from([1,2]),
        boards: vec![
            PlaceWagerBoard {
                game_type: GameType::Normal.into(),
                selections: vec![
                    PlaceWagerSelection { name: "primary".to_string(), values: Vec::from([1, 2, 3, 4, 5, 6]) }
                ],
            },
            PlaceWagerBoard {
                game_type: GameType::Normal.into(),
                selections: vec![
                    PlaceWagerSelection { name: "primary".to_string(), values: Vec::from([11, 12, 13, 14, 15, 16]) },
                ],
            },
        ],
        quick_pick: false,
    });

    let place_wager_response = wagering_client.place_wager(place_wager_request).await.expect("Failed to place wager");
    let wager = place_wager_response.into_inner().wager.unwrap();

    // TODO: add plenty of other assertions
    assert_eq!(wager.draws[0].id, draws[0].id, "First wager should be for the first draw");
    assert_eq!(wager.draws[1].id, draws[1].id, "Second wager should be for the second draw");
}
