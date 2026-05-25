//! Upload image metadata migration storing private MinIO object keys and owner-scoped image metadata.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS upload_images (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                purpose TEXT NOT NULL,
                original_filename TEXT NOT NULL,
                bucket TEXT NOT NULL,
                object_key TEXT NOT NULL UNIQUE,
                image_type TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size_bytes BIGINT NOT NULL,
                sha256 TEXT NOT NULL,
                etag TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_upload_images_user_purpose_created ON upload_images(user_id, purpose, created_at)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS upload_images")
            .await?;
        Ok(())
    }
}
