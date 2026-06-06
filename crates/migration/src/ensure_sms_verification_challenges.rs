//! Backfills SMS verification storage on databases created before phone auth.

use sea_orm_migration::prelude::*;

/// Compatibility migration for existing databases missing SMS verification challenges.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260606_000002_ensure_sms_verification_challenges"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the SMS challenge table and indexes when old databases missed them.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
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
            "CREATE INDEX IF NOT EXISTS idx_sms_verification_phone_created ON sms_verification_challenges(phone, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sms_verification_out_id ON sms_verification_challenges(out_id)",
        )
        .await?;
        Ok(())
    }

    /// Keeps the table because other authentication flows may reference it.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn up_creates_sms_verification_challenges_and_is_idempotent() {
        let db = Database::connect("sqlite::memory:").await.expect("connect");
        let manager = SchemaManager::new(&db);

        Migration.up(&manager).await.expect("first migration run");
        Migration
            .up(&manager)
            .await
            .expect("idempotent migration run");

        assert!(
            manager
                .has_table("sms_verification_challenges")
                .await
                .expect("sms challenge table")
        );
        assert!(
            manager
                .has_column("sms_verification_challenges", "out_id")
                .await
                .expect("sms out_id")
        );
    }
}
