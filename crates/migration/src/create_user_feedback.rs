//! User feedback migration creating feedback rows and an unlimited image association table.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260516_000006_create_user_feedback"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_feedback (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                contact TEXT NULL,
                page TEXT NULL,
                client_platform TEXT NULL,
                client_version TEXT NULL,
                device_model TEXT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_feedback_images (
                feedback_id TEXT NOT NULL REFERENCES user_feedback(id) ON DELETE CASCADE,
                upload_image_id TEXT NOT NULL REFERENCES upload_images(id) ON DELETE RESTRICT,
                sort_order INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (feedback_id, upload_image_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_feedback_user_created ON user_feedback(user_id, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_feedback_status_created ON user_feedback(status, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_feedback_deleted_status_created ON user_feedback(is_deleted, status, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_feedback_images_feedback_order ON user_feedback_images(feedback_id, sort_order)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS user_feedback_images")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS user_feedback")
            .await?;
        Ok(())
    }
}
