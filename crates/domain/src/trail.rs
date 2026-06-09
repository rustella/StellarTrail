//! Trail domain models for reusable uploaded route assets, map links, and context annotations.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    trip::FieldVersions,
    validation::{
        FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
    },
};

/// Supported source file formats for uploaded trails.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailSourceFormat {
    Gpx,
    Kml,
    Fit,
}

impl TrailSourceFormat {
    /// Returns the stable storage key used in database rows and API payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gpx => "gpx",
            Self::Kml => "kml",
            Self::Fit => "fit",
        }
    }

    /// Parses a database source-format key.
    pub fn from_key(value: &str) -> Result<Self, ValidationError> {
        match value {
            "gpx" => Ok(Self::Gpx),
            "kml" => Ok(Self::Kml),
            "fit" => Ok(Self::Fit),
            _ => Err(ValidationError::single(
                "source_format",
                "must be one of gpx, kml, or fit",
            )),
        }
    }
}

/// A single WGS84 trail point. GeoJSON output uses `[lng, lat, elevation?]`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrailPoint {
    pub lng: f64,
    pub lat: f64,
    pub elevation_m: Option<f64>,
    pub time: Option<String>,
}

/// WGS84 bounding box for map viewport fitting.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrailBounds {
    pub min_lng: f64,
    pub min_lat: f64,
    pub max_lng: f64,
    pub max_lat: f64,
}

/// Persisted reusable trail asset with normalized geometry and summary metrics.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trail {
    pub id: String,
    pub owner_user_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub source_format: TrailSourceFormat,
    pub original_filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256_hex: String,
    pub bucket: String,
    pub object_key: String,
    pub normalized_points: Vec<TrailPoint>,
    pub simplified_geojson: JsonValue,
    pub bounds: Option<TrailBounds>,
    pub distance_m: f64,
    pub ascent_m: f64,
    pub descent_m: f64,
    pub min_elevation_m: Option<f64>,
    pub max_elevation_m: Option<f64>,
    pub start_elevation_m: Option<f64>,
    pub end_elevation_m: Option<f64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub point_count: i64,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// List-friendly trail asset shape that omits full point arrays.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrailSummary {
    pub id: String,
    pub owner_user_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub source_format: TrailSourceFormat,
    pub original_filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256_hex: String,
    pub bounds: Option<TrailBounds>,
    pub distance_m: f64,
    pub ascent_m: f64,
    pub descent_m: f64,
    pub min_elevation_m: Option<f64>,
    pub max_elevation_m: Option<f64>,
    pub start_elevation_m: Option<f64>,
    pub end_elevation_m: Option<f64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub point_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Trail> for TrailSummary {
    fn from(value: &Trail) -> Self {
        Self {
            id: value.id.clone(),
            owner_user_id: value.owner_user_id.clone(),
            display_name: value.display_name.clone(),
            description: value.description.clone(),
            source_format: value.source_format,
            original_filename: value.original_filename.clone(),
            content_type: value.content_type.clone(),
            size_bytes: value.size_bytes,
            sha256_hex: value.sha256_hex.clone(),
            bounds: value.bounds.clone(),
            distance_m: value.distance_m,
            ascent_m: value.ascent_m,
            descent_m: value.descent_m,
            min_elevation_m: value.min_elevation_m,
            max_elevation_m: value.max_elevation_m,
            start_elevation_m: value.start_elevation_m,
            end_elevation_m: value.end_elevation_m,
            start_time: value.start_time.clone(),
            end_time: value.end_time.clone(),
            point_count: value.point_count,
            created_at: value.created_at.clone(),
            updated_at: value.updated_at.clone(),
        }
    }
}

/// Normalized data used to create a trail row after safe file parsing.
#[derive(Clone, Debug)]
pub struct TrailCreateDraft {
    pub display_name: String,
    pub description: Option<String>,
    pub source_format: TrailSourceFormat,
    pub original_filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256_hex: String,
    pub bucket: String,
    pub object_key: String,
    pub normalized_points: Vec<TrailPoint>,
    pub simplified_geojson: JsonValue,
    pub bounds: Option<TrailBounds>,
    pub distance_m: f64,
    pub ascent_m: f64,
    pub descent_m: f64,
    pub min_elevation_m: Option<f64>,
    pub max_elevation_m: Option<f64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub point_count: i64,
}

impl TrailCreateDraft {
    /// Normalizes user-visible metadata before persistence.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.display_name = normalize_required_text(
            std::mem::take(&mut self.display_name),
            120,
            "display_name",
            &mut errors,
        );
        self.description =
            normalize_optional_text(self.description.take(), 1000, "description", &mut errors);
        if self.point_count <= 0 || self.normalized_points.is_empty() {
            errors.push(FieldViolation::new(
                "file",
                "must contain valid coordinates",
            ));
        }
        if self.distance_m.is_sign_negative() || !self.distance_m.is_finite() {
            errors.push(FieldViolation::new("distance_m", "must be a finite value"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Patchable user-owned trail metadata.
#[derive(Clone, Debug, Default)]
pub struct TrailMetadataPatch {
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
}

impl TrailMetadataPatch {
    /// Normalizes sparse metadata fields before a patch is persisted.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        if let Some(display_name) = self.display_name.take() {
            self.display_name = Some(normalize_required_text(
                display_name,
                120,
                "display_name",
                &mut errors,
            ));
        }
        if let Some(description) = self.description.take() {
            self.description = Some(normalize_optional_text(
                description,
                1000,
                "description",
                &mut errors,
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// A context-level link between a reusable trail and a trip or outdoor experience.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrailLink {
    pub trail_id: String,
    pub linked_by_user_id: String,
    pub role: String,
    pub sort_order: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub trail: TrailSummary,
}

/// Map-ready trail link that includes already-simplified render geometry.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapTrailLink {
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

/// A trip-scoped trail row used by the all-trips map overview endpoint.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripOverviewTrail {
    pub trip_id: String,
    pub trip_title: String,
    pub trip_start_date: Option<String>,
    pub trip_end_date: Option<String>,
    pub link: MapTrailLink,
}

/// Context-free annotation asset that can be attached to a trip or outdoor experience.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapAnnotation {
    pub id: String,
    pub owner_user_id: String,
    pub trail_id: Option<String>,
    pub lng: f64,
    pub lat: f64,
    pub elevation_m: Option<f64>,
    pub trail_point_index: Option<i64>,
    pub annotation_type: String,
    pub title: Option<String>,
    pub note: Option<String>,
    pub field_versions: FieldVersions,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Writable annotation payload accepted inside map contexts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapAnnotationDraft {
    pub trail_id: Option<String>,
    pub lng: f64,
    pub lat: f64,
    pub elevation_m: Option<f64>,
    pub trail_point_index: Option<i64>,
    pub annotation_type: String,
    pub title: Option<String>,
    pub note: Option<String>,
}

impl MapAnnotationDraft {
    /// Validates WGS84 coordinates, optional trail point indexes, and user-visible text.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        validate_wgs84(self.lng, self.lat, &mut errors);
        if let Some(elevation) = self.elevation_m
            && !elevation.is_finite()
        {
            errors.push(FieldViolation::new("elevation_m", "must be a finite value"));
        }
        if self.trail_point_index.is_some_and(|index| index < 0) {
            errors.push(FieldViolation::new(
                "trail_point_index",
                "must be greater than or equal to 0",
            ));
        }
        self.annotation_type = normalize_required_text(
            std::mem::take(&mut self.annotation_type),
            40,
            "annotation_type",
            &mut errors,
        );
        self.title = normalize_optional_text(self.title.take(), 120, "title", &mut errors);
        self.note = normalize_optional_text(self.note.take(), 2000, "note", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Sparse annotation metadata patch.
#[derive(Clone, Debug, Default)]
pub struct MapAnnotationPatch {
    pub annotation_type: Option<String>,
    pub title: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub elevation_m: Option<Option<f64>>,
}

impl MapAnnotationPatch {
    /// Normalizes sparse annotation changes before persistence.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        if let Some(annotation_type) = self.annotation_type.take() {
            self.annotation_type = Some(normalize_required_text(
                annotation_type,
                40,
                "annotation_type",
                &mut errors,
            ));
        }
        if let Some(title) = self.title.take() {
            self.title = Some(normalize_optional_text(title, 120, "title", &mut errors));
        }
        if let Some(note) = self.note.take() {
            self.note = Some(normalize_optional_text(note, 2000, "note", &mut errors));
        }
        if let Some(Some(elevation)) = self.elevation_m
            && !elevation.is_finite()
        {
            errors.push(FieldViolation::new("elevation_m", "must be a finite value"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }

    /// Returns the field names touched by this patch.
    pub fn touched_fields(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.annotation_type.is_some() {
            fields.push("annotation_type");
        }
        if self.title.is_some() {
            fields.push("title");
        }
        if self.note.is_some() {
            fields.push("note");
        }
        if self.elevation_m.is_some() {
            fields.push("elevation_m");
        }
        fields
    }
}

fn validate_wgs84(lng: f64, lat: f64, errors: &mut Vec<FieldViolation>) {
    if !lng.is_finite() || !(-180.0..=180.0).contains(&lng) {
        errors.push(FieldViolation::new(
            "lng",
            "must be a WGS84 longitude between -180 and 180",
        ));
    }
    if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
        errors.push(FieldViolation::new(
            "lat",
            "must be a WGS84 latitude between -90 and 90",
        ));
    }
}
