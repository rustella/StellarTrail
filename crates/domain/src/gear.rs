//! Gear inventory domain model module defining categories, statuses, draft validation, statistics, and route gear suggestions.

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime, format_description::well_known::Iso8601};

use crate::validation::{
    FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
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

/// Complete gear domain object representing one user gear record from the database.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearItem {
    pub id: String,
    pub user_id: String,
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub capacity: Option<String>,
    pub size: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub warmth_index: Option<String>,
    pub waterproof_index: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub expiry_or_warranty_date: Option<String>,
    pub purchase_location: Option<String>,
    pub status: GearStatus,
    pub storage_location: Option<String>,
    pub tags: Vec<String>,
    pub share_enabled: bool,
    pub share_status: GearShareStatus,
    pub notes: Option<String>,
    pub archived_at: Option<String>,
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
    pub color: Option<String>,
    pub material: Option<String>,
    pub capacity: Option<String>,
    pub size: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub warmth_index: Option<String>,
    pub waterproof_index: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub expiry_or_warranty_date: Option<String>,
    pub purchase_location: Option<String>,
    pub status: GearStatus,
    pub storage_location: Option<String>,
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
        self.color = normalize_optional_text(self.color.take(), 40, "color", &mut errors);
        self.material = normalize_optional_text(self.material.take(), 100, "material", &mut errors);
        self.capacity = normalize_optional_text(self.capacity.take(), 50, "capacity", &mut errors);
        self.size = normalize_optional_text(self.size.take(), 50, "size", &mut errors);
        self.description =
            normalize_optional_text(self.description.take(), 100, "description", &mut errors);
        self.warmth_index =
            normalize_optional_text(self.warmth_index.take(), 30, "warmth_index", &mut errors);
        self.waterproof_index = normalize_optional_text(
            self.waterproof_index.take(),
            30,
            "waterproof_index",
            &mut errors,
        );
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
        self.notes = normalize_optional_text(self.notes.take(), 100, "notes", &mut errors);

        if let Some(weight_g) = self.weight_g {
            if !(0..=1_000_000).contains(&weight_g) {
                errors.push(FieldViolation::new(
                    "weight_g",
                    "must be between 0 and 1000000",
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

        validate_date(self.purchase_date.as_deref(), "purchase_date", &mut errors);
        validate_date(
            self.expiry_or_warranty_date.as_deref(),
            "expiry_or_warranty_date",
            &mut errors,
        );
        if let (Some(purchase_date), Some(expiry_date)) = (
            parse_date(self.purchase_date.as_deref()),
            parse_date(self.expiry_or_warranty_date.as_deref()),
        ) {
            if expiry_date < purchase_date {
                errors.push(FieldViolation::new(
                    "expiry_or_warranty_date",
                    "must not be earlier than purchase_date",
                ));
            }
        }

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

/// Stable data boundary for `RouteGearSuggestion`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteGearSuggestion {
    pub route_id: String,
    pub gear_category: String,
    pub gear_name: String,
    pub required_level: RequiredLevel,
    pub reason: String,
}

/// Stable enum boundary for `RequiredLevel`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredLevel {
    Required,
    Recommended,
    Optional,
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
            color: Some("".to_owned()),
            material: None,
            capacity: None,
            size: None,
            description: Some("夜间备用".to_owned()),
            weight_g: Some(85),
            warmth_index: None,
            waterproof_index: Some("IPX4".to_owned()),
            purchase_date: Some("2026-01-01".to_owned()),
            purchase_price_cents: Some(9900),
            expiry_or_warranty_date: Some("2027-01-01".to_owned()),
            purchase_location: Some("京东".to_owned()),
            status: GearStatus::Available,
            storage_location: Some("装备柜".to_owned()),
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
        assert_eq!(draft.color, None);
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
        draft.purchase_price_cents = Some(-1);
        draft.purchase_date = Some("2026/01/01".to_owned());

        let error = draft.validate_and_normalize().unwrap_err();

        let fields: Vec<_> = error.fields.into_iter().map(|field| field.field).collect();
        assert!(fields.contains(&"name".to_owned()));
        assert!(fields.contains(&"description".to_owned()));
        assert!(fields.contains(&"weight_g".to_owned()));
        assert!(fields.contains(&"purchase_price_cents".to_owned()));
        assert!(fields.contains(&"purchase_date".to_owned()));
    }
}
