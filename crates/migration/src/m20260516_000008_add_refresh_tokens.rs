//! Adds refresh-token metadata to sessions so clients can rotate short-lived access tokens without re-login.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refresh_token_hash TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refresh_expires_at TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refreshed_at TEXT NULL")
            .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_refresh_token_hash ON sessions(refresh_token_hash)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic for indexes created by up; SQLite cannot safely drop added columns.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_sessions_refresh_token_hash")
            .await?;
        Ok(())
    }
}
