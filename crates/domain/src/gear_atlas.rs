//! Public gear atlas domain models and validation.
//!
//! Gear atlas records are public-market gear snapshots. They deliberately keep
//! a much smaller field set than personal gear records so user-specific
//! purchase, storage, status, and note data cannot leak into public reads.

use serde::{Deserialize, Serialize};

use crate::{
    gear::{GearCategory, GearSpecs, SUPPORTED_CURRENCIES, normalize_specs, now_rfc3339},
    validation::{
        FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
    },
};

/// Review status for one public gear atlas submission.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}

impl GearAtlasStatus {
    pub const ALL: [Self; 3] = [Self::Pending, Self::Approved, Self::Rejected];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
    }
}

impl std::fmt::Display for GearAtlasStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Source that created a gear atlas submission.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasSourceType {
    #[default]
    Manual,
    UserGear,
}

impl GearAtlasSourceType {
    pub const ALL: [Self; 2] = [Self::Manual, Self::UserGear];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::UserGear => "user_gear",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|source_type| source_type.as_str() == value)
    }
}

impl std::fmt::Display for GearAtlasSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Sort options supported by public gear atlas list reads.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasSort {
    #[default]
    ApprovedAtDesc,
    NameAsc,
    WeightDesc,
    OfficialPriceDesc,
}

/// Complete persisted atlas item, including review metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasItem {
    pub id: String,
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub specs: GearSpecs,
    pub source_type: GearAtlasSourceType,
    pub submitted_by_user_id: String,
    pub source_user_gear_id: Option<String>,
    pub status: GearAtlasStatus,
    pub rejection_reason: Option<String>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_at: Option<String>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Writable public atlas draft created from a manual form or a personal gear snapshot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasDraft {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub specs: GearSpecs,
    pub source_type: GearAtlasSourceType,
    pub submitted_by_user_id: String,
    pub source_user_gear_id: Option<String>,
}

impl GearAtlasDraft {
    /// Validates and normalizes only the public fields allowed in the atlas.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.name =
            normalize_required_text(std::mem::take(&mut self.name), 100, "name", &mut errors);
        self.brand = normalize_optional_text(self.brand.take(), 80, "brand", &mut errors);
        self.model = normalize_optional_text(self.model.take(), 80, "model", &mut errors);
        self.description =
            normalize_optional_text(self.description.take(), 100, "description", &mut errors);
        self.submitted_by_user_id = normalize_required_text(
            std::mem::take(&mut self.submitted_by_user_id),
            128,
            "submitted_by_user_id",
            &mut errors,
        );
        self.source_user_gear_id = normalize_optional_text(
            self.source_user_gear_id.take(),
            128,
            "source_user_gear_id",
            &mut errors,
        );

        if let Some(weight_g) = self.weight_g {
            if !(0..=1_000_000).contains(&weight_g) {
                errors.push(FieldViolation::new(
                    "weight_g",
                    "must be between 0 and 1000000",
                ));
            }
        }

        if let Some(price) = self.official_price_cents {
            if price < 0 {
                errors.push(FieldViolation::new(
                    "official_price_cents",
                    "must be greater than or equal to 0",
                ));
            }
        }
        self.official_price_currency = normalize_official_price_currency(
            self.official_price_cents,
            self.official_price_currency.take(),
            &mut errors,
        );
        self.specs = normalize_specs(self.category, std::mem::take(&mut self.specs), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Builds a public atlas draft from a personal gear item by copying only the
/// approved public snapshot fields.
pub fn draft_from_personal_gear(user_id: &str, item: &crate::gear::GearItem) -> GearAtlasDraft {
    GearAtlasDraft {
        category: item.category,
        name: item.name.clone(),
        brand: item.brand.clone(),
        model: item.model.clone(),
        description: item.description.clone(),
        weight_g: item.weight_g,
        official_price_cents: item.official_price_cents,
        official_price_currency: item.official_price_currency.clone(),
        specs: item.specs.clone(),
        source_type: GearAtlasSourceType::UserGear,
        submitted_by_user_id: user_id.to_owned(),
        source_user_gear_id: Some(item.id.clone()),
    }
}

fn normalize_official_price_currency(
    price_cents: Option<i64>,
    currency: Option<String>,
    errors: &mut Vec<FieldViolation>,
) -> Option<String> {
    let _price_cents = price_cents?;
    let normalized = currency
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "CNY".to_owned());
    if !SUPPORTED_CURRENCIES.contains(&normalized.as_str()) {
        errors.push(FieldViolation::new(
            "official_price_currency",
            "must be one of CNY, USD, EUR, JPY, HKD",
        ));
    }
    Some(normalized)
}

/// Returns the current UTC timestamp for atlas review transitions.
pub fn now_atlas_rfc3339() -> String {
    now_rfc3339()
}
