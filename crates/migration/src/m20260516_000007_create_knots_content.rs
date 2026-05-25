//! Knots content migration creating DB-backed public outdoor skill knot metadata, localization, taxonomy, media, and import audit tables.

use sea_orm_migration::prelude::*;

/// Migration adding Knots3D-backed outdoor skill tables.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(KNOTS_SCHEMA_SQL).await?;
        db.execute_unprepared(
            "INSERT INTO skill_categories(id, slug) VALUES ('knots', 'knots') ON CONFLICT(id) DO NOTHING",
        )
        .await?;
        db.execute_unprepared(
            "INSERT INTO skill_category_localizations(category_id, locale, title, summary) \
             VALUES ('knots', 'zh-CN', '绳结', '户外、露营、钓鱼、航海等场景常用绳结技能。') \
             ON CONFLICT(category_id, locale) DO UPDATE SET title = excluded.title, summary = excluded.summary",
        )
        .await?;
        db.execute_unprepared(
            "INSERT INTO skill_category_localizations(category_id, locale, title, summary) \
             VALUES ('knots', 'en', 'Knots', 'Outdoor knots for camping, fishing, sailing, and field skills.') \
             ON CONFLICT(category_id, locale) DO UPDATE SET title = excluded.title, summary = excluded.summary",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic while preserving shared skill category tables owned by other features.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DELETE FROM skill_category_localizations WHERE category_id = 'knots'",
        )
        .await?;
        db.execute_unprepared("DELETE FROM skill_categories WHERE id = 'knots'")
            .await?;
        for table in [
            "knot_raw_metadata",
            "knot_import_runs",
            "knot_media_assets",
            "knot_type_memberships",
            "knot_type_localizations",
            "knot_types",
            "knot_category_memberships",
            "knot_category_localizations",
            "knot_categories",
            "knot_localizations",
            "knots",
        ] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}

const KNOTS_SCHEMA_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS skill_categories (
        id TEXT PRIMARY KEY,
        slug TEXT NOT NULL UNIQUE
    );

    CREATE TABLE IF NOT EXISTS skill_category_localizations (
        category_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT NOT NULL,
        PRIMARY KEY (category_id, locale),
        FOREIGN KEY (category_id) REFERENCES skill_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knots (
        id TEXT PRIMARY KEY,
        source_name TEXT NOT NULL,
        source_url TEXT NULL,
        source_slug_en TEXT NOT NULL,
        source_slug_zh TEXT NULL,
        difficulty TEXT NULL
    );

    CREATE TABLE IF NOT EXISTS knot_localizations (
        knot_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT NOT NULL,
        description TEXT NULL,
        steps_json TEXT NOT NULL DEFAULT '[]',
        PRIMARY KEY (knot_id, locale),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_categories (
        id TEXT PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS knot_category_localizations (
        category_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        PRIMARY KEY (category_id, locale),
        FOREIGN KEY (category_id) REFERENCES knot_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_category_memberships (
        knot_id TEXT NOT NULL,
        category_id TEXT NOT NULL,
        PRIMARY KEY (knot_id, category_id),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        FOREIGN KEY (category_id) REFERENCES knot_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_types (
        id TEXT PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS knot_type_localizations (
        type_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        PRIMARY KEY (type_id, locale),
        FOREIGN KEY (type_id) REFERENCES knot_types(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_type_memberships (
        knot_id TEXT NOT NULL,
        type_id TEXT NOT NULL,
        PRIMARY KEY (knot_id, type_id),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        FOREIGN KEY (type_id) REFERENCES knot_types(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_media_assets (
        knot_id TEXT NOT NULL,
        asset_id TEXT NOT NULL,
        media_type TEXT NOT NULL,
        path TEXT NOT NULL,
        mime_type TEXT NOT NULL,
        width INTEGER NULL,
        height INTEGER NULL,
        attribution TEXT NULL,
        license_note TEXT NULL,
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        PRIMARY KEY (knot_id, asset_id)
    );

    CREATE TABLE IF NOT EXISTS knot_import_runs (
        source TEXT NOT NULL,
        item_count INTEGER NOT NULL,
        imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS knot_raw_metadata (
        knot_id TEXT PRIMARY KEY,
        raw_json TEXT NOT NULL,
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_knot_localizations_locale_slug ON knot_localizations(locale, slug);
    CREATE INDEX IF NOT EXISTS idx_knot_localizations_locale_title ON knot_localizations(locale, title);
    CREATE INDEX IF NOT EXISTS idx_knot_category_memberships_category ON knot_category_memberships(category_id, knot_id);
    CREATE INDEX IF NOT EXISTS idx_knot_type_memberships_type ON knot_type_memberships(type_id, knot_id);
    CREATE INDEX IF NOT EXISTS idx_knot_media_assets_knot ON knot_media_assets(knot_id);
"#;
