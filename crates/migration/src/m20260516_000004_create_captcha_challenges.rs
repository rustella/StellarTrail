//! Image captcha challenge migration creating one-time ticket, answer digest, expiry, and consumed-state fields.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // The challenge table stores only answer digests and consumed state, never reusable plaintext captcha answers.
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS captcha_challenges (
                id TEXT PRIMARY KEY,
                account TEXT NOT NULL,
                ticket TEXT NOT NULL UNIQUE,
                answer_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                consumed_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_captcha_challenges_ticket ON captcha_challenges(ticket)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_captcha_challenges_account_created ON captcha_challenges(account, created_at)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS captcha_challenges")
            .await?;
        Ok(())
    }
}
