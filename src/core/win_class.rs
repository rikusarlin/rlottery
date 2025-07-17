use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WinClassType {
    Factor,
    Constant,
    Percentage,
    External,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WinClass {
    pub id: Uuid,
    pub game_id: Uuid,
    pub name: String,
    pub r#type: WinClassType,
    pub factor: Option<u32>,
    pub constant: Option<u64>,
    pub percentage: Option<u32>,
    pub min_cap: Option<u64>,
    pub max_cap: Option<u64>,
}
