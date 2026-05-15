use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::DatabaseConfig;

pub async fn connect_database(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    let mut url = config.url.clone();
    if url.starts_with("sqlite://") && !url.contains('?') && url != "sqlite::memory:" {
        url.push_str("?mode=rwc");
    }
    let mut options = ConnectOptions::new(url);
    options.sqlx_logging(false);
    Database::connect(options).await
}
