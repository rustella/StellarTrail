use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use sea_orm_migration::prelude::MigratorTrait;
use serde::{Deserialize, Serialize};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{GearAtlasExternalImportAction, GearAtlasRepository},
};
use stellartrail_domain::{
    gear_atlas::{GearAtlasExternalImportDraft, GearAtlasItem},
    locale::Locale,
};
use stellartrail_importer::{
    GearImportSource, ParsedGearImport, Translator, discover_urls_from_index, parse_import_page,
    read_url_file,
};
use stellartrail_migration::Migrator;
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use tokio::time::sleep;
use url::Url;

const DEFAULT_8264_SCAN_START_ID: u64 = 2_074_200;
const DEFAULT_8264_SCAN_END_ID: u64 = 1;
const DEFAULT_8264_CONSECUTIVE_MISS_LIMIT: usize = 20_000;
const DEFAULT_8264_LIST_MAX_PAGES_PER_SCOPE: u64 = 1_500;
const SOURCE8264_LIST_EMPTY_PAGE_LIMIT: usize = 10;
const SOURCE8264_LIST_STAGNANT_PAGE_LIMIT: usize = 20;
const SOURCE8264_LIST_DISCOVERY_METHOD: &str = "authorized_8264_equipmentlist";
const SOURCE8264_EQUIPMENTLIST_URL: &str = "https://m.8264.com/zhuangbei-equipmentlist.html";
const SOURCE8264_GLOBAL_LIST_ORDERS: [&str; 3] = ["score", "lastpost", "heats"];
const DISCOVERY_STDOUT_SAMPLE_LIMIT: usize = 100;

#[derive(Parser, Debug)]
#[command(name = "import-gear-atlas-all")]
#[command(about = "Conservative multi-source gear atlas importer")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Subcommand, Debug)]
enum CommandKind {
    /// Probe source robots/index reachability without importing.
    Probe(SourceOptions),
    /// Discover candidate product URLs from approved indexes and sitemaps.
    Discover(SourceOptions),
    /// Fetch explicit URLs and print import drafts without writing the database.
    DryRun(ImportOptions),
    /// Fetch explicit URLs and write pending atlas submissions into a test database.
    Write(WriteOptions),
    /// Backfill generated display localizations for pending external imports in a test database.
    BackfillLocalizations(BackfillLocalizationOptions),
}

#[derive(Args, Clone, Debug)]
struct SourceOptions {
    /// Source to include. Repeat to select multiple; defaults to all known sources.
    #[arg(long = "source", value_enum)]
    sources: Vec<CliSource>,
    /// Enable source-specific full discovery instead of a small probe sample.
    #[arg(long)]
    full_scan: bool,
    /// Explicitly ignore robots for an authorized source. Currently only 8264 is allowed.
    #[arg(long = "ignore-robots", alias = "allow-robots-unreadable")]
    ignore_robots: Vec<String>,
    /// Maximum URLs to report per source.
    #[arg(long, default_value = "20")]
    max_items_per_source: String,
    /// Delay between network requests.
    #[arg(long, default_value_t = 1500)]
    delay_ms: u64,
    /// JSONL destination for discovered URLs.
    #[arg(long)]
    output_discovery_file: Option<PathBuf>,
    /// Optional JSON resume state file for long-running discovery.
    #[arg(long)]
    resume_state_file: Option<PathBuf>,
    /// Optional JSONL cache of already discovered URLs, used for de-duplication.
    #[arg(long)]
    source_url_cache_file: Option<PathBuf>,
    /// Starting id for authorized 8264 id-range discovery.
    #[arg(long = "8264-scan-start", default_value_t = DEFAULT_8264_SCAN_START_ID)]
    scan_8264_start_id: u64,
    /// Ending id for authorized 8264 id-range discovery.
    #[arg(long = "8264-scan-end", default_value_t = DEFAULT_8264_SCAN_END_ID)]
    scan_8264_end_id: u64,
    /// Stop 8264 id scanning after this many consecutive misses.
    #[arg(
        long = "8264-consecutive-miss-limit",
        default_value_t = DEFAULT_8264_CONSECUTIVE_MISS_LIMIT
    )]
    scan_8264_consecutive_miss_limit: usize,
    /// Only run 8264 equipment-list discovery and skip slow id-range fallback scanning.
    #[arg(long = "8264-skip-id-fallback")]
    skip_8264_id_fallback: bool,
    /// Maximum pages to scan for each 8264 equipment-list scope.
    #[arg(
        long = "8264-list-max-pages-per-scope",
        default_value_t = DEFAULT_8264_LIST_MAX_PAGES_PER_SCOPE
    )]
    list_8264_max_pages_per_scope: u64,
    /// Include 8264 brand-filter scopes in addition to global and category scopes.
    #[arg(long = "8264-include-brand-scopes")]
    include_8264_brand_scopes: bool,
}

#[derive(Args, Clone, Debug)]
struct ImportOptions {
    /// Explicit detail URL to import. Repeat for multiple URLs.
    #[arg(long = "url")]
    urls: Vec<String>,
    /// File containing one explicit URL per line; blank lines and # comments are ignored.
    #[arg(long)]
    input_url_file: Option<PathBuf>,
    /// Discovery JSONL file produced by `discover --output-discovery-file`.
    #[arg(long)]
    from_discovery_file: Option<PathBuf>,
    /// Explicitly ignore robots for an authorized source. Currently only 8264 is allowed.
    #[arg(long = "ignore-robots", alias = "allow-robots-unreadable")]
    ignore_robots: Vec<String>,
    /// Maximum URLs to process per source.
    #[arg(long, default_value = "20")]
    max_items_per_source: String,
    /// Delay between network requests.
    #[arg(long, default_value_t = 1500)]
    delay_ms: u64,
    /// Optional JSONL destination for dry-run previews or write results.
    #[arg(long)]
    output_file: Option<PathBuf>,
    /// Import batch id. Required for write.
    #[arg(long)]
    batch_id: Option<String>,
    /// Existing user id that owns imported pending submissions. Required for write.
    #[arg(long)]
    submitter_user_id: Option<String>,
    /// Database URL. Required for write.
    #[arg(long)]
    database_url: Option<String>,
    /// Translation provider/config label. Required for write.
    #[arg(long)]
    translation_provider: Option<String>,
}

#[derive(Args, Clone, Debug)]
struct WriteOptions {
    #[command(flatten)]
    import: ImportOptions,
    /// Required safety flag; without it the write command exits before DB access.
    #[arg(long)]
    write: bool,
}

#[derive(Args, Clone, Debug)]
struct BackfillLocalizationOptions {
    /// Required safety flag; without it the command exits before DB access.
    #[arg(long)]
    write: bool,
    /// Database URL. Falls back to DATABASE_URL when omitted.
    #[arg(long)]
    database_url: Option<String>,
    /// Translation provider/config label. Required for write.
    #[arg(long)]
    translation_provider: Option<String>,
    /// Maximum pending rows to backfill in one run.
    #[arg(long, default_value_t = 500)]
    limit: u64,
    /// Optional JSONL destination for backfill results.
    #[arg(long)]
    output_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliSource {
    #[value(name = "8264")]
    Source8264,
    PackWizard,
    Trailspace,
    GearAtlas,
    GearKr,
    OutdoorGearReview,
}

impl CliSource {
    fn into_source(self) -> GearImportSource {
        match self {
            Self::Source8264 => GearImportSource::Source8264,
            Self::PackWizard => GearImportSource::PackWizard,
            Self::Trailspace => GearImportSource::Trailspace,
            Self::GearAtlas => GearImportSource::GearAtlas,
            Self::GearKr => GearImportSource::GearKr,
            Self::OutdoorGearReview => GearImportSource::OutdoorGearReview,
        }
    }
}

#[derive(Debug, Serialize)]
struct ProbeReport {
    source: String,
    robots_url: Option<String>,
    status: String,
    warning: Option<String>,
}

#[derive(Debug, Serialize)]
struct DiscoverReport {
    source: String,
    index_url: Option<String>,
    discovered_count: usize,
    written_count: usize,
    duplicate_count: usize,
    sample_urls: Vec<String>,
    warning: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DiscoveredUrl {
    source: String,
    url: String,
    discovery_method: String,
    discovered_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DiscoveryResumeState {
    positions: BTreeMap<String, u64>,
}

#[derive(Debug, Serialize)]
struct DiscoveryWriteSummary {
    written_count: usize,
    duplicate_count: usize,
}

#[derive(Debug)]
struct DiscoveryBatch {
    discovered_count: usize,
    written_count: usize,
    duplicate_count: usize,
    sample_urls: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Source8264ListScopeKind {
    Global,
    Category,
    Brand,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Source8264ListScope {
    kind: Source8264ListScopeKind,
    order: String,
    pcid: Option<String>,
    cid: Option<String>,
    bid: Option<String>,
}

impl Source8264ListScope {
    fn global(order: &str) -> Self {
        Self {
            kind: Source8264ListScopeKind::Global,
            order: order.to_owned(),
            pcid: None,
            cid: None,
            bid: None,
        }
    }

    fn category(order: &str, pcid: String, cid: String) -> Self {
        Self {
            kind: Source8264ListScopeKind::Category,
            order: order.to_owned(),
            pcid: Some(pcid),
            cid: Some(cid),
            bid: None,
        }
    }

    fn brand(order: &str, bid: String) -> Self {
        Self {
            kind: Source8264ListScopeKind::Brand,
            order: order.to_owned(),
            pcid: None,
            cid: None,
            bid: Some(bid),
        }
    }

    fn resume_key(&self) -> String {
        match self.kind {
            Source8264ListScopeKind::Global => {
                format!("8264_list_global:{}", self.order)
            }
            Source8264ListScopeKind::Category => format!(
                "8264_list_category:{}:{}:{}",
                self.order,
                self.pcid.as_deref().unwrap_or_default(),
                self.cid.as_deref().unwrap_or_default()
            ),
            Source8264ListScopeKind::Brand => format!(
                "8264_list_brand:{}:{}",
                self.order,
                self.bid.as_deref().unwrap_or_default()
            ),
        }
    }

    fn page_url(&self, page: u64) -> String {
        format!(
            "{SOURCE8264_EQUIPMENTLIST_URL}?order={}&pcid={}&cid={}&bid={}&min=&max=&page={page}",
            self.order,
            self.pcid.as_deref().unwrap_or_default(),
            self.cid.as_deref().unwrap_or_default(),
            self.bid.as_deref().unwrap_or_default()
        )
    }
}

#[derive(Debug, Serialize)]
struct DryRunItem {
    url: String,
    source: String,
    parsed: Option<ParsedGearImport>,
    draft: Option<GearAtlasExternalImportDraft>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct WriteItem {
    url: String,
    source: String,
    action: String,
    item_id: Option<String>,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct BackfillLocalizationItem {
    item_id: String,
    source_name: Option<String>,
    canonical_name: String,
    display_name: String,
    action: String,
    translation_status: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        CommandKind::Probe(options) => write_json(run_probe(options).await?),
        CommandKind::Discover(options) => write_json(run_discover(options).await?),
        CommandKind::DryRun(options) => write_json(run_dry_run(options).await?),
        CommandKind::Write(options) => write_json(run_write(options).await?),
        CommandKind::BackfillLocalizations(options) => {
            write_json(run_backfill_localizations(options).await?)
        }
    }
}

async fn run_probe(options: SourceOptions) -> Result<Vec<ProbeReport>> {
    let mut reports = Vec::new();
    for source in selected_sources(&options.sources) {
        let Some(robots_url) = robots_url(source) else {
            reports.push(ProbeReport {
                source: source.key().to_owned(),
                robots_url: None,
                status: "skipped".to_owned(),
                warning: Some("source has no configured robots probe".to_owned()),
            });
            continue;
        };
        let result = fetch_url(robots_url);
        let allowed = allows_robots_override(source, &options.ignore_robots);
        match result {
            Ok(body) => reports.push(ProbeReport {
                source: source.key().to_owned(),
                robots_url: Some(robots_url.to_owned()),
                status: "ok".to_owned(),
                warning: crawl_warning(source, &body),
            }),
            Err(error) if source == GearImportSource::Source8264 && allowed => {
                reports.push(ProbeReport {
                    source: source.key().to_owned(),
                    robots_url: Some(robots_url.to_owned()),
                    status: "override".to_owned(),
                    warning: Some(format!(
                        "robots unreadable; continuing only because override was explicit: {error:#}"
                    )),
                });
            }
            Err(error) => reports.push(ProbeReport {
                source: source.key().to_owned(),
                robots_url: Some(robots_url.to_owned()),
                status: "blocked".to_owned(),
                warning: Some(format!("{error:#}")),
            }),
        }
        sleep(Duration::from_millis(options.delay_ms)).await;
    }
    Ok(reports)
}

async fn run_discover(options: SourceOptions) -> Result<Vec<DiscoverReport>> {
    validate_robots_overrides(&options.ignore_robots)?;
    if options.full_scan
        && selected_sources(&options.sources).contains(&GearImportSource::Source8264)
        && !allows_robots_override(GearImportSource::Source8264, &options.ignore_robots)
    {
        bail!("8264 full scan requires explicit authorization flag: pass --ignore-robots 8264");
    }
    let mut resume_state = read_resume_state(options.resume_state_file.as_ref())?;
    let mut seen_urls = read_discovered_url_set(options.source_url_cache_file.as_ref())?;
    if let Some(path) = options.output_discovery_file.as_ref() {
        seen_urls.extend(read_discovered_url_set(Some(path))?);
    }
    let mut reports = Vec::new();
    for source in selected_sources(&options.sources) {
        if source == GearImportSource::Source8264 {
            let batch = discover_8264_urls(&options, &mut resume_state, &mut seen_urls);
            let (batch, warning) = match batch {
                Ok(batch) => (batch, None),
                Err(error) => (
                    DiscoveryBatch {
                        discovered_count: 0,
                        written_count: 0,
                        duplicate_count: 0,
                        sample_urls: Vec::new(),
                    },
                    Some(format!("{error:#}")),
                ),
            };
            reports.push(DiscoverReport {
                source: source.key().to_owned(),
                index_url: discovery_index_url(source).map(str::to_owned),
                discovered_count: batch.discovered_count,
                written_count: batch.written_count,
                duplicate_count: batch.duplicate_count,
                sample_urls: batch.sample_urls,
                warning,
            });
            write_resume_state(options.resume_state_file.as_ref(), &resume_state)?;
            sleep(request_delay(source, options.delay_ms)).await;
            continue;
        }
        let discovered = discover_source_urls(source, &options, &mut resume_state).await;
        let (urls, warning) = match discovered {
            Ok(urls) => (urls, None),
            Err(error) => (Vec::new(), Some(format!("{error:#}"))),
        };
        let discovered_at = now_rfc3339()?;
        let items = urls
            .iter()
            .map(|url| DiscoveredUrl {
                source: source.key().to_owned(),
                url: url.clone(),
                discovery_method: discovery_method(source, options.full_scan).to_owned(),
                discovered_at: discovered_at.clone(),
            })
            .collect::<Vec<_>>();
        let summary = write_discovery_items(
            &items,
            &mut seen_urls,
            options.output_discovery_file.as_ref(),
            options.source_url_cache_file.as_ref(),
        )?;
        reports.push(DiscoverReport {
            source: source.key().to_owned(),
            index_url: discovery_index_url(source).map(str::to_owned),
            discovered_count: urls.len(),
            written_count: summary.written_count,
            duplicate_count: summary.duplicate_count,
            sample_urls: urls
                .into_iter()
                .take(DISCOVERY_STDOUT_SAMPLE_LIMIT)
                .collect(),
            warning,
        });
        write_resume_state(options.resume_state_file.as_ref(), &resume_state)?;
        sleep(request_delay(source, options.delay_ms)).await;
    }
    Ok(reports)
}

async fn run_dry_run(options: ImportOptions) -> Result<Vec<DryRunItem>> {
    validate_robots_overrides(&options.ignore_robots)?;
    let translator = Translator::new(
        options
            .translation_provider
            .clone()
            .unwrap_or_else(|| "dry-run-rule-based".to_owned()),
    )?;
    let urls = collect_urls(&options)?;
    let mut items = Vec::new();
    for url in limited_urls_by_source(urls, parse_item_limit(&options.max_items_per_source)?)? {
        let item = match dry_run_one_url(&url, &translator, &options).await {
            Ok(item) => item,
            Err(error) => DryRunItem {
                source: source_from_url(&url)
                    .map(|source| source.key().to_owned())
                    .unwrap_or_else(|_| "unknown".to_owned()),
                url,
                parsed: None,
                draft: None,
                error: Some(format!("{error:#}")),
            },
        };
        let source = item.source.clone();
        write_jsonl_item(options.output_file.as_ref(), &item)?;
        items.push(item);
        sleep(request_delay(source_from_key(&source), options.delay_ms)).await;
    }
    Ok(items)
}

async fn dry_run_one_url(
    url: &str,
    translator: &Translator,
    options: &ImportOptions,
) -> Result<DryRunItem> {
    let source = source_from_url(url)?;
    ensure_source_allowed(source, &options.ignore_robots)?;
    let body = fetch_source_url(source, url)?;
    let parsed = parse_import_page(url, &body)?;
    let batch_id = options
        .batch_id
        .clone()
        .unwrap_or_else(|| dry_run_batch_id().unwrap_or_else(|_| "dry-run".to_owned()));
    let draft = parsed
        .clone()
        .into_draft("dry-run-submitter", &batch_id, translator)?;
    Ok(DryRunItem {
        url: url.to_owned(),
        source: source.key().to_owned(),
        parsed: Some(parsed),
        draft: Some(draft),
        error: None,
    })
}

async fn run_write(options: WriteOptions) -> Result<Vec<WriteItem>> {
    if !options.write {
        bail!("write command requires explicit --write");
    }
    ensure_test_import_env("write")?;
    let database_url = options
        .import
        .database_url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok());
    let database_url = required_option(&database_url, "--database-url or DATABASE_URL")?;
    let submitter_user_id =
        required_option(&options.import.submitter_user_id, "--submitter-user-id")?;
    let batch_id = required_option(&options.import.batch_id, "--batch-id")?;
    let provider = required_option(
        &options.import.translation_provider,
        "--translation-provider",
    )?;
    let translator = Translator::new(provider)?;
    let config = DatabaseConfig::new(database_url.to_owned()).context("invalid database URL")?;
    let db = connect_database(&config)
        .await
        .context("connect database")?;
    Migrator::up(&db, None).await.context("run migrations")?;
    let repo = GearAtlasRepository::new(db);
    validate_robots_overrides(&options.import.ignore_robots)?;
    let urls = collect_urls(&options.import)?;
    let mut items = Vec::new();
    for url in limited_urls_by_source(
        urls,
        parse_item_limit(&options.import.max_items_per_source)?,
    )? {
        let item = match write_one_url(
            &repo,
            &url,
            submitter_user_id,
            batch_id,
            &translator,
            &options.import.ignore_robots,
        )
        .await
        {
            Ok(item) => item,
            Err(error) => WriteItem {
                source: source_from_url(&url)
                    .map(|source| source.key().to_owned())
                    .unwrap_or_else(|_| "unknown".to_owned()),
                url,
                action: "error".to_owned(),
                item_id: None,
                status: "error".to_owned(),
                error: Some(format!("{error:#}")),
            },
        };
        let source = item.source.clone();
        write_jsonl_item(options.import.output_file.as_ref(), &item)?;
        items.push(item);
        sleep(request_delay(
            source_from_key(&source),
            options.import.delay_ms,
        ))
        .await;
    }
    Ok(items)
}

async fn run_backfill_localizations(
    options: BackfillLocalizationOptions,
) -> Result<Vec<BackfillLocalizationItem>> {
    if !options.write {
        bail!("backfill-localizations command requires explicit --write");
    }
    ensure_test_import_env("backfill-localizations")?;
    let database_url = options
        .database_url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok());
    let database_url = required_option(&database_url, "--database-url or DATABASE_URL")?;
    let provider = required_option(&options.translation_provider, "--translation-provider")?;
    let translator = Translator::new(provider)?;
    let config = DatabaseConfig::new(database_url.to_owned()).context("invalid database URL")?;
    let db = connect_database(&config)
        .await
        .context("connect database")?;
    Migrator::up(&db, None).await.context("run migrations")?;
    let repo = GearAtlasRepository::new(db);
    let candidates = repo
        .list_external_import_localization_backfill_candidates(
            Locale::En,
            Locale::ZhCn,
            options.limit,
        )
        .await
        .context("list localization backfill candidates")?;
    let mut results = Vec::with_capacity(candidates.len());
    for item in candidates {
        let result = backfill_zh_localization(&repo, &translator, &item).await?;
        write_jsonl_item(options.output_file.as_ref(), &result)?;
        results.push(result);
    }
    Ok(results)
}

async fn backfill_zh_localization(
    repo: &GearAtlasRepository,
    translator: &Translator,
    item: &GearAtlasItem,
) -> Result<BackfillLocalizationItem> {
    let localization =
        translator.translate_localization(&item.name, &item.variants, &item.specs, Locale::ZhCn)?;
    repo.upsert_item_localization(&item.id, &localization)
        .await
        .with_context(|| format!("upsert zh-CN localization for {}", item.id))?;
    Ok(BackfillLocalizationItem {
        item_id: item.id.clone(),
        source_name: item.source_name.clone(),
        canonical_name: item.name.clone(),
        display_name: localization.name,
        action: "upserted_zh-CN_localization".to_owned(),
        translation_status: localization.translation_status,
    })
}

async fn write_one_url(
    repo: &GearAtlasRepository,
    url: &str,
    submitter_user_id: &str,
    batch_id: &str,
    translator: &Translator,
    ignore_robots: &[String],
) -> Result<WriteItem> {
    let source = source_from_url(url)?;
    ensure_source_allowed(source, ignore_robots)?;
    let body = fetch_source_url(source, url)?;
    let parsed = parse_import_page(url, &body)?;
    if should_skip_low_confidence_write(&parsed) {
        return Ok(WriteItem {
            url: url.to_owned(),
            source: source.key().to_owned(),
            action: "skipped_low_confidence".to_owned(),
            item_id: None,
            status: "skipped".to_owned(),
            error: None,
        });
    }
    let mut draft = parsed.into_draft(submitter_user_id, batch_id, translator)?;
    draft
        .validate_and_normalize()
        .context("invalid import draft")?;
    let result = repo
        .upsert_external_import(&draft)
        .await
        .context("upsert external import")?;
    Ok(WriteItem {
        url: url.to_owned(),
        source: source.key().to_owned(),
        action: action_name(result.action).to_owned(),
        item_id: Some(result.item.id),
        status: result.item.status.as_str().to_owned(),
        error: None,
    })
}

fn should_skip_low_confidence_write(item: &ParsedGearImport) -> bool {
    let low_signal_source = item.source_key.starts_with("gearatlas:")
        || item.source_key.starts_with("gearkr:")
        || item.source_key.starts_with("outdoorgearreview:");
    low_signal_source
        && item.specs.is_empty()
        && item.weight_g.is_none()
        && item.official_price_cents.is_none()
        && item.source_rating_score.is_none()
        && item.source_rating_count.is_none()
}

fn collect_urls(options: &ImportOptions) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    urls.extend(options.urls.iter().cloned());
    if let Some(path) = options.input_url_file.as_deref() {
        urls.extend(read_url_file(path)?);
    }
    if let Some(path) = options.from_discovery_file.as_deref() {
        urls.extend(read_discovery_file(path)?);
    }
    if urls.is_empty() {
        bail!("dry-run/write require explicit --url, --input-url-file, or --from-discovery-file");
    }
    Ok(urls)
}

fn read_discovery_file(path: &std::path::Path) -> Result<Vec<String>> {
    let file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut urls = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.with_context(|| format!("failed to read {}", path.display()))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            urls.push(trimmed.to_owned());
            continue;
        }
        let item: DiscoveredUrl = serde_json::from_str(trimmed)
            .with_context(|| format!("invalid discovery JSONL row in {}", path.display()))?;
        urls.push(item.url);
    }
    Ok(urls)
}

fn read_discovered_url_set(path: Option<&PathBuf>) -> Result<BTreeSet<String>> {
    let Some(path) = path else {
        return Ok(BTreeSet::new());
    };
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    Ok(read_discovery_file(path)?.into_iter().collect())
}

fn read_resume_state(path: Option<&PathBuf>) -> Result<DiscoveryResumeState> {
    let Some(path) = path else {
        return Ok(DiscoveryResumeState::default());
    };
    if !path.exists() {
        return Ok(DiscoveryResumeState::default());
    }
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("invalid resume state {}", path.display()))
}

fn write_resume_state(path: Option<&PathBuf>, state: &DiscoveryResumeState) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn write_discovery_items(
    items: &[DiscoveredUrl],
    seen_urls: &mut BTreeSet<String>,
    output_path: Option<&PathBuf>,
    cache_path: Option<&PathBuf>,
) -> Result<DiscoveryWriteSummary> {
    let mut written = Vec::new();
    let mut duplicates = 0;
    for item in items {
        if seen_urls.insert(item.url.clone()) {
            written.push(item);
        } else {
            duplicates += 1;
        }
    }
    append_jsonl_items(output_path, &written)?;
    if cache_path != output_path {
        append_jsonl_items(cache_path, &written)?;
    }
    Ok(DiscoveryWriteSummary {
        written_count: written.len(),
        duplicate_count: duplicates,
    })
}

fn append_jsonl_items<T: Serialize>(path: Option<&PathBuf>, items: &[&T]) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    for item in items {
        writeln!(file, "{}", serde_json::to_string(item)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn write_jsonl_item<T: Serialize>(path: Option<&PathBuf>, item: &T) -> Result<()> {
    append_jsonl_items(path, &[item])
}

fn limited_urls_by_source(
    urls: Vec<String>,
    max_items_per_source: Option<usize>,
) -> Result<Vec<String>> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for url in urls {
        let url = normalize_import_url(&url)?;
        if !seen.insert(url.clone()) {
            continue;
        }
        let source = source_from_url(&url)?;
        let count = counts.entry(source.key().to_owned()).or_default();
        if max_items_per_source.is_some_and(|limit| *count >= limit) {
            continue;
        }
        *count += 1;
        selected.push(url);
    }
    Ok(selected)
}

fn normalize_import_url(url: &str) -> Result<String> {
    let mut parsed = Url::parse(url).with_context(|| format!("invalid url: {url}"))?;
    parsed.set_fragment(None);
    Ok(parsed.to_string())
}

fn selected_sources(sources: &[CliSource]) -> Vec<GearImportSource> {
    if sources.is_empty() {
        return vec![
            GearImportSource::Source8264,
            GearImportSource::PackWizard,
            GearImportSource::Trailspace,
            GearImportSource::GearAtlas,
            GearImportSource::GearKr,
            GearImportSource::OutdoorGearReview,
        ];
    }
    sources.iter().map(|source| source.into_source()).collect()
}

async fn discover_source_urls(
    source: GearImportSource,
    options: &SourceOptions,
    resume_state: &mut DiscoveryResumeState,
) -> Result<Vec<String>> {
    let limit = parse_item_limit(&options.max_items_per_source)?;
    match source {
        GearImportSource::Source8264 => bail!("8264 discovery is handled by the streaming scanner"),
        GearImportSource::PackWizard | GearImportSource::GearAtlas => {
            discover_recursive_sitemap_urls(source, options, limit)
        }
        GearImportSource::OutdoorGearReview => discover_outdoor_gear_review_urls(options, limit),
        GearImportSource::Trailspace => discover_trailspace_urls(options, limit),
        GearImportSource::GearKr => discover_gearkr_urls(options, resume_state, limit),
    }
}

fn discover_recursive_sitemap_urls(
    source: GearImportSource,
    options: &SourceOptions,
    limit: Option<usize>,
) -> Result<Vec<String>> {
    let Some(index_url) = discovery_index_url(source) else {
        return Ok(Vec::new());
    };
    let mut urls = BTreeSet::new();
    let mut visited_sitemaps = BTreeSet::new();
    let mut sitemap_queue = vec![index_url.to_owned()];
    while let Some(sitemap_url) = sitemap_queue.pop() {
        if !visited_sitemaps.insert(sitemap_url.clone()) {
            continue;
        }
        let body = fetch_source_url(source, &sitemap_url)?;
        for loc in extract_xml_locs(&body) {
            if looks_like_sitemap_url(&loc) {
                sitemap_queue.push(loc);
            }
        }
        urls.extend(discover_urls_from_index(
            source,
            &body,
            limit.unwrap_or(usize::MAX),
        ));
        if limit.is_some_and(|limit| urls.len() >= limit) {
            break;
        }
        std::thread::sleep(request_delay(source, options.delay_ms));
    }
    Ok(urls.into_iter().take(limit.unwrap_or(usize::MAX)).collect())
}

fn looks_like_sitemap_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.ends_with(".xml") || lower.contains("sitemap")
}

fn discover_trailspace_urls(options: &SourceOptions, limit: Option<usize>) -> Result<Vec<String>> {
    let source = GearImportSource::Trailspace;
    let limit = limit.unwrap_or(usize::MAX);
    let mut product_urls = BTreeSet::new();
    let mut visited_pages = BTreeSet::new();
    let mut pending_pages = vec!["https://www.trailspace.com/gear/".to_owned()];
    while let Some(page_url) = pending_pages.pop() {
        if product_urls.len() >= limit || !visited_pages.insert(page_url.clone()) {
            continue;
        }
        let body = fetch_source_url(source, &page_url)?;
        for link in extract_html_links(&page_url, &body) {
            if !is_allowed_trailspace_gear_url(&link) {
                continue;
            }
            if looks_like_trailspace_product_url(&link) {
                product_urls.insert(link);
                if product_urls.len() >= limit {
                    break;
                }
            } else if !visited_pages.contains(&link) && pending_pages.len() < 2_000 {
                pending_pages.push(link);
            }
        }
        std::thread::sleep(request_delay(source, options.delay_ms));
    }
    Ok(product_urls.into_iter().collect())
}

fn extract_html_links(base_url: &str, body: &str) -> Vec<String> {
    let Ok(base) = Url::parse(base_url) else {
        return Vec::new();
    };
    let mut links = BTreeSet::new();
    let mut rest = body;
    while let Some(index) = rest.find("href=") {
        rest = &rest[index + "href=".len()..];
        let Some(quote) = rest.chars().next() else {
            break;
        };
        if quote != '"' && quote != '\'' {
            continue;
        }
        let after_quote = &rest[quote.len_utf8()..];
        let Some(end) = after_quote.find(quote) else {
            break;
        };
        let href = after_quote[..end].trim();
        if let Ok(mut url) = base.join(href) {
            url.set_fragment(None);
            links.insert(url.to_string());
        }
        rest = &after_quote[end + quote.len_utf8()..];
    }
    links.into_iter().collect()
}

fn is_allowed_trailspace_gear_url(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let Some(host) = parsed
        .host_str()
        .map(|host| host.trim_start_matches("www."))
    else {
        return false;
    };
    if host != "trailspace.com" || parsed.query().is_some() {
        return false;
    }
    let path = parsed.path();
    path.starts_with("/gear/")
        && !path.starts_with("/gear/write-review/")
        && !path.contains("/search/")
        && !path.contains("/out/")
        && !path.contains("/site-utilities/")
}

fn looks_like_trailspace_product_url(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let segments = parsed
        .path()
        .trim_start_matches("/gear/")
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .count();
    segments >= 2
}

fn discover_outdoor_gear_review_urls(
    options: &SourceOptions,
    limit: Option<usize>,
) -> Result<Vec<String>> {
    let source = GearImportSource::OutdoorGearReview;
    let Some(index_url) = discovery_index_url(source) else {
        return Ok(Vec::new());
    };
    let limit = limit.unwrap_or(usize::MAX);
    let mut urls = BTreeSet::new();
    let body = fetch_source_url(source, index_url)?;
    let product_sitemaps = extract_xml_locs(&body)
        .into_iter()
        .filter(|loc| loc.contains("/product-sitemap"))
        .collect::<Vec<_>>();
    for sitemap_url in product_sitemaps {
        let child_body = fetch_source_url(source, &sitemap_url)?;
        urls.extend(discover_urls_from_index(source, &child_body, limit));
        if urls.len() >= limit {
            break;
        }
        std::thread::sleep(request_delay(source, options.delay_ms));
    }
    Ok(urls.into_iter().take(limit).collect())
}

fn discover_gearkr_urls(
    options: &SourceOptions,
    resume_state: &mut DiscoveryResumeState,
    limit: Option<usize>,
) -> Result<Vec<String>> {
    let source = GearImportSource::GearKr;
    let mut urls = BTreeSet::new();
    let mut page = resume_state
        .positions
        .get("gearkr_page")
        .copied()
        .unwrap_or(1)
        .max(1);
    loop {
        if limit.is_some_and(|limit| urls.len() >= limit) {
            break;
        }
        let url = format!("http://gearkr.com/?rest_route=/wp/v2/posts&per_page=100&page={page}");
        let body = match fetch_source_url(source, &url) {
            Ok(body) => body,
            Err(error) if !urls.is_empty() => {
                eprintln!(
                    "stopping GearKr discovery at page {page} after {} collected URLs: {error:#}",
                    urls.len()
                );
                break;
            }
            Err(error) => return Err(error),
        };
        let posts: serde_json::Value = serde_json::from_str(&body)
            .with_context(|| format!("invalid GearKr REST response for page {page}"))?;
        let Some(posts) = posts.as_array() else {
            break;
        };
        if posts.is_empty() {
            break;
        }
        for post in posts {
            if let Some(link) = post.get("link").and_then(|value| value.as_str()) {
                urls.insert(link.to_owned());
                if limit.is_some_and(|limit| urls.len() >= limit) {
                    break;
                }
            }
        }
        page += 1;
        resume_state
            .positions
            .insert("gearkr_page".to_owned(), page);
        std::thread::sleep(request_delay(source, options.delay_ms));
    }
    Ok(urls.into_iter().collect())
}

fn discover_8264_urls(
    options: &SourceOptions,
    resume_state: &mut DiscoveryResumeState,
    seen_urls: &mut BTreeSet<String>,
) -> Result<DiscoveryBatch> {
    if !options.full_scan {
        bail!("8264 discovery requires --full-scan and explicit --ignore-robots 8264");
    }
    ensure_source_allowed(GearImportSource::Source8264, &options.ignore_robots)?;
    let mut batch = DiscoveryBatch {
        discovered_count: 0,
        written_count: 0,
        duplicate_count: 0,
        sample_urls: Vec::new(),
    };
    let limit = parse_item_limit(&options.max_items_per_source)?.unwrap_or(usize::MAX);

    discover_8264_equipmentlist_urls(options, resume_state, seen_urls, &mut batch, limit)?;
    if batch.discovered_count >= limit {
        return Ok(batch);
    }
    if options.skip_8264_id_fallback {
        write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
        return Ok(batch);
    }

    let mut current_id = resume_state
        .positions
        .get("8264_last_id")
        .copied()
        .unwrap_or(options.scan_8264_start_id);
    let end_id = options.scan_8264_end_id;
    let descending = current_id >= end_id;
    let mut consecutive_misses = 0usize;
    while batch.discovered_count < limit {
        if descending && current_id < end_id {
            break;
        }
        if !descending && current_id > end_id {
            break;
        }
        let url = format!("https://m.8264.com/zhuangbei-equipmentDetail-{current_id}-1.html");
        match fetch_source_url(GearImportSource::Source8264, &url) {
            Ok(body) if looks_like_8264_detail(&body, current_id) => {
                let _ = record_discovered_url(
                    &mut batch,
                    seen_urls,
                    GearImportSource::Source8264,
                    url,
                    discovery_method(GearImportSource::Source8264, options.full_scan),
                    options.output_discovery_file.as_ref(),
                    options.source_url_cache_file.as_ref(),
                )?;
                consecutive_misses = 0;
            }
            _ => {
                consecutive_misses += 1;
                if consecutive_misses >= options.scan_8264_consecutive_miss_limit {
                    break;
                }
            }
        }
        resume_state
            .positions
            .insert("8264_last_id".to_owned(), current_id);
        if current_id % 25 == 0 {
            write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
        }
        current_id = if descending {
            current_id.saturating_sub(1)
        } else {
            current_id.saturating_add(1)
        };
        std::thread::sleep(request_delay(
            GearImportSource::Source8264,
            options.delay_ms,
        ));
    }
    write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
    Ok(batch)
}

fn discover_8264_equipmentlist_urls(
    options: &SourceOptions,
    resume_state: &mut DiscoveryResumeState,
    seen_urls: &mut BTreeSet<String>,
    batch: &mut DiscoveryBatch,
    limit: usize,
) -> Result<()> {
    if limit == 0 {
        return Ok(());
    }

    let index_body = fetch_source_url(GearImportSource::Source8264, SOURCE8264_EQUIPMENTLIST_URL)
        .with_context(|| format!("failed to fetch {SOURCE8264_EQUIPMENTLIST_URL}"))?;
    for url in extract_8264_detail_urls(&index_body) {
        if batch.discovered_count >= limit {
            return Ok(());
        }
        let _ = record_discovered_url(
            batch,
            seen_urls,
            GearImportSource::Source8264,
            url,
            SOURCE8264_LIST_DISCOVERY_METHOD,
            options.output_discovery_file.as_ref(),
            options.source_url_cache_file.as_ref(),
        )?;
    }

    let mut scopes = BTreeSet::new();
    for order in SOURCE8264_GLOBAL_LIST_ORDERS {
        scopes.insert(Source8264ListScope::global(order));
    }
    scopes.extend(extract_8264_category_list_scopes(&index_body, "score"));
    if options.include_8264_brand_scopes {
        scopes.extend(extract_8264_brand_list_scopes(&index_body, "score"));
    }

    for scope in scopes {
        if batch.discovered_count >= limit {
            break;
        }
        discover_8264_equipmentlist_scope(options, resume_state, seen_urls, batch, limit, &scope)?;
    }
    write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
    Ok(())
}

fn discover_8264_equipmentlist_scope(
    options: &SourceOptions,
    resume_state: &mut DiscoveryResumeState,
    seen_urls: &mut BTreeSet<String>,
    batch: &mut DiscoveryBatch,
    limit: usize,
    scope: &Source8264ListScope,
) -> Result<()> {
    let resume_key = scope.resume_key();
    let mut page = resume_state
        .positions
        .get(&resume_key)
        .copied()
        .unwrap_or(1);
    let mut empty_pages = 0usize;
    let mut stagnant_pages = 0usize;

    while page <= options.list_8264_max_pages_per_scope && batch.discovered_count < limit {
        let page_url = scope.page_url(page);
        let body = match fetch_source_url(GearImportSource::Source8264, &page_url) {
            Ok(body) => body,
            Err(_) => {
                empty_pages += 1;
                stagnant_pages += 1;
                if empty_pages >= SOURCE8264_LIST_EMPTY_PAGE_LIMIT
                    || stagnant_pages >= SOURCE8264_LIST_STAGNANT_PAGE_LIMIT
                {
                    break;
                }
                page += 1;
                resume_state.positions.insert(resume_key.clone(), page);
                std::thread::sleep(request_delay(
                    GearImportSource::Source8264,
                    options.delay_ms,
                ));
                continue;
            }
        };

        let urls = extract_8264_detail_urls(&body);
        if urls.is_empty() {
            empty_pages += 1;
            stagnant_pages += 1;
        } else {
            empty_pages = 0;
            let mut page_new_urls = 0usize;
            for url in urls {
                if batch.discovered_count >= limit {
                    break;
                }
                if record_discovered_url(
                    batch,
                    seen_urls,
                    GearImportSource::Source8264,
                    url,
                    SOURCE8264_LIST_DISCOVERY_METHOD,
                    options.output_discovery_file.as_ref(),
                    options.source_url_cache_file.as_ref(),
                )? {
                    page_new_urls += 1;
                }
            }
            if page_new_urls == 0 {
                stagnant_pages += 1;
            } else {
                stagnant_pages = 0;
            }
        }

        page += 1;
        resume_state.positions.insert(resume_key.clone(), page);
        if page % 10 == 0 {
            write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
        }
        if empty_pages >= SOURCE8264_LIST_EMPTY_PAGE_LIMIT
            || stagnant_pages >= SOURCE8264_LIST_STAGNANT_PAGE_LIMIT
        {
            break;
        }
        std::thread::sleep(request_delay(
            GearImportSource::Source8264,
            options.delay_ms,
        ));
    }
    write_resume_state(options.resume_state_file.as_ref(), resume_state)?;
    Ok(())
}

fn record_discovered_url(
    batch: &mut DiscoveryBatch,
    seen_urls: &mut BTreeSet<String>,
    source: GearImportSource,
    url: String,
    discovery_method: &str,
    output_path: Option<&PathBuf>,
    cache_path: Option<&PathBuf>,
) -> Result<bool> {
    batch.discovered_count += 1;
    if batch.sample_urls.len() < DISCOVERY_STDOUT_SAMPLE_LIMIT {
        batch.sample_urls.push(url.clone());
    }
    let item = DiscoveredUrl {
        source: source.key().to_owned(),
        url,
        discovery_method: discovery_method.to_owned(),
        discovered_at: now_rfc3339()?,
    };
    let summary = write_discovery_items(
        std::slice::from_ref(&item),
        seen_urls,
        output_path,
        cache_path,
    )?;
    batch.written_count += summary.written_count;
    batch.duplicate_count += summary.duplicate_count;
    Ok(summary.written_count > 0)
}

fn source_from_url(url: &str) -> Result<GearImportSource> {
    let parsed = Url::parse(url).with_context(|| format!("invalid url: {url}"))?;
    GearImportSource::from_url(&parsed).with_context(|| format!("unsupported source host: {url}"))
}

fn source_from_key(source_key: &str) -> GearImportSource {
    match source_key {
        "8264" => GearImportSource::Source8264,
        "packwizard" => GearImportSource::PackWizard,
        "trailspace" => GearImportSource::Trailspace,
        "gearatlas" => GearImportSource::GearAtlas,
        "gearkr" => GearImportSource::GearKr,
        "outdoorgearreview" => GearImportSource::OutdoorGearReview,
        _ => GearImportSource::PackWizard,
    }
}

fn ensure_source_allowed(source: GearImportSource, ignore_robots: &[String]) -> Result<()> {
    if source == GearImportSource::Source8264 && !allows_robots_override(source, ignore_robots) {
        bail!("8264 full scan requires explicit authorization flag: pass --ignore-robots 8264");
    }
    Ok(())
}

fn validate_robots_overrides(ignore_robots: &[String]) -> Result<()> {
    for value in ignore_robots {
        if !value.eq_ignore_ascii_case("8264") {
            bail!("--ignore-robots is only supported for 8264 in this importer");
        }
    }
    Ok(())
}

fn allows_robots_override(source: GearImportSource, ignore_robots: &[String]) -> bool {
    let source_key = source.key();
    ignore_robots
        .iter()
        .any(|value| value.eq_ignore_ascii_case(source_key))
}

fn parse_item_limit(value: &str) -> Result<Option<usize>> {
    if value.eq_ignore_ascii_case("unlimited") {
        return Ok(None);
    }
    value
        .parse::<usize>()
        .map(Some)
        .with_context(|| format!("invalid item limit {value:?}"))
}

fn discovery_method(source: GearImportSource, full_scan: bool) -> &'static str {
    match (source, full_scan) {
        (GearImportSource::Source8264, true) => "authorized_8264_full_scan",
        (GearImportSource::Source8264, false) => "authorized_8264_seed_probe",
        (GearImportSource::GearKr, true) => "wp_rest_full_scan",
        (GearImportSource::GearAtlas, true) => "recursive_wp_sitemap",
        (_, true) => "full_sitemap",
        (_, false) => "sitemap_sample",
    }
}

fn extract_xml_locs(body: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<loc>") {
        let after = &rest[start + "<loc>".len()..];
        let Some(end) = after.find("</loc>") else {
            break;
        };
        let value = after[..end].trim();
        if !value.is_empty() {
            values.push(value.to_owned());
        }
        rest = &after[end + "</loc>".len()..];
    }
    values
}

fn extract_8264_detail_urls(body: &str) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for token in body.split(['"', '\'', '<', '>', ' ', '\n', '\r', '\t']) {
        if let Some(normalized) = normalize_8264_detail_token(token) {
            urls.insert(normalized);
        }
    }
    urls.into_iter().collect()
}

fn normalize_8264_detail_token(token: &str) -> Option<String> {
    let normalized = if token.starts_with("//") {
        format!("https:{token}")
    } else if token.starts_with('/') {
        format!("https://m.8264.com{token}")
    } else if token.starts_with("http://") || token.starts_with("https://") {
        token.to_owned()
    } else if token.starts_with("zhuangbei-") {
        format!("https://m.8264.com/{token}")
    } else {
        return None;
    };
    let id = source_id_from_8264_token(&normalized)?;
    Some(format!(
        "https://m.8264.com/zhuangbei-equipmentDetail-{id}-1.html"
    ))
}

fn source_id_from_8264_token(token: &str) -> Option<String> {
    for marker in ["equipmentDetail-", "zhuangbei-"] {
        let Some(after_marker) = token.split(marker).nth(1) else {
            continue;
        };
        let id: String = after_marker
            .chars()
            .take_while(|value| value.is_ascii_digit())
            .collect();
        if !id.is_empty() {
            return Some(id);
        }
    }
    None
}

fn extract_8264_category_list_scopes(body: &str, order: &str) -> Vec<Source8264ListScope> {
    let mut scopes = BTreeSet::new();
    for href in extract_html_href_values(body) {
        if !href.contains("equipmentlist") {
            continue;
        }
        let href = href.replace("&amp;", "&");
        let Some(cid) = extract_query_value(&href, "cid").filter(|value| !value.is_empty()) else {
            continue;
        };
        let pcid = extract_query_value(&href, "pcid").unwrap_or_default();
        scopes.insert(Source8264ListScope::category(order, pcid, cid));
    }
    scopes.into_iter().collect()
}

fn extract_8264_brand_list_scopes(body: &str, order: &str) -> Vec<Source8264ListScope> {
    let mut scopes = BTreeSet::new();
    for href in extract_html_href_values(body) {
        if !href.contains("equipmentlist") {
            continue;
        }
        let href = href.replace("&amp;", "&");
        let Some(bid) = extract_query_value(&href, "bid").filter(|value| !value.is_empty()) else {
            continue;
        };
        scopes.insert(Source8264ListScope::brand(order, bid));
    }
    scopes.into_iter().collect()
}

fn extract_html_href_values(body: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("href=") {
        let after_attr = &rest[start + "href=".len()..];
        let mut chars = after_attr.chars();
        let Some(quote) = chars.next() else {
            break;
        };
        if quote != '"' && quote != '\'' {
            rest = after_attr;
            continue;
        }
        let after_quote = &after_attr[quote.len_utf8()..];
        let Some(end) = after_quote.find(quote) else {
            break;
        };
        values.push(after_quote[..end].to_owned());
        rest = &after_quote[end + quote.len_utf8()..];
    }
    values
}

fn extract_query_value(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    for pair in query.split('&') {
        let (pair_key, pair_value) = pair.split_once('=').unwrap_or((pair, ""));
        if pair_key == key {
            return Some(pair_value.to_owned());
        }
    }
    None
}

fn looks_like_8264_detail(body: &str, id: u64) -> bool {
    body.contains(&format!("equipmentDetail-{id}-"))
        && body.contains("8264")
        && (body.contains("装备") || body.contains("点评") || body.contains("参数"))
}

fn request_delay(source: GearImportSource, configured_ms: u64) -> Duration {
    let minimum_ms = if source == GearImportSource::OutdoorGearReview {
        30_000
    } else {
        0
    };
    Duration::from_millis(configured_ms.max(minimum_ms))
}

fn robots_url(source: GearImportSource) -> Option<&'static str> {
    match source {
        GearImportSource::Source8264 => Some("https://m.8264.com/robots.txt"),
        GearImportSource::PackWizard => Some("https://packwizard.com/robots.txt"),
        GearImportSource::Trailspace => Some("https://www.trailspace.com/robots.txt"),
        GearImportSource::GearAtlas => Some("https://gearatlas.com/robots.txt"),
        GearImportSource::GearKr => Some("http://gearkr.com/robots.txt"),
        GearImportSource::OutdoorGearReview => Some("https://www.outdoorgearreview.com/robots.txt"),
    }
}

fn discovery_index_url(source: GearImportSource) -> Option<&'static str> {
    match source {
        GearImportSource::Source8264 => None,
        GearImportSource::PackWizard => Some("https://packwizard.com/sitemap.xml"),
        GearImportSource::Trailspace => Some("https://www.trailspace.com/sitemap.xml"),
        GearImportSource::GearAtlas => Some("https://gearatlas.com/wp-sitemap.xml"),
        GearImportSource::GearKr => Some("http://gearkr.com/?feed=rss2"),
        GearImportSource::OutdoorGearReview => {
            Some("https://www.outdoorgearreview.com/sitemap.xml")
        }
    }
}

fn crawl_warning(source: GearImportSource, robots_body: &str) -> Option<String> {
    if source == GearImportSource::OutdoorGearReview
        && robots_body.to_ascii_lowercase().contains("crawl-delay")
    {
        Some("crawl-delay detected; importer enforces a minimum 30000 ms delay".to_owned())
    } else {
        None
    }
}

fn fetch_source_url(source: GearImportSource, url: &str) -> Result<String> {
    fetch_url_with_agent(url, user_agent_for_source(source))
}

fn fetch_url(url: &str) -> Result<String> {
    fetch_url_with_agent(
        url,
        "StellarTrailImporter/0.1 (+https://github.com/rustella/StellarTrail)",
    )
}

fn fetch_url_with_agent(url: &str, user_agent: &str) -> Result<String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "30", "-A", user_agent, url])
        .output()
        .with_context(|| format!("failed to spawn curl for {url}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("curl failed for {url}: {}", stderr.trim());
    }
    String::from_utf8(output.stdout).with_context(|| format!("non UTF-8 response from {url}"))
}

fn user_agent_for_source(source: GearImportSource) -> &'static str {
    match source {
        GearImportSource::Source8264 => {
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148"
        }
        GearImportSource::GearAtlas => "Mozilla/5.0 StellarTrailImporter/0.1",
        _ => "StellarTrailImporter/0.1 (+https://github.com/rustella/StellarTrail)",
    }
}

fn required_option<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("write requires {name}"))
}

fn ensure_test_import_env(command: &str) -> Result<()> {
    match std::env::var("STELLARTRAIL_IMPORT_ENV") {
        Ok(value) if value == "test" => Ok(()),
        _ => bail!("{command} requires STELLARTRAIL_IMPORT_ENV=test"),
    }
}

fn action_name(action: GearAtlasExternalImportAction) -> &'static str {
    action.as_str()
}

fn dry_run_batch_id() -> Result<String> {
    Ok(format!(
        "dry-run-{}",
        OffsetDateTime::now_utc().format(&Iso8601::DEFAULT)?
    ))
}

fn now_rfc3339() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .context("format timestamp")
}

fn write_json<T: Serialize>(value: T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_8264_equipment_list_short_urls() {
        let html = r#"
            <a href="/zhuangbei-2081055-1.html">OSPREY Kestrel</a>
            <a href="https://bbs.8264.com/zhuangbei-equipmentDetail-2074165-1.html">detail</a>
            <a href="https://m.8264.com/zhuangbei-equipmentlist.html?page=2">list</a>
        "#;

        let urls = extract_8264_detail_urls(html);

        assert_eq!(
            urls,
            vec![
                "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html",
                "https://m.8264.com/zhuangbei-equipmentDetail-2081055-1.html",
            ]
        );
    }

    #[test]
    fn extracts_8264_equipment_list_scopes() {
        let html = r#"
            <a href="https://m.8264.com/index.php?d=forum&amp;c=dianping&amp;m=equipmentlist&amp;act=equipmentlist&amp;order=score&amp;typename=小型背包（30L以下）&amp;pcid=12&amp;cid=146&amp;bid=&amp;page=1">小型背包</a>
            <a href="https://m.8264.com/index.php?d=forum&amp;c=dianping&amp;m=equipmentlist&amp;act=equipmentlist&amp;order=&amp;brand=OSPREY&amp;pcid=&amp;cid=&amp;bid=663&amp;page=1">OSPREY</a>
            <a href="https://m.8264.com/zhuangbei-equipmentlist.html?order=score&amp;page=2">page</a>
        "#;

        let categories = extract_8264_category_list_scopes(html, "score");
        let brands = extract_8264_brand_list_scopes(html, "score");

        assert_eq!(categories.len(), 1);
        assert_eq!(
            categories[0].resume_key(),
            "8264_list_category:score:12:146"
        );
        assert_eq!(brands.len(), 1);
        assert_eq!(brands[0].resume_key(), "8264_list_brand:score:663");
    }

    #[test]
    fn builds_8264_static_equipment_list_urls() {
        let global = Source8264ListScope::global("score");
        let category = Source8264ListScope::category("score", "12".to_owned(), "146".to_owned());
        let brand = Source8264ListScope::brand("score", "663".to_owned());

        assert_eq!(
            global.page_url(2),
            "https://m.8264.com/zhuangbei-equipmentlist.html?order=score&pcid=&cid=&bid=&min=&max=&page=2"
        );
        assert_eq!(
            category.page_url(25),
            "https://m.8264.com/zhuangbei-equipmentlist.html?order=score&pcid=12&cid=146&bid=&min=&max=&page=25"
        );
        assert_eq!(
            brand.page_url(1),
            "https://m.8264.com/zhuangbei-equipmentlist.html?order=score&pcid=&cid=&bid=663&min=&max=&page=1"
        );
    }
}
