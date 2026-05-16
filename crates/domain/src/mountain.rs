//! 山峰内容领域模型，描述山峰摘要和难度等级。

use serde::{Deserialize, Serialize};

/// MountainSummary 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MountainSummary {
    pub id: String,
    pub name: String,
    pub province: String,
    pub elevation_m: Option<i32>,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

/// DifficultyLevel 枚举，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Leisure,
    Beginner,
    Intermediate,
    Advanced,
    Technical,
}
