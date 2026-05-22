//! HTTP DTOs for public gear atlas browsing, user submissions, and administrator review.

use serde::{Deserialize, Serialize};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs, GearVariants},
    gear_atlas::{
        GearAtlasDraft, GearAtlasItem, GearAtlasSort, GearAtlasSourceType, GearAtlasStatus,
    },
};

/// Query parameters accepted by the public gear atlas list endpoint.
#[derive(Debug, Deserialize)]
pub struct ListGearAtlasQuery {
    pub category: Option<GearCategory>,
    pub q: Option<String>,
    #[serde(default)]
    pub sort: GearAtlasSort,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub locale: Option<String>,
}

/// Query parameters for a user's own submission list.
#[derive(Debug, Deserialize)]
pub struct ListMyGearAtlasSubmissionsQuery {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Query parameters for the administrator review queue.
#[derive(Debug, Deserialize)]
pub struct ListAdminGearAtlasSubmissionsQuery {
    pub status: Option<GearAtlasStatus>,
    pub category: Option<GearCategory>,
    pub q: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Manual gear atlas submission body. It exposes only public atlas fields.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGearAtlasSubmissionRequest {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    #[serde(default)]
    pub variants: GearVariants,
    #[serde(default)]
    pub specs: GearSpecs,
}

impl CreateGearAtlasSubmissionRequest {
    /// Converts the HTTP body into a pending atlas draft owned by the current user.
    pub fn into_draft(self, user_id: &str) -> GearAtlasDraft {
        GearAtlasDraft {
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
            source_type: GearAtlasSourceType::Manual,
            submitted_by_user_id: user_id.to_owned(),
            source_user_gear_id: None,
        }
    }
}

/// Administrator body for replacing editable public fields on one submission.
pub type UpdateGearAtlasSubmissionRequest = CreateGearAtlasSubmissionRequest;

/// Administrator reject body.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectGearAtlasSubmissionRequest {
    pub reason: Option<String>,
}

/// Public gear atlas item returned to unauthenticated clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasPublicItemResponse {
    pub id: String,
    pub category: GearCategory,
    pub category_label: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub approved_at: Option<String>,
    pub source_name: Option<String>,
    pub source_url: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&GearAtlasItem> for GearAtlasPublicItemResponse {
    fn from(item: &GearAtlasItem) -> Self {
        Self::from_item_and_category_label(item, item.category.label().to_owned())
    }
}

impl GearAtlasPublicItemResponse {
    /// Builds a public response with a locale-resolved category label.
    pub fn from_item_and_category_label(item: &GearAtlasItem, category_label: String) -> Self {
        Self {
            id: item.id.clone(),
            category: item.category,
            category_label,
            name: item.name.clone(),
            brand: item.brand.clone(),
            model: item.model.clone(),
            description: item.description.clone(),
            weight_g: item.weight_g,
            official_price_cents: item.official_price_cents,
            official_price_currency: item.official_price_currency.clone(),
            variants: item.variants.clone(),
            specs: item.specs.clone(),
            approved_at: item.approved_at.clone(),
            source_name: item.source_name.clone(),
            source_url: item.source_url.clone(),
            source_rating_score: item.source_rating_score,
            source_rating_count: item.source_rating_count,
            created_at: item.created_at.clone(),
            updated_at: item.updated_at.clone(),
        }
    }
}

/// Submission item returned to users and administrators.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasSubmissionResponse {
    #[serde(flatten)]
    pub public: GearAtlasPublicItemResponse,
    pub source_type: GearAtlasSourceType,
    pub source_user_gear_id: Option<String>,
    pub status: GearAtlasStatus,
    pub rejection_reason: Option<String>,
    pub reviewed_at: Option<String>,
}

impl From<&GearAtlasItem> for GearAtlasSubmissionResponse {
    fn from(item: &GearAtlasItem) -> Self {
        Self {
            public: GearAtlasPublicItemResponse::from(item),
            source_type: item.source_type,
            source_user_gear_id: item.source_user_gear_id.clone(),
            status: item.status,
            rejection_reason: item.rejection_reason.clone(),
            reviewed_at: item.reviewed_at.clone(),
        }
    }
}

/// Paginated public gear atlas list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGearAtlasResponse {
    pub items: Vec<GearAtlasPublicItemResponse>,
    pub next_cursor: Option<String>,
}

/// Paginated submission list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGearAtlasSubmissionsResponse {
    pub items: Vec<GearAtlasSubmissionResponse>,
    pub next_cursor: Option<String>,
}
