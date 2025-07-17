use super::draw::Draw;
use super::board::Board;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wager {
    pub id: Uuid,
    pub user_id: Uuid,
    pub draws: Vec<Draw>,
    pub boards: Vec<Board>,
    pub stake: u32,
    pub price: u32,
    pub created_at: DateTime<Utc>,
}
