//! Alias-only backfill support for existing Knots3D knot localizations.
//!
//! This module intentionally updates only `knot_localizations.aliases_json`.
//! It never calls the destructive full import path, preserving already-uploaded
//! media resource mappings in production.

use std::{fs, path::PathBuf};

use anyhow::{Context, bail};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait,
    Value as SeaValue,
};
use serde_json::Value;
use stellartrail_db::{DatabaseConfig, connect_database};
use stellartrail_domain::skill::{KnotSeed, Locale};

use crate::{parse_knots3d_item, parse_knots3d_metadata_value};

const DEFAULT_EXPECTED_ITEMS: i64 = 225;
const DEFAULT_EXPECTED_LOCALIZATIONS: i64 = 450;
const DEFAULT_EXPECTED_MEDIA_RESOURCES: i64 = 1350;
const DEFAULT_EXPECTED_KNOT_MEDIA_RESOURCES: i64 = 1350;

/// Source used by the alias-only backfill.
#[derive(Clone, Debug)]
pub enum AliasBackfillSource {
    /// Read one raw Knots3D JSON item per row from `knot_raw_metadata`.
    RawDb,
    /// Read a local metadata JSON document with a top-level `items` array.
    Metadata(PathBuf),
}

/// Expected production row counts used as fail-closed safety guards.
#[derive(Clone, Debug)]
pub struct AliasBackfillExpectations {
    pub items: i64,
    pub localizations: i64,
    pub media_resources: i64,
    pub knot_media_resources: i64,
}

impl Default for AliasBackfillExpectations {
    fn default() -> Self {
        Self {
            items: DEFAULT_EXPECTED_ITEMS,
            localizations: DEFAULT_EXPECTED_LOCALIZATIONS,
            media_resources: DEFAULT_EXPECTED_MEDIA_RESOURCES,
            knot_media_resources: DEFAULT_EXPECTED_KNOT_MEDIA_RESOURCES,
        }
    }
}

/// Runtime options for an alias-only backfill run.
#[derive(Clone, Debug)]
pub struct AliasBackfillOptions {
    pub database_url: String,
    pub source: AliasBackfillSource,
    pub dry_run: bool,
    pub expectations: AliasBackfillExpectations,
}

/// Human-readable report returned by dry-run and write-mode backfill runs.
#[derive(Clone, Debug)]
pub struct AliasBackfillReport {
    pub source: &'static str,
    pub source_items: i64,
    pub localization_rows_seen: i64,
    pub en_alias_knots: i64,
    pub en_alias_rows: i64,
    pub zh_cn_alias_knots: i64,
    pub zh_cn_alias_rows: i64,
    pub total_alias_rows: i64,
    pub dry_run: bool,
    pub would_update_localization_rows: i64,
    pub updated_localization_rows: Option<i64>,
    pub media_resources_before: i64,
    pub media_resources_after: Option<i64>,
    pub knot_media_resources_before: i64,
    pub knot_media_resources_after: Option<i64>,
}

impl AliasBackfillReport {
    /// Formats the report as stable `key=value` lines for shell operators.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("source={}", self.source),
            format!("source_items={}", self.source_items),
            format!("localization_rows_seen={}", self.localization_rows_seen),
            format!("en_alias_knots={}", self.en_alias_knots),
            format!("en_alias_rows={}", self.en_alias_rows),
            format!("zh_cn_alias_knots={}", self.zh_cn_alias_knots),
            format!("zh_cn_alias_rows={}", self.zh_cn_alias_rows),
            format!("total_alias_rows={}", self.total_alias_rows),
            format!("dry_run={}", self.dry_run),
            format!(
                "would_update_localization_rows={}",
                self.would_update_localization_rows
            ),
            format!("media_resources_before={}", self.media_resources_before),
            format!(
                "knot_media_resources_before={}",
                self.knot_media_resources_before
            ),
        ];
        if let Some(value) = self.updated_localization_rows {
            lines.push(format!("updated_localization_rows={value}"));
        }
        if let Some(value) = self.media_resources_after {
            lines.push(format!("media_resources_after={value}"));
        }
        if let Some(value) = self.knot_media_resources_after {
            lines.push(format!("knot_media_resources_after={value}"));
        }
        lines
    }
}

struct AliasUpdate {
    knot_id: String,
    locale: Locale,
    aliases_json: String,
}

/// Runs the alias-only backfill with safety guards and optional dry-run mode.
pub async fn backfill_knot_localization_aliases(
    options: AliasBackfillOptions,
) -> anyhow::Result<AliasBackfillReport> {
    let db = connect_database(&DatabaseConfig::new(options.database_url)?).await?;
    ensure_alias_column(&db).await?;

    let (source, seeds) = load_source(&db, &options.source).await?;
    let backend = db.get_database_backend();
    let source_items = seeds.len() as i64;
    let localization_rows_seen = count_table(&db, backend, "knot_localizations").await?;
    let media_resources_before = count_table(&db, backend, "media_resources").await?;
    let knot_media_resources_before = count_table(&db, backend, "knot_media_resources").await?;

    check_expected_count("source_items", source_items, options.expectations.items)?;
    check_expected_count(
        "localization_rows_seen",
        localization_rows_seen,
        options.expectations.localizations,
    )?;
    check_expected_count(
        "media_resources_before",
        media_resources_before,
        options.expectations.media_resources,
    )?;
    check_expected_count(
        "knot_media_resources_before",
        knot_media_resources_before,
        options.expectations.knot_media_resources,
    )?;

    let (updates, mut report) = build_report(source, &seeds, options.dry_run)?;
    report.source_items = source_items;
    report.localization_rows_seen = localization_rows_seen;
    report.media_resources_before = media_resources_before;
    report.knot_media_resources_before = knot_media_resources_before;

    if options.dry_run {
        return Ok(report);
    }

    let tx = db.begin().await?;
    let mut updated = 0i64;
    for update in &updates {
        let result = tx
            .execute(statement(
                backend,
                "UPDATE knot_localizations SET aliases_json = ? WHERE knot_id = ? AND locale = ?",
                vec![
                    update.aliases_json.clone().into(),
                    update.knot_id.clone().into(),
                    update.locale.as_str().to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() != 1 {
            bail!(
                "expected to update one localization row for knot_id={} locale={}, updated {}",
                update.knot_id,
                update.locale.as_str(),
                result.rows_affected()
            );
        }
        updated += 1;
    }

    let media_resources_after = count_table(&tx, backend, "media_resources").await?;
    let knot_media_resources_after = count_table(&tx, backend, "knot_media_resources").await?;
    if media_resources_after != media_resources_before {
        bail!(
            "media_resources count changed from {media_resources_before} to {media_resources_after}"
        );
    }
    if knot_media_resources_after != knot_media_resources_before {
        bail!(
            "knot_media_resources count changed from {knot_media_resources_before} to {knot_media_resources_after}"
        );
    }

    tx.commit().await?;
    report.updated_localization_rows = Some(updated);
    report.media_resources_after = Some(media_resources_after);
    report.knot_media_resources_after = Some(knot_media_resources_after);
    Ok(report)
}

async fn load_source(
    db: &DatabaseConnection,
    source: &AliasBackfillSource,
) -> anyhow::Result<(&'static str, Vec<KnotSeed>)> {
    match source {
        AliasBackfillSource::RawDb => load_raw_db_source(db).await.map(|seeds| ("raw_db", seeds)),
        AliasBackfillSource::Metadata(path) => {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read metadata {}", path.display()))?;
            let value: Value = serde_json::from_str(&content)
                .with_context(|| format!("metadata {} must be JSON", path.display()))?;
            let seeds = parse_knots3d_metadata_value(&value)
                .with_context(|| format!("failed to parse metadata {}", path.display()))?;
            Ok(("metadata", seeds))
        }
    }
}

async fn load_raw_db_source(db: &DatabaseConnection) -> anyhow::Result<Vec<KnotSeed>> {
    let rows = db
        .query_all(statement(
            db.get_database_backend(),
            "SELECT knot_id, raw_json FROM knot_raw_metadata ORDER BY knot_id",
            vec![],
        ))
        .await?;
    let mut seeds = Vec::with_capacity(rows.len());
    for row in rows {
        let knot_id: String = row.try_get("", "knot_id")?;
        let raw_json: String = row.try_get("", "raw_json")?;
        let value = serde_json::from_str::<Value>(&raw_json)
            .with_context(|| format!("raw_json for {knot_id} must be JSON"))?;
        let mut seed = parse_knots3d_item(&value)
            .with_context(|| format!("failed to parse raw metadata for {knot_id}"))?;
        seed.id = knot_id;
        seeds.push(seed);
    }
    Ok(seeds)
}

fn build_report(
    source: &'static str,
    seeds: &[KnotSeed],
    dry_run: bool,
) -> anyhow::Result<(Vec<AliasUpdate>, AliasBackfillReport)> {
    let mut updates = Vec::new();
    let mut en_alias_knots = 0i64;
    let mut en_alias_rows = 0i64;
    let mut zh_cn_alias_knots = 0i64;
    let mut zh_cn_alias_rows = 0i64;

    for seed in seeds {
        for localization in &seed.localizations {
            if localization.aliases.is_empty() {
                continue;
            }
            match localization.locale {
                Locale::En => {
                    en_alias_knots += 1;
                    en_alias_rows += localization.aliases.len() as i64;
                }
                Locale::ZhCn => {
                    zh_cn_alias_knots += 1;
                    zh_cn_alias_rows += localization.aliases.len() as i64;
                }
            }
            updates.push(AliasUpdate {
                knot_id: seed.id.clone(),
                locale: localization.locale,
                aliases_json: serde_json::to_string(&localization.aliases)?,
            });
        }
    }

    let would_update_localization_rows = updates.len() as i64;
    Ok((
        updates,
        AliasBackfillReport {
            source,
            source_items: seeds.len() as i64,
            localization_rows_seen: 0,
            en_alias_knots,
            en_alias_rows,
            zh_cn_alias_knots,
            zh_cn_alias_rows,
            total_alias_rows: en_alias_rows + zh_cn_alias_rows,
            dry_run,
            would_update_localization_rows,
            updated_localization_rows: None,
            media_resources_before: 0,
            media_resources_after: None,
            knot_media_resources_before: 0,
            knot_media_resources_after: None,
        },
    ))
}

async fn ensure_alias_column(db: &DatabaseConnection) -> anyhow::Result<()> {
    db.query_all(statement(
        db.get_database_backend(),
        "SELECT aliases_json FROM knot_localizations LIMIT 1",
        vec![],
    ))
    .await
    .context("knot_localizations.aliases_json is missing; run migrations before backfill")?;
    Ok(())
}

async fn count_table<C>(
    connection: &C,
    backend: DatabaseBackend,
    table: &str,
) -> anyhow::Result<i64>
where
    C: ConnectionTrait,
{
    let row = connection
        .query_one(statement(
            backend,
            format!("SELECT COUNT(*) AS count FROM {table}"),
            vec![],
        ))
        .await?
        .with_context(|| format!("missing count row for {table}"))?;
    Ok(row.try_get("", "count")?)
}

fn check_expected_count(label: &str, actual: i64, expected: i64) -> anyhow::Result<()> {
    if actual != expected {
        bail!("{label} expected {expected}, got {actual}");
    }
    Ok(())
}

fn statement(backend: DatabaseBackend, sql: impl Into<String>, values: Vec<SeaValue>) -> Statement {
    let sql = sql.into();
    let sql = if matches!(backend, DatabaseBackend::Postgres) {
        postgres_placeholders(&sql)
    } else {
        sql
    };
    Statement::from_sql_and_values(backend, sql, values)
}

fn postgres_placeholders(sql: &str) -> String {
    let mut converted = String::with_capacity(sql.len());
    let mut index = 1;
    let mut in_single_quote = false;
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            converted.push(ch);
            if in_single_quote && chars.peek() == Some(&'\'') {
                converted.push(chars.next().expect("peeked escaped quote"));
            } else {
                in_single_quote = !in_single_quote;
            }
            continue;
        }

        if ch == '?' && !in_single_quote {
            converted.push('$');
            converted.push_str(&index.to_string());
            index += 1;
        } else {
            converted.push(ch);
        }
    }

    converted
}
