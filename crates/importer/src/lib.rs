//! Conservative multi-source gear atlas importer.
//!
//! The importer only extracts short factual fields from explicitly discovered
//! public pages. It deliberately avoids images, user reviews, long descriptions,
//! affiliate links, and retailer-restricted data.

use std::{collections::BTreeSet, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs, GearVariants},
    gear_atlas::{
        GearAtlasExternalImportDraft, GearAtlasLocalizationDraft, GearAtlasLocalizationTranslator,
    },
    locale::Locale,
};
use url::Url;

/// Supported external sources for the first importer implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GearImportSource {
    Source8264,
    PackWizard,
    Trailspace,
    GearAtlas,
    GearKr,
    OutdoorGearReview,
}

impl GearImportSource {
    pub fn key(self) -> &'static str {
        match self {
            Self::Source8264 => "8264",
            Self::PackWizard => "packwizard",
            Self::Trailspace => "trailspace",
            Self::GearAtlas => "gearatlas",
            Self::GearKr => "gearkr",
            Self::OutdoorGearReview => "outdoorgearreview",
        }
    }

    pub fn source_name(self) -> &'static str {
        match self {
            Self::Source8264 => "8264 户外用品点评",
            Self::PackWizard => "PackWizard",
            Self::Trailspace => "Trailspace",
            Self::GearAtlas => "GearAtlas",
            Self::GearKr => "GearKr 旗客",
            Self::OutdoorGearReview => "OutdoorGearReview",
        }
    }

    pub fn default_locale(self) -> Locale {
        match self {
            Self::Source8264 | Self::GearKr => Locale::ZhCn,
            Self::PackWizard | Self::Trailspace | Self::GearAtlas | Self::OutdoorGearReview => {
                Locale::En
            }
        }
    }

    pub fn from_url(url: &Url) -> Option<Self> {
        let host = url.host_str()?.trim_start_matches("www.");
        if host == "m.8264.com" || host == "bbs.8264.com" || host == "8264.com" {
            Some(Self::Source8264)
        } else if host == "packwizard.com" {
            Some(Self::PackWizard)
        } else if host == "trailspace.com" {
            Some(Self::Trailspace)
        } else if host == "gearatlas.com" {
            Some(Self::GearAtlas)
        } else if host == "gearkr.com" {
            Some(Self::GearKr)
        } else if host == "outdoorgearreview.com" {
            Some(Self::OutdoorGearReview)
        } else {
            None
        }
    }
}

/// Parsed source fact set before repository validation and review insertion.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedGearImport {
    pub source_key: String,
    pub source_name: String,
    pub source_url: String,
    pub source_locale: Locale,
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
    pub canonical_key: String,
    pub detail_score: i32,
    pub warnings: Vec<String>,
}

impl ParsedGearImport {
    pub fn into_draft(
        self,
        submitter_user_id: &str,
        batch_id: &str,
        translator: &Translator,
    ) -> Result<GearAtlasExternalImportDraft> {
        let target_locale = match self.source_locale {
            Locale::ZhCn => Locale::En,
            Locale::En => Locale::ZhCn,
        };
        let translated_localization = translator.translate_localization(
            &self.name,
            &self.variants,
            &self.specs,
            target_locale,
        )?;
        let source_localization = GearAtlasLocalizationDraft {
            locale: self.source_locale,
            name: self.name.clone(),
            description: self.description.clone(),
            variants: self.variants.clone(),
            specs: self.specs.clone(),
            translation_status: Some("source".to_owned()),
            translation_provider: None,
            translated_at: None,
        };
        Ok(GearAtlasExternalImportDraft {
            category: self.category,
            name: self.name,
            brand: self.brand,
            model: self.model,
            description: self.description,
            weight_g: self.weight_g,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_currency,
            variants: self.variants,
            specs: self.specs,
            submitted_by_user_id: submitter_user_id.to_owned(),
            source_key: self.source_key,
            source_name: self.source_name,
            source_url: Some(self.source_url),
            source_license_note: Some(
                "facts and source link only; no images, reviews, or long descriptions".to_owned(),
            ),
            import_batch_id: Some(batch_id.to_owned()),
            source_rating_score: self.source_rating_score,
            source_rating_count: self.source_rating_count,
            canonical_key: Some(self.canonical_key),
            source_locale: Some(self.source_locale),
            detail_score: Some(self.detail_score),
            localizations: vec![source_localization, translated_localization],
        })
    }
}

/// Deterministic translation helper used by the importer boundary.
///
/// The first implementation intentionally keeps translation simple and
/// auditable. Production runs still require an explicit provider flag so a write
/// cannot silently store one-language rows.
#[derive(Clone, Debug)]
pub struct Translator {
    inner: GearAtlasLocalizationTranslator,
}

impl Translator {
    pub fn new(provider: impl Into<String>) -> Result<Self> {
        let Some(inner) = GearAtlasLocalizationTranslator::new(provider) else {
            bail!("translation provider is required for write runs");
        };
        Ok(Self { inner })
    }

    /// Builds a generated display localization for an existing canonical item.
    ///
    /// Backfill callers use this to add a target-locale row without mutating the
    /// source row. The description is generated rather than copied from any third
    /// party source.
    pub fn translate_localization(
        &self,
        name: &str,
        variants: &GearVariants,
        specs: &GearSpecs,
        locale: Locale,
    ) -> Result<GearAtlasLocalizationDraft> {
        Ok(self
            .inner
            .translate_localization(name, variants, specs, locale))
    }
}

/// Parses one fetched page into a conservative gear import fact set.
pub fn parse_import_page(url: &str, html: &str) -> Result<ParsedGearImport> {
    let parsed_url = Url::parse(url).with_context(|| format!("invalid source url: {url}"))?;
    let source = GearImportSource::from_url(&parsed_url)
        .with_context(|| format!("unsupported source host: {url}"))?;
    match source {
        GearImportSource::Source8264 => parse_8264(&parsed_url, html),
        GearImportSource::PackWizard => parse_packwizard(&parsed_url, html),
        GearImportSource::Trailspace => parse_english_page(&parsed_url, html, source),
        GearImportSource::GearAtlas => parse_english_page(&parsed_url, html, source),
        GearImportSource::GearKr => parse_gearkr(&parsed_url, html),
        GearImportSource::OutdoorGearReview => parse_english_page(&parsed_url, html, source),
    }
}

/// Reads explicit URL files with comments and blank lines.
pub fn read_url_file(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read URL file {}", path.display()))?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect())
}

/// Extracts sitemap or feed URLs without trying to crawl disallowed paths.
pub fn discover_urls_from_index(source: GearImportSource, body: &str, limit: usize) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for loc in extract_xml_values(body, "loc")
        .into_iter()
        .chain(extract_xml_values(body, "link"))
    {
        let keep = match source {
            GearImportSource::PackWizard => {
                loc.contains("/gear/tent/")
                    || loc.contains("/gear/backpacks/")
                    || loc.contains("/gear/sleeping-bags/")
                    || loc.contains("/gear/sleeping-pads/")
                    || loc.contains("/gear/insulated-jackets/")
            }
            GearImportSource::Trailspace => loc.contains("/gear/") && !loc.contains("/search/"),
            GearImportSource::GearAtlas => loc.contains("gearatlas.com/"),
            GearImportSource::GearKr => loc.contains("gearkr.com/"),
            GearImportSource::OutdoorGearReview => loc.contains("outdoorgearreview.com/"),
            GearImportSource::Source8264 => false,
        };
        if keep {
            urls.insert(loc);
        }
        if urls.len() >= limit {
            break;
        }
    }
    urls.into_iter().collect()
}

fn parse_8264(url: &Url, html: &str) -> Result<ParsedGearImport> {
    let text = html_to_text(html);
    let name = clean_title(
        &extract_tag_text(html, "title")
            .or_else(|| text_after_class(html, "namebox"))
            .unwrap_or_else(|| "8264 imported gear".to_owned()),
    )
    .replace(" - 8264手机触屏版", "");
    let category = map_category(&text);
    let specs = extract_chinese_specs(&text, category);
    let price = extract_price_cents(&text, '¥').or_else(|| extract_price_cents(&text, '￥'));
    let rating_score = text_after_class(html, "starvalue").and_then(|value| parse_f64(&value));
    let rating_count = text_after_class(html, "dpnum").and_then(|value| parse_first_i32(&value));
    let source_key = format!("8264:{}", source_id_from_8264(url));
    Ok(build_import(
        url,
        GearImportSource::Source8264,
        source_key,
        Locale::ZhCn,
        category,
        name,
        None,
        None,
        Some("来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。".to_owned()),
        None,
        price.map(|price| (price, "CNY".to_owned())),
        GearVariants::new(),
        specs,
        rating_score,
        rating_count,
    ))
}

fn parse_packwizard(url: &Url, html: &str) -> Result<ParsedGearImport> {
    let title = meta_content(html, "og:title")
        .or_else(|| extract_tag_text(html, "title"))
        .unwrap_or_else(|| "PackWizard imported gear".to_owned());
    let name = clean_title(&title);
    let text = html_to_text(html);
    let category = map_packwizard_path_category(url.path())
        .unwrap_or_else(|| map_category(&format!("{} {}", url.path(), text)));
    let specs = extract_english_specs(&text, category);
    let weight_g = specs.get("weight").and_then(|value| parse_weight_g(value));
    let price = extract_price_cents(&text, '$').map(|price| (price, "USD".to_owned()));
    let source_key = format!("packwizard:{}", stable_url_id(url));
    Ok(build_import(
        url,
        GearImportSource::PackWizard,
        source_key,
        Locale::En,
        category,
        name,
        None,
        None,
        Some("Pending external-import gear atlas item from public fact fields.".to_owned()),
        weight_g,
        price,
        GearVariants::new(),
        specs
            .into_iter()
            .filter(|(key, _)| key != "weight")
            .collect(),
        None,
        None,
    ))
}

fn parse_english_page(url: &Url, html: &str, source: GearImportSource) -> Result<ParsedGearImport> {
    let title = meta_content(html, "og:title")
        .or_else(|| extract_tag_text(html, "title"))
        .unwrap_or_else(|| format!("{} imported gear", source.source_name()));
    let name = clean_title(&title)
        .replace(" Reviews - Trailspace", "")
        .replace(" - Trailspace", "")
        .replace(" | GearAtlas.com", "");
    let text = html_to_text(html);
    let category = map_category(&format!("{} {}", url.path(), text));
    let specs = extract_english_specs(&text, category);
    let source_key = format!("{}:{}", source.key(), stable_url_id(url));
    Ok(build_import(
        url,
        source,
        source_key,
        source.default_locale(),
        category,
        name,
        None,
        None,
        Some("Pending external-import gear atlas item from public fact fields.".to_owned()),
        specs
            .get("weight")
            .and_then(|value| parse_weight_g(value))
            .or_else(|| parse_weight_g(&text)),
        None,
        GearVariants::new(),
        specs
            .into_iter()
            .filter(|(key, _)| key != "weight")
            .collect(),
        None,
        None,
    ))
}

fn parse_gearkr(url: &Url, html: &str) -> Result<ParsedGearImport> {
    let title =
        extract_tag_text(html, "title").unwrap_or_else(|| "GearKr imported gear".to_owned());
    let name = clean_title(&title)
        .split('|')
        .next()
        .unwrap_or("GearKr imported gear")
        .trim()
        .to_owned();
    let text = html_to_text(html);
    let category = map_category(&text);
    let source_key = format!("gearkr:{}", stable_url_id(url));
    Ok(build_import(
        url,
        GearImportSource::GearKr,
        source_key,
        Locale::ZhCn,
        category,
        name,
        None,
        None,
        Some("来自 GearKr 公开页面的事实字段候选，已保留来源链接供审核。".to_owned()),
        None,
        None,
        GearVariants::new(),
        extract_chinese_specs(&text, category),
        None,
        None,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_import(
    url: &Url,
    source: GearImportSource,
    source_key: String,
    locale: Locale,
    category: GearCategory,
    name: String,
    brand: Option<String>,
    model: Option<String>,
    description: Option<String>,
    weight_g: Option<i32>,
    price: Option<(i64, String)>,
    variants: GearVariants,
    specs: GearSpecs,
    source_rating_score: Option<f64>,
    source_rating_count: Option<i32>,
) -> ParsedGearImport {
    let (official_price_cents, official_price_currency) = price
        .map(|(cents, currency)| (Some(cents), Some(currency)))
        .unwrap_or((None, None));
    let canonical_key = canonical_key(
        category,
        brand.as_deref(),
        model.as_deref(),
        &name,
        &specs,
        weight_g,
    );
    let detail_score = detail_score(&ParsedGearImport {
        source_key: source_key.clone(),
        source_name: source.source_name().to_owned(),
        source_url: url.as_str().to_owned(),
        source_locale: locale,
        category,
        name: name.clone(),
        brand: brand.clone(),
        model: model.clone(),
        description: description.clone(),
        weight_g,
        official_price_cents,
        official_price_currency: official_price_currency.clone(),
        variants: variants.clone(),
        specs: specs.clone(),
        source_rating_score,
        source_rating_count,
        canonical_key: canonical_key.clone(),
        detail_score: 0,
        warnings: Vec::new(),
    });
    ParsedGearImport {
        source_key,
        source_name: source.source_name().to_owned(),
        source_url: url.as_str().to_owned(),
        source_locale: locale,
        category,
        name,
        brand,
        model,
        description,
        weight_g,
        official_price_cents,
        official_price_currency,
        variants,
        specs,
        source_rating_score,
        source_rating_count,
        canonical_key,
        detail_score,
        warnings: Vec::new(),
    }
}

fn canonical_key(
    category: GearCategory,
    brand: Option<&str>,
    model: Option<&str>,
    name: &str,
    specs: &GearSpecs,
    weight_g: Option<i32>,
) -> String {
    let mut input = format!(
        "{}|{}|{}|{}|{}",
        category.as_str(),
        canonical_part(brand),
        canonical_part(model),
        canonical_part(Some(name)),
        weight_g.unwrap_or_default()
    );
    for key in ["capacity", "people_count", "temperature_or_r_value", "type"] {
        if let Some(value) = specs.get(key) {
            input.push('|');
            input.push_str(key);
            input.push('=');
            input.push_str(&canonical_part(Some(value)));
        }
    }
    let digest = Sha256::digest(input.as_bytes());
    format!("external-gear:v1:{}", hex::encode(&digest[..16]))
}

fn canonical_part(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn detail_score(item: &ParsedGearImport) -> i32 {
    10 + item.brand.as_ref().map(|_| 8).unwrap_or_default()
        + item.model.as_ref().map(|_| 8).unwrap_or_default()
        + item.description.as_ref().map(|_| 3).unwrap_or_default()
        + item.weight_g.map(|_| 10).unwrap_or_default()
        + item.official_price_cents.map(|_| 6).unwrap_or_default()
        + item.source_rating_score.map(|_| 4).unwrap_or_default()
        + item.source_rating_count.map(|_| 4).unwrap_or_default()
        + (item.specs.len() as i32 * 5)
        + (item.variants.len() as i32 * 3)
}

fn map_category(text: &str) -> GearCategory {
    let lower = text.to_lowercase();
    if contains_any(&lower, &["背包", "腰包", "防水袋", "backpack", "pack"]) {
        GearCategory::BackpackSystem
    } else if contains_any(
        &lower,
        &[
            "帐篷",
            "睡袋",
            "防潮垫",
            "垫子",
            "tent",
            "sleeping bag",
            "sleeping pad",
        ],
    ) {
        GearCategory::SleepSystem
    } else if contains_any(&lower, &["炉具", "餐具", "水壶", "stove", "cook", "bottle"]) {
        GearCategory::KitchenSystem
    } else if contains_any(
        &lower,
        &[
            "登山鞋",
            "徒步鞋",
            "越野跑鞋",
            "登山杖",
            "shoe",
            "boot",
            "trekking pole",
        ],
    ) {
        GearCategory::WalkingSystem
    } else if contains_any(
        &lower,
        &[
            "服装", "帽子", "手套", "雪套", "护膝", "jacket", "shirt", "glove",
        ],
    ) {
        GearCategory::ClothingSystem
    } else if contains_any(
        &lower,
        &["头灯", "手电", "headlamp", "flashlight", "lantern"],
    ) {
        GearCategory::LightingSystem
    } else if contains_any(
        &lower,
        &["gps", "户外手表", "watch", "battery", "power bank"],
    ) {
        GearCategory::ElectronicsSystem
    } else if contains_any(
        &lower,
        &[
            "冰镐",
            "冰爪",
            "安全带",
            "头盔",
            "绳索",
            "helmet",
            "rope",
            "harness",
        ],
    ) {
        GearCategory::TechnicalGear
    } else {
        GearCategory::OtherGear
    }
}

fn map_packwizard_path_category(path: &str) -> Option<GearCategory> {
    let lower = path.to_lowercase();
    if lower.contains("/gear/tent")
        || lower.contains("/gear/sleeping-bag")
        || lower.contains("/gear/sleeping-pad")
    {
        Some(GearCategory::SleepSystem)
    } else if lower.contains("/gear/backpack") {
        Some(GearCategory::BackpackSystem)
    } else if lower.contains("/gear/insulated-jacket")
        || lower.contains("/gear/jacket")
        || lower.contains("/gear/shirt")
        || lower.contains("/gear/glove")
    {
        Some(GearCategory::ClothingSystem)
    } else if lower.contains("/gear/stove")
        || lower.contains("/gear/cook")
        || lower.contains("/gear/bottle")
    {
        Some(GearCategory::KitchenSystem)
    } else if lower.contains("/gear/headlamp")
        || lower.contains("/gear/flashlight")
        || lower.contains("/gear/lantern")
    {
        Some(GearCategory::LightingSystem)
    } else if lower.contains("/gear/shoe")
        || lower.contains("/gear/boot")
        || lower.contains("/gear/trekking-pole")
    {
        Some(GearCategory::WalkingSystem)
    } else {
        None
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn extract_chinese_specs(text: &str, category: GearCategory) -> GearSpecs {
    let mut specs = GearSpecs::new();
    if matches!(
        category,
        GearCategory::BackpackSystem | GearCategory::OtherGear
    ) && let Some(value) =
        value_after_labels(text, &["容量:", "容量："]).or_else(|| capacity_from_text(text))
    {
        specs.insert("capacity".to_owned(), value);
    }
    if matches!(category, GearCategory::SleepSystem)
        && let Some(value) = value_after_labels(text, &["适用人数:", "适用人数："])
    {
        specs.insert("people_count".to_owned(), value);
    }
    if matches!(category, GearCategory::SleepSystem)
        && let Some(value) = value_after_labels(text, &["填充物:", "填充物："])
    {
        specs.insert("filling".to_owned(), value);
    }
    specs
}

fn extract_english_specs(text: &str, category: GearCategory) -> GearSpecs {
    let mut specs = GearSpecs::new();
    if matches!(
        category,
        GearCategory::BackpackSystem | GearCategory::OtherGear
    ) && let Some(value) = value_after_labels(text, &["Capacity:", "capacity:"])
    {
        specs.insert("capacity".to_owned(), value);
    }
    if matches!(category, GearCategory::SleepSystem)
        && let Some(value) = value_after_labels(text, &["Capacity:", "Sleeps:", "People:"])
    {
        specs.insert("people_count".to_owned(), value);
    }
    if let Some(weight) = value_after_labels(text, &["Weight:", "weight:"]) {
        specs.insert("weight".to_owned(), weight);
    }
    specs
}

fn value_after_labels(text: &str, labels: &[&str]) -> Option<String> {
    for label in labels {
        if let Some(start) = text.find(label) {
            let rest = text[start + label.len()..].trim();
            let value = rest
                .split(['\n', '\r', ';', '；'])
                .next()
                .unwrap_or_default()
                .split("  ")
                .next()
                .unwrap_or_default()
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let value = truncate_at_next_field(&value);
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn truncate_at_next_field(value: &str) -> String {
    let mut end = value.len();
    for marker in [
        " 价格",
        " 推荐",
        " 重量",
        " 亮度",
        " MSRP",
        " Price:",
        " Weight:",
        " Capacity:",
        " Sleeps:",
        " People:",
        " R-value:",
    ] {
        if let Some(index) = value.find(marker)
            && index > 0
        {
            end = end.min(index);
        }
    }
    value[..end].trim().to_owned()
}

fn capacity_from_text(text: &str) -> Option<String> {
    let chars = text.chars().collect::<Vec<_>>();
    for (index, marker) in chars.iter().enumerate() {
        if *marker != '升' && *marker != 'L' {
            continue;
        }
        let mut start = index;
        while start > 0 {
            let ch = chars[start - 1];
            if ch.is_ascii_digit() || ch == '.' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == index {
            continue;
        }
        let number = chars[start..index].iter().collect::<String>();
        let Ok(value) = number.parse::<f64>() else {
            continue;
        };
        if (1.0..=150.0).contains(&value) {
            return Some(format!("{number}L"));
        }
    }
    None
}

fn html_to_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_entity = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                text.push(' ');
            }
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            '&' if !in_tag => {
                in_entity = true;
                text.push(' ');
            }
            ';' if in_entity => in_entity = false,
            _ if in_tag || in_entity => {}
            _ => text.push(ch),
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_tag_text(html: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = html.to_lowercase().find(&open)?;
    let after_open = html[start..].find('>')? + start + 1;
    let end = html[after_open..].to_lowercase().find(&close)? + after_open;
    Some(html_to_text(&html[after_open..end]).trim().to_owned())
}

fn meta_content(html: &str, property: &str) -> Option<String> {
    let needle = format!("property=\"{property}\"");
    let start = html
        .find(&needle)
        .or_else(|| html.find(&format!("name=\"{property}\"")))?;
    let tag_end = html[start..].find('>')? + start;
    let tag = &html[start..tag_end];
    let content_start = tag.find("content=\"")? + "content=\"".len();
    let rest = &tag[content_start..];
    let content_end = rest.find('"')?;
    Some(rest[..content_end].trim().to_owned())
}

fn text_after_class(html: &str, class_name: &str) -> Option<String> {
    let start = html.find(class_name)?;
    let after = html[start..].find('>')? + start + 1;
    let end = html[after..].find('<')? + after;
    let text = html_to_text(&html[after..end]);
    if text.trim().is_empty() {
        None
    } else {
        Some(text.trim().to_owned())
    }
}

fn clean_title(title: &str) -> String {
    title
        .replace("&amp;", "&")
        .replace("&#8211;", "-")
        .split(" | ")
        .next()
        .unwrap_or(title)
        .trim()
        .chars()
        .take(100)
        .collect()
}

fn source_id_from_8264(url: &Url) -> String {
    if let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "tid") {
        return value.into_owned();
    }
    let path = url.path();
    path.split("equipmentDetail-")
        .nth(1)
        .and_then(|rest| rest.split('-').next())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| path.trim_matches('/'))
        .to_owned()
}

fn stable_url_id(url: &Url) -> String {
    let digest = Sha256::digest(url.as_str().as_bytes());
    hex::encode(&digest[..12])
}

fn extract_price_cents(text: &str, marker: char) -> Option<i64> {
    let start = text.find(marker)? + marker.len_utf8();
    let value = text[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    let (yuan, cents) = value.split_once('.').unwrap_or((value.as_str(), "0"));
    let yuan = yuan.parse::<i64>().ok()?;
    let cents = cents.chars().take(2).collect::<String>();
    let cents = format!("{cents:0<2}").parse::<i64>().ok()?;
    Some(yuan * 100 + cents)
}

fn parse_first_i32(text: &str) -> Option<i32> {
    text.chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn parse_f64(text: &str) -> Option<f64> {
    text.trim()
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .parse()
        .ok()
}

fn parse_weight_g(text: &str) -> Option<i32> {
    let lower = text.to_lowercase();
    let weight_pos = lower.find("weight").unwrap_or(0);
    let snippet = &lower[weight_pos..lower.len().min(weight_pos + 120)];
    let number = snippet
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    let value = number.parse::<f64>().ok()?;
    if snippet.contains("kg") {
        Some((value * 1000.0).round() as i32)
    } else if snippet.contains("lb") {
        Some((value * 453.592).round() as i32)
    } else if snippet.contains("oz") {
        Some((value * 28.3495).round() as i32)
    } else if snippet.contains('g') {
        Some(value.round() as i32)
    } else {
        None
    }
}

fn extract_xml_values(body: &str, tag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut rest = body;
    while let Some(start) = rest.find(&open) {
        let after = &rest[start + open.len()..];
        let Some(end) = after.find(&close) else {
            break;
        };
        let value = after[..end].trim();
        if !value.is_empty() {
            values.push(value.to_owned());
        }
        rest = &after[end + close.len()..];
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_file_ignores_comments_and_blanks() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("urls.txt");
        std::fs::write(
            &path,
            "\n# comment\nhttps://example.test/a\n\nhttps://example.test/b\n",
        )
        .expect("write urls");
        let urls = read_url_file(&path).expect("read url file");
        assert_eq!(urls, ["https://example.test/a", "https://example.test/b"]);
    }

    #[test]
    fn sitemap_discovery_filters_packwizard_detail_urls() {
        let urls = discover_urls_from_index(
            GearImportSource::PackWizard,
            "<url><loc>https://packwizard.com/gear</loc></url><url><loc>https://packwizard.com/gear/tent/abc</loc></url>",
            10,
        );
        assert_eq!(urls, ["https://packwizard.com/gear/tent/abc"]);
    }
}
