use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selection {
    pub id: Uuid,
    pub name: String,
    pub values: Vec<i32>,
}
