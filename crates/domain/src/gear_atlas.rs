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
    ExternalImport,
}

impl GearAtlasSourceType {
    pub const ALL: [Self; 3] = [Self::Manual, Self::UserGear, Self::ExternalImport];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::UserGear => "user_gear",
            Self::ExternalImport => "external_import",
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
    pub source_key: Option<String>,
    pub source_name: Option<String>,
    pub source_url: Option<String>,
    pub source_license_note: Option<String>,
    pub import_batch_id: Option<String>,
    pub imported_at: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
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

/// Writable draft created by a conservative external source import.
///
/// The public fields mirror a normal atlas submission, while source metadata is
/// kept separate so clients can audit where imported facts came from without
/// exposing crawler-only identifiers such as `source_key`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasExternalImportDraft {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub specs: GearSpecs,
    pub submitted_by_user_id: String,
    pub source_key: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub source_license_note: Option<String>,
    pub import_batch_id: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
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

impl GearAtlasExternalImportDraft {
    /// Validates source-audit metadata and the public atlas fields that will be reviewed.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut public_draft = GearAtlasDraft {
            category: self.category,
            name: std::mem::take(&mut self.name),
            brand: self.brand.take(),
            model: self.model.take(),
            description: self.description.take(),
            weight_g: self.weight_g,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_currency.take(),
            specs: std::mem::take(&mut self.specs),
            source_type: GearAtlasSourceType::ExternalImport,
            submitted_by_user_id: std::mem::take(&mut self.submitted_by_user_id),
            source_user_gear_id: None,
        };
        let mut errors = match public_draft.validate_and_normalize() {
            Ok(()) => Vec::new(),
            Err(error) => error.fields,
        };

        self.category = public_draft.category;
        self.name = public_draft.name;
        self.brand = public_draft.brand;
        self.model = public_draft.model;
        self.description = public_draft.description;
        self.weight_g = public_draft.weight_g;
        self.official_price_cents = public_draft.official_price_cents;
        self.official_price_currency = public_draft.official_price_currency;
        self.specs = public_draft.specs;
        self.submitted_by_user_id = public_draft.submitted_by_user_id;

        self.source_key = normalize_required_text(
            std::mem::take(&mut self.source_key),
            160,
            "source_key",
            &mut errors,
        );
        self.source_name = normalize_required_text(
            std::mem::take(&mut self.source_name),
            80,
            "source_name",
            &mut errors,
        );
        self.source_url =
            normalize_optional_text(self.source_url.take(), 500, "source_url", &mut errors);
        self.source_license_note = normalize_optional_text(
            self.source_license_note.take(),
            240,
            "source_license_note",
            &mut errors,
        );
        self.import_batch_id = normalize_optional_text(
            self.import_batch_id.take(),
            128,
            "import_batch_id",
            &mut errors,
        );

        if let Some(score) = self.source_rating_score {
            if !score.is_finite() || !(0.0..=10.0).contains(&score) {
                errors.push(FieldViolation::new(
                    "source_rating_score",
                    "must be between 0 and 10",
                ));
            }
        }
        if let Some(count) = self.source_rating_count {
            if !(0..=1_000_000).contains(&count) {
                errors.push(FieldViolation::new(
                    "source_rating_count",
                    "must be between 0 and 1000000",
                ));
            }
        }

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
