//! HTTP DTOs for public gear atlas browsing, user submissions, and administrator review.

use serde::{Deserialize, Serialize};
use stellartrail_domain::{
    deletion::DeletedFilter,
    gear::{GearCategory, GearSpecs, GearVariants},
    gear_atlas::{
        GearAtlasDraft, GearAtlasItem, GearAtlasLocalizationDraft,
        GearAtlasLocalizationReviewStatus, GearAtlasReviewChange, GearAtlasSort,
        GearAtlasSourceType, GearAtlasStatus,
    },
    locale::Locale,
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
    #[serde(default)]
    pub deleted: DeletedFilter,
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

/// Administrator body for editing one locale-specific display row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateGearAtlasLocalizationRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub variants: GearVariants,
    #[serde(default)]
    pub specs: GearSpecs,
    pub translation_provider: Option<String>,
    #[serde(default)]
    pub mark_reviewed: bool,
}

/// Administrator body for generating one locale-specific display draft.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateGearAtlasLocalizationDraftRequest {
    #[serde(default)]
    pub overwrite_reviewed: bool,
    pub translation_provider: Option<String>,
}

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
    pub is_deleted: bool,
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
            is_deleted: item.is_deleted,
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
    pub review_changes: GearAtlasReviewChangesResponse,
    pub reviewed_at: Option<String>,
}

pub type GearAtlasReviewChangesResponse = Vec<GearAtlasReviewChange>;

impl From<&GearAtlasItem> for GearAtlasSubmissionResponse {
    fn from(item: &GearAtlasItem) -> Self {
        Self {
            public: GearAtlasPublicItemResponse::from(item),
            source_type: item.source_type,
            source_user_gear_id: item.source_user_gear_id.clone(),
            status: item.status,
            rejection_reason: item.rejection_reason.clone(),
            review_changes: item.review_changes.clone(),
            reviewed_at: item.reviewed_at.clone(),
        }
    }
}

/// Submission item returned only from administrator endpoints, including source audit metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasAdminSubmissionResponse {
    #[serde(flatten)]
    pub submission: GearAtlasSubmissionResponse,
    pub review_locale: Locale,
    pub display_name: String,
    pub display_description: Option<String>,
    pub display_variants: GearVariants,
    pub display_specs: GearSpecs,
    pub display_category_label: String,
    pub localization_statuses: Vec<GearAtlasLocalizationReviewStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub localizations: Vec<GearAtlasLocalizationResponse>,
    pub source_name: Option<String>,
    pub source_url: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
}

impl From<&GearAtlasItem> for GearAtlasAdminSubmissionResponse {
    fn from(item: &GearAtlasItem) -> Self {
        Self::from_item_and_display(
            item,
            item,
            item.category.label().to_owned(),
            Locale::ZhCn,
            Vec::new(),
            Vec::new(),
        )
    }
}

impl GearAtlasAdminSubmissionResponse {
    /// Builds an admin response that keeps editable canonical fields separate
    /// from locale-resolved display fields.
    pub fn from_item_and_display(
        item: &GearAtlasItem,
        display_item: &GearAtlasItem,
        display_category_label: String,
        review_locale: Locale,
        localization_statuses: Vec<GearAtlasLocalizationReviewStatus>,
        localizations: Vec<GearAtlasLocalizationResponse>,
    ) -> Self {
        Self {
            submission: GearAtlasSubmissionResponse::from(item),
            review_locale,
            display_name: display_item.name.clone(),
            display_description: display_item.description.clone(),
            display_variants: display_item.variants.clone(),
            display_specs: display_item.specs.clone(),
            display_category_label,
            localization_statuses,
            localizations,
            source_name: item.source_name.clone(),
            source_url: item.source_url.clone(),
            source_rating_score: item.source_rating_score,
            source_rating_count: item.source_rating_count,
        }
    }
}

/// Locale-specific atlas display content returned only to administrators.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasLocalizationResponse {
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub translation_status: Option<String>,
    pub translation_provider: Option<String>,
    pub translated_at: Option<String>,
}

impl From<&GearAtlasLocalizationDraft> for GearAtlasLocalizationResponse {
    fn from(localization: &GearAtlasLocalizationDraft) -> Self {
        Self {
            locale: localization.locale,
            name: localization.name.clone(),
            description: localization.description.clone(),
            variants: localization.variants.clone(),
            specs: localization.specs.clone(),
            translation_status: localization.translation_status.clone(),
            translation_provider: localization.translation_provider.clone(),
            translated_at: localization.translated_at.clone(),
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

/// Paginated administrator gear atlas submission list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListAdminGearAtlasSubmissionsResponse {
    pub items: Vec<GearAtlasAdminSubmissionResponse>,
    pub next_cursor: Option<String>,
}
