use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub title: String,
    pub category: SkillCategory,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

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
