//! Backfills gear archive columns on databases created before folded gear schema.

use sea_orm_migration::prelude::*;

/// Compatibility migration for existing gear tables missing archive state fields.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260607_000001_ensure_user_gear_archive_fields"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds missing gear archive columns and indexes only on older databases.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        if !manager.has_column("user_gear_items", "is_deleted").await? {
            db.execute_unprepared(
                "ALTER TABLE user_gear_items ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT FALSE",
            )
            .await?;
        }
        if !manager.has_column("user_gear_items", "archived_at").await? {
            db.execute_unprepared("ALTER TABLE user_gear_items ADD COLUMN archived_at TEXT NULL")
                .await?;
        }
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_gear_user_archived_created ON user_gear_items(user_id, archived_at, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_gear_user_deleted_archived_created ON user_gear_items(user_id, is_deleted, archived_at, created_at)",
        )
        .await?;
        Ok(())
    }

    /// Keeps archive columns in place because dropping them would lose user state.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn up_backfills_missing_gear_archive_columns_and_is_idempotent() {
        let db = Database::connect("sqlite::memory:").await.expect("connect");
        db.execute_unprepared(
            r#"CREATE TABLE user_gear_items (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await
        .expect("create legacy gear table");
        let manager = SchemaManager::new(&db);

        Migration.up(&manager).await.expect("first migration run");
        Migration
            .up(&manager)
            .await
            .expect("idempotent migration run");

        assert!(
            manager
                .has_column("user_gear_items", "is_deleted")
                .await
                .expect("is_deleted")
        );
        assert!(
            manager
                .has_column("user_gear_items", "archived_at")
                .await
                .expect("archived_at")
        );
    }
}
