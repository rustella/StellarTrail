//! User gear table migration creating gear fields, archive state, and common query indexes.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260516_000002_create_user_gear_items"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Gear table fields map directly to the domain model, reducing implicit conversions in the repository.
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_gear_items (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                brand TEXT NULL,
                model TEXT NULL,
                description TEXT NULL,
                weight_g INTEGER NULL,
                official_price_cents BIGINT NULL,
                official_price_currency TEXT NULL,
                purchase_date TEXT NULL,
                purchase_price_cents BIGINT NULL,
                purchase_price_currency TEXT NULL,
                purchase_location TEXT NULL,
                status TEXT NOT NULL DEFAULT 'available',
                storage_location TEXT NULL,
                atlas_item_id TEXT NULL,
                selected_variant_key TEXT NULL,
                selected_variant_label TEXT NULL,
                specs_json TEXT NOT NULL DEFAULT '{}',
                quantity INTEGER NOT NULL DEFAULT 1,
                tags_json TEXT NOT NULL DEFAULT '[]',
                share_enabled BOOLEAN NOT NULL DEFAULT FALSE,
                share_status TEXT NOT NULL DEFAULT 'not_shared',
                notes TEXT NULL,
                archived_at TEXT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_user_archived_created ON user_gear_items(user_id, archived_at, created_at)")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_user_category ON user_gear_items(user_id, category)")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_user_status ON user_gear_items(user_id, status)")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_user_purchase_date ON user_gear_items(user_id, purchase_date)")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_items_atlas_item ON user_gear_items(atlas_item_id)")
            .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_user_gear_user_deleted_archived_created ON user_gear_items(user_id, is_deleted, archived_at, created_at)")
            .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS user_gear_items")
            .await?;
        Ok(())
    }
}
