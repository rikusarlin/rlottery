use tokio_postgres::{Client, Error};
use uuid::Uuid;
use tracing::{info, error};
use crate::core::draw::{Draw, DrawStatus};
use chrono::Utc;

pub async fn get_active_draws(client: &Client, game_id: Uuid) -> Result<Vec<Draw>, Error> {
    info!("Attempting to get active draws for game_id: {}", game_id);
    let rows = client
        .query(
            "SELECT id, game_id, status, created_at, modified_at, open_time, close_time, draw_time, winset_calculated_at, winset_confirmed_at FROM draw WHERE game_id = $1 AND status IN ('Created', 'Open')",
            &[&game_id],
        )
        .await?;

    let mut draws = Vec::new();
    for row in rows {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Created" => DrawStatus::Created,
            "Open" => DrawStatus::Open,
            "Closed" => DrawStatus::Closed,
            "Drawn" => DrawStatus::Drawn,
            "WinsetCalculated" => DrawStatus::WinsetCalculated,
            "WinsetConfirmed" => DrawStatus::WinsetConfirmed,
            "Finalized" => DrawStatus::Finalized,
            "Cancelled" => DrawStatus::Cancelled,
            _ => {
                error!("Unknown draw status in database: '{}'", status_str);
                continue;
            }
        };

        draws.push(Draw {
            id: row.get("id"),
            game_id: row.get("game_id"),
            status,
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            open_time: row.get("open_time"),
            close_time: row.get("close_time"),
            draw_time: row.get("draw_time"),
            winset_calculated_at: row.get("winset_calculated_at"),
            winset_confirmed_at: row.get("winset_confirmed_at"),
            winning_numbers: Vec::new(), // TODO: Deserialize winning_numbers
        });
    }
    info!("Found {} active draws for game_id: {}", draws.len(), game_id);
    Ok(draws)
}

pub async fn insert_draw(client: &Client, draw: &Draw) -> Result<(), Error> {
    info!("Attempting to insert draw: {:?}", draw);
    client
        .execute(
            "INSERT INTO draw (game_id, status, created_at, modified_at, open_time, close_time, draw_time) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[&draw.game_id, &draw.status.to_string(), &draw.created_at, &draw.modified_at, &draw.open_time, &draw.close_time, &draw.draw_time],
        )
        .await?;
    info!("Successfully inserted draw: {:?}", draw);
    Ok(())
}

pub async fn get_created_draws_ready_to_open(client: &Client, game_id: Uuid) -> Result<Vec<Draw>, Error> {
    info!("Attempting to get created draws ready to open for game_id: {}", game_id);
    let rows = client
        .query(
            "SELECT id, game_id, status, created_at, modified_at, open_time, close_time, draw_time, winset_calculated_at, winset_confirmed_at FROM draw WHERE game_id = $1 AND status = 'Created' AND open_time <= $2",
            &[&game_id, &Utc::now()],
        )
        .await?;

    let mut draws = Vec::new();
    for row in rows {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Created" => DrawStatus::Created,
            "Open" => DrawStatus::Open,
            "Closed" => DrawStatus::Closed,
            "Drawn" => DrawStatus::Drawn,
            "WinsetCalculated" => DrawStatus::WinsetCalculated,
            "WinsetConfirmed" => DrawStatus::WinsetConfirmed,
            "Finalized" => DrawStatus::Finalized,
            "Cancelled" => DrawStatus::Cancelled,
            _ => {
                error!("Unknown draw status in database: '{}'", status_str);
                continue;
            }
        };

        draws.push(Draw {
            id: row.get("id"),
            game_id: row.get("game_id"),
            status,
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            open_time: row.get("open_time"),
            close_time: row.get("close_time"),
            draw_time: row.get("draw_time"),
            winset_calculated_at: row.get("winset_calculated_at"),
            winset_confirmed_at: row.get("winset_confirmed_at"),
            winning_numbers: Vec::new(), // TODO: Deserialize winning_numbers
        });
    }
    info!("Found {} created draws ready to open for game_id: {}", draws.len(), game_id);
    Ok(draws)
}

pub async fn update_draw_status(client: &Client, draw_id: i32, new_status: DrawStatus) -> Result<(), Error> {
    info!("Attempting to update draw {} status to {:?}", draw_id, new_status);
    client
        .execute(
            "UPDATE draw SET status = $1, modified_at = $2 WHERE id = $3",
            &[&new_status.to_string(), &Utc::now(), &draw_id],
        )
        .await?;
    info!("Successfully updated draw {} status to {:?}", draw_id, new_status);
    Ok(())
}
