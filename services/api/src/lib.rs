//! Public StellarTrail API crate entrypoint that assembles configuration, database, cache, DB-backed seeds, object storage, email, and route state.

pub mod api_usage;
pub mod cache;
pub mod config;
pub mod dto;
pub mod email;
pub mod error;
pub mod extractors;
pub mod object_store;
pub mod routes;
pub mod services;
pub mod state;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{connect_database, repositories::GearTemplateRepository};
use stellartrail_domain::gear_template::default_system_gear_templates;
use stellartrail_migration::Migrator;

use config::ApiConfig;
use email::{EmailSender, NoopEmailSender, SmtpEmailSender};
use object_store::MinioObjectStore;
use state::AppState;

/// Creates the database connection, runs migrations, seeds DB-backed defaults, and builds AppState from configuration.
pub async fn build_state(config: ApiConfig) -> anyhow::Result<AppState> {
    let cache = cache::Cache::from_config(&config.redis_cache).await?;
    // Startup connects to the database before running migrations so routes never see an uninitialized schema.
    let db = connect_database(&config.database).await?;
    migrate_database(&db).await?;
    seed_system_gear_templates(&db).await?;
    let object_store =
        MinioObjectStore::from_config(&config.minio, &config.object_storage.bucket).await?;
    let email_sender: Arc<dyn EmailSender> = if config.mail.enabled {
        Arc::new(SmtpEmailSender::from_config(&config.mail)?)
    } else {
        Arc::new(NoopEmailSender)
    };
    Ok(
        AppState::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            Arc::new(services::wechat::HttpWechatCodeSessionClient),
            cache,
            Arc::new(object_store),
            email_sender,
        ),
    )
}

async fn seed_system_gear_templates(db: &DatabaseConnection) -> anyhow::Result<()> {
    GearTemplateRepository::new(db.clone())
        .replace_system_templates("system_seed", &default_system_gear_templates())
        .await?;
    Ok(())
}

/// Runs database migrations so the schema reaches the current version before the service starts.
pub async fn migrate_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
