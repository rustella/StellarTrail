//! HTTP DTOs for solo/team trips and editable section records.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use stellartrail_domain::{
    trip::{
        FieldVersions, OutdoorExperience, OutdoorExperienceDraft, TripDraft, TripInvitation,
        TripSectionKey, TripSummary, TripTimeBucket, TripType,
    },
    validation::{FieldViolation, ValidationError},
};

/// List response for profile-visible outdoor experiences.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListOutdoorExperiencesResponse {
    pub items: Vec<OutdoorExperience>,
}

/// Request body for creating or replacing a structured outdoor experience.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutdoorExperienceRequest {
    pub title: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub day_count: Option<i64>,
    pub companion_count: Option<i64>,
    pub route_summary: Option<String>,
    pub gear_summary: Option<String>,
    pub food_summary: Option<String>,
    pub budget_summary: Option<String>,
    pub notes: Option<String>,
}

impl OutdoorExperienceRequest {
    pub fn into_draft(self) -> OutdoorExperienceDraft {
        OutdoorExperienceDraft {
            title: self.title,
            start_date: self.start_date,
            end_date: self.end_date,
            day_count: self.day_count,
            companion_count: self.companion_count,
            route_summary: self.route_summary,
            gear_summary: self.gear_summary,
            food_summary: self.food_summary,
            budget_summary: self.budget_summary,
            notes: self.notes,
        }
    }
}

/// Query parameters for the current user's trip index.
#[derive(Debug, Deserialize)]
pub struct ListTripsQuery {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub bucket: Option<TripBucketQuery>,
    pub trip_type: Option<TripTypeQuery>,
    pub today: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TripBucketQuery {
    Ongoing,
    Upcoming,
    Past,
    Undated,
    All,
}

impl TripBucketQuery {
    pub fn into_filter(self) -> Option<TripTimeBucket> {
        match self {
            Self::Ongoing => Some(TripTimeBucket::Ongoing),
            Self::Upcoming => Some(TripTimeBucket::Upcoming),
            Self::Past => Some(TripTimeBucket::Past),
            Self::Undated => Some(TripTimeBucket::Undated),
            Self::All => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TripTypeQuery {
    Solo,
    Team,
    All,
}

impl TripTypeQuery {
    pub fn into_filter(self) -> Option<TripType> {
        match self {
            Self::Solo => Some(TripType::Solo),
            Self::Team => Some(TripType::Team),
            Self::All => None,
        }
    }
}

/// Paginated current-user trip response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListTripsResponse {
    pub items: Vec<TripSummary>,
    pub next_cursor: Option<String>,
}

/// Query parameters for the homepage trip reminder.
#[derive(Debug, Deserialize)]
pub struct TripHomeHighlightQuery {
    pub today: Option<String>,
}

/// Status bucket for the homepage trip reminder.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TripHomeHighlightStatus {
    Ongoing,
    Upcoming,
}

/// Single homepage trip reminder item.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripHomeHighlightItem {
    pub trip: TripSummary,
    pub status: TripHomeHighlightStatus,
    pub days_until_start: i64,
    pub days_until_end: i64,
}

/// Response body for the homepage trip reminder.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripHomeHighlightResponse {
    pub item: Option<TripHomeHighlightItem>,
}

/// Request body used to create a new trip.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTripRequest {
    pub trip_type: TripType,
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub route_use_slope_adjustment: bool,
    #[serde(default)]
    pub route_use_high_altitude_adjustment: bool,
    pub route_start_altitude_m: Option<i32>,
}

impl CreateTripRequest {
    /// Converts the HTTP body into a domain draft.
    pub fn into_draft(self) -> TripDraft {
        TripDraft {
            trip_type: self.trip_type,
            title: self.title,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            route_use_slope_adjustment: self.route_use_slope_adjustment,
            route_use_high_altitude_adjustment: self.route_use_high_altitude_adjustment,
            route_start_altitude_m: self.route_start_altitude_m,
        }
    }
}

/// Field-level metadata shared by PATCH requests.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConflictPatchMeta {
    #[serde(default)]
    pub base_field_versions: FieldVersions,
    #[serde(default)]
    pub force_fields: BTreeSet<String>,
}

/// Request body used to patch root trip metadata.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTripRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub start_date: Option<Option<String>>,
    pub end_date: Option<Option<String>>,
    pub route_use_slope_adjustment: Option<bool>,
    pub route_use_high_altitude_adjustment: Option<bool>,
    pub route_start_altitude_m: Option<Option<i32>>,
    #[serde(flatten)]
    pub meta: ConflictPatchMeta,
}

impl UpdateTripRequest {
    /// Builds a sparse change map so unrelated fields can be edited concurrently.
    pub fn into_changes(
        self,
    ) -> Result<(BTreeMap<String, JsonValue>, ConflictPatchMeta), ValidationError> {
        let mut changes = BTreeMap::new();
        if let Some(title) = self.title {
            changes.insert("title".to_owned(), JsonValue::String(title));
        }
        if let Some(description) = self.description {
            changes.insert(
                "description".to_owned(),
                description
                    .map(JsonValue::String)
                    .unwrap_or(JsonValue::Null),
            );
        }
        if let Some(start_date) = self.start_date {
            changes.insert(
                "start_date".to_owned(),
                start_date.map(JsonValue::String).unwrap_or(JsonValue::Null),
            );
        }
        if let Some(end_date) = self.end_date {
            changes.insert(
                "end_date".to_owned(),
                end_date.map(JsonValue::String).unwrap_or(JsonValue::Null),
            );
        }
        if let Some(use_slope) = self.route_use_slope_adjustment {
            changes.insert(
                "route_use_slope_adjustment".to_owned(),
                JsonValue::Bool(use_slope),
            );
        }
        if let Some(use_high_altitude) = self.route_use_high_altitude_adjustment {
            changes.insert(
                "route_use_high_altitude_adjustment".to_owned(),
                JsonValue::Bool(use_high_altitude),
            );
        }
        if let Some(start_altitude) = self.route_start_altitude_m {
            changes.insert(
                "route_start_altitude_m".to_owned(),
                start_altitude
                    .map(|value| JsonValue::Number(value.into()))
                    .unwrap_or(JsonValue::Null),
            );
        }
        if changes.is_empty() {
            return Err(ValidationError::single(
                "fields",
                "at least one editable field is required",
            ));
        }
        Ok((changes, self.meta))
    }
}

/// Request body for changing visible trip sections.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTripSectionsRequest {
    pub enabled_sections: Vec<TripSectionKey>,
    #[serde(flatten)]
    pub meta: ConflictPatchMeta,
}

/// Request body for importing one existing packing list as member gear snapshots.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportPackingListRequest {
    pub packing_list_id: String,
}

/// Response body after an owner creates an invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateTripInvitationResponse {
    pub invitation: TripInvitation,
}

/// Generic create request used by typed section-record endpoints.
#[derive(Debug, Deserialize)]
pub struct CreateTripRecordRequest {
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(flatten)]
    pub payload: JsonMap<String, JsonValue>,
}

impl CreateTripRecordRequest {
    /// Returns the free-form payload as a JSON object.
    pub fn into_payload(self) -> JsonValue {
        JsonValue::Object(self.payload)
    }
}

/// Generic sparse PATCH request for member profiles and section records.
#[derive(Debug, Deserialize)]
pub struct PatchTripFieldsRequest {
    #[serde(default)]
    pub base_field_versions: FieldVersions,
    #[serde(default)]
    pub force_fields: BTreeSet<String>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, JsonValue>,
}

impl PatchTripFieldsRequest {
    /// Validates that the sparse PATCH contains at least one domain field.
    pub fn into_parts(
        self,
    ) -> Result<(BTreeMap<String, JsonValue>, ConflictPatchMeta), ValidationError> {
        if self.fields.is_empty() {
            return Err(ValidationError::new(vec![FieldViolation::new(
                "fields",
                "at least one editable field is required",
            )]));
        }
        Ok((
            self.fields,
            ConflictPatchMeta {
                base_field_versions: self.base_field_versions,
                force_fields: self.force_fields,
            },
        ))
    }
}
