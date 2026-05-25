//! API usage aggregation migration for privacy-preserving per-route call counters.
//!
//! The table stores daily aggregates only. It intentionally avoids raw request
//! paths, query strings, request bodies, authorization headers, tokens, IP
//! addresses, and user agents so usage reporting cannot become an accidental
//! request log.

use sea_orm_migration::prelude::*;

/// Migration that adds the daily API usage aggregate table.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260518_000003_create_api_usage_daily"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the daily aggregate table and read indexes used by admin queries.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS api_usage_daily (
                bucket_date TEXT NOT NULL,
                user_key TEXT NOT NULL,
                user_id TEXT NULL REFERENCES users(id),
                method TEXT NOT NULL,
                route_pattern TEXT NOT NULL,
                status_code INTEGER NOT NULL,
                call_count INTEGER NOT NULL DEFAULT 0,
                first_called_at TEXT NOT NULL,
                last_called_at TEXT NOT NULL,
                PRIMARY KEY (bucket_date, user_key, method, route_pattern, status_code)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_api_usage_daily_user_date ON api_usage_daily(user_key, bucket_date)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_api_usage_daily_route_date ON api_usage_daily(route_pattern, bucket_date)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_api_usage_daily_method_date ON api_usage_daily(method, bucket_date)",
        )
        .await?;
        Ok(())
    }

    /// Drops usage aggregates before rolling back earlier authentication tables.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS api_usage_daily")
            .await?;
        Ok(())
    }
}
