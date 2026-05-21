//! Conservative Chinese gear-atlas import helpers.
//!
//! The first supported source is 8264 mobile equipment detail HTML. The parser
//! extracts only fact-like fields that can be reviewed later: name, category,
//! RMB price, rating summary, and source link. It deliberately ignores images,
//! introduction body text, and individual review text.

use std::path::PathBuf;

use anyhow::Context;
use serde::Serialize;
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs},
    gear_atlas::GearAtlasExternalImportDraft,
};

const SOURCE_NAME_8264: &str = "8264 户外用品点评";
const SOURCE_LICENSE_NOTE_8264: &str = "Imported from 8264 mobile public equipment detail page; facts and source link only; no images or review text copied.";
const IMPORT_DESCRIPTION_8264: &str =
    "来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。";

/// Parsed source record ready to become a pending atlas submission.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GearAtlasCnSourceRecord {
    pub source_key: String,
    pub source_name: String,
    pub source_url: String,
    pub source_license_note: String,
    pub name: String,
    pub source_category_label: Option<String>,
    pub category: GearCategory,
    pub official_price_cents: Option<i64>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
}

impl GearAtlasCnSourceRecord {
    /// Converts a parsed source record into a DB draft owned by an import user.
    pub fn into_external_import_draft(
        self,
        submitter_user_id: String,
        import_batch_id: Option<String>,
    ) -> GearAtlasExternalImportDraft {
        GearAtlasExternalImportDraft {
            category: self.category,
            name: self.name,
            brand: None,
            model: None,
            description: Some(IMPORT_DESCRIPTION_8264.to_owned()),
            weight_g: None,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_cents.map(|_| "CNY".to_owned()),
            specs: GearSpecs::new(),
            submitted_by_user_id: submitter_user_id,
            source_key: self.source_key,
            source_name: self.source_name,
            source_url: Some(self.source_url),
            source_license_note: Some(self.source_license_note),
            import_batch_id,
            source_rating_score: self.source_rating_score,
            source_rating_count: self.source_rating_count,
        }
    }
}

/// Parsed command-line options for the Chinese gear atlas import CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GearAtlasCnImportArgs {
    pub urls: Vec<String>,
    pub input_url_file: Option<PathBuf>,
    pub database_url: Option<String>,
    pub write: bool,
    pub submitter_user_id: Option<String>,
    pub import_batch_id: Option<String>,
    pub delay_ms: u64,
    pub limit: usize,
    pub allow_robots_unavailable: bool,
}

impl GearAtlasCnImportArgs {
    /// Parses arguments without a program name.
    pub fn parse_from<I, S>(args: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut parsed = Self {
            urls: Vec::new(),
            input_url_file: None,
            database_url: std::env::var("DATABASE_URL").ok(),
            write: false,
            submitter_user_id: None,
            import_batch_id: None,
            delay_ms: 1_500,
            limit: 20,
            allow_robots_unavailable: false,
        };
        let mut args = args.into_iter().map(Into::into);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--url" => parsed.urls.push(next_value(&mut args, "--url")?),
                "--input-url-file" => {
                    parsed.input_url_file =
                        Some(PathBuf::from(next_value(&mut args, "--input-url-file")?));
                }
                "--database-url" => {
                    parsed.database_url = Some(next_value(&mut args, "--database-url")?)
                }
                "--write" => parsed.write = true,
                "--dry-run" => parsed.write = false,
                "--submitter-user-id" => {
                    parsed.submitter_user_id = Some(next_value(&mut args, "--submitter-user-id")?);
                }
                "--import-batch-id" => {
                    parsed.import_batch_id = Some(next_value(&mut args, "--import-batch-id")?);
                }
                "--delay-ms" => {
                    parsed.delay_ms = next_value(&mut args, "--delay-ms")?
                        .parse()
                        .context("--delay-ms must be an integer")?;
                }
                "--limit" => {
                    parsed.limit = next_value(&mut args, "--limit")?
                        .parse()
                        .context("--limit must be an integer")?;
                }
                "--poc-allow-robots-unavailable" => parsed.allow_robots_unavailable = true,
                "--help" | "-h" => anyhow::bail!("{}", gear_atlas_cn_usage()),
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }
        if parsed.limit == 0 || parsed.limit > 20 {
            anyhow::bail!("--limit must be between 1 and 20");
        }
        if parsed.write {
            if parsed.database_url.is_none() {
                anyhow::bail!("--database-url is required when --write is set");
            }
            if parsed.submitter_user_id.is_none() {
                anyhow::bail!("--submitter-user-id is required when --write is set");
            }
        }
        Ok(parsed)
    }
}

/// Returns the CLI usage text.
pub fn gear_atlas_cn_usage() -> &'static str {
    "usage: import-gear-atlas-cn [--url <8264-mobile-detail-url> ...] [--input-url-file <path>] [--dry-run|--write] [--database-url <url>] [--submitter-user-id <user-id>] [--import-batch-id <id>] [--delay-ms <ms>] [--limit <1..20>] [--poc-allow-robots-unavailable]"
}

/// Parses a URL file where empty lines and `#` comments are ignored.
pub fn parse_url_file_content(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

/// Validates a supported 8264 mobile equipment detail URL.
pub fn validate_8264_mobile_detail_url(url: &str) -> anyhow::Result<String> {
    if !(url.starts_with("https://m.8264.com/") || url.starts_with("http://m.8264.com/")) {
        anyhow::bail!("only m.8264.com mobile equipment detail URLs are supported: {url}");
    }
    source_id_from_8264_url(url)
        .map(|id| canonical_8264_source_url(&id))
        .ok_or_else(|| anyhow::anyhow!("unsupported 8264 equipment detail URL: {url}"))
}

/// Extracts conservative public fields from an 8264 mobile detail HTML page.
pub fn parse_8264_mobile_gear_page(
    html: &str,
    source_url: &str,
) -> anyhow::Result<GearAtlasCnSourceRecord> {
    let source_id = source_id_from_8264_url(source_url)
        .or_else(|| hidden_tid(html))
        .ok_or_else(|| anyhow::anyhow!("missing 8264 source id"))?;
    let name = class_text(html, "namebox").context("missing 8264 equipment name")?;
    let rating_score = class_text(html, "starvalue").and_then(|value| value.parse::<f64>().ok());
    let rating_count =
        class_text(html, "dpnum").and_then(|value| parse_count_before(&value, "点评"));
    let facts = class_segment(html, "feleibox", "<div class=\"introduction")
        .map(|segment| span_texts(&segment))
        .unwrap_or_default();
    let source_category_label = facts
        .iter()
        .find(|value| !value.contains('¥') && !value.contains('￥'))
        .cloned();
    let category = source_category_label
        .as_deref()
        .map(map_8264_category)
        .unwrap_or(GearCategory::OtherGear);
    let official_price_cents = facts
        .iter()
        .find_map(|value| parse_rmb_price_cents(value))
        .or_else(|| parse_rmb_price_cents(html));
    let canonical_url = canonical_8264_source_url(&source_id);

    Ok(GearAtlasCnSourceRecord {
        source_key: format!("8264:{source_id}"),
        source_name: SOURCE_NAME_8264.to_owned(),
        source_url: canonical_url,
        source_license_note: SOURCE_LICENSE_NOTE_8264.to_owned(),
        name,
        source_category_label,
        category,
        official_price_cents,
        source_rating_score: rating_score,
        source_rating_count: rating_count,
    })
}

/// Maps first-pass 8264 Chinese category labels into StellarTrail gear categories.
pub fn map_8264_category(label: &str) -> GearCategory {
    if contains_any(label, &["背包", "腰包", "防水袋", "旅行包"]) {
        GearCategory::BackpackSystem
    } else if contains_any(label, &["帐篷", "睡袋", "垫子", "防潮垫"]) {
        GearCategory::SleepSystem
    } else if contains_any(label, &["炉具", "餐具", "水壶", "营地灯"]) {
        GearCategory::KitchenSystem
    } else if contains_any(label, &["登山鞋", "徒步鞋", "越野跑鞋", "登山杖"]) {
        GearCategory::WalkingSystem
    } else if contains_any(label, &["服装", "帽子", "手套", "雪套", "护膝"]) {
        GearCategory::ClothingSystem
    } else if contains_any(label, &["头灯", "手电"]) {
        GearCategory::LightingSystem
    } else if contains_any(label, &["GPS", "户外手表", "电子数码"]) {
        GearCategory::ElectronicsSystem
    } else if contains_any(
        label,
        &["攀登器材", "冰镐", "冰爪", "安全带", "头盔", "绳索"],
    ) {
        GearCategory::TechnicalGear
    } else {
        GearCategory::OtherGear
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> anyhow::Result<String> {
    args.next()
        .ok_or_else(|| anyhow::anyhow!("{flag} requires a value"))
}

fn source_id_from_8264_url(url: &str) -> Option<String> {
    let marker = if url.contains("equipmentDetail-") {
        "equipmentDetail-"
    } else if url.contains("/zhuangbei-") {
        "/zhuangbei-"
    } else {
        return None;
    };
    let rest = url.split_once(marker)?.1;
    let id: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if id.is_empty() || id == "0" {
        None
    } else {
        Some(id)
    }
}

fn hidden_tid(html: &str) -> Option<String> {
    let marker = "name=\"tid\" value=\"";
    let rest = html.split_once(marker)?.1;
    let id: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if id.is_empty() { None } else { Some(id) }
}

fn canonical_8264_source_url(source_id: &str) -> String {
    format!("https://m.8264.com/zhuangbei-equipmentDetail-{source_id}-1.html")
}

fn class_text(html: &str, class_name: &str) -> Option<String> {
    let marker = format!("class=\"{class_name}\"");
    let start = html.find(&marker)?;
    let open_end = html[start..].find('>')? + start + 1;
    let close = html[open_end..].find('<')? + open_end;
    cleaned_text(&html[open_end..close])
}

fn class_segment(html: &str, class_name: &str, end_marker: &str) -> Option<String> {
    let marker = format!("class=\"{class_name}\"");
    let start = html.find(&marker)?;
    let end = html[start..]
        .find(end_marker)
        .map(|offset| start + offset)
        .unwrap_or(html.len());
    Some(html[start..end].to_owned())
}

fn span_texts(segment: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = segment;
    while let Some(start) = rest.find("<span") {
        rest = &rest[start..];
        let Some(open_end) = rest.find('>') else {
            break;
        };
        let after_open = open_end + 1;
        let Some(close) = rest[after_open..].find("</span>") else {
            break;
        };
        if let Some(value) = cleaned_text(&rest[after_open..after_open + close]) {
            values.push(value);
        }
        rest = &rest[after_open + close + "</span>".len()..];
    }
    values
}

fn cleaned_text(raw: &str) -> Option<String> {
    let without_tags = strip_tags(raw);
    let decoded = decode_html_entities(&without_tags);
    let mut normalized = String::new();
    let mut last_space = false;
    for ch in decoded.chars() {
        if ch.is_whitespace() {
            if !last_space {
                normalized.push(' ');
                last_space = true;
            }
        } else {
            normalized.push(ch);
            last_space = false;
        }
    }
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn strip_tags(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn decode_html_entities(raw: &str) -> String {
    raw.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn parse_rmb_price_cents(value: &str) -> Option<i64> {
    let start = value.find('¥').or_else(|| value.find('￥'))?;
    let amount: String = value[start + '¥'.len_utf8()..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == ',')
        .filter(|ch| *ch != ',')
        .collect();
    if amount.is_empty() {
        return None;
    }
    decimal_to_cents(&amount)
}

fn decimal_to_cents(amount: &str) -> Option<i64> {
    let (yuan, cents) = amount.split_once('.').unwrap_or((amount, ""));
    let yuan: i64 = yuan.parse().ok()?;
    let cents = match cents.len() {
        0 => 0,
        1 => cents.parse::<i64>().ok()? * 10,
        _ => cents.get(..2)?.parse::<i64>().ok()?,
    };
    Some(yuan * 100 + cents)
}

fn parse_count_before(value: &str, suffix: &str) -> Option<i32> {
    let prefix = value.split_once(suffix).map(|(prefix, _)| prefix)?;
    let digits: String = prefix.chars().filter(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
