//! Mountain content domain model describing mountain summaries and difficulty levels.

use serde::{Deserialize, Serialize};

/// Stable data boundary for `MountainSummary`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MountainSummary {
    pub id: String,
    pub name: String,
    pub province: String,
    pub elevation_m: Option<i32>,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

/// Stable enum boundary for `DifficultyLevel`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Leisure,
    Beginner,
    Intermediate,
    Advanced,
    Technical,
}
