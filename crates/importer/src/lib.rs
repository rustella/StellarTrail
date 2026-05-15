use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use stellartrail_domain::{mountain::DifficultyLevel, route::RoutePointType, skill::SkillCategory};

#[derive(Clone, Debug, Default, Serialize)]
pub struct ContentCatalog {
    pub mountains: Vec<MountainContent>,
    pub routes: Vec<RouteContent>,
    pub skills: Vec<SkillContent>,
    pub gear_templates: Vec<GearTemplate>,
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoutePoint {
    #[serde(rename = "type")]
    pub point_type: RoutePointType,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteGearSuggestion {
    pub gear_category: String,
    pub gear_name: String,
    pub required_level: String,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteSkillLink {
    pub skill_id: String,
    pub reason: Option<String>,
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub categories: Vec<GearTemplateCategory>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTemplateCategory {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub items: Vec<String>,
}

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

pub fn read_yaml_file<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

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

fn read_skill_dir(dir: PathBuf) -> anyhow::Result<Vec<SkillContent>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_markdown_files(&dir, &mut paths)?;
    paths.sort();
    paths.into_iter().map(read_skill_file).collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_front_matter_returns_metadata_and_body() {
        let (front_matter, body) = split_front_matter("---\nid: demo\n---\n# Demo\n").unwrap();
        assert_eq!(front_matter.trim(), "id: demo");
        assert_eq!(body, "# Demo\n");
    }
}
