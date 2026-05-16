//! 数据库 crate 入口，导出连接、配置和 repository 层能力。

pub mod config;
pub mod connection;
pub mod repositories;

pub use config::{DatabaseConfig, DatabaseConfigError, DatabaseKind};
pub use connection::connect_database;
