//! One-time verification challenge schema for captcha and SMS flows.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260516_000004_create_captcha_challenges"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // The captcha challenge table stores only answer digests and consumed state, never reusable plaintext answers.
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
        // SMS verification relies on Aliyun-issued tickets and never stores plaintext SMS codes.
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS sms_verification_challenges (
                id TEXT PRIMARY KEY,
                phone TEXT NOT NULL,
                purpose TEXT NOT NULL,
                out_id TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                consumed_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sms_verification_phone_purpose_created ON sms_verification_challenges(phone, purpose, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sms_verification_out_id ON sms_verification_challenges(out_id)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS sms_verification_challenges")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS captcha_challenges")
            .await?;
        Ok(())
    }
}
