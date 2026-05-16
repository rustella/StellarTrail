//! Content importer crate that reads YAML/Markdown seed content and converts it into the domain content catalog.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use stellartrail_domain::{
    mountain::DifficultyLevel,
    route::RoutePointType,
    skill::{
        KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
        SkillCategory,
    },
};

/// Content catalog aggregate holding mountains, routes, skills, and gear templates loaded from seed files.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ContentCatalog {
    pub mountains: Vec<MountainContent>,
    pub routes: Vec<RouteContent>,
    pub skills: Vec<SkillContent>,
    pub gear_templates: Vec<GearTemplate>,
}

/// Stable data boundary for `MountainContent`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MountainContent {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub province: String,
    pub city: Option<String>,
    pub area: Option<String>,
    pub elevation_m: Option<i32>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub summary: String,
    #[serde(default)]
    pub best_seasons: Vec<String>,
    pub difficulty_level: DifficultyLevel,
    pub status: String,
}

/// Stable data boundary for `RouteContent`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteContent {
    pub id: String,
    pub mountain_id: Option<String>,
    pub title: String,
    pub province: String,
    pub city: Option<String>,
    pub route_type: String,
    pub difficulty_level: DifficultyLevel,
    pub distance_m: Option<i32>,
    pub ascent_m: Option<i32>,
    pub descent_m: Option<i32>,
    pub duration_min: Option<i32>,
    #[serde(default)]
    pub best_seasons: Vec<String>,
    pub summary: String,
    pub transport_info: Option<String>,
    pub permit_info: Option<String>,
    pub risk_summary: Option<String>,
    pub status: String,
    #[serde(default)]
    pub points: Vec<RoutePoint>,
    #[serde(default)]
    pub gear_suggestions: Vec<RouteGearSuggestion>,
    #[serde(default)]
    pub skill_links: Vec<RouteSkillLink>,
}

/// Stable data boundary for `RoutePoint`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoutePoint {
    #[serde(rename = "type")]
    pub point_type: RoutePointType,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

/// Stable data boundary for `RouteGearSuggestion`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteGearSuggestion {
    pub gear_category: String,
    pub gear_name: String,
    pub required_level: String,
    pub reason: Option<String>,
}

/// Stable data boundary for `RouteSkillLink`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteSkillLink {
    pub skill_id: String,
    pub reason: Option<String>,
}

/// Stable data boundary for `SkillContent`, exposed by or reused within this module.
#[derive(Clone, Debug, Serialize)]
pub struct SkillContent {
    pub id: String,
    pub title: String,
    pub category: SkillCategory,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
    pub related_gear_categories: Vec<String>,
    pub body_markdown: String,
}

/// Stable data boundary for `GearTemplate`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub categories: Vec<GearTemplateCategory>,
}

/// Stable data boundary for `GearTemplateCategory`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTemplateCategory {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub items: Vec<String>,
}

/// Stable data boundary for `SkillFrontMatter`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
struct SkillFrontMatter {
    id: String,
    title: String,
    category: SkillCategory,
    difficulty_level: DifficultyLevel,
    summary: String,
    #[serde(default)]
    related_gear_categories: Vec<String>,
}

/// Reads the full content catalog and aggregates mountains, routes, skills, and gear templates.
pub fn read_content_catalog(root: impl AsRef<Path>) -> anyhow::Result<ContentCatalog> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(ContentCatalog::default());
    }

    let mut mountains: Vec<MountainContent> = read_yaml_dir(root.join("mountains"))?;
    mountains.sort_by(|left, right| left.id.cmp(&right.id));

    let mut routes: Vec<RouteContent> = read_yaml_dir(root.join("routes"))?;
    for route in &mut routes {
        route.points.sort_by_key(|point| point.sort_order);
    }
    routes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut skills = read_skill_dir(root.join("skills"))?;
    skills.sort_by(|left, right| left.id.cmp(&right.id));

    let mut gear_templates: Vec<GearTemplate> = read_yaml_dir(root.join("gear-templates"))?;
    gear_templates.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(ContentCatalog {
        mountains,
        routes,
        skills,
        gear_templates,
    })
}

/// Runs the `read yaml file` server-side flow while preserving input validation, error propagation, and state invariants.
pub fn read_yaml_file<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

/// Runs the `read yaml dir` server-side flow while preserving input validation, error propagation, and state invariants.
fn read_yaml_dir<T>(dir: PathBuf) -> anyhow::Result<Vec<T>>
where
    T: DeserializeOwned,
{
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&dir)
        .with_context(|| format!("failed to list {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    paths
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml" | "yml")
            )
        })
        .map(read_yaml_file)
        .collect()
}

/// Runs the `read skill dir` server-side flow while preserving input validation, error propagation, and state invariants.
fn read_skill_dir(dir: PathBuf) -> anyhow::Result<Vec<SkillContent>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_markdown_files(&dir, &mut paths)?;
    paths.sort();
    paths.into_iter().map(read_skill_file).collect()
}

/// Runs the `collect markdown files` server-side flow while preserving input validation, error propagation, and state invariants.
fn collect_markdown_files(dir: &Path, paths: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to list {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_markdown_files(&path, paths)?;
        } else if matches!(path.extension().and_then(|ext| ext.to_str()), Some("md")) {
            paths.push(path);
        }
    }
    Ok(())
}

/// Runs the `read skill file` server-side flow while preserving input validation, error propagation, and state invariants.
fn read_skill_file(path: PathBuf) -> anyhow::Result<SkillContent> {
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let (front_matter, body_markdown) = split_front_matter(&content)
        .with_context(|| format!("failed to parse front matter {}", path.display()))?;
    let front_matter: SkillFrontMatter = serde_yaml::from_str(front_matter)
        .with_context(|| format!("failed to parse skill metadata {}", path.display()))?;
    Ok(SkillContent {
        id: front_matter.id,
        title: front_matter.title,
        category: front_matter.category,
        difficulty_level: front_matter.difficulty_level,
        summary: front_matter.summary,
        related_gear_categories: front_matter.related_gear_categories,
        body_markdown: body_markdown.trim().to_owned(),
    })
}

/// Splits Markdown front matter from the body and returns empty metadata when front matter is absent.
fn split_front_matter(content: &str) -> anyhow::Result<(&str, &str)> {
    let content = content
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow!("missing opening ---"))?;
    let end = content
        .find("\n---")
        .ok_or_else(|| anyhow!("missing closing ---"))?;
    let front_matter = &content[..end];
    let mut body = &content[end + "\n---".len()..];
    if let Some(rest) = body.strip_prefix('\r') {
        body = rest;
    }
    if let Some(rest) = body.strip_prefix('\n') {
        body = rest;
    }
    Ok((front_matter, body))
}

/// Reads Knots3D bilingual metadata JSON from disk and converts it into DB import seeds.
pub fn read_knots3d_metadata(path: impl AsRef<Path>) -> anyhow::Result<Vec<KnotSeed>> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Knots3D metadata {}", path.display()))?;
    parse_knots3d_metadata(&content)
        .with_context(|| format!("failed to parse Knots3D metadata {}", path.display()))
}

/// Parses Knots3D bilingual metadata JSON into internal import seeds.
pub fn parse_knots3d_metadata(raw: &str) -> anyhow::Result<Vec<KnotSeed>> {
    let value: Value = serde_json::from_str(raw).context("metadata must be JSON")?;
    let items = value
        .get("items")
        .and_then(Value::as_array)
        .context("metadata JSON must contain items array")?;

    let mut seeds = Vec::with_capacity(items.len());
    for item in items {
        let id = text(item, "id")
            .or_else(|| text(item, "english_slug"))
            .context("metadata item missing id")?;
        let english_slug = text(item, "english_slug").unwrap_or_else(|| id.clone());
        let zh_slug = text(item, "zh_slug");
        let en_language = item.pointer("/languages/en");
        let zh_language = item.pointer("/languages/zh-CN");

        let english_title = text(item, "english_name")
            .or_else(|| nested_text(en_language, "h1"))
            .or_else(|| nested_text(en_language, "title"))
            .unwrap_or_else(|| english_slug.clone());
        let chinese_title = text(item, "chinese_name")
            .or_else(|| nested_text(zh_language, "h1"))
            .or_else(|| zh_slug.clone())
            .unwrap_or_else(|| english_title.clone());
        let english_summary = text(item, "english_summary")
            .or_else(|| nested_text(en_language, "summary"))
            .or_else(|| {
                nested_text(en_language, "meta_description").and_then(|s| strip_title_prefix(&s))
            })
            .unwrap_or_default();
        let chinese_summary = text(item, "chinese_summary")
            .or_else(|| nested_text(zh_language, "summary"))
            .or_else(|| {
                nested_text(zh_language, "meta_description").and_then(|s| strip_title_prefix(&s))
            })
            .unwrap_or_else(|| english_summary.clone());
        let english_url = text(item, "english_url").or_else(|| nested_text(en_language, "url"));
        let chinese_slug = zh_slug.clone().unwrap_or_else(|| chinese_title.clone());

        seeds.push(KnotSeed {
            id: id.clone(),
            source_name: "Knots 3D".to_owned(),
            source_url: english_url,
            source_slug_en: english_slug.clone(),
            source_slug_zh: zh_slug,
            difficulty: None,
            localizations: vec![
                KnotLocalizationSeed {
                    locale: Locale::En,
                    slug: english_slug,
                    title: english_title,
                    summary: english_summary,
                    description: section_description(en_language),
                    steps: section_steps(en_language),
                },
                KnotLocalizationSeed {
                    locale: Locale::ZhCn,
                    slug: chinese_slug,
                    title: chinese_title,
                    summary: chinese_summary,
                    description: section_description(zh_language),
                    steps: section_steps(zh_language),
                },
            ],
            categories: parse_categories(item),
            types: parse_types(item),
            media: parse_media(item, &id),
            raw_metadata: item.clone(),
        });
    }

    Ok(seeds)
}

fn text(item: &Value, key: &str) -> Option<String> {
    item.get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn nested_text(item: Option<&Value>, key: &str) -> Option<String> {
    item?
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn strip_title_prefix(value: &str) -> Option<String> {
    value
        .split_once(':')
        .map(|(_, rest)| rest.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn section_description(language: Option<&Value>) -> Option<String> {
    let sections = language?.get("sections")?.as_array()?;
    sections
        .iter()
        .filter_map(|section| section.get("text")?.as_str())
        .map(str::trim)
        .find(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn section_steps(language: Option<&Value>) -> Vec<String> {
    language
        .and_then(|value| value.get("sections"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|section| section.get("heading").and_then(Value::as_str))
        .map(str::trim)
        .filter(|heading| !heading.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_categories(item: &Value) -> Vec<KnotCategorySeed> {
    item.get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|category| {
            taxonomy_seed(category).map(|(id, en, zh)| KnotCategorySeed {
                id: id.clone(),
                localizations: vec![(Locale::En, id.clone(), en), (Locale::ZhCn, id, zh)],
            })
        })
        .collect()
}

fn parse_types(item: &Value) -> Vec<KnotTypeSeed> {
    item.get("types")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|knot_type| {
            taxonomy_seed(knot_type).map(|(id, en, zh)| KnotTypeSeed {
                id: id.clone(),
                localizations: vec![(Locale::En, id.clone(), en), (Locale::ZhCn, id, zh)],
            })
        })
        .collect()
}

fn taxonomy_seed(value: &Value) -> Option<(String, String, String)> {
    let id = text(value, "slug")?;
    let en = text(value, "en").unwrap_or_else(|| id.clone());
    let zh = text(value, "zh").unwrap_or_else(|| en.clone());
    Some((id, en, zh))
}

fn parse_media(item: &Value, knot_id: &str) -> Vec<KnotMediaAssetSeed> {
    let Some(media) = item.get("local_media") else {
        return Vec::new();
    };
    let mapping = [
        (
            "local_thumbnail",
            "thumbnail",
            "thumbnail.webp",
            "image/webp",
        ),
        ("local_preview", "preview", "preview.webp", "image/webp"),
        ("local_draw_gif", "draw_gif", "draw.gif", "image/gif"),
        (
            "local_360_gif",
            "turntable_gif",
            "turntable.gif",
            "image/gif",
        ),
        ("local_draw_mp4", "draw_mp4", "draw.mp4", "video/mp4"),
        (
            "local_360_mp4",
            "turntable_mp4",
            "turntable.mp4",
            "video/mp4",
        ),
    ];
    mapping
        .iter()
        .filter_map(|(source_key, media_type, filename, mime_type)| {
            let _source_path = media.get(*source_key)?.as_str()?;
            Some(KnotMediaAssetSeed {
                id: (*media_type).to_owned(),
                media_type: (*media_type).to_owned(),
                path: format!("skills/knots/{knot_id}/{filename}"),
                mime_type: (*mime_type).to_owned(),
                width: None,
                height: None,
                attribution: Some("Knots 3D".to_owned()),
                license_note: Some("Use only after authorization is confirmed.".to_owned()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the `split front matter returns metadata and body` server-side flow while preserving input validation, error propagation, and state invariants.
    #[test]
    fn split_front_matter_returns_metadata_and_body() {
        let (front_matter, body) = split_front_matter("---\nid: demo\n---\n# Demo\n").unwrap();
        assert_eq!(front_matter.trim(), "id: demo");
        assert_eq!(body, "# Demo\n");
    }
}
