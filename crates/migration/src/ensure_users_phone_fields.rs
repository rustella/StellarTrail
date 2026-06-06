//! Backfills phone-auth columns on databases created before folded auth schema.

use sea_orm_migration::prelude::*;

/// Compatibility migration for existing databases whose base users table predates phone auth.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260606_000001_ensure_users_phone_fields"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds missing phone columns only on older databases.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        if !manager.has_column("users", "phone").await? {
            db.execute_unprepared("ALTER TABLE users ADD COLUMN phone TEXT NULL")
                .await?;
        }
        if !manager.has_column("users", "phone_bound_at").await? {
            db.execute_unprepared("ALTER TABLE users ADD COLUMN phone_bound_at TEXT NULL")
                .await?;
        }
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_phone ON users(phone) WHERE phone IS NOT NULL",
        )
        .await?;
        Ok(())
    }

    /// Keeps columns in place because dropping columns is unsafe across supported databases.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn up_backfills_missing_phone_columns_and_is_idempotent() {
        let db = Database::connect("sqlite::memory:").await.expect("connect");
        db.execute_unprepared(
            r#"CREATE TABLE users (
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
        .await
        .expect("create legacy users table");
        let manager = SchemaManager::new(&db);

        Migration.up(&manager).await.expect("first migration run");
        Migration
            .up(&manager)
            .await
            .expect("idempotent migration run");

        assert!(manager.has_column("users", "phone").await.expect("phone"));
        assert!(
            manager
                .has_column("users", "phone_bound_at")
                .await
                .expect("phone_bound_at")
        );
    }
}
