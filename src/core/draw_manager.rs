use chrono::{DateTime, Utc};
use crate::core::draw::{Draw, DrawStatus};
use uuid::Uuid;

pub struct DrawManager;

impl DrawManager {
    pub fn new_draw(
        game_id: Uuid,
        open_time: DateTime<Utc>,
        close_time: DateTime<Utc>,
    ) -> Draw {
        Draw {
            id: 0, // This will be set by the database upon insertion
            game_id,
            status: DrawStatus::Created,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            open_time,
            close_time,
            draw_time: None,
            winset_calculated_at: None,
            winset_confirmed_at: None,
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

    // Placeholder for scheduling logic
    pub async fn schedule_draws() {
        // This will involve interacting with tokio-cron-scheduler
        println!("Scheduling draws...");
    }
}
