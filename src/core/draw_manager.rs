use chrono::{DateTime, Utc, Duration, Timelike};
use crate::core::draw::{Draw, DrawStatus, WinningNumbers};
use crate::core::draw_level::DrawLevel;
use crate::core::rng::Rng;
use uuid::Uuid;
use tokio_postgres::Client;
use crate::config::app_config::GameConfig;
use crate::db::draws;
use tokio_cron_scheduler::{JobScheduler, Job};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

pub struct DrawManager;

impl DrawManager {
    

    pub fn new_draw(
        game_id: Uuid,
        open_time: DateTime<Utc>,
        close_time: DateTime<Utc>,
        draw_time: DateTime<Utc>,
    ) -> Draw {
        Draw {
            id: 0, // This will be set by the database upon insertion
            game_id,
            status: DrawStatus::Created,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            open_time,
            close_time,
            draw_time: Some(draw_time),
            winset_calculated_at: None,
            winset_confirmed_at: None,
            winning_numbers: Vec::new(),
        }
    }

    pub fn transition_draw_status(
        draw: &mut Draw,
        new_status: DrawStatus,
    ) -> Result<(), String> {
        match (&draw.status, &new_status) {
            (DrawStatus::Created, DrawStatus::Open) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                Ok(())
            }
            (DrawStatus::Open, DrawStatus::Closed) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                Ok(())
            }
            (DrawStatus::Closed, DrawStatus::Drawn) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                draw.draw_time = Some(Utc::now());
                Ok(())
            }
            (DrawStatus::Drawn, DrawStatus::WinsetCalculated) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                draw.winset_calculated_at = Some(Utc::now());
                Ok(())
            }
            (DrawStatus::WinsetCalculated, DrawStatus::WinsetConfirmed) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                draw.winset_confirmed_at = Some(Utc::now());
                Ok(())
            }
            (DrawStatus::WinsetConfirmed, DrawStatus::Finalized) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                Ok(())
            }
            (_, DrawStatus::Cancelled) => {
                draw.status = new_status;
                draw.modified_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Invalid draw status transition from {:?} to {:?}",
                draw.status,
                new_status
            )),
        }
    }

    pub fn is_draw_open(draw: &Draw) -> bool {
        draw.status == DrawStatus::Open && Utc::now() >= draw.open_time && Utc::now() < draw.close_time
    }

    pub fn draw_winning_numbers(
        draw: &mut Draw,
        draw_levels: &[DrawLevel],
        seed: [u8; 64],
    ) {
        let mut rng = Rng::new(seed);
        let mut winning_numbers_vec = Vec::new();

        for level in draw_levels {
            let mut numbers = Vec::new();
            while numbers.len() < level.number_of_selections as usize {
                let num = (rng.next_u64() % (level.max_value - level.min_value + 1) as u64) as i32
                    + level.min_value;
                if !numbers.contains(&num) {
                    numbers.push(num);
                }
            }
            numbers.sort_unstable();
            winning_numbers_vec.push(WinningNumbers {
                draw_level_id: level.id,
                numbers,
            });
        }
        draw.winning_numbers = winning_numbers_vec;
    }

    async fn check_and_create_draws(client: Arc<Mutex<Client>>, game_config: GameConfig) {
        info!("Checking and creating draws for game_id: {}", game_config.id);
        let client_locked = client.lock().await;

        let game_id = uuid::Uuid::parse_str(&game_config.id).expect("Invalid game ID in config");
        let open_draws_config = game_config.open_draws;
        let closed_state_duration = Duration::seconds(game_config.closed_state_duration_seconds as i64);

        // Transition created draws to open
        match draws::get_created_draws_ready_to_open(&client_locked, game_id).await {
            Ok(mut created_draws) => {
                info!("Found {} created draws ready to open for game_id: {}", created_draws.len(), game_id);
                for draw in created_draws.iter_mut() {
                    if let Ok(_) = DrawManager::transition_draw_status(draw, DrawStatus::Open) {
                        match draws::update_draw_status(&client_locked, draw.id, DrawStatus::Open).await {
                            Ok(_) => info!("Successfully transitioned draw {} to Open", draw.id),
                            Err(e) => error!("Failed to update draw {} status to Open: {}", draw.id, e),
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to get created draws ready to open: {}", e);
            }
        }

        match draws::get_active_draws(&*client_locked, game_id).await {
            Ok(mut active_draws) => {
                info!("Found {} active draws for game_id: {}", active_draws.len(), game_id);

                while active_draws.len() < open_draws_config as usize {
                    let now = Utc::now();
                    let mut last_scheduled_draw_time = now;

                    if let Some(latest_draw) = active_draws.iter().max_by_key(|d| d.draw_time) {
                        if let Some(dt) = latest_draw.draw_time {
                            last_scheduled_draw_time = dt;
                        }
                    }

                    let mut next_scheduled_draw_time = last_scheduled_draw_time;

                    // Calculate next_scheduled_draw_time based on schedule
                    match &game_config.schedule {
                        crate::config::app_config::ScheduleConfig::Daily { time } => {
                            let parts: Vec<&str> = time.split(':').collect();
                            let hour = parts[0].parse::<u32>().unwrap();
                            let minute = parts[1].parse::<u32>().unwrap();

                            next_scheduled_draw_time = next_scheduled_draw_time.with_hour(hour).unwrap().with_minute(minute).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
                            if next_scheduled_draw_time <= last_scheduled_draw_time {
                                next_scheduled_draw_time = next_scheduled_draw_time + Duration::days(1);
                            }
                        },
                        _ => unimplemented!(), // TODO: Implement other schedule types
                    }

                    let draw_time = next_scheduled_draw_time;
                    let close_time = draw_time - closed_state_duration;
                    let open_time = Utc::now(); // Current timestamp upon initial creation

                    let new_draw = DrawManager::new_draw(game_id, open_time, close_time, draw_time);
                    match draws::insert_draw(&*client_locked, &new_draw).await {
                        Ok(_) => {
                            info!("Successfully inserted new draw: {:?}", new_draw);
                            active_draws.push(new_draw);
                            info!("Created new draw. Total active draws: {}", active_draws.len());
                        },
                        Err(e) => {
                            error!("Failed to insert draw: {}", e);
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to get active draws: {}", e);
            }
        }
    }

    pub async fn schedule_draws(client: Arc<Mutex<Client>>, game_config: GameConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _game_id = uuid::Uuid::parse_str(&game_config.id).expect("Invalid game ID in config");
        let _open_draws_config = game_config.open_draws;

        // Run once on startup
        DrawManager::check_and_create_draws(client.clone(), game_config.clone()).await;

        let sched = JobScheduler::new().await?;
        let client_clone = client.clone();
        let game_config_clone = game_config.clone();

        let job = Job::new_async("1/10 * * * * *", move |_uuid, _l| {
            let client_clone = client_clone.clone();
            let game_config_clone = game_config_clone.clone();
            Box::pin(async move {
                info!("Cron job triggered: Checking and creating draws.");
                DrawManager::check_and_create_draws(client_clone, game_config_clone).await;
            })
        })?;
        sched.add(job).await.expect("Failed to add job to scheduler");

        sched.start().await?;
        Ok(())
    }
}
