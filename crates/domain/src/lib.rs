//! 领域模型 crate 入口，导出装备、山峰、路线、技能、用户和通用校验模块。

pub mod gear;
pub mod mountain;
pub mod pagination;
pub mod route;
pub mod skill;
pub mod user;
pub mod validation;

pub type Id = uuid::Uuid;
