//! Creates user-owned gear packing lists and item rows for pre-hike packing checks.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260524_000001_create_gear_packing_lists"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS gear_packing_lists (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                route_name TEXT NULL,
                duration_label TEXT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS gear_packing_list_items (
                id TEXT PRIMARY KEY,
                packing_list_id TEXT NOT NULL REFERENCES gear_packing_lists(id),
                user_id TEXT NOT NULL REFERENCES users(id),
                gear_id TEXT NOT NULL REFERENCES user_gear_items(id),
                planned_quantity INTEGER NOT NULL DEFAULT 1,
                packed_quantity INTEGER NOT NULL DEFAULT 0,
                packed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (packing_list_id, gear_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_packing_lists_user_deleted_updated ON gear_packing_lists(user_id, is_deleted, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_packing_items_list_created ON gear_packing_list_items(packing_list_id, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gear_packing_items_user_gear ON gear_packing_list_items(user_id, gear_id)",
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and removes the tables created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_packing_items_user_gear")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_packing_items_list_created")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gear_packing_lists_user_deleted_updated")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_packing_list_items")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS gear_packing_lists")
            .await?;
        Ok(())
    }
}
