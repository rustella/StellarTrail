//! Imports conservative Chinese gear-atlas source records into the review queue.

use std::{env, fs, process::ExitCode, thread, time::Duration};

use sea_orm_migration::prelude::MigratorTrait;
use serde_json::json;
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{GearAtlasExternalImportAction, GearAtlasRepository},
};
use stellartrail_importer::gear_atlas_cn::{
    GearAtlasCnImportArgs, gear_atlas_cn_usage, parse_8264_mobile_gear_page,
    parse_url_file_content, validate_8264_mobile_detail_url,
};
use stellartrail_migration::Migrator;

const USER_AGENT: &str = "StellarTrailGearAtlasPOC/0.1 (+https://github.com/rustella/StellarTrail)";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("import-gear-atlas-cn failed: {error:#}");
            eprintln!("{}", gear_atlas_cn_usage());
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let args = GearAtlasCnImportArgs::parse_from(env::args().skip(1))?;
    let urls = load_urls(&args)?;
    ensure_robots_for_8264(&args, &urls)?;

    let mut records = Vec::with_capacity(urls.len());
    for (index, url) in urls.iter().enumerate() {
        if index > 0 {
            thread::sleep(Duration::from_millis(args.delay_ms));
        }
        let html = curl_get(url, 20)?;
        records.push(parse_8264_mobile_gear_page(&html, url)?);
    }

    if !args.write {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    let submitter_user_id = args
        .submitter_user_id
        .clone()
        .expect("validated by argument parser");
    let db = connect_database(&DatabaseConfig::new(
        args.database_url
            .clone()
            .expect("validated by argument parser"),
    )?)
    .await?;
    Migrator::up(&db, None).await?;
    let repo = GearAtlasRepository::new(db);
    let mut created = 0;
    let mut updated = 0;
    let mut skipped_approved = 0;
    let mut items = Vec::with_capacity(records.len());
    for record in records {
        let source_key = record.source_key.clone();
        let mut draft = record
            .into_external_import_draft(submitter_user_id.clone(), args.import_batch_id.clone());
        draft.validate_and_normalize().map_err(|error| {
            anyhow::anyhow!("invalid source record {source_key}: {:?}", error.fields)
        })?;
        let result = repo.upsert_external_import(&draft).await?;
        match result.action {
            GearAtlasExternalImportAction::Created => created += 1,
            GearAtlasExternalImportAction::Updated => updated += 1,
            GearAtlasExternalImportAction::SkippedApproved => skipped_approved += 1,
        }
        items.push(json!({
            "source_key": source_key,
            "id": result.item.id,
            "action": result.action.as_str(),
            "status": result.item.status.as_str(),
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "created": created,
            "updated": updated,
            "skipped_approved": skipped_approved,
            "items": items,
        }))?
    );
    Ok(())
}

fn load_urls(args: &GearAtlasCnImportArgs) -> anyhow::Result<Vec<String>> {
    let mut urls = args.urls.clone();
    if let Some(path) = &args.input_url_file {
        let raw = fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
        urls.extend(parse_url_file_content(&raw));
    }
    if urls.is_empty() {
        anyhow::bail!("at least one --url or --input-url-file entry is required");
    }
    if urls.len() > args.limit {
        urls.truncate(args.limit);
    }
    urls.into_iter()
        .map(|url| validate_8264_mobile_detail_url(&url))
        .collect()
}

fn ensure_robots_for_8264(args: &GearAtlasCnImportArgs, urls: &[String]) -> anyhow::Result<()> {
    if !urls
        .iter()
        .any(|url| url.starts_with("https://m.8264.com/"))
    {
        return Ok(());
    }
    match curl_get("https://m.8264.com/robots.txt", 15) {
        Ok(robots) => {
            if !robots_allows_path(&robots, "/zhuangbei-equipmentDetail-") {
                anyhow::bail!("m.8264.com robots.txt does not allow the equipment detail path");
            }
            Ok(())
        }
        Err(error) if args.allow_robots_unavailable => {
            eprintln!(
                "warning: robots.txt unavailable for POC import; continuing because --poc-allow-robots-unavailable was set: {error:#}"
            );
            Ok(())
        }
        Err(error) => anyhow::bail!(
            "robots.txt unavailable; rerun only with --poc-allow-robots-unavailable after manual approval: {error:#}"
        ),
    }
}

fn curl_get(url: &str, timeout_seconds: u64) -> anyhow::Result<String> {
    let timeout = timeout_seconds.to_string();
    let output = std::process::Command::new("curl")
        .args([
            "-fL",
            "-A",
            USER_AGENT,
            "-sS",
            "--max-time",
            timeout.as_str(),
            url,
        ])
        .output()
        .map_err(|error| anyhow::anyhow!("failed to run curl for {url}: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "curl failed for {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn robots_allows_path(raw: &str, path: &str) -> bool {
    let mut applies = false;
    let mut allowed = true;
    for line in raw.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("user-agent:") {
            applies = lower
                .split_once(':')
                .map(|(_, value)| value.trim() == "*")
                .unwrap_or(false);
            continue;
        }
        if applies && lower.starts_with("disallow:") {
            let rule = line
                .split_once(':')
                .map(|(_, value)| value.trim())
                .unwrap_or("");
            if !rule.is_empty() && (rule == "/" || path.starts_with(rule)) {
                allowed = false;
            }
        }
        if applies && lower.starts_with("allow:") {
            let rule = line
                .split_once(':')
                .map(|(_, value)| value.trim())
                .unwrap_or("");
            if !rule.is_empty() && path.starts_with(rule) {
                allowed = true;
            }
        }
    }
    allowed
}
