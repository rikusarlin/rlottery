use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LotteryOperator {
    pub id: u32,
    pub name: String,
}
