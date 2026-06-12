//! Migration creating the public gear atlas submission table.
//!
//! The schema intentionally stores only public market-equipment fields. User
//! purchase, location, status, notes, tags, and tag colors stay in personal gear
//! tables and are never represented here.

use sea_orm_migration::prelude::*;

/// Single migration type for the gear atlas table.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260518_000005_create_gear_atlas_items"
    }
}

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
                variants_json TEXT NOT NULL DEFAULT '[]',
                specs_json TEXT NOT NULL DEFAULT '{}',
                submitted_snapshot_json TEXT NULL,
                review_changes_json TEXT NULL,
                source_type TEXT NOT NULL,
                submitted_by_user_id TEXT NOT NULL REFERENCES users(id),
                source_user_gear_id TEXT NULL REFERENCES user_gear_items(id),
                status TEXT NOT NULL DEFAULT 'pending',
                rejection_reason TEXT NULL,
                reviewed_by_user_id TEXT NULL REFERENCES users(id),
                reviewed_at TEXT NULL,
                approved_at TEXT NULL,
                source_key TEXT NULL,
                source_name TEXT NULL,
                source_url TEXT NULL,
                source_license_note TEXT NULL,
                import_batch_id TEXT NULL,
                imported_at TEXT NULL,
                source_rating_score DOUBLE PRECISION NULL,
                source_rating_count INTEGER NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
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
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_gear_atlas_source_key ON gear_atlas_items(source_key) WHERE source_key IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_import_batch ON gear_atlas_items(import_batch_id, imported_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_deleted_status_approved_created ON gear_atlas_items(is_deleted, status, approved_at, created_at)",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS gear_atlas_import_sources (
                source_key TEXT PRIMARY KEY,
                canonical_key TEXT NOT NULL,
                atlas_item_id TEXT NOT NULL REFERENCES gear_atlas_items(id) ON DELETE CASCADE,
                source_name TEXT NOT NULL,
                source_url TEXT NULL,
                source_locale TEXT NOT NULL,
                detail_score INTEGER NOT NULL DEFAULT 0,
                last_seen_batch_id TEXT NULL,
                last_seen_at TEXT NOT NULL,
                last_action TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_import_sources_canonical ON gear_atlas_import_sources(canonical_key, detail_score)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_import_sources_item ON gear_atlas_import_sources(atlas_item_id)",
        )
        .await?;
        Ok(())
    }

    /// Drops the public gear atlas table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_import_sources_item")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_import_sources_canonical")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_atlas_import_sources")
            .await?;
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_gear_atlas_deleted_status_approved_created",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_import_batch")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_source_key")
            .await?;
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
