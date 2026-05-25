//! Gear packing-list domain model for route-specific pre-hike preparation.

use serde::{Deserialize, Serialize};

use crate::validation::{
    FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
};

/// Writable packing-list metadata supplied by the current user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListDraft {
    pub name: String,
    pub route_name: Option<String>,
    pub duration_label: Option<String>,
}

impl GearPackingListDraft {
    /// Validates and trims packing-list metadata before persistence.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.name =
            normalize_required_text(std::mem::take(&mut self.name), 100, "name", &mut errors);
        self.route_name =
            normalize_optional_text(self.route_name.take(), 100, "route_name", &mut errors);
        self.duration_label = normalize_optional_text(
            self.duration_label.take(),
            80,
            "duration_label",
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// User-owned packing list persisted in the database.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingList {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub route_name: Option<String>,
    pub duration_label: Option<String>,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Aggregate counters shown on packing-list index and detail surfaces.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GearPackingListStats {
    pub item_count: i64,
    pub packed_count: i64,
    pub total_weight_g: i64,
}

/// Packing-list summary returned by paginated list reads.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListSummary {
    pub list: GearPackingList,
    pub stats: GearPackingListStats,
}

/// One item inside a packing list.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListItem {
    pub id: String,
    pub packing_list_id: String,
    pub user_id: String,
    pub gear_id: String,
    pub planned_quantity: i32,
    pub packed_quantity: i32,
    pub packed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Full packing-list detail with item rows and aggregate counters.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListDetail {
    pub list: GearPackingList,
    pub stats: GearPackingListStats,
    pub items: Vec<GearPackingListItem>,
}

/// Normalizes a bulk add request into a deduplicated list of gear ids.
pub fn normalize_gear_ids(values: Vec<String>) -> Result<Vec<String>, ValidationError> {
    let mut errors = Vec::new();
    let mut normalized = Vec::new();
    for raw in values {
        let value = raw.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 128 {
            errors.push(FieldViolation::new(
                "gear_ids",
                "each gear id must be at most 128 characters",
            ));
            continue;
        }
        if !normalized.iter().any(|existing| existing == value) {
            normalized.push(value.to_owned());
        }
    }
    if normalized.is_empty() {
        errors.push(FieldViolation::new(
            "gear_ids",
            "must contain at least one id",
        ));
    }
    if normalized.len() > 200 {
        errors.push(FieldViolation::new(
            "gear_ids",
            "must contain at most 200 ids",
        ));
        normalized.truncate(200);
    }
    if errors.is_empty() {
        Ok(normalized)
    } else {
        Err(ValidationError::new(errors))
    }
}
