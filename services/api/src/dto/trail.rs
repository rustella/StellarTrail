//! HTTP DTOs for reusable trails, map state, and context annotations.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use stellartrail_domain::{
    trail::{
        MapAnnotation, MapAnnotationDraft, MapAnnotationPatch, MapTrailLink, Trail, TrailBounds,
        TrailMetadataPatch, TrailSummary, TripOverviewTrail,
    },
    trip::FieldVersions,
};

use crate::config::{MapConfig, MapStyleConfig};

/// Current user's trail library list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListTrailsResponse {
    pub items: Vec<TrailSummary>,
}

/// Sparse metadata patch for an owned trail.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTrailRequest {
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
}

impl UpdateTrailRequest {
    pub fn into_patch(self) -> TrailMetadataPatch {
        TrailMetadataPatch {
            display_name: self.display_name,
            description: self.description,
        }
    }
}

/// Request body for linking an existing trail into a map context.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrailLinkRequest {
    pub trail_id: String,
}

/// Client-visible map configuration. Service tokens are never included.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapStyleOption {
    pub id: String,
    pub label: String,
    pub style_url: String,
    pub request_origins: Vec<String>,
}

impl MapStyleOption {
    fn from_config(config: &MapStyleConfig, public_origin: &str) -> Self {
        Self {
            id: config.id.clone(),
            label: config.label.clone(),
            style_url: hosted_style_url(public_origin, &config.id),
            request_origins: config.request_origins.clone(),
        }
    }
}

/// Client-visible map configuration. Service tokens are never included.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapConfigResponse {
    pub provider: String,
    pub public_key: Option<String>,
    pub coordinate_system: String,
    pub enabled: bool,
    pub styles: Vec<MapStyleOption>,
    pub default_style_id: String,
}

impl MapConfigResponse {
    pub fn from_config(config: &MapConfig, public_origin: &str) -> Self {
        let public_key = config.public_key.clone();
        Self {
            provider: config.provider.clone(),
            enabled: public_key.as_ref().is_some_and(|key| !key.is_empty()),
            public_key,
            coordinate_system: "WGS84".to_owned(),
            styles: config
                .styles
                .iter()
                .map(|style| MapStyleOption::from_config(style, public_origin))
                .collect(),
            default_style_id: config.default_style_id.clone(),
        }
    }
}

fn hosted_style_url(public_origin: &str, style_id: &str) -> String {
    format!(
        "{}/api/v1/map/styles/{}/style.json",
        public_origin.trim_end_matches('/'),
        style_id
    )
}

/// Trip map state including linked trails and trip-scoped annotations.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripMapStateResponse {
    pub map: MapConfigResponse,
    pub trails: Vec<MapTrailLink>,
    pub annotations: Vec<MapAnnotation>,
}

/// Outdoor experience map state including linked trails and context-scoped annotations.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutdoorExperienceMapStateResponse {
    pub map: MapConfigResponse,
    pub trails: Vec<MapTrailLink>,
    pub annotations: Vec<MapAnnotation>,
}

/// One map-renderable trail in the user's all-trips overview map.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripOverviewMapTrail {
    pub trip_id: String,
    pub trip_title: String,
    pub trip_start_date: Option<String>,
    pub trip_end_date: Option<String>,
    pub trail_id: String,
    pub linked_by_user_id: String,
    pub role: String,
    pub sort_order: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub trail: TrailSummary,
    pub simplified_geojson: JsonValue,
}

impl TripOverviewMapTrail {
    pub fn from_domain(value: TripOverviewTrail, simplified_geojson: JsonValue) -> Self {
        Self {
            trip_id: value.trip_id,
            trip_title: value.trip_title,
            trip_start_date: value.trip_start_date,
            trip_end_date: value.trip_end_date,
            trail_id: value.link.trail_id,
            linked_by_user_id: value.link.linked_by_user_id,
            role: value.link.role,
            sort_order: value.link.sort_order,
            notes: value.link.notes,
            created_at: value.link.created_at,
            updated_at: value.link.updated_at,
            trail: value.link.trail,
            simplified_geojson,
        }
    }
}

/// Aggregate counters for the all-trips overview map.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TripsMapOverviewStats {
    pub trip_count: usize,
    pub trail_count: usize,
    pub rendered_point_count: usize,
    pub total_distance_m: f64,
    pub total_ascent_m: f64,
    pub total_descent_m: f64,
}

/// All-trips overview map response optimized for one request and one map source.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripsMapOverviewResponse {
    pub map: MapConfigResponse,
    pub trails: Vec<TripOverviewMapTrail>,
    pub bounds: Option<TrailBounds>,
    pub stats: TripsMapOverviewStats,
    pub truncated: bool,
}

/// Request body for creating a map annotation in one context.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapAnnotationRequest {
    pub trail_id: Option<String>,
    pub lng: f64,
    pub lat: f64,
    pub elevation_m: Option<f64>,
    pub trail_point_index: Option<i64>,
    pub annotation_type: String,
    pub title: Option<String>,
    pub note: Option<String>,
}

impl MapAnnotationRequest {
    pub fn into_draft(self) -> MapAnnotationDraft {
        MapAnnotationDraft {
            trail_id: self.trail_id,
            lng: self.lng,
            lat: self.lat,
            elevation_m: self.elevation_m,
            trail_point_index: self.trail_point_index,
            annotation_type: self.annotation_type,
            title: self.title,
            note: self.note,
        }
    }
}

/// Request body for patching annotation text/type metadata.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateMapAnnotationRequest {
    pub annotation_type: Option<String>,
    pub title: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub elevation_m: Option<Option<f64>>,
    #[serde(default)]
    pub base_field_versions: FieldVersions,
    #[serde(default)]
    pub force_fields: BTreeSet<String>,
}

impl UpdateMapAnnotationRequest {
    pub fn into_parts(self) -> (MapAnnotationPatch, FieldVersions, BTreeSet<String>) {
        (
            MapAnnotationPatch {
                annotation_type: self.annotation_type,
                title: self.title,
                note: self.note,
                elevation_m: self.elevation_m,
            },
            self.base_field_versions,
            self.force_fields,
        )
    }
}

/// Created trail response used by upload routes.
pub type TrailUploadResponse = Trail;
