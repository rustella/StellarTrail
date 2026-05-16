//! StellarTrail API crate 公共入口，组装配置、数据库、缓存、内容目录和路由状态。

pub mod cache;
pub mod config;
pub mod dto;
pub mod error;
pub mod routes;
pub mod services;
pub mod state;

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::connect_database;
use stellartrail_importer::read_content_catalog;
use stellartrail_migration::Migrator;

use config::ApiConfig;
use state::AppState;

/// 根据配置创建数据库连接、执行迁移、加载内容目录并组装 AppState。
pub async fn build_state(config: ApiConfig) -> anyhow::Result<AppState> {
    let content = read_content_catalog(&config.content_dir)?;
    let cache = cache::Cache::from_config(&config.redis_cache).await?;
    // 服务启动时先建立数据库连接，再运行 migration，避免路由接入未初始化 schema。
    let db = connect_database(&config.database).await?;
    migrate_database(&db).await?;
    Ok(AppState::new_with_content_and_cache(
        config, db, content, cache,
    ))
}

/// 执行数据库 migration，确保服务启动前 schema 达到当前版本。
pub async fn migrate_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
