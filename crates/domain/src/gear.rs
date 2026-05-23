//! Gear inventory domain model module defining categories, statuses, draft validation, and statistics.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Iso8601, Date, OffsetDateTime};

use crate::validation::{
    normalize_optional_text, normalize_required_text, FieldViolation, ValidationError,
};

/// Stable enum boundary for `GearCategory`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearCategory {
    BackpackSystem,
    SleepSystem,
    KitchenSystem,
    WalkingSystem,
    ClothingSystem,
    LightingSystem,
    FirstAidSystem,
    ElectronicsSystem,
    TechnicalGear,
    OtherGear,
    Consumable,
}

impl GearCategory {
    pub const ALL: [Self; 11] = [
        Self::BackpackSystem,
        Self::SleepSystem,
        Self::KitchenSystem,
        Self::WalkingSystem,
        Self::ClothingSystem,
        Self::LightingSystem,
        Self::FirstAidSystem,
        Self::ElectronicsSystem,
        Self::TechnicalGear,
        Self::OtherGear,
        Self::Consumable,
    ];

    /// Runs the `as str` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackpackSystem => "backpack_system",
            Self::SleepSystem => "sleep_system",
            Self::KitchenSystem => "kitchen_system",
            Self::WalkingSystem => "walking_system",
            Self::ClothingSystem => "clothing_system",
            Self::LightingSystem => "lighting_system",
            Self::FirstAidSystem => "first_aid_system",
            Self::ElectronicsSystem => "electronics_system",
            Self::TechnicalGear => "technical_gear",
            Self::OtherGear => "other_gear",
            Self::Consumable => "consumable",
        }
    }

    /// Runs the `label` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn label(self) -> &'static str {
        match self {
            Self::BackpackSystem => "背负系统",
            Self::SleepSystem => "睡眠系统",
            Self::KitchenSystem => "餐厨系统",
            Self::WalkingSystem => "行走系统",
            Self::ClothingSystem => "衣物系统",
            Self::LightingSystem => "照明系统",
            Self::FirstAidSystem => "急救系统",
            Self::ElectronicsSystem => "电子系统",
            Self::TechnicalGear => "技术装备",
            Self::OtherGear => "其它装备",
            Self::Consumable => "消耗品",
        }
    }

    /// Runs the `from key` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == value)
    }
}

impl std::fmt::Display for GearCategory {
    /// Runs the `fmt` server-side flow while preserving input validation, error propagation, and state invariants.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable enum boundary for `GearStatus`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearStatus {
    #[default]
    Available,
    InUse,
    Maintenance,
    Damaged,
    Lost,
    Retired,
    Sold,
    Idle,
}

impl GearStatus {
    pub const ALL: [Self; 8] = [
        Self::Available,
        Self::InUse,
        Self::Maintenance,
        Self::Damaged,
        Self::Lost,
        Self::Retired,
        Self::Sold,
        Self::Idle,
    ];

    /// Runs the `as str` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::InUse => "in_use",
            Self::Maintenance => "maintenance",
            Self::Damaged => "damaged",
            Self::Lost => "lost",
            Self::Retired => "retired",
            Self::Sold => "sold",
            Self::Idle => "idle",
        }
    }

    /// Runs the `label` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "可用",
            Self::InUse => "使用中",
            Self::Maintenance => "维修中",
            Self::Damaged => "已损坏",
            Self::Lost => "已丢失",
            Self::Retired => "已退役",
            Self::Sold => "已售出",
            Self::Idle => "闲置",
        }
    }

    /// Runs the `from key` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
    }
}

impl std::fmt::Display for GearStatus {
    /// Runs the `fmt` server-side flow while preserving input validation, error propagation, and state invariants.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable enum boundary for `GearShareStatus`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearShareStatus {
    #[default]
    NotShared,
    Pending,
    Approved,
    Rejected,
    Withdrawn,
}

impl GearShareStatus {
    pub const ALL: [Self; 5] = [
        Self::NotShared,
        Self::Pending,
        Self::Approved,
        Self::Rejected,
        Self::Withdrawn,
    ];

    /// Runs the `as str` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotShared => "not_shared",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Runs the `from key` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
    }
}

impl std::fmt::Display for GearShareStatus {
    /// Runs the `fmt` server-side flow while preserving input validation, error propagation, and state invariants.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable enum boundary for `GearTab`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearTab {
    #[default]
    Available,
    History,
}

impl GearTab {
    /// Returns the stable wire key used by API query parameters and JSON payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::History => "history",
        }
    }

    /// Parses a stable wire key into a gear tab.
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "available" => Some(Self::Available),
            "history" => Some(Self::History),
            _ => None,
        }
    }
}

/// Stable enum boundary for `GearSort`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearSort {
    #[default]
    CreatedAtDesc,
    CreatedAtAsc,
    PurchaseDateDesc,
    NameAsc,
    WeightDesc,
    PriceDesc,
}

impl GearSort {
    /// Returns the stable wire key used by API query parameters and JSON payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreatedAtDesc => "created_at_desc",
            Self::CreatedAtAsc => "created_at_asc",
            Self::PurchaseDateDesc => "purchase_date_desc",
            Self::NameAsc => "name_asc",
            Self::WeightDesc => "weight_desc",
            Self::PriceDesc => "price_desc",
        }
    }

    /// Parses a stable wire key into a gear list sort mode.
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "created_at_desc" => Some(Self::CreatedAtDesc),
            "created_at_asc" => Some(Self::CreatedAtAsc),
            "purchase_date_desc" => Some(Self::PurchaseDateDesc),
            "name_asc" => Some(Self::NameAsc),
            "weight_desc" => Some(Self::WeightDesc),
            "price_desc" => Some(Self::PriceDesc),
            _ => None,
        }
    }
}

pub type GearSpecs = BTreeMap<String, String>;

/// Public size/fit variant for a gear atlas item.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GearVariant {
    pub key: String,
    pub label: String,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub weight_g: Option<i32>,
}

pub type GearVariants = Vec<GearVariant>;

/// Complete gear domain object representing one user gear record from the database.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearItem {
    pub id: String,
    pub user_id: String,
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
    pub status: GearStatus,
    pub storage_location: Option<String>,
    pub atlas_item_id: Option<String>,
    pub selected_variant_key: Option<String>,
    pub selected_variant_label: Option<String>,
    pub quantity: i32,
    pub specs: GearSpecs,
    pub tags: Vec<String>,
    pub share_enabled: bool,
    pub share_status: GearShareStatus,
    pub notes: Option<String>,
    pub archived_at: Option<String>,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Writable gear draft containing fields validated before create, update, or import operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearDraft {
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
    pub status: GearStatus,
    pub storage_location: Option<String>,
    pub atlas_item_id: Option<String>,
    pub selected_variant_key: Option<String>,
    pub selected_variant_label: Option<String>,
    pub quantity: i32,
    pub specs: GearSpecs,
    pub tags: Vec<String>,
    pub share_enabled: bool,
    pub share_status: GearShareStatus,
    pub notes: Option<String>,
}

impl GearDraft {
    /// Validates a gear draft and normalizes text, tags, status, and related fields.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.name =
            normalize_required_text(std::mem::take(&mut self.name), 100, "name", &mut errors);
        self.brand = normalize_optional_text(self.brand.take(), 80, "brand", &mut errors);
        self.model = normalize_optional_text(self.model.take(), 80, "model", &mut errors);
        self.description =
            normalize_optional_text(self.description.take(), 100, "description", &mut errors);
        self.purchase_location = normalize_optional_text(
            self.purchase_location.take(),
            100,
            "purchase_location",
            &mut errors,
        );
        self.storage_location = normalize_optional_text(
            self.storage_location.take(),
            100,
            "storage_location",
            &mut errors,
        );
        self.atlas_item_id =
            normalize_optional_text(self.atlas_item_id.take(), 128, "atlas_item_id", &mut errors);
        self.selected_variant_label = normalize_optional_text(
            self.selected_variant_label.take(),
            80,
            "selected_variant_label",
            &mut errors,
        );
        self.selected_variant_key = normalize_optional_text(
            self.selected_variant_key.take(),
            80,
            "selected_variant_key",
            &mut errors,
        );
        if self.selected_variant_label.is_some() && self.selected_variant_key.is_none() {
            if let Some(label) = self.selected_variant_label.as_deref() {
                self.selected_variant_key = Some(variant_key_from_label(label, 0));
            }
        }
        self.notes = normalize_optional_text(self.notes.take(), 1000, "notes", &mut errors);

        if !(1..=9_999).contains(&self.quantity) {
            errors.push(FieldViolation::new(
                "quantity",
                "must be between 1 and 9999",
            ));
        }

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

        if let Some(price) = self.purchase_price_cents {
            if price < 0 {
                errors.push(FieldViolation::new(
                    "purchase_price_cents",
                    "must be greater than or equal to 0",
                ));
            }
        }

        self.official_price_currency = normalize_price_currency(
            self.official_price_cents,
            self.official_price_currency.take(),
            "official_price_currency",
            &mut errors,
        );
        self.purchase_price_currency = normalize_price_currency(
            self.purchase_price_cents,
            self.purchase_price_currency.take(),
            "purchase_price_currency",
            &mut errors,
        );

        validate_date(self.purchase_date.as_deref(), "purchase_date", &mut errors);
        self.specs = normalize_specs(self.category, std::mem::take(&mut self.specs), &mut errors);

        self.tags = normalize_tags(std::mem::take(&mut self.tags), &mut errors);
        self.share_status = derive_share_status(self.share_enabled, self.share_status);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Runs the `derive share status` server-side flow while preserving input validation, error propagation, and state invariants.
pub fn derive_share_status(share_enabled: bool, previous: GearShareStatus) -> GearShareStatus {
    if share_enabled {
        match previous {
            GearShareStatus::Approved | GearShareStatus::Rejected => previous,
            _ => GearShareStatus::Pending,
        }
    } else if matches!(
        previous,
        GearShareStatus::Pending | GearShareStatus::Approved
    ) {
        GearShareStatus::Withdrawn
    } else {
        GearShareStatus::NotShared
    }
}

pub const SUPPORTED_CURRENCIES: [&str; 5] = ["CNY", "USD", "EUR", "JPY", "HKD"];

const LEGACY_SPEC_KEYS: [&str; 6] = [
    "color",
    "material",
    "capacity",
    "warmth_index",
    "waterproof_index",
    "expiry_or_warranty_date",
];

/// Returns the first-version spec keys supported for a gear category.
pub fn allowed_spec_keys(category: GearCategory) -> &'static [&'static str] {
    match category {
        GearCategory::BackpackSystem => &[
            "capacity",
            "recommended_load",
            "back_length",
            "waterproof_rating",
        ],
        GearCategory::SleepSystem => &[
            "type",
            "people_count",
            "temperature_or_r_value",
            "filling",
            "fill_weight",
            "packed_size",
            "waterproof_rating",
        ],
        GearCategory::KitchenSystem => &[
            "fuel_type",
            "capacity",
            "power",
            "people_count",
            "packed_size",
        ],
        GearCategory::WalkingSystem => &["terrain", "waterproof_rating", "material", "support"],
        GearCategory::ClothingSystem => &[
            "layer",
            "warmth_rating",
            "waterproof_rating",
            "breathability_rating",
            "season",
        ],
        GearCategory::LightingSystem => &[
            "max_brightness",
            "runtime",
            "battery_type",
            "charging_port",
            "waterproof_rating",
            "beam_distance",
        ],
        GearCategory::FirstAidSystem => &[
            "kit_size",
            "expiry_date",
            "people_count",
            "days",
            "waterproof_packaging",
        ],
        GearCategory::ElectronicsSystem => &[
            "battery_capacity",
            "rated_energy",
            "output_power",
            "ports",
            "waterproof_rating",
            "working_temperature",
        ],
        GearCategory::TechnicalGear => &[
            "certification",
            "strength",
            "specification",
            "length",
            "material",
            "retirement_date",
        ],
        GearCategory::OtherGear => &[
            "use_case",
            "specification",
            "capacity",
            "waterproof_rating",
            "accessories",
        ],
        GearCategory::Consumable => &[
            "type",
            "net_content",
            "expiry_date",
            "storage_condition",
            "restock_threshold",
        ],
    }
}

/// Checks whether a spec key is valid for the given category.
pub fn is_allowed_spec_key(category: GearCategory, key: &str) -> bool {
    allowed_spec_keys(category).contains(&key) || LEGACY_SPEC_KEYS.contains(&key)
}

/// Builds a stable variant key from a human label.
pub fn variant_key_from_label(label: &str, index: usize) -> String {
    let mut key = String::new();
    for ch in label.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
        } else if !key.ends_with('-') {
            key.push('-');
        }
    }
    let key = key.trim_matches('-');
    if key.is_empty() {
        format!("variant-{index}")
    } else {
        key.chars().take(80).collect()
    }
}

fn normalize_price_currency(
    price_cents: Option<i64>,
    currency: Option<String>,
    field: &str,
    errors: &mut Vec<FieldViolation>,
) -> Option<String> {
    let _price_cents = price_cents?;
    let normalized = normalize_currency(currency).unwrap_or_else(|| "CNY".to_owned());
    if !SUPPORTED_CURRENCIES.contains(&normalized.as_str()) {
        errors.push(FieldViolation::new(
            field,
            "must be one of CNY, USD, EUR, JPY, HKD",
        ));
    }
    Some(normalized)
}

fn normalize_currency(value: Option<String>) -> Option<String> {
    let currency = value?.trim().to_ascii_uppercase();
    if currency.is_empty() {
        None
    } else {
        Some(currency)
    }
}

/// Normalizes specs by trimming values, enforcing supported keys, and deduplicating by key.
pub fn normalize_specs(
    category: GearCategory,
    values: GearSpecs,
    errors: &mut Vec<FieldViolation>,
) -> GearSpecs {
    let mut normalized = GearSpecs::new();
    for (raw_key, raw_value) in values {
        let key = raw_key.trim();
        if key.is_empty() {
            continue;
        }
        if !is_allowed_spec_key(category, key) {
            errors.push(FieldViolation::new(
                format!("specs.{key}"),
                "is not supported for this category",
            ));
            continue;
        }
        let value = raw_value.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 100 {
            errors.push(FieldViolation::new(
                format!("specs.{key}"),
                "must be at most 100 characters",
            ));
            continue;
        }
        normalized.insert(key.to_owned(), value.to_owned());
    }
    if normalized.len() > 32 {
        errors.push(FieldViolation::new(
            "specs",
            "must contain at most 32 fields",
        ));
        normalized = normalized.into_iter().take(32).collect();
    }
    normalized
}

/// Normalizes atlas size variants while preserving user-visible labels.
pub fn normalize_variants(values: GearVariants, errors: &mut Vec<FieldViolation>) -> GearVariants {
    let mut normalized = GearVariants::new();
    for (index, mut raw) in values.into_iter().enumerate() {
        let label = raw.label.trim();
        if label.is_empty() {
            continue;
        }
        if label.chars().count() > 80 {
            errors.push(FieldViolation::new(
                format!("variants.{index}.label"),
                "must be at most 80 characters",
            ));
            continue;
        }
        let key = raw.key.trim();
        let key = if key.is_empty() {
            variant_key_from_label(label, index)
        } else {
            key.to_owned()
        };
        if key.chars().count() > 80 {
            errors.push(FieldViolation::new(
                format!("variants.{index}.key"),
                "must be at most 80 characters",
            ));
            continue;
        }
        if normalized.iter().any(|existing| existing.key == key) {
            continue;
        }
        if let Some(price) = raw.official_price_cents {
            if price < 0 {
                errors.push(FieldViolation::new(
                    format!("variants.{index}.official_price_cents"),
                    "must be greater than or equal to 0",
                ));
                raw.official_price_cents = None;
            }
        }
        raw.official_price_currency = normalize_price_currency(
            raw.official_price_cents,
            raw.official_price_currency.take(),
            &format!("variants.{index}.official_price_currency"),
            errors,
        );
        if let Some(weight_g) = raw.weight_g {
            if !(0..=1_000_000).contains(&weight_g) {
                errors.push(FieldViolation::new(
                    format!("variants.{index}.weight_g"),
                    "must be between 0 and 1000000",
                ));
                raw.weight_g = None;
            }
        }
        normalized.push(GearVariant {
            key,
            label: label.to_owned(),
            official_price_cents: raw.official_price_cents,
            official_price_currency: raw.official_price_currency,
            weight_g: raw.weight_g,
        });
    }
    if normalized.len() > 50 {
        errors.push(FieldViolation::new(
            "variants",
            "must contain at most 50 variants",
        ));
        normalized.truncate(50);
    }
    normalized
}

/// Sanitizes tag lists by removing empty values, enforcing length limits, and deduplicating entries.
pub fn normalize_tags(values: Vec<String>, errors: &mut Vec<FieldViolation>) -> Vec<String> {
    let mut normalized = Vec::new();
    for raw in values {
        let tag = raw.trim();
        if tag.is_empty() {
            continue;
        }
        if tag.chars().count() > 20 {
            errors.push(FieldViolation::new(
                "tags",
                "each tag must be at most 20 characters",
            ));
            continue;
        }
        if !normalized.iter().any(|existing| existing == tag) {
            normalized.push(tag.to_owned());
        }
    }
    if normalized.len() > 20 {
        errors.push(FieldViolation::new("tags", "must contain at most 20 tags"));
        normalized.truncate(20);
    }
    normalized
}

/// Runs the `now rfc3339` server-side flow while preserving input validation, error propagation, and state invariants.
pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .expect("RFC3339 timestamp formatting should be infallible")
}

/// Runs the `validate date` server-side flow while preserving input validation, error propagation, and state invariants.
fn validate_date(value: Option<&str>, field: &str, errors: &mut Vec<FieldViolation>) {
    if value.is_some() && parse_date(value).is_none() {
        errors.push(FieldViolation::new(field, "must use YYYY-MM-DD format"));
    }
}

/// Runs the `parse date` server-side flow while preserving input validation, error propagation, and state invariants.
fn parse_date(value: Option<&str>) -> Option<Date> {
    let value = value?;
    Date::parse(
        value,
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .ok()
}

/// Stable data boundary for `GearCategoryCount`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearCategoryCount {
    pub category: GearCategory,
    pub label: String,
    pub count: i64,
}

/// Stable data boundary for `GearStatusCount`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearStatusCount {
    pub status: GearStatus,
    pub label: String,
    pub count: i64,
}

/// Stable data boundary for `GearStats`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearStats {
    pub current_count: i64,
    pub archived_count: i64,
    pub total_value_cents: i64,
    pub total_weight_g: i64,
    pub by_category: Vec<GearCategoryCount>,
    pub by_status: Vec<GearStatusCount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the `valid draft` server-side flow while preserving input validation, error propagation, and state invariants.
    fn valid_draft() -> GearDraft {
        GearDraft {
            category: GearCategory::LightingSystem,
            name: "  头灯  ".to_owned(),
            brand: Some("  Naturehike  ".to_owned()),
            model: Some("TD-02".to_owned()),
            description: Some("夜间备用".to_owned()),
            weight_g: Some(85),
            official_price_cents: Some(12900),
            official_price_currency: Some("cny".to_owned()),
            purchase_date: Some("2026-01-01".to_owned()),
            purchase_price_cents: Some(9900),
            purchase_price_currency: None,
            purchase_location: Some("京东".to_owned()),
            status: GearStatus::Available,
            storage_location: Some("装备柜".to_owned()),
            atlas_item_id: None,
            selected_variant_key: None,
            selected_variant_label: None,
            quantity: 1,
            specs: GearSpecs::from([
                ("waterproof_rating".to_owned(), " IPX4 ".to_owned()),
                ("max_brightness".to_owned(), "450 lm".to_owned()),
            ]),
            tags: vec![" 夜徒 ".to_owned(), "夜徒".to_owned(), "照明".to_owned()],
            share_enabled: true,
            share_status: GearShareStatus::NotShared,
            notes: Some("充满电后入库".to_owned()),
        }
    }

    /// Runs the `gear enums serialize as snake case` server-side flow while preserving input validation, error propagation, and state invariants.
    #[test]
    fn gear_enums_serialize_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&GearCategory::BackpackSystem).unwrap(),
            "\"backpack_system\""
        );
        assert_eq!(
            serde_json::to_string(&GearStatus::InUse).unwrap(),
            "\"in_use\""
        );
        assert_eq!(
            serde_json::to_string(&GearShareStatus::NotShared).unwrap(),
            "\"not_shared\""
        );
    }

    /// Validates that draft normalization trims nulls, deduplicates tags, and derives share status.
    /// Preserves the server-side validation, error propagation, and state invariants for this case.
    #[test]
    fn draft_validation_trims_nulls_dedupes_tags_and_derives_share_status() {
        let mut draft = valid_draft();

        draft.validate_and_normalize().unwrap();

        assert_eq!(draft.name, "头灯");
        assert_eq!(draft.brand.as_deref(), Some("Naturehike"));
        assert_eq!(draft.official_price_currency.as_deref(), Some("CNY"));
        assert_eq!(draft.purchase_price_currency.as_deref(), Some("CNY"));
        assert_eq!(
            draft.specs.get("waterproof_rating").map(String::as_str),
            Some("IPX4")
        );
        assert_eq!(draft.tags, vec!["夜徒", "照明"]);
        assert_eq!(draft.share_status, GearShareStatus::Pending);
    }

    /// Runs the `draft validation rejects required and bounded fields` server-side flow while preserving input validation, error propagation, and state invariants.
    #[test]
    fn draft_validation_rejects_required_and_bounded_fields() {
        let mut draft = valid_draft();
        draft.name = "  ".to_owned();
        draft.description = Some("太".repeat(101));
        draft.weight_g = Some(-1);
        draft.quantity = 0;
        draft.official_price_cents = Some(-1);
        draft.purchase_price_cents = Some(-1);
        draft.purchase_price_currency = Some("GBP".to_owned());
        draft.purchase_date = Some("2026/01/01".to_owned());
        draft
            .specs
            .insert("opening_style".to_owned(), "拉链".to_owned());

        let error = draft.validate_and_normalize().unwrap_err();

        let fields: Vec<_> = error.fields.into_iter().map(|field| field.field).collect();
        assert!(fields.contains(&"name".to_owned()));
        assert!(fields.contains(&"description".to_owned()));
        assert!(fields.contains(&"weight_g".to_owned()));
        assert!(fields.contains(&"quantity".to_owned()));
        assert!(fields.contains(&"official_price_cents".to_owned()));
        assert!(fields.contains(&"purchase_price_cents".to_owned()));
        assert!(fields.contains(&"purchase_price_currency".to_owned()));
        assert!(fields.contains(&"purchase_date".to_owned()));
        assert!(fields.contains(&"specs.opening_style".to_owned()));
    }
}
