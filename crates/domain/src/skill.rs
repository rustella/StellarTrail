use serde::{Deserialize, Serialize};

use crate::mountain::DifficultyLevel;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub title: String,
    pub category: SkillCategory,
    pub difficulty_level: DifficultyLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Knot,
    Camping,
    FirstAid,
    Packing,
    Navigation,
    Weather,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl Locale {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" | "cn" => Some(Self::ZhCn),
            "en" | "en-us" | "en-gb" => Some(Self::En),
            _ => None,
        }
    }

    pub fn fallbacks(self) -> [Self; 2] {
        match self {
            Self::ZhCn => [Self::ZhCn, Self::En],
            Self::En => [Self::En, Self::ZhCn],
        }
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillCategorySummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub item_count: u32,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillCategoriesResponse {
    pub items: Vec<SkillCategorySummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageInfo {
    pub limit: u32,
    pub offset: u32,
    pub next_offset: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotListResponse {
    pub locale: Locale,
    pub items: Vec<KnotSummary>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub difficulty: Option<String>,
    pub categories: Vec<KnotTaxonomyItem>,
    pub types: Vec<KnotTaxonomyItem>,
    pub media: Vec<KnotMediaAsset>,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotDetail {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    pub steps: Vec<String>,
    pub difficulty: Option<String>,
    pub categories: Vec<KnotTaxonomyItem>,
    pub types: Vec<KnotTaxonomyItem>,
    pub media: Vec<KnotMediaAsset>,
    pub locale: Locale,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotTaxonomyItem {
    pub id: String,
    pub slug: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotMediaAsset {
    pub id: String,
    pub media_type: String,
    pub url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
    pub license_note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotSeed {
    pub id: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub source_slug_en: String,
    pub source_slug_zh: Option<String>,
    pub difficulty: Option<String>,
    pub localizations: Vec<KnotLocalizationSeed>,
    pub categories: Vec<KnotCategorySeed>,
    pub types: Vec<KnotTypeSeed>,
    pub media: Vec<KnotMediaAssetSeed>,
    pub raw_metadata: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotLocalizationSeed {
    pub locale: Locale,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    pub steps: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotCategorySeed {
    pub id: String,
    pub localizations: Vec<(Locale, String, String)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotTypeSeed {
    pub id: String,
    pub localizations: Vec<(Locale, String, String)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotMediaAssetSeed {
    pub id: String,
    pub media_type: String,
    pub path: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub attribution: Option<String>,
    pub license_note: Option<String>,
}
