//! HTTP DTOs for user-owned gear packing lists and pre-hike packed-state checks.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use stellartrail_domain::{
    gear::GearItem,
    gear_packing::{
        GearPackingListDetail, GearPackingListDraft, GearPackingListItem, GearPackingListStats,
        GearPackingListSummary, normalize_gear_ids,
    },
    validation::{FieldViolation, ValidationError},
};

use crate::dto::gear::GearSummaryResponse;

/// Query parameters for packing-list index pagination.
#[derive(Debug, Deserialize)]
pub struct ListGearPackingListsQuery {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Request body used to create a packing list.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGearPackingListRequest {
    pub name: String,
    pub route_name: Option<String>,
    pub duration_label: Option<String>,
}

impl CreateGearPackingListRequest {
    /// Converts the HTTP body into a validated domain draft.
    pub fn into_draft(self) -> GearPackingListDraft {
        GearPackingListDraft {
            name: self.name,
            route_name: self.route_name,
            duration_label: self.duration_label,
        }
    }
}

/// Request body used to update packing-list metadata.
pub type UpdateGearPackingListRequest = CreateGearPackingListRequest;

/// Request body for bulk-adding gear to a packing list.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddGearPackingItemsRequest {
    pub gear_ids: Vec<String>,
}

impl AddGearPackingItemsRequest {
    /// Returns trimmed, deduplicated gear ids or a validation error.
    pub fn normalized_gear_ids(self) -> Result<Vec<String>, ValidationError> {
        normalize_gear_ids(self.gear_ids)
    }
}

/// Request body for toggling one packing-list item.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateGearPackingItemRequest {
    pub packed: Option<bool>,
    pub planned_quantity: Option<i32>,
    pub packed_quantity: Option<i32>,
}

impl UpdateGearPackingItemRequest {
    /// Validates quantity bounds and requires at least one updated field.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        if self.packed.is_none()
            && self.planned_quantity.is_none()
            && self.packed_quantity.is_none()
        {
            errors.push(FieldViolation::new(
                "item",
                "must include packed, planned_quantity, or packed_quantity",
            ));
        }
        if self
            .planned_quantity
            .is_some_and(|value| !(1..=9_999).contains(&value))
        {
            errors.push(FieldViolation::new(
                "planned_quantity",
                "must be between 1 and 9999",
            ));
        }
        if self
            .packed_quantity
            .is_some_and(|value| !(0..=9_999).contains(&value))
        {
            errors.push(FieldViolation::new(
                "packed_quantity",
                "must be between 0 and 9999",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Packing-list aggregate counters flattened for API clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListStatsResponse {
    pub item_count: i64,
    pub packed_count: i64,
    pub total_weight_g: i64,
}

impl From<GearPackingListStats> for GearPackingListStatsResponse {
    fn from(value: GearPackingListStats) -> Self {
        Self {
            item_count: value.item_count,
            packed_count: value.packed_count,
            total_weight_g: value.total_weight_g,
        }
    }
}

/// Packing-list summary returned by index reads.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListSummaryResponse {
    pub id: String,
    pub name: String,
    pub route_name: Option<String>,
    pub duration_label: Option<String>,
    pub item_count: i64,
    pub packed_count: i64,
    pub total_weight_g: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<GearPackingListSummary> for GearPackingListSummaryResponse {
    fn from(value: GearPackingListSummary) -> Self {
        Self {
            id: value.list.id,
            name: value.list.name,
            route_name: value.list.route_name,
            duration_label: value.list.duration_label,
            item_count: value.stats.item_count,
            packed_count: value.stats.packed_count,
            total_weight_g: value.stats.total_weight_g,
            created_at: value.list.created_at,
            updated_at: value.list.updated_at,
        }
    }
}

/// Paginated packing-list index response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGearPackingListsResponse {
    pub items: Vec<GearPackingListSummaryResponse>,
    pub next_cursor: Option<String>,
}

/// One packing-list item with a gear summary and current packed state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListItemResponse {
    pub id: String,
    pub gear_id: String,
    pub planned_quantity: i32,
    pub packed_quantity: i32,
    pub packed: bool,
    pub unavailable: bool,
    pub unavailable_reason: Option<String>,
    pub gear: GearSummaryResponse,
    pub created_at: String,
    pub updated_at: String,
}

impl GearPackingListItemResponse {
    /// Builds an item response while deriving unavailable state from the linked gear row.
    pub fn from_item_and_gear(
        item: GearPackingListItem,
        gear: &GearItem,
        tag_colors: &HashMap<String, String>,
    ) -> Self {
        let unavailable_reason = gear.is_deleted.then(|| "deleted".to_owned());
        Self {
            id: item.id,
            gear_id: item.gear_id,
            planned_quantity: item.planned_quantity,
            packed_quantity: item.packed_quantity,
            packed: item.packed,
            unavailable: unavailable_reason.is_some(),
            unavailable_reason,
            gear: GearSummaryResponse::from_item(gear, tag_colors),
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

/// Full packing-list detail response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearPackingListDetailResponse {
    pub id: String,
    pub name: String,
    pub route_name: Option<String>,
    pub duration_label: Option<String>,
    pub stats: GearPackingListStatsResponse,
    pub items: Vec<GearPackingListItemResponse>,
    pub created_at: String,
    pub updated_at: String,
}

impl GearPackingListDetailResponse {
    /// Flattens a repository detail plus rendered item responses into the API shape.
    pub fn from_detail(
        detail: GearPackingListDetail,
        items: Vec<GearPackingListItemResponse>,
    ) -> Self {
        Self {
            id: detail.list.id,
            name: detail.list.name,
            route_name: detail.list.route_name,
            duration_label: detail.list.duration_label,
            stats: detail.stats.into(),
            items,
            created_at: detail.list.created_at,
            updated_at: detail.list.updated_at,
        }
    }
}
