//! Adds account-level phone authentication and SMS verification challenge tracking.

use sea_orm_migration::prelude::*;

/// Migration that extends users with a verified phone identity.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds nullable phone credentials and one-time SMS challenge records.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE users ADD COLUMN phone TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN phone_bound_at TEXT NULL")
            .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_phone ON users(phone) WHERE phone IS NOT NULL",
        )
        .await?;
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

    /// Drops SMS challenge storage and phone columns for local rollback.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS sms_verification_challenges")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_users_phone")
            .await?;
        db.execute_unprepared("ALTER TABLE users DROP COLUMN phone_bound_at")
            .await?;
        db.execute_unprepared("ALTER TABLE users DROP COLUMN phone")
            .await?;
        Ok(())
    }
}
