//! 数据库迁移 crate 入口，按顺序注册 StellarTrail 的所有 schema migration。

use sea_orm_migration::prelude::*;

mod m20260516_000001_create_users_sessions;
mod m20260516_000002_create_user_gear_items;
mod m20260516_000003_add_password_auth;
mod m20260516_000004_create_captcha_challenges;

/// SeaORM migrator 实现，按注册顺序执行所有 schema migration。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    /// 执行 `migrations` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // migration 顺序即 schema 演进顺序，新 migration 必须追加在列表末尾。
        vec![
            Box::new(m20260516_000001_create_users_sessions::Migration),
            Box::new(m20260516_000002_create_user_gear_items::Migration),
            Box::new(m20260516_000003_add_password_auth::Migration),
            Box::new(m20260516_000004_create_captcha_challenges::Migration),
        ]
    }
}
