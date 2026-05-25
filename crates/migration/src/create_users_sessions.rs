//! Initial users, sessions, and email verification schema.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260516_000001_create_users_sessions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // The initial migration uses explicit DDL to keep SQLite/Postgres/MySQL fields aligned.
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                wechat_openid TEXT UNIQUE NULL,
                username TEXT NULL,
                email TEXT NULL,
                password_hash TEXT NULL,
                failed_login_attempts INTEGER NOT NULL DEFAULT 0,
                last_failed_login_at TEXT NULL,
                nickname TEXT NULL,
                avatar_url TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                token_hash TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                refresh_token_hash TEXT NULL,
                refresh_expires_at TEXT NULL,
                refreshed_at TEXT NULL,
                created_at TEXT NOT NULL,
                revoked_at TEXT NULL
            )"#,
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
                failed_attempts INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )"#,
        )
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
            "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_refresh_token_hash ON sessions(refresh_token_hash)",
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
        db.execute_unprepared("DROP TABLE IF EXISTS sessions")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS users").await?;
        Ok(())
    }
}
