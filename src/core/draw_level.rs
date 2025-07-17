use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrawLevel {
    pub id: Uuid,
    pub game_id: Uuid,
    pub name: String,
    pub number_of_selections: u32,
    pub min_value: u32,
    pub max_value: u32,
}
