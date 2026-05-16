//! Public StellarTrail API crate entrypoint that assembles configuration, database, cache, content catalog, and route state.

pub mod cache;
pub mod config;
pub mod dto;
pub mod error;
pub mod object_store;
pub mod routes;
pub mod services;
pub mod state;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::connect_database;
use stellartrail_importer::read_content_catalog;
use stellartrail_migration::Migrator;

use config::ApiConfig;
use object_store::MinioObjectStore;
use state::AppState;

/// Creates the database connection, runs migrations, loads the content catalog, and builds AppState from configuration.
pub async fn build_state(config: ApiConfig) -> anyhow::Result<AppState> {
    let content = read_content_catalog(&config.content_dir)?;
    let cache = cache::Cache::from_config(&config.redis_cache).await?;
    // Startup connects to the database before running migrations so routes never see an uninitialized schema.
    let db = connect_database(&config.database).await?;
    migrate_database(&db).await?;
    let object_store = MinioObjectStore::from_config(&config.object_storage).await?;
    Ok(
        AppState::new_with_content_and_wechat_client_cache_and_object_store(
            config,
            db,
            content,
            Arc::new(services::wechat::CurlWechatCodeSessionClient),
            cache,
            Arc::new(object_store),
        ),
    )
}

/// Runs database migrations so the schema reaches the current version before the service starts.
pub async fn migrate_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
