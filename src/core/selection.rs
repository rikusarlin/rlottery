use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selection {
    pub id: Uuid,
    pub board_id: Uuid,
    pub draw_level_id: Uuid,
    pub value: i32,
}
