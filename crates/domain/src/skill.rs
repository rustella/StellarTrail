//! Outdoor skill domain model describing skill summaries and skill categories.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl Locale {
    /// Returns the public BCP-47 locale tag used by API responses and headers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    /// Parses supported locale aliases from headers while rejecting encoded or unknown values.
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" | "cn" => Some(Self::ZhCn),
            "en" | "en-us" | "en-gb" => Some(Self::En),
            _ => None,
        }
    }

    /// Returns locale fallback order for DB localization lookup.
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

/// Outdoor skill category summary returned by `GET /api/skills`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillCategorySummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub item_count: u32,
    pub href: String,
}

/// Outdoor skill categories response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillCategoriesResponse {
    pub items: Vec<SkillCategorySummary>,
}

/// Offset pagination metadata for list APIs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageInfo {
    pub limit: u32,
    pub offset: u32,
    pub next_offset: Option<u32>,
}

/// Knot list response with locale-resolved fields only.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotListResponse {
    pub locale: Locale,
    pub items: Vec<KnotSummary>,
    pub page: PageInfo,
}

/// Locale-resolved filter option for the public knot catalog.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotFilterOption {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub count: u32,
}

/// Public knot filters response with counts across the full catalog.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotFiltersResponse {
    pub locale: Locale,
    pub categories: Vec<KnotFilterOption>,
    pub difficulties: Vec<KnotFilterOption>,
}

/// Knot summary for list cards.
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

/// Knot detail response with metadata and media URLs.
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

/// Locale-resolved category/type metadata for a knot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotTaxonomyItem {
    pub id: String,
    pub slug: String,
    pub title: String,
}

/// Public media metadata. Binary data is stored in object storage and exposed through a configured public URL.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotMediaAsset {
    pub id: String,
    pub media_type: String,
    pub url: String,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size_bytes: i64,
    pub attribution: Option<String>,
    pub license_note: Option<String>,
}

/// Import seed for a knot parsed from Knots3D metadata.
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

/// Import seed for localized knot text.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotLocalizationSeed {
    pub locale: Locale,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    pub steps: Vec<String>,
}

/// Import seed for knot category taxonomy.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotCategorySeed {
    pub id: String,
    pub localizations: Vec<(Locale, String, String)>,
}

/// Import seed for knot type taxonomy.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotTypeSeed {
    pub id: String,
    pub localizations: Vec<(Locale, String, String)>,
}

/// Import seed for media files associated with a knot.
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
