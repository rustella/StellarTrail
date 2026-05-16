//! 用户装备表 migration，建立装备字段、归档状态和常用查询索引。

use sea_orm_migration::prelude::*;

/// 单个数据库 migration 类型，由 SeaORM migration 框架调用 up/down。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// 执行 schema 升级逻辑。
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // 装备表字段直接对应领域模型，减少 repository 映射时的隐式转换。
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_gear_items (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                brand TEXT NULL,
                model TEXT NULL,
                color TEXT NULL,
                material TEXT NULL,
                capacity TEXT NULL,
                size TEXT NULL,
                description TEXT NULL,
                weight_g INTEGER NULL,
                warmth_index TEXT NULL,
                waterproof_index TEXT NULL,
                purchase_date TEXT NULL,
                purchase_price_cents BIGINT NULL,
                expiry_or_warranty_date TEXT NULL,
                purchase_location TEXT NULL,
                status TEXT NOT NULL DEFAULT 'available',
                storage_location TEXT NULL,
                tags_json TEXT NOT NULL DEFAULT '[]',
                share_enabled BOOLEAN NOT NULL DEFAULT FALSE,
                share_status TEXT NOT NULL DEFAULT 'not_shared',
                notes TEXT NULL,
                archived_at TEXT NULL,
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
        Ok(())
    }

    /// 执行 schema 回滚逻辑，尽量撤销 up 中创建的表或索引。
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS user_gear_items")
            .await?;
        Ok(())
    }
}
