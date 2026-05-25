//! Public content localization migration for gear templates, gear atlas items, and gear category labels.

use sea_orm_migration::prelude::*;

/// Adds locale-specific public text tables while leaving user-entered private text unchanged.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260521_000001_add_public_content_localizations"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates localization tables and seeds Chinese fallbacks plus English category labels.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(PUBLIC_CONTENT_LOCALIZATION_SCHEMA_SQL)
            .await?;
        db.execute_unprepared(BACKFILL_GEAR_TEMPLATE_LOCALIZATIONS_SQL)
            .await?;
        db.execute_unprepared(BACKFILL_GEAR_ATLAS_LOCALIZATIONS_SQL)
            .await?;
        for sql in GEAR_CATEGORY_LABEL_SEEDS {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drops localization tables introduced by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_category_localizations_locale")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_atlas_item_localizations_locale")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_template_item_localizations_locale")
            .await?;
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_gear_template_category_localizations_locale",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_template_localizations_locale")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_category_localizations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_atlas_item_localizations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_template_item_localizations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_template_category_localizations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_template_localizations")
            .await?;
        Ok(())
    }
}

const PUBLIC_CONTENT_LOCALIZATION_SCHEMA_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS gear_template_localizations (
        template_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        title TEXT NOT NULL,
        PRIMARY KEY (template_id, locale),
        FOREIGN KEY (template_id) REFERENCES gear_templates(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS gear_template_category_localizations (
        template_id TEXT NOT NULL,
        category_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        PRIMARY KEY (template_id, category_id, locale),
        FOREIGN KEY (template_id, category_id) REFERENCES gear_template_categories(template_id, id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS gear_template_item_localizations (
        template_id TEXT NOT NULL,
        category_id TEXT NOT NULL,
        item_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        PRIMARY KEY (template_id, category_id, item_id, locale),
        FOREIGN KEY (template_id, category_id, item_id) REFERENCES gear_template_items(template_id, category_id, id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS gear_atlas_item_localizations (
        atlas_item_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        description TEXT NULL,
        PRIMARY KEY (atlas_item_id, locale),
        FOREIGN KEY (atlas_item_id) REFERENCES gear_atlas_items(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS gear_category_localizations (
        category TEXT NOT NULL,
        locale TEXT NOT NULL,
        label TEXT NOT NULL,
        PRIMARY KEY (category, locale)
    );

    CREATE INDEX IF NOT EXISTS idx_gear_template_localizations_locale
        ON gear_template_localizations(locale, template_id);
    CREATE INDEX IF NOT EXISTS idx_gear_template_category_localizations_locale
        ON gear_template_category_localizations(locale, template_id, category_id);
    CREATE INDEX IF NOT EXISTS idx_gear_template_item_localizations_locale
        ON gear_template_item_localizations(locale, template_id, category_id, item_id);
    CREATE INDEX IF NOT EXISTS idx_gear_atlas_item_localizations_locale
        ON gear_atlas_item_localizations(locale, atlas_item_id);
    CREATE INDEX IF NOT EXISTS idx_gear_category_localizations_locale
        ON gear_category_localizations(locale, category);
"#;

const BACKFILL_GEAR_TEMPLATE_LOCALIZATIONS_SQL: &str = r#"
    INSERT INTO gear_template_localizations(template_id, locale, title)
    SELECT id, 'zh-CN', title FROM gear_templates WHERE true
    ON CONFLICT(template_id, locale) DO NOTHING;

    INSERT INTO gear_template_category_localizations(template_id, category_id, locale, name)
    SELECT template_id, id, 'zh-CN', name FROM gear_template_categories WHERE true
    ON CONFLICT(template_id, category_id, locale) DO NOTHING;

    INSERT INTO gear_template_item_localizations(template_id, category_id, item_id, locale, name)
    SELECT template_id, category_id, id, 'zh-CN', name FROM gear_template_items WHERE true
    ON CONFLICT(template_id, category_id, item_id, locale) DO NOTHING;
"#;

const BACKFILL_GEAR_ATLAS_LOCALIZATIONS_SQL: &str = r#"
    INSERT INTO gear_atlas_item_localizations(atlas_item_id, locale, name, description)
    SELECT id, 'zh-CN', name, description FROM gear_atlas_items WHERE true
    ON CONFLICT(atlas_item_id, locale) DO NOTHING;
"#;

const GEAR_CATEGORY_LABEL_SEEDS: &[&str] = &[
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('backpack_system', 'zh-CN', '背负系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('backpack_system', 'en', 'Backpack System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('sleep_system', 'zh-CN', '睡眠系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('sleep_system', 'en', 'Sleep System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('kitchen_system', 'zh-CN', '餐厨系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('kitchen_system', 'en', 'Kitchen System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('walking_system', 'zh-CN', '行走系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('walking_system', 'en', 'Walking System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('clothing_system', 'zh-CN', '衣物系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('clothing_system', 'en', 'Clothing System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('lighting_system', 'zh-CN', '照明系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('lighting_system', 'en', 'Lighting System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('first_aid_system', 'zh-CN', '急救系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('first_aid_system', 'en', 'First Aid System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('electronics_system', 'zh-CN', '电子系统') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('electronics_system', 'en', 'Electronics System') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('technical_gear', 'zh-CN', '技术装备') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('technical_gear', 'en', 'Technical Gear') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('other_gear', 'zh-CN', '其它装备') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('other_gear', 'en', 'Other Gear') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('consumable', 'zh-CN', '消耗品') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
    "INSERT INTO gear_category_localizations(category, locale, label) VALUES ('consumable', 'en', 'Consumables') ON CONFLICT(category, locale) DO UPDATE SET label = excluded.label",
];
