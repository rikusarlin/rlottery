use super::selection::Selection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash, Deserialize, Display)]
pub enum GameType {
    NORMAL,
    SYSTEM,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub wager_id: Uuid,
    pub game_type: GameType,
    pub system_game_level: u32,
    pub selections: Vec<Selection>,
}
