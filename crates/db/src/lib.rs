//! Database crate entrypoint exporting connection, configuration, and repository-layer capabilities.

pub mod config;
pub mod connection;
pub mod repositories;

pub use config::{DatabaseConfig, DatabaseConfigError, DatabaseKind};
pub use connection::connect_database;
