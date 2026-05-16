//! 路线内容领域模型，描述路线摘要、地理信息和路线点类型。

use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

/// RouteSummary 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
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

/// RoutePointType 枚举，定义当前模块对外暴露或内部复用的稳定数据边界。
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
