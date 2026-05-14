use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MountainSummary {
    pub id: String,
    pub name: String,
    pub province: String,
    pub elevation_m: Option<i32>,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Leisure,
    Beginner,
    Intermediate,
    Advanced,
    Technical,
}
