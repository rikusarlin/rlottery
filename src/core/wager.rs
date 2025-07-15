use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
pub enum GameType {
    NORMAL,
    SYSTEM,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wager {
    pub id: Uuid,
    pub user_id: Uuid,
    pub draw_id: i32,
    pub game_type: GameType,
    pub system_game_level: u32,
    pub stake: f64,
    pub price: f64,
    pub created_at: DateTime<Utc>,
}
