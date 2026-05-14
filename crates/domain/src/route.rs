use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteSummary {
    pub id: String,
    pub title: String,
    pub province: String,
    pub difficulty_level: DifficultyLevel,
    pub distance_m: Option<i32>,
    pub ascent_m: Option<i32>,
    pub duration_min: Option<i32>,
    pub best_seasons: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePointType {
    Start,
    End,
    Camp,
    Water,
    Supply,
    Danger,
    Viewpoint,
    Exit,
}
