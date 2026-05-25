//! Password login migration adding username, email, password digest, failure counters, and email verification codes.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Password login extends the users table compatibly so existing WeChat users can keep null username/email fields.
        db.execute_unprepared("ALTER TABLE users ADD COLUMN username TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN email TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN password_hash TEXT NULL")
            .await?;
        db.execute_unprepared(
            "ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN last_failed_login_at TEXT NULL")
            .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username) WHERE username IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE email IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS email_verification_codes (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                purpose TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                consumed_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_email_verification_email_purpose ON email_verification_codes(email, purpose, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_email_verification_code_hash ON email_verification_codes(code_hash)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS email_verification_codes")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_users_email")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_users_username")
            .await?;
        Ok(())
    }
}
