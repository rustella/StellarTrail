//! Gear inventory HTTP DTOs that convert query parameters and request bodies into domain gear drafts and response payloads.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use stellartrail_domain::gear::{
    GearCategory, GearDraft, GearItem, GearShareStatus, GearSort, GearSpecs, GearStatus, GearTab,
};

/// Stable data boundary for `ListGearQuery`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct ListGearQuery {
    #[serde(default)]
    pub tab: GearTab,
    pub category: Option<GearCategory>,
    pub status: Option<GearStatus>,
    pub q: Option<String>,
    #[serde(default)]
    pub sort: GearSort,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Stable data boundary for `GearStatsQuery`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct GearStatsQuery {
    #[serde(default)]
    pub tab: GearTab,
}

/// Query parameters for the per-user spec-field ranking endpoint.
#[derive(Debug, Deserialize)]
pub struct GearSpecKeyRankingsQuery {
    pub category: GearCategory,
}

/// Query parameters for tag suggestions backed by Redis-only user tag history.
#[derive(Debug, Deserialize)]
pub struct GearTagSuggestionsQuery {
    pub limit: Option<usize>,
}

/// Stable data boundary for `GearExportQuery`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct GearExportQuery {
    #[serde(default)]
    pub tab: GearTab,
    #[serde(default = "default_csv_format")]
    pub format: String,
}

/// Runs the `default csv format` server-side flow while preserving input validation, error propagation, and state invariants.
fn default_csv_format() -> String {
    "csv".to_owned()
}

/// Stable data boundary for `CreateGearRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGearRequest {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub purchase_price_currency: Option<String>,
    pub purchase_location: Option<String>,
    pub status: Option<GearStatus>,
    pub storage_location: Option<String>,
    #[serde(default)]
    pub specs: GearSpecs,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tag_colors: Option<HashMap<String, String>>,
    #[serde(default)]
    pub share_enabled: bool,
    pub notes: Option<String>,
}

impl CreateGearRequest {
    /// Runs the `into draft` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn into_draft(self) -> GearDraft {
        GearDraft {
            category: self.category,
            name: self.name,
            brand: self.brand,
            model: self.model,
            description: self.description,
            weight_g: self.weight_g,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_currency,
            purchase_date: self.purchase_date,
            purchase_price_cents: self.purchase_price_cents,
            purchase_price_currency: self.purchase_price_currency,
            purchase_location: self.purchase_location,
            status: self.status.unwrap_or_default(),
            storage_location: self.storage_location,
            specs: self.specs,
            tags: self.tags,
            share_enabled: self.share_enabled,
            share_status: GearShareStatus::NotShared,
            notes: self.notes,
        }
    }
}

/// Stable data boundary for `UpdateGearRequest`, exposed by or reused within this module.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateGearRequest {
    pub category: Option<GearCategory>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub purchase_price_currency: Option<String>,
    pub purchase_location: Option<String>,
    pub status: Option<GearStatus>,
    pub storage_location: Option<String>,
    pub specs: Option<GearSpecs>,
    pub tags: Option<Vec<String>>,
    pub tag_colors: Option<HashMap<String, String>>,
    pub share_enabled: Option<bool>,
    pub notes: Option<String>,
}

impl UpdateGearRequest {
    /// Runs the `merge into` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn merge_into(self, existing: &GearItem) -> GearDraft {
        GearDraft {
            category: self.category.unwrap_or(existing.category),
            name: self.name.unwrap_or_else(|| existing.name.clone()),
            brand: self.brand.or_else(|| existing.brand.clone()),
            model: self.model.or_else(|| existing.model.clone()),
            description: self.description.or_else(|| existing.description.clone()),
            weight_g: self.weight_g.or(existing.weight_g),
            official_price_cents: self.official_price_cents.or(existing.official_price_cents),
            official_price_currency: self
                .official_price_currency
                .or_else(|| existing.official_price_currency.clone()),
            purchase_date: self
                .purchase_date
                .or_else(|| existing.purchase_date.clone()),
            purchase_price_cents: self.purchase_price_cents.or(existing.purchase_price_cents),
            purchase_price_currency: self
                .purchase_price_currency
                .or_else(|| existing.purchase_price_currency.clone()),
            purchase_location: self
                .purchase_location
                .or_else(|| existing.purchase_location.clone()),
            status: self.status.unwrap_or(existing.status),
            storage_location: self
                .storage_location
                .or_else(|| existing.storage_location.clone()),
            specs: self.specs.unwrap_or_else(|| existing.specs.clone()),
            tags: self.tags.unwrap_or_else(|| existing.tags.clone()),
            share_enabled: self.share_enabled.unwrap_or(existing.share_enabled),
            share_status: existing.share_status,
            notes: self.notes.or_else(|| existing.notes.clone()),
        }
    }
}

/// Stable data boundary for `GearSummaryResponse`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearSummaryResponse {
    pub id: String,
    pub category: GearCategory,
    pub category_label: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub status: GearStatus,
    pub status_label: String,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub purchase_price_currency: Option<String>,
    pub purchase_date: Option<String>,
    pub specs: GearSpecs,
    pub tags: Vec<String>,
    pub tag_colors: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&GearItem> for GearSummaryResponse {
    /// Runs the `from` server-side flow while preserving input validation, error propagation, and state invariants.
    fn from(item: &GearItem) -> Self {
        Self {
            id: item.id.clone(),
            category: item.category,
            category_label: item.category.label().to_owned(),
            name: item.name.clone(),
            brand: item.brand.clone(),
            model: item.model.clone(),
            status: item.status,
            status_label: item.status.label().to_owned(),
            weight_g: item.weight_g,
            official_price_cents: item.official_price_cents,
            official_price_currency: item.official_price_currency.clone(),
            purchase_price_cents: item.purchase_price_cents,
            purchase_price_currency: item.purchase_price_currency.clone(),
            purchase_date: item.purchase_date.clone(),
            specs: item.specs.clone(),
            tags: item.tags.clone(),
            tag_colors: HashMap::new(),
            created_at: item.created_at.clone(),
            updated_at: item.updated_at.clone(),
        }
    }
}

impl GearSummaryResponse {
    /// Builds a list response item with only colors for tags present on that item.
    pub fn from_item(item: &GearItem, all_tag_colors: &HashMap<String, String>) -> Self {
        Self {
            id: item.id.clone(),
            category: item.category,
            category_label: item.category.label().to_owned(),
            name: item.name.clone(),
            brand: item.brand.clone(),
            model: item.model.clone(),
            status: item.status,
            status_label: item.status.label().to_owned(),
            weight_g: item.weight_g,
            official_price_cents: item.official_price_cents,
            official_price_currency: item.official_price_currency.clone(),
            purchase_price_cents: item.purchase_price_cents,
            purchase_price_currency: item.purchase_price_currency.clone(),
            purchase_date: item.purchase_date.clone(),
            specs: item.specs.clone(),
            tags: item.tags.clone(),
            tag_colors: tag_colors_for_tags(&item.tags, all_tag_colors),
            created_at: item.created_at.clone(),
            updated_at: item.updated_at.clone(),
        }
    }
}

/// Detail/create/update response for one gear item plus user-level tag color mapping.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearItemResponse {
    #[serde(flatten)]
    pub item: GearItem,
    pub tag_colors: HashMap<String, String>,
}

impl GearItemResponse {
    /// Builds a gear item response with only colors for tags present on that item.
    pub fn from_item(item: GearItem, all_tag_colors: &HashMap<String, String>) -> Self {
        let tag_colors = tag_colors_for_tags(&item.tags, all_tag_colors);
        Self { item, tag_colors }
    }
}

/// Stable data boundary for `ListGearResponse`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGearResponse {
    pub items: Vec<GearSummaryResponse>,
    pub next_cursor: Option<String>,
}

/// Stable data boundary for `GearCategoryFilterResponse`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearCategoryFilterResponse {
    pub id: String,
    pub label: String,
    pub count: i64,
}

/// Stable data boundary for `GearCategoriesResponse`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearCategoriesResponse {
    pub items: Vec<GearCategoryFilterResponse>,
}

/// Redis-backed ranking response for spec keys the current user often fills in a category.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearSpecKeyRankingsResponse {
    pub keys: Vec<String>,
}

/// Suggested tag plus the current user-level color preference, when one exists.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTagSuggestionResponse {
    pub tag: String,
    pub color: Option<String>,
}

/// Redis-backed tag suggestions for tags the current user often adds.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearTagSuggestionsResponse {
    pub items: Vec<GearTagSuggestionResponse>,
}

/// Stable data boundary for `ImportGearsRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct ImportGearsRequest {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub items: Vec<CreateGearRequest>,
}

/// Stable data boundary for `ImportGearsResponse`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct ImportGearsResponse {
    pub created_count: usize,
    pub updated_count: usize,
    pub failed_count: usize,
    pub errors: Vec<ImportGearError>,
}

/// Stable data boundary for `ImportGearError`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct ImportGearError {
    pub row: usize,
    pub field: String,
    pub message: String,
}

/// Filters a user tag-color map down to tags present on the gear item.
fn tag_colors_for_tags(
    tags: &[String],
    all_tag_colors: &HashMap<String, String>,
) -> HashMap<String, String> {
    tags.iter()
        .filter_map(|tag| {
            all_tag_colors
                .get(tag)
                .map(|color| (tag.clone(), color.clone()))
        })
        .collect()
}
