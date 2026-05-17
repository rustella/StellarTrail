//! Public StellarTrail API crate entrypoint that assembles configuration, database, cache, content catalog, and route state.

pub mod cache;
pub mod config;
pub mod dto;
pub mod email;
pub mod error;
pub mod object_store;
pub mod routes;
pub mod services;
pub mod state;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::connect_database;
use stellartrail_domain::{
    mountain::DifficultyLevel,
    skill::{
        KnotCategorySeed, KnotLocalizationSeed, KnotSeed, KnotTypeSeed, Locale, SkillCategory,
    },
};
use stellartrail_importer::{ContentCatalog, SkillContent, read_content_catalog};
use stellartrail_migration::Migrator;

use config::ApiConfig;
use email::{EmailSender, NoopEmailSender, SmtpEmailSender};
use object_store::MinioObjectStore;
use state::AppState;

/// Creates the database connection, runs migrations, loads the content catalog, and builds AppState from configuration.
pub async fn build_state(config: ApiConfig) -> anyhow::Result<AppState> {
    let content = read_content_catalog(&config.content_dir)?;
    let cache = cache::Cache::from_config(&config.redis_cache).await?;
    // Startup connects to the database before running migrations so routes never see an uninitialized schema.
    let db = connect_database(&config.database).await?;
    migrate_database(&db).await?;
    seed_public_knots_from_content(&db, &content).await?;
    let object_store = MinioObjectStore::from_config(&config.object_storage).await?;
    let email_sender: Arc<dyn EmailSender> = if config.mail.enabled {
        Arc::new(SmtpEmailSender::from_config(&config.mail)?)
    } else {
        Arc::new(NoopEmailSender)
    };
    Ok(
        AppState::new_with_content_and_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            content,
            Arc::new(services::wechat::CurlWechatCodeSessionClient),
            cache,
            Arc::new(object_store),
            email_sender,
        ),
    )
}

async fn seed_public_knots_from_content(
    db: &DatabaseConnection,
    content: &ContentCatalog,
) -> anyhow::Result<()> {
    let seeds = content
        .skills
        .iter()
        .filter(|skill| matches!(skill.category, SkillCategory::Knot))
        .map(knot_seed_from_skill)
        .collect::<Vec<_>>();
    if seeds.is_empty() {
        return Ok(());
    }
    stellartrail_db::repositories::KnotRepository::new(db.clone(), "/assets")
        .replace_all_knots("content/skills", &seeds)
        .await?;
    Ok(())
}

fn knot_seed_from_skill(skill: &SkillContent) -> KnotSeed {
    let steps = extract_numbered_steps(&skill.body_markdown);
    let difficulty = match skill.difficulty_level {
        DifficultyLevel::Leisure => "leisure",
        DifficultyLevel::Beginner => "beginner",
        DifficultyLevel::Intermediate => "intermediate",
        DifficultyLevel::Advanced => "advanced",
        DifficultyLevel::Technical => "technical",
    };
    KnotSeed {
        id: skill.id.clone(),
        source_name: "StellarTrail content".to_owned(),
        source_url: None,
        source_slug_en: skill.id.clone(),
        source_slug_zh: Some(skill.id.clone()),
        difficulty: Some(difficulty.to_owned()),
        localizations: vec![KnotLocalizationSeed {
            locale: Locale::ZhCn,
            slug: skill.id.clone(),
            title: skill.title.clone(),
            summary: skill.summary.clone(),
            description: Some(skill.body_markdown.clone()),
            steps,
        }],
        categories: vec![KnotCategorySeed {
            id: "general-knots".to_owned(),
            localizations: vec![
                (Locale::ZhCn, "sheng-jie".to_owned(), "绳结".to_owned()),
                (Locale::En, "knots".to_owned(), "Knots".to_owned()),
            ],
        }],
        types: vec![KnotTypeSeed {
            id: "outdoor-knots".to_owned(),
            localizations: vec![
                (
                    Locale::ZhCn,
                    "hu-wai-sheng-jie".to_owned(),
                    "户外绳结".to_owned(),
                ),
                (
                    Locale::En,
                    "outdoor-knots".to_owned(),
                    "Outdoor Knots".to_owned(),
                ),
            ],
        }],
        media: Vec::new(),
        raw_metadata: serde_json::json!({
            "source": "content/skills",
            "id": skill.id,
        }),
    }
}

fn extract_numbered_steps(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (number, rest) = line.split_once('.')?;
            if number.chars().all(|ch| ch.is_ascii_digit()) {
                let step = rest.trim();
                if !step.is_empty() {
                    return Some(step.to_owned());
                }
            }
            None
        })
        .collect()
}

/// Runs database migrations so the schema reaches the current version before the service starts.
pub async fn migrate_database(db: &DatabaseConnection) -> anyhow::Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
