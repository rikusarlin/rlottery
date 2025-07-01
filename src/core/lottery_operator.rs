use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LotteryOperator {
    pub id: i32,
    pub name: String,
}
