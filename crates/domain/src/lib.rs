//! Domain model crate entrypoint exporting administrator roles, gear, gear templates, skill, user, and shared validation modules.

pub mod admin;
pub mod feedback;
pub mod gear;
pub mod gear_atlas;
pub mod gear_template;
pub mod pagination;
pub mod skill;
pub mod upload;
pub mod user;
pub mod validation;

pub type Id = uuid::Uuid;
