//! Adds a compact owner-purpose index for cumulative upload quota checks.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_upload_images_user_purpose ON upload_images(user_id, purpose)",
            )
            .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_upload_images_user_purpose")
            .await?;
        Ok(())
    }
}
