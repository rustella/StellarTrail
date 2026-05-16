//! Domain model crate entrypoint exporting gear, mountain, route, skill, user, and shared validation modules.

pub mod feedback;
pub mod gear;
pub mod mountain;
pub mod pagination;
pub mod route;
pub mod skill;
pub mod upload;
pub mod user;
pub mod validation;

pub type Id = uuid::Uuid;
