//! Migration creating the public gear atlas submission table.
//!
//! The schema intentionally stores only public market-equipment fields. User
//! purchase, location, status, notes, tags, and tag colors stay in personal gear
//! tables and are never represented here.

use sea_orm_migration::prelude::*;

/// Single migration type for the gear atlas table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the gear atlas table and read/review indexes.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS gear_atlas_items (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                brand TEXT NULL,
                model TEXT NULL,
                description TEXT NULL,
                weight_g INTEGER NULL,
                official_price_cents BIGINT NULL,
                official_price_currency TEXT NULL,
                specs_json TEXT NOT NULL DEFAULT '{}',
                source_type TEXT NOT NULL,
                submitted_by_user_id TEXT NOT NULL REFERENCES users(id),
                source_user_gear_id TEXT NULL REFERENCES user_gear_items(id),
                status TEXT NOT NULL DEFAULT 'pending',
                rejection_reason TEXT NULL,
                reviewed_by_user_id TEXT NULL REFERENCES users(id),
                reviewed_at TEXT NULL,
                approved_at TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_status_approved_created ON gear_atlas_items(status, approved_at, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_status_category ON gear_atlas_items(status, category)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_submitter_created ON gear_atlas_items(submitted_by_user_id, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_source_gear_status ON gear_atlas_items(submitted_by_user_id, source_user_gear_id, status)",
        )
        .await?;
        Ok(())
    }

    /// Drops the public gear atlas table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_source_gear_status")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_submitter_created")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_status_category")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_status_approved_created")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_atlas_items")
            .await?;
        Ok(())
    }
}
