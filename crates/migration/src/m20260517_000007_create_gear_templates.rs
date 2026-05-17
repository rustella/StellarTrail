//! Gear template migration creating DB-backed public equipment checklist templates.

use sea_orm_migration::prelude::*;

/// Migration adding public gear template tables.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates gear template tables and ordering indexes.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(GEAR_TEMPLATE_SCHEMA_SQL).await?;
        Ok(())
    }

    /// Drops only gear template tables introduced by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_template_items_order")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_template_categories_order")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_templates_status_order")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_template_items")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_template_categories")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_templates")
            .await?;
        Ok(())
    }
}

const GEAR_TEMPLATE_SCHEMA_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS gear_templates (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        source TEXT NOT NULL DEFAULT 'system_seed',
        status TEXT NOT NULL DEFAULT 'active',
        sort_order INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CHECK (status IN ('active', 'archived'))
    );

    CREATE TABLE IF NOT EXISTS gear_template_categories (
        template_id TEXT NOT NULL,
        id TEXT NOT NULL,
        name TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (template_id, id),
        FOREIGN KEY (template_id) REFERENCES gear_templates(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS gear_template_items (
        template_id TEXT NOT NULL,
        category_id TEXT NOT NULL,
        id TEXT NOT NULL,
        name TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (template_id, category_id, id),
        FOREIGN KEY (template_id, category_id) REFERENCES gear_template_categories(template_id, id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_gear_templates_status_order ON gear_templates(status, sort_order, id);
    CREATE INDEX IF NOT EXISTS idx_gear_template_categories_order ON gear_template_categories(template_id, sort_order, id);
    CREATE INDEX IF NOT EXISTS idx_gear_template_items_order ON gear_template_items(template_id, category_id, sort_order, id);
"#;
