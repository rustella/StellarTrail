//! Database crate entrypoint exporting connection, configuration, and repository-layer capabilities.

pub mod config;
pub mod connection;
pub mod repositories;

pub use config::{DatabaseConfig, DatabaseConfigError, DatabaseKind};
pub use connection::connect_database;
pub use repositories::{
    AuthRepository, GearRepository, GearTemplateRepository, KnotRepository, ListGearOptions,
    UserRecord, hash_token,
};
