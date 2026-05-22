//! Conservative Chinese gear-atlas import helpers.
//!
//! The first supported source is 8264 mobile equipment detail HTML. The parser
//! extracts only fact-like fields that can be reviewed later: name, category,
//! RMB price, structured specs, rating summary, and source link. It deliberately
//! ignores images, introduction body text, and individual review text.

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
    pub brand: Option<String>,
    pub model: Option<String>,
    pub source_category_label: Option<String>,
    pub category: GearCategory,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub specs: GearSpecs,
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
            brand: self.brand,
            model: self.model,
            description: Some(IMPORT_DESCRIPTION_8264.to_owned()),
            weight_g: self.weight_g,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_cents.map(|_| "CNY".to_owned()),
            specs: self.specs,
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
    let namebox = class_text(html, "namebox").context("missing 8264 equipment name")?;
    let name = html_title_text(html)
        .and_then(clean_8264_page_title)
        .filter(|title| title.chars().count() > namebox.chars().count() && title.contains(&namebox))
        .unwrap_or(namebox);
    let rating_score = class_text(html, "starvalue").and_then(|value| value.parse::<f64>().ok());
    let rating_count =
        class_text(html, "dpnum").and_then(|value| parse_count_before(&value, "点评"));
    let facts_segment = class_segment(html, "feleibox", "<div class=\"introduction");
    let primary_facts = facts_segment.as_deref().map(fact_texts).unwrap_or_default();
    let introduction_facts = class_segment(html, "introductioncon", "</div>")
        .as_deref()
        .map(text_lines)
        .unwrap_or_default();
    let facts = dedupe_preserving_order(
        primary_facts
            .iter()
            .cloned()
            .chain(introduction_facts)
            .collect(),
    );
    let source_category_label = primary_facts
        .iter()
        .find(|value| {
            !value.contains('¥')
                && !value.contains('￥')
                && !value.contains(':')
                && !value.contains('：')
        })
        .cloned();
    let category = source_category_label
        .as_deref()
        .map(map_8264_category)
        .unwrap_or(GearCategory::OtherGear);
    let official_price_cents = facts
        .iter()
        .find_map(|value| parse_rmb_price_cents(value))
        .or_else(|| parse_rmb_price_cents(html));
    let labeled_facts = parse_labeled_facts(&facts);
    let (brand, model) = infer_brand_model(&name);
    let weight_g = labeled_facts
        .iter()
        .find_map(|(label, value)| parse_weight_g_for_label(label, value));
    let specs = specs_from_labeled_facts(category, &labeled_facts);
    let canonical_url = canonical_8264_source_url(&source_id);

    Ok(GearAtlasCnSourceRecord {
        source_key: format!("8264:{source_id}"),
        source_name: SOURCE_NAME_8264.to_owned(),
        source_url: canonical_url,
        source_license_note: SOURCE_LICENSE_NOTE_8264.to_owned(),
        name,
        brand,
        model,
        source_category_label,
        category,
        weight_g,
        official_price_cents,
        specs,
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
    } else if contains_any(label, &["登山鞋", "徒步鞋", "越野跑鞋", "登山杖", "徒步杖"])
    {
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

fn html_title_text(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let close = html[start..].find("</title>")? + start;
    cleaned_text(&html[start..close])
}

fn clean_8264_page_title(title: String) -> Option<String> {
    let without_suffix = title
        .strip_suffix("- 8264手机触屏版")
        .or_else(|| title.strip_suffix(" - 8264手机触屏版"))
        .unwrap_or(title.as_str())
        .trim();
    if without_suffix.is_empty() {
        None
    } else {
        Some(without_suffix.to_owned())
    }
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

fn fact_texts(segment: &str) -> Vec<String> {
    let mut values = span_texts(segment);
    values.extend(text_lines(segment));
    dedupe_preserving_order(values)
}

fn text_lines(segment: &str) -> Vec<String> {
    let mut text = String::with_capacity(segment.len());
    let mut in_tag = false;
    for ch in segment.chars() {
        match ch {
            '<' => {
                in_tag = true;
                text.push('\n');
            }
            '>' => {
                in_tag = false;
                text.push('\n');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    decode_html_entities(&text)
        .lines()
        .filter_map(cleaned_text)
        .filter(|line| !line.starts_with("function") && !line.starts_with('$'))
        .collect()
}

fn dedupe_preserving_order(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.iter().any(|existing| existing == &value) {
            deduped.push(value);
        }
    }
    deduped
}

fn parse_labeled_facts(values: &[String]) -> Vec<(String, String)> {
    values
        .iter()
        .filter_map(|value| {
            let (label, raw_value) = value.split_once('：').or_else(|| value.split_once(':'))?;
            let label = label.trim();
            let raw_value = raw_value.trim();
            if label.is_empty() || raw_value.is_empty() {
                return None;
            }
            Some((label.to_owned(), raw_value.to_owned()))
        })
        .collect()
}

fn specs_from_labeled_facts(category: GearCategory, facts: &[(String, String)]) -> GearSpecs {
    let mut specs = GearSpecs::new();
    for (label, value) in facts {
        if let Some(key) = map_8264_spec_key(category, label) {
            insert_spec(&mut specs, key, value);
        }
    }
    specs
}

fn map_8264_spec_key(category: GearCategory, label: &str) -> Option<&'static str> {
    let normalized = label.replace(' ', "");
    match category {
        GearCategory::BackpackSystem => backpack_spec_key(&normalized),
        GearCategory::SleepSystem => sleep_spec_key(&normalized),
        GearCategory::KitchenSystem => kitchen_spec_key(&normalized),
        GearCategory::WalkingSystem => walking_spec_key(&normalized),
        GearCategory::ClothingSystem => clothing_spec_key(&normalized),
        GearCategory::LightingSystem => lighting_spec_key(&normalized),
        GearCategory::ElectronicsSystem => electronics_spec_key(&normalized),
        GearCategory::TechnicalGear => technical_spec_key(&normalized),
        GearCategory::FirstAidSystem => first_aid_spec_key(&normalized),
        GearCategory::Consumable => consumable_spec_key(&normalized),
        GearCategory::OtherGear => other_spec_key(&normalized),
    }
}

fn backpack_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["容量", "升数"]) {
        Some("capacity")
    } else if contains_any(label, &["建议负重", "推荐负重", "负重"]) {
        Some("recommended_load")
    } else if contains_any(label, &["背长"]) {
        Some("back_length")
    } else if contains_any(label, &["尺寸", "规格"]) {
        Some("backpack_size")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["材质", "面料"]) {
        Some("material")
    } else {
        None
    }
}

fn sleep_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["类型", "款式"]) {
        Some("type")
    } else if contains_any(label, &["人数", "适用人数"]) {
        Some("people_count")
    } else if contains_any(label, &["温标", "r值", "R值"]) {
        Some("temperature_or_r_value")
    } else if contains_any(label, &["充绒量"])
        || (contains_any(label, &["填充", "充绒"]) && contains_any(label, &["重量"]))
    {
        Some("fill_weight")
    } else if contains_any(label, &["填充", "蓬松"]) {
        Some("filling")
    } else if contains_any(label, &["收纳", "包装尺寸"]) {
        Some("packed_size")
    } else if contains_any(label, &["尺寸", "规格", "尺码"]) {
        Some("size")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["材质", "面料"]) {
        Some("material")
    } else {
        None
    }
}

fn kitchen_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["燃料"]) {
        Some("fuel_type")
    } else if contains_any(label, &["容量"]) {
        Some("capacity")
    } else if contains_any(label, &["功率", "火力"]) {
        Some("power")
    } else if contains_any(label, &["人数"]) {
        Some("people_count")
    } else if contains_any(label, &["收纳", "尺寸", "规格"]) {
        Some("packed_size")
    } else if contains_any(label, &["材质"]) {
        Some("material")
    } else {
        None
    }
}

fn walking_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["长度", "尺码", "尺寸", "规格"]) {
        Some("size_or_length")
    } else if contains_any(label, &["地形", "路面"]) {
        Some("terrain")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["材质", "面料"]) {
        Some("material")
    } else if contains_any(label, &["支撑", "缓震", "锁定", "锁紧"]) {
        Some("support")
    } else {
        None
    }
}

fn clothing_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["尺码", "尺寸", "规格"]) {
        Some("size")
    } else if contains_any(label, &["层", "类型"]) {
        Some("layer")
    } else if contains_any(label, &["保暖", "温标"]) {
        Some("warmth_rating")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["透气"]) {
        Some("breathability_rating")
    } else if contains_any(label, &["季节"]) {
        Some("season")
    } else if contains_any(label, &["材质", "面料"]) {
        Some("material")
    } else {
        None
    }
}

fn lighting_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["亮度", "流明"]) {
        Some("max_brightness")
    } else if contains_any(label, &["续航", "运行时间", "使用时间"]) {
        Some("runtime")
    } else if contains_any(label, &["电池"]) {
        Some("battery_type")
    } else if contains_any(label, &["充电", "接口"]) {
        Some("charging_port")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["照射距离", "光束距离", "距离"]) {
        Some("beam_distance")
    } else {
        None
    }
}

fn electronics_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["电池容量", "容量"]) {
        Some("battery_capacity")
    } else if contains_any(label, &["额定能量"]) {
        Some("rated_energy")
    } else if contains_any(label, &["输出", "功率"]) {
        Some("output_power")
    } else if contains_any(label, &["接口", "端口"]) {
        Some("ports")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["工作温度", "温度"]) {
        Some("working_temperature")
    } else {
        None
    }
}

fn technical_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["认证"]) {
        Some("certification")
    } else if contains_any(label, &["强度", "拉力", "承重"]) {
        Some("strength")
    } else if contains_any(label, &["长度"]) {
        Some("length")
    } else if contains_any(label, &["材质"]) {
        Some("material")
    } else if contains_any(label, &["规格", "尺寸"]) {
        Some("specification")
    } else {
        None
    }
}

fn first_aid_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["规格", "尺寸"]) {
        Some("kit_size")
    } else if contains_any(label, &["有效期", "保质期"]) {
        Some("expiry_date")
    } else if contains_any(label, &["人数"]) {
        Some("people_count")
    } else if contains_any(label, &["天数"]) {
        Some("days")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_packaging")
    } else {
        None
    }
}

fn consumable_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["类型"]) {
        Some("type")
    } else if contains_any(label, &["净含量", "含量"]) {
        Some("net_content")
    } else if contains_any(label, &["数量"]) {
        Some("quantity")
    } else if contains_any(label, &["有效期", "保质期"]) {
        Some("expiry_date")
    } else if contains_any(label, &["储存", "保存"]) {
        Some("storage_condition")
    } else {
        None
    }
}

fn other_spec_key(label: &str) -> Option<&'static str> {
    if contains_any(label, &["用途", "使用场景"]) {
        Some("use_case")
    } else if contains_any(label, &["规格", "尺寸", "长度"]) {
        Some("specification")
    } else if contains_any(label, &["容量"]) {
        Some("capacity")
    } else if contains_any(label, &["防水"]) {
        Some("waterproof_rating")
    } else if contains_any(label, &["配件"]) {
        Some("accessories")
    } else if contains_any(label, &["材质"]) {
        Some("material")
    } else {
        None
    }
}

fn insert_spec(specs: &mut GearSpecs, key: &str, value: &str) {
    let value = clean_spec_value(key, value);
    let value = value.trim();
    if !value.is_empty() && value.chars().count() <= 100 {
        specs.insert(key.to_owned(), value.to_owned());
    }
}

fn clean_spec_value(key: &str, value: &str) -> String {
    if matches!(
        key,
        "size" | "size_or_length" | "backpack_size" | "specification"
    ) {
        return strip_weight_clause(value);
    }
    value.to_owned()
}

fn strip_weight_clause(value: &str) -> String {
    let Some(index) = ["重量", "总重", "自重"]
        .into_iter()
        .filter_map(|marker| value.find(marker))
        .min()
    else {
        return value.to_owned();
    };
    value[..index]
        .trim()
        .trim_end_matches([':', '：', ';', '；', ',', '，', ' '])
        .to_owned()
}

fn parse_weight_g_for_label(label: &str, value: &str) -> Option<i32> {
    let label = label.replace(' ', "");
    if contains_any(&label, &["重量", "总重", "自重"]) && !contains_any(&label, &["填充", "充绒"])
    {
        return parse_weight_g(value);
    }
    if contains_any(value, &["重量", "总重", "自重"]) {
        return parse_weight_g(value);
    }
    None
}

fn parse_weight_g(value: &str) -> Option<i32> {
    let lower = value.to_ascii_lowercase();
    let (unit_index, multiplier) = [
        ("kg", 1000.0),
        ("千克", 1000.0),
        ("公斤", 1000.0),
        ("g", 1.0),
        ("克", 1.0),
    ]
    .into_iter()
    .filter_map(|(unit, multiplier)| lower.find(unit).map(|index| (index, multiplier)))
    .min_by_key(|(index, _)| *index)?;
    let prefix = &lower[..unit_index];
    let amount = trailing_decimal(prefix)?;
    let grams = amount * multiplier;
    if grams.is_finite() && grams >= 0.0 && grams <= i32::MAX as f64 {
        Some(grams.round() as i32)
    } else {
        None
    }
}

fn trailing_decimal(value: &str) -> Option<f64> {
    let mut chars = value.chars().rev().skip_while(|ch| ch.is_whitespace());
    let mut reversed = String::new();
    for ch in &mut chars {
        if ch.is_ascii_digit() || ch == '.' {
            reversed.push(ch);
        } else if !reversed.is_empty() {
            break;
        }
    }
    if reversed.is_empty() {
        return None;
    }
    let number: String = reversed.chars().rev().collect();
    number.parse().ok()
}

fn infer_brand_model(name: &str) -> (Option<String>, Option<String>) {
    if let Some((brand, model)) = split_parenthesized_brand(name) {
        return (Some(brand), Some(model));
    }
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() < 2 || !looks_like_ascii_brand(parts[0]) {
        return (None, None);
    }
    let mut brand_parts = vec![parts[0]];
    let mut model_start = 1;
    if parts.len() >= 3
        && (is_short_chinese_alias(parts[1]) || is_parenthesized_alias(parts[1]))
        && !looks_like_category_word(parts[1])
    {
        brand_parts.push(parts[1]);
        model_start = 2;
    }
    let model = parts[model_start..].join(" ");
    if model.is_empty() {
        (None, None)
    } else {
        (Some(brand_parts.join(" ")), Some(model))
    }
}

fn split_parenthesized_brand(name: &str) -> Option<(String, String)> {
    let (close, close_len) = name
        .find('）')
        .map(|index| (index, '）'.len_utf8()))
        .or_else(|| name.find(')').map(|index| (index, ')'.len_utf8())))?;
    let brand = name[..close + close_len].trim();
    let model = name[close + close_len..].trim();
    if brand.is_empty()
        || model.is_empty()
        || brand.chars().count() > 80
        || !brand.chars().any(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    Some((brand.to_owned(), model.to_owned()))
}

fn looks_like_ascii_brand(value: &str) -> bool {
    value.chars().any(|ch| ch.is_ascii_alphabetic())
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '\'' | '-' | '_' | '&' | '.'))
}

fn is_short_chinese_alias(value: &str) -> bool {
    value.chars().count() <= 6
        && value
            .chars()
            .all(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

fn is_parenthesized_alias(value: &str) -> bool {
    (value.starts_with('(') && value.ends_with(')'))
        || (value.starts_with('（') && value.ends_with('）'))
}

fn looks_like_category_word(value: &str) -> bool {
    contains_any(
        value,
        &[
            "背包",
            "睡袋",
            "帐篷",
            "登山鞋",
            "徒步鞋",
            "越野跑鞋",
            "登山杖",
            "徒步杖",
            "头灯",
            "手电",
            "炉具",
            "水壶",
        ],
    )
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
