use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LotteryOperatorConfig {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawLevelConfig {
    pub name: String,
    pub selections: u32,
    pub dependent_on: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WagerClassConfig {
    pub name: String,
    pub selections: Vec<String>,
    pub number_of_selections: Vec<u32>,
    pub stake_min: u32,
    pub stake_max: u32,
    pub stake_increment: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleConfig {
    Daily { time: String },
    Weekly { days: Vec<String>, time: String },
    Interval { minutes: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub id: String,
    pub lottery_operator_id: i32,
    pub name: String,
    pub draw_levels: Vec<DrawLevelConfig>,
    pub wager_classes: Vec<WagerClassConfig>,
    pub open_draws: u32,
    pub allowed_participations: Vec<u32>,
    pub closed_state_duration_seconds: u64,
    pub schedule: ScheduleConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub lottery_operator: LotteryOperatorConfig,
    pub game: GameConfig,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        settings.try_deserialize()
    }
}
