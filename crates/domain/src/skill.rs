//! 户外技能领域模型，描述技能摘要和技能分类。

use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

/// SkillSummary 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub title: String,
    pub category: SkillCategory,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

/// SkillCategory 枚举，定义当前模块对外暴露或内部复用的稳定数据边界。
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
