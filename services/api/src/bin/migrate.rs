//! Database migration command entrypoint for local development, deployment pipelines, and one-off schema maintenance.

use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_api::config::ApiConfig;
use stellartrail_db::connect_database;
use stellartrail_migration::Migrator;

/// Runs the `main` server-side flow while preserving input validation, error propagation, and state invariants.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let command = std::env::args().nth(1).unwrap_or_else(|| "up".to_owned());
    let config = ApiConfig::from_env()?;
    let db = connect_database(&config.database).await?;
    match command.as_str() {
        "up" => Migrator::up(&db, None).await?,
        "down" => Migrator::down(&db, None).await?,
        "fresh" => Migrator::fresh(&db).await?,
        other => anyhow::bail!("unsupported migrate command: {other}"),
    }
    Ok(())
}
