//! Backfills gear atlas import-source and localized fact columns on older databases.

use sea_orm_migration::prelude::*;

/// Compatibility migration for existing gear atlas databases missing importer i18n schema.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260611_000001_ensure_gear_atlas_import_i18n"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds localized fact columns and source-mapping tables only when absent.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for (column, ddl) in [
            (
                "variants_json",
                "ALTER TABLE gear_atlas_item_localizations ADD COLUMN variants_json TEXT NULL",
            ),
            (
                "specs_json",
                "ALTER TABLE gear_atlas_item_localizations ADD COLUMN specs_json TEXT NULL",
            ),
            (
                "translation_status",
                "ALTER TABLE gear_atlas_item_localizations ADD COLUMN translation_status TEXT NULL",
            ),
            (
                "translation_provider",
                "ALTER TABLE gear_atlas_item_localizations ADD COLUMN translation_provider TEXT NULL",
            ),
            (
                "translated_at",
                "ALTER TABLE gear_atlas_item_localizations ADD COLUMN translated_at TEXT NULL",
            ),
        ] {
            if !manager
                .has_column("gear_atlas_item_localizations", column)
                .await?
            {
                db.execute_unprepared(ddl).await?;
            }
        }
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

    /// Keeps import audit data in place because it is part of review provenance.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
