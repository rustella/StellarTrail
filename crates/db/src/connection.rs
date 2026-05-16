//! Database connection module responsible for creating a SeaORM DatabaseConnection from configuration.

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::DatabaseConfig;

/// Runs the `connect database` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn connect_database(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    let mut url = config.url.clone();
    if url.starts_with("sqlite://") && !url.contains('?') && url != "sqlite::memory:" {
        url.push_str("?mode=rwc");
    }
    let mut options = ConnectOptions::new(url);
    options.sqlx_logging(false);
    Database::connect(options).await
}
