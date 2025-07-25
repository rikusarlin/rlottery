use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Win {
    pub id: Uuid,
    pub wager_id: Uuid,
    pub win_class_id: Uuid,
    pub amount: u64,
}
