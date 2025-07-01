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
    pub factor: Option<f32>,
    pub constant: Option<f32>,
    pub percentage: Option<f32>,
    pub min_cap: Option<f32>,
    pub max_cap: Option<f32>,
}
