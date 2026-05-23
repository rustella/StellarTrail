//! Adds soft-delete flags to top-level user and atlas records.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for table in [
            "user_gear_items",
            "gear_atlas_items",
            "user_feedback",
            "upload_images",
        ] {
            db.execute_unprepared(&format!(
                "ALTER TABLE {table} ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT FALSE"
            ))
            .await?;
        }
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_gear_user_deleted_archived_created ON user_gear_items(user_id, is_deleted, archived_at, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_deleted_status_approved_created ON gear_atlas_items(is_deleted, status, approved_at, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_feedback_deleted_status_created ON user_feedback(is_deleted, status, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_upload_images_user_purpose_deleted_created ON upload_images(user_id, purpose, is_deleted, created_at)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and removes the indexes and columns created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_upload_images_user_purpose_deleted_created",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_feedback_deleted_status_created")
            .await?;
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_gear_atlas_deleted_status_approved_created",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_gear_user_deleted_archived_created")
            .await?;
        for table in [
            "upload_images",
            "user_feedback",
            "gear_atlas_items",
            "user_gear_items",
        ] {
            db.execute_unprepared(&format!("ALTER TABLE {table} DROP COLUMN is_deleted"))
                .await?;
        }
        Ok(())
    }
}
