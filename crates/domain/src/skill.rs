//! Outdoor skill domain model describing skill summaries and skill categories.

use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

/// Stable data boundary for `SkillSummary`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub title: String,
    pub category: SkillCategory,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

/// Stable enum boundary for `SkillCategory`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Knot,
    Camping,
    FirstAid,
    Packing,
    Navigation,
    Weather,
}
