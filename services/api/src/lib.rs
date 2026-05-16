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

pub async fn build_state(config: ApiConfig) -> anyhow::Result<AppState> {
    let content = read_content_catalog(&config.content_dir)?;
    let cache = cache::Cache::from_config(&config.redis_cache).await?;
    let db = connect_database(&config.database).await?;
    migrate_database(&db).await?;
    Ok(AppState::new_with_content_and_cache(
        config, db, content, cache,
    ))
}

pub async fn migrate_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
