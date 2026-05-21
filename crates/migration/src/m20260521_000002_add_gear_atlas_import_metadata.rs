//! Adds source-audit metadata for conservative public gear atlas imports.
//!
//! Imported rows remain normal atlas submissions that require administrator
//! review, but these columns preserve enough provenance to inspect where a
//! public fact came from and to refresh the same source record idempotently.

use sea_orm_migration::prelude::*;

/// Adds nullable import metadata to public gear atlas items.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates source-audit columns and indexes for idempotent external imports.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in ADD_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_gear_atlas_source_key \
             ON gear_atlas_items(source_key) WHERE source_key IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_atlas_import_batch \
             ON gear_atlas_items(import_batch_id, imported_at)",
        )
        .await?;
        Ok(())
    }

    /// Drops the import metadata added by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_import_batch")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_source_key")
            .await?;
        for sql in DROP_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}

const ADD_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE gear_atlas_items ADD COLUMN source_key TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN source_name TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN source_url TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN source_license_note TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN import_batch_id TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN imported_at TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN source_rating_score DOUBLE PRECISION NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN source_rating_count INTEGER NULL",
];

const DROP_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE gear_atlas_items DROP COLUMN source_rating_count",
    "ALTER TABLE gear_atlas_items DROP COLUMN source_rating_score",
    "ALTER TABLE gear_atlas_items DROP COLUMN imported_at",
    "ALTER TABLE gear_atlas_items DROP COLUMN import_batch_id",
    "ALTER TABLE gear_atlas_items DROP COLUMN source_license_note",
    "ALTER TABLE gear_atlas_items DROP COLUMN source_url",
    "ALTER TABLE gear_atlas_items DROP COLUMN source_name",
    "ALTER TABLE gear_atlas_items DROP COLUMN source_key",
];
