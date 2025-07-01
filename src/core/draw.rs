use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawStatus {
    Created,
    Open,
    Closed,
    Drawn,
    WinsetCalculated,
    WinsetConfirmed,
    Finalized,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Draw {
    pub id: i32,
    pub game_id: Uuid,
    pub status: DrawStatus,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub draw_time: Option<DateTime<Utc>>,
    pub winset_calculated_at: Option<DateTime<Utc>>,
    pub winset_confirmed_at: Option<DateTime<Utc>>,
}
