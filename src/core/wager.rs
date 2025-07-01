use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wager {
    pub id: Uuid,
    pub user_id: Uuid,
    pub draw_id: i32,
    pub created_at: DateTime<Utc>,
}
