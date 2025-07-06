use super::selection::Selection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub id: Uuid,
    pub wager_id: Uuid,
    pub selections: Vec<Selection>,
}
