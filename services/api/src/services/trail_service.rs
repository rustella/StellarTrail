//! Trail upload service for parsing GPX/KML/KMZ/FIT files into WGS84 geometry, writing source files to object storage, and persisting reusable trail assets.

use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::Path,
};

use fitparser::{FitDataRecord, Value as FitValue, profile::MesgNum};
use geojson::{Feature, Geometry as GeoJsonGeometry, Value as GeoJsonValue};
use kml::{
    Kml,
    types::{Geometry as KmlGeometry, LineString as KmlLineString},
};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use stellartrail_db::repositories::{TrailRepository, TripRepository};
use stellartrail_domain::{
    trail::{MapTrailLink, Trail, TrailBounds, TrailCreateDraft, TrailPoint, TrailSourceFormat},
    validation::FieldViolation,
};
use uuid::Uuid;

use crate::{error::ApiError, object_store::PutObjectRequest, state::AppState};

/// Fully parsed trail geometry and summary metadata before object storage persistence.
#[derive(Clone, Debug)]
pub struct ParsedTrailUpload {
    pub source_format: TrailSourceFormat,
    pub storage_extension: String,
    pub original_filename: String,
    pub display_name: String,
    pub content_type: String,
    pub points: Vec<TrailPoint>,
    pub simplified_geojson: JsonValue,
    pub bounds: Option<TrailBounds>,
    pub distance_m: f64,
    pub ascent_m: f64,
    pub descent_m: f64,
    pub min_elevation_m: Option<f64>,
    pub max_elevation_m: Option<f64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// Parses, stores, and persists a user-owned trail library asset.
pub async fn upload_trail_to_library(
    state: &AppState,
    user_id: &str,
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: Vec<u8>,
) -> Result<Trail, ApiError> {
    let parsed = parse_trail_upload(
        original_filename,
        declared_content_type,
        &bytes,
        state.config().trail.upload_max_bytes,
        state.config().trail.upload_max_points,
        state.config().trail.max_simplified_points,
    )?;
    let sha256_hex = sha256_hex(&bytes);
    let repo = TrailRepository::new(state.db().clone());
    if let Some(existing) = repo.get_owned_by_sha256(user_id, &sha256_hex).await? {
        return Ok(existing);
    }
    persist_parsed_trail(state, user_id, parsed, bytes).await
}

/// Parses, stores, persists, and links a new user-owned trail to an existing trip.
pub async fn upload_trail_to_trip(
    state: &AppState,
    user_id: &str,
    trip_id: &str,
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: Vec<u8>,
) -> Result<Option<MapTrailLink>, ApiError> {
    if TripRepository::new(state.db().clone())
        .detail_for_user(user_id, trip_id)
        .await?
        .is_none()
    {
        return Ok(None);
    }
    let repo = TrailRepository::new(state.db().clone());
    if let Some((links, _)) = repo.trip_map_state(user_id, trip_id).await?
        && links.len() as u64 >= state.config().trail.max_trails_per_trip
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "trail_id",
            format!(
                "trip can link at most {} trails",
                state.config().trail.max_trails_per_trip
            ),
        )]));
    }

    let trail = upload_trail_to_library(
        state,
        user_id,
        original_filename,
        declared_content_type,
        bytes,
    )
    .await?;
    repo.link_trail_to_trip(
        user_id,
        trip_id,
        &trail.id,
        state.config().trail.max_trails_per_trip,
    )
    .await
    .map_err(ApiError::from)
}

/// Parses a supported trail file into normalized WGS84 points and derived map metadata.
pub fn parse_trail_upload(
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: &[u8],
    max_bytes: u64,
    max_points: u64,
    max_simplified_points: u64,
) -> Result<ParsedTrailUpload, ApiError> {
    if bytes.is_empty() {
        return Err(validation_error("file", "must not be empty"));
    }
    if bytes.len() as u64 > max_bytes {
        return Err(ApiError::PayloadTooLarge { max_bytes });
    }
    let file = validate_upload_identity(original_filename, declared_content_type, bytes)?;
    let points = match file.kind {
        UploadFileKind::Gpx => parse_gpx_points(bytes)?,
        UploadFileKind::Kml => parse_kml_points(bytes)?,
        UploadFileKind::Kmz => {
            let kml_bytes = extract_kmz_kml(bytes, max_bytes)?;
            parse_kml_points(&kml_bytes)?
        }
        UploadFileKind::Fit => parse_fit_points(bytes)?,
    };
    if points.is_empty() {
        return Err(validation_error("file", "must contain valid coordinates"));
    }
    if points.len() as u64 > max_points {
        return Err(validation_error(
            "file",
            format!("must contain at most {max_points} points"),
        ));
    }
    let stats = compute_stats(&points);
    let simplified = simplify_points(&points, max_simplified_points as usize);
    Ok(ParsedTrailUpload {
        source_format: file.kind.source_format(),
        storage_extension: file.kind.storage_extension().to_owned(),
        original_filename: file.original_filename,
        display_name: file.display_name,
        content_type: file.content_type,
        simplified_geojson: trail_geojson(&simplified)?,
        points,
        bounds: stats.bounds,
        distance_m: stats.distance_m,
        ascent_m: stats.ascent_m,
        descent_m: stats.descent_m,
        min_elevation_m: stats.min_elevation_m,
        max_elevation_m: stats.max_elevation_m,
        start_time: stats.start_time,
        end_time: stats.end_time,
    })
}

async fn persist_parsed_trail(
    state: &AppState,
    user_id: &str,
    parsed: ParsedTrailUpload,
    bytes: Vec<u8>,
) -> Result<Trail, ApiError> {
    let size_bytes = i64::try_from(bytes.len()).map_err(ApiError::internal)?;
    let sha256_hex = sha256_hex(&bytes);
    let object_key = format!(
        "trails/{user_id}/{}.{}",
        Uuid::new_v4(),
        parsed.storage_extension
    );
    let object_store = state.object_store();
    object_store
        .put_object(PutObjectRequest {
            bucket: None,
            object_key: object_key.clone(),
            content_type: parsed.content_type.clone(),
            bytes,
            metadata: HashMap::from([
                (
                    "original_filename".to_owned(),
                    parsed.original_filename.clone(),
                ),
                ("sha256".to_owned(), sha256_hex.clone()),
                (
                    "source_format".to_owned(),
                    parsed.source_format.as_str().to_owned(),
                ),
            ]),
            cache_control: None,
        })
        .await
        .map_err(ApiError::internal)?;

    let point_count = parsed.points.len() as i64;
    let mut draft = TrailCreateDraft {
        display_name: parsed.display_name,
        description: None,
        source_format: parsed.source_format,
        original_filename: parsed.original_filename,
        content_type: parsed.content_type,
        size_bytes,
        sha256_hex,
        bucket: state.config().object_storage.bucket.clone(),
        object_key: object_key.clone(),
        normalized_points: parsed.points,
        simplified_geojson: parsed.simplified_geojson,
        bounds: parsed.bounds,
        distance_m: parsed.distance_m,
        ascent_m: parsed.ascent_m,
        descent_m: parsed.descent_m,
        min_elevation_m: parsed.min_elevation_m,
        max_elevation_m: parsed.max_elevation_m,
        start_time: parsed.start_time,
        end_time: parsed.end_time,
        point_count,
    };
    draft.validate_and_normalize()?;

    let repo = TrailRepository::new(state.db().clone());
    match repo.create(user_id, &draft).await {
        Ok(trail) => Ok(trail),
        Err(error) => {
            let _ = object_store.delete_object(&object_key).await;
            Err(ApiError::from(error))
        }
    }
}

#[derive(Clone, Debug)]
struct UploadIdentity {
    kind: UploadFileKind,
    original_filename: String,
    display_name: String,
    content_type: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UploadFileKind {
    Gpx,
    Kml,
    Kmz,
    Fit,
}

impl UploadFileKind {
    fn from_extension(extension: &str) -> Result<Self, ApiError> {
        match extension {
            "gpx" => Ok(Self::Gpx),
            "kml" => Ok(Self::Kml),
            "kmz" => Ok(Self::Kmz),
            "fit" => Ok(Self::Fit),
            _ => Err(ApiError::UnsupportedMediaType(
                "trail upload supports GPX, KML, KMZ, and FIT files".to_owned(),
            )),
        }
    }

    fn source_format(self) -> TrailSourceFormat {
        match self {
            Self::Gpx => TrailSourceFormat::Gpx,
            Self::Kml | Self::Kmz => TrailSourceFormat::Kml,
            Self::Fit => TrailSourceFormat::Fit,
        }
    }

    fn storage_extension(self) -> &'static str {
        match self {
            Self::Gpx => "gpx",
            Self::Kml => "kml",
            Self::Kmz => "kmz",
            Self::Fit => "fit",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Gpx => "GPX",
            Self::Kml => "KML",
            Self::Kmz => "KMZ",
            Self::Fit => "FIT",
        }
    }
}

fn validate_upload_identity(
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: &[u8],
) -> Result<UploadIdentity, ApiError> {
    let original_filename = original_filename
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| validation_error("file", "filename is required"))?;
    let extension = Path::new(original_filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .ok_or_else(|| validation_error("file", "filename extension is required"))?;
    let kind = UploadFileKind::from_extension(extension.as_str())?;
    let content_type = normalized_content_type(declared_content_type, kind);
    validate_declared_content_type(declared_content_type, kind)?;
    validate_magic(bytes, kind)?;
    Ok(UploadIdentity {
        kind,
        original_filename: safe_filename(original_filename),
        display_name: display_name_from_filename(original_filename),
        content_type,
    })
}

fn validate_declared_content_type(
    declared_content_type: Option<&str>,
    kind: UploadFileKind,
) -> Result<(), ApiError> {
    let Some(content_type) = declared_content_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let allowed = match kind {
        UploadFileKind::Gpx => [
            "application/gpx+xml",
            "application/xml",
            "text/xml",
            "text/plain",
            "application/octet-stream",
        ]
        .as_slice(),
        UploadFileKind::Kml => [
            "application/vnd.google-earth.kml+xml",
            "application/xml",
            "text/xml",
            "text/plain",
            "application/octet-stream",
        ]
        .as_slice(),
        UploadFileKind::Kmz => [
            "application/vnd.google-earth.kmz",
            "application/kmz",
            "application/x-kmz",
            "application/zip",
            "application/x-zip-compressed",
            "application/octet-stream",
        ]
        .as_slice(),
        UploadFileKind::Fit => [
            "application/vnd.ant.fit",
            "application/fit",
            "application/octet-stream",
        ]
        .as_slice(),
    };
    if allowed
        .iter()
        .any(|allowed_type| content_type.eq_ignore_ascii_case(allowed_type))
    {
        Ok(())
    } else {
        Err(ApiError::UnsupportedMediaType(format!(
            "content type {content_type} does not match {} upload",
            kind.label()
        )))
    }
}

fn normalized_content_type(declared_content_type: Option<&str>, kind: UploadFileKind) -> String {
    if kind == UploadFileKind::Kmz {
        return "application/vnd.google-earth.kmz".to_owned();
    }
    declared_content_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| match kind {
            UploadFileKind::Gpx => "application/gpx+xml".to_owned(),
            UploadFileKind::Kml => "application/vnd.google-earth.kml+xml".to_owned(),
            UploadFileKind::Kmz => unreachable!("KMZ content type is canonicalized above"),
            UploadFileKind::Fit => "application/vnd.ant.fit".to_owned(),
        })
}

fn validate_magic(bytes: &[u8], kind: UploadFileKind) -> Result<(), ApiError> {
    match kind {
        UploadFileKind::Gpx => {
            let root = xml_root_tag(bytes)?;
            if root.as_deref() == Some("gpx") {
                Ok(())
            } else {
                Err(validation_error("file", "GPX root tag must be gpx"))
            }
        }
        UploadFileKind::Kml => {
            let root = xml_root_tag(bytes)?;
            if root.as_deref() == Some("kml") {
                Ok(())
            } else {
                Err(validation_error("file", "KML root tag must be kml"))
            }
        }
        UploadFileKind::Kmz => {
            if has_zip_magic(bytes) {
                Ok(())
            } else {
                Err(validation_error("file", "KMZ archive must be a ZIP file"))
            }
        }
        UploadFileKind::Fit => {
            if bytes.len() >= 12 && &bytes[8..12] == b".FIT" {
                Ok(())
            } else {
                Err(validation_error("file", "FIT header magic must be .FIT"))
            }
        }
    }
}

fn has_zip_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && matches!(&bytes[..4], b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08")
}

fn extract_kmz_kml(bytes: &[u8], max_kml_bytes: u64) -> Result<Vec<u8>, ApiError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| ApiError::BadRequest(format!("invalid KMZ archive: {error}")))?;
    let mut fallback_index = None;
    let mut doc_index = None;

    for index in 0..archive.len() {
        let entry_name = {
            let entry = archive.by_index(index).map_err(|error| {
                ApiError::BadRequest(format!("invalid KMZ archive entry: {error}"))
            })?;
            entry.name().to_owned()
        };
        let normalized_name = entry_name
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_ascii_lowercase();
        if normalized_name.ends_with('/') {
            continue;
        }
        if normalized_name == "doc.kml" {
            doc_index = Some(index);
            break;
        }
        if fallback_index.is_none() && normalized_name.ends_with(".kml") {
            fallback_index = Some(index);
        }
    }

    let kml_index = doc_index
        .or(fallback_index)
        .ok_or_else(|| validation_error("file", "KMZ archive must contain a KML file"))?;
    let mut entry = archive
        .by_index(kml_index)
        .map_err(|error| ApiError::BadRequest(format!("invalid KMZ KML entry: {error}")))?;
    if entry.size() > max_kml_bytes {
        return Err(ApiError::PayloadTooLarge {
            max_bytes: max_kml_bytes,
        });
    }

    let mut kml_bytes = Vec::new();
    let mut limited_entry = entry.by_ref().take(max_kml_bytes.saturating_add(1));
    limited_entry
        .read_to_end(&mut kml_bytes)
        .map_err(|error| ApiError::BadRequest(format!("invalid KMZ KML entry: {error}")))?;
    if kml_bytes.len() as u64 > max_kml_bytes {
        return Err(ApiError::PayloadTooLarge {
            max_bytes: max_kml_bytes,
        });
    }
    if kml_bytes.is_empty() {
        return Err(validation_error("file", "KMZ KML file must not be empty"));
    }
    Ok(kml_bytes)
}

fn parse_gpx_points(bytes: &[u8]) -> Result<Vec<TrailPoint>, ApiError> {
    let gpx = gpx::read(Cursor::new(bytes))
        .map_err(|error| ApiError::BadRequest(format!("invalid GPX file: {error}")))?;
    let mut points = Vec::new();
    for track in &gpx.tracks {
        for segment in &track.segments {
            for point in &segment.points {
                points.push(point_from_gpx(point)?);
            }
        }
    }
    if points.is_empty() {
        for route in &gpx.routes {
            for point in &route.points {
                points.push(point_from_gpx(point)?);
            }
        }
    }
    Ok(points.into_iter().filter(valid_point).collect())
}

fn point_from_gpx(point: &gpx::Waypoint) -> Result<TrailPoint, ApiError> {
    let geo = point.point();
    Ok(TrailPoint {
        lng: geo.x(),
        lat: geo.y(),
        elevation_m: point.elevation,
        time: point
            .time
            .map(|time| time.format())
            .transpose()
            .map_err(|error| {
                ApiError::BadRequest(format!("invalid GPX point timestamp: {error}"))
            })?,
    })
}

fn parse_kml_points(bytes: &[u8]) -> Result<Vec<TrailPoint>, ApiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ApiError::BadRequest("KML file must be valid UTF-8".to_owned()))?;
    let kml = text
        .parse::<Kml<f64>>()
        .map_err(|error| ApiError::BadRequest(format!("invalid KML file: {error}")))?;
    let mut points = Vec::new();
    collect_kml_points(&kml, &mut points);
    Ok(points.into_iter().filter(valid_point).collect())
}

fn collect_kml_points(kml: &Kml<f64>, points: &mut Vec<TrailPoint>) {
    match kml {
        Kml::KmlDocument(document) => {
            for child in &document.elements {
                collect_kml_points(child, points);
            }
        }
        Kml::Document { elements, .. } => {
            for child in elements {
                collect_kml_points(child, points);
            }
        }
        Kml::Folder(folder) => {
            for child in &folder.elements {
                collect_kml_points(child, points);
            }
        }
        Kml::Placemark(placemark) => {
            if let Some(geometry) = &placemark.geometry {
                collect_kml_geometry_points(geometry, points);
            }
        }
        Kml::LineString(line) => collect_kml_line_points(line, points),
        Kml::MultiGeometry(multi) => {
            for geometry in &multi.geometries {
                collect_kml_geometry_points(geometry, points);
            }
        }
        _ => {}
    }
}

fn collect_kml_geometry_points(geometry: &KmlGeometry<f64>, points: &mut Vec<TrailPoint>) {
    match geometry {
        KmlGeometry::LineString(line) => collect_kml_line_points(line, points),
        KmlGeometry::MultiGeometry(multi) => {
            for geometry in &multi.geometries {
                collect_kml_geometry_points(geometry, points);
            }
        }
        _ => {}
    }
}

fn collect_kml_line_points(line: &KmlLineString<f64>, points: &mut Vec<TrailPoint>) {
    points.extend(line.coords.iter().map(|coord| TrailPoint {
        lng: coord.x,
        lat: coord.y,
        elevation_m: coord.z,
        time: None,
    }));
}

fn parse_fit_points(bytes: &[u8]) -> Result<Vec<TrailPoint>, ApiError> {
    let mut reader = Cursor::new(bytes);
    let records = fitparser::from_reader(&mut reader)
        .map_err(|error| ApiError::BadRequest(format!("invalid FIT file: {error}")))?;
    let points = records
        .iter()
        .filter(|record| record.kind() == MesgNum::Record)
        .filter_map(point_from_fit_record)
        .filter(valid_point)
        .collect::<Vec<_>>();
    Ok(points)
}

fn point_from_fit_record(record: &FitDataRecord) -> Option<TrailPoint> {
    let lat = fit_field_f64(record, "position_lat").map(semicircles_to_degrees)?;
    let lng = fit_field_f64(record, "position_long").map(semicircles_to_degrees)?;
    let elevation_m =
        fit_field_f64(record, "enhanced_altitude").or_else(|| fit_field_f64(record, "altitude"));
    let time = record
        .fields()
        .iter()
        .find(|field| field.name() == "timestamp")
        .map(|field| field.value().to_string());
    Some(TrailPoint {
        lng,
        lat,
        elevation_m,
        time,
    })
}

fn fit_field_f64(record: &FitDataRecord, name: &str) -> Option<f64> {
    record
        .fields()
        .iter()
        .find(|field| field.name() == name)
        .and_then(|field| fit_value_to_f64(field.value()))
}

fn fit_value_to_f64(value: &FitValue) -> Option<f64> {
    match value {
        FitValue::Byte(value) => Some((*value).into()),
        FitValue::Enum(value) => Some((*value).into()),
        FitValue::SInt8(value) => Some((*value).into()),
        FitValue::UInt8(value) => Some((*value).into()),
        FitValue::SInt16(value) => Some((*value).into()),
        FitValue::UInt16(value) => Some((*value).into()),
        FitValue::SInt32(value) => Some((*value).into()),
        FitValue::UInt32(value) => Some((*value).into()),
        FitValue::Float32(value) => Some((*value).into()),
        FitValue::Float64(value) => Some(*value),
        FitValue::UInt8z(value) => Some((*value).into()),
        FitValue::UInt16z(value) => Some((*value).into()),
        FitValue::UInt32z(value) => Some((*value).into()),
        FitValue::SInt64(value) => Some(*value as f64),
        FitValue::UInt64(value) => Some(*value as f64),
        FitValue::UInt64z(value) => Some(*value as f64),
        FitValue::Array(_) | FitValue::String(_) | FitValue::Timestamp(_) | FitValue::Invalid => {
            None
        }
    }
}

fn semicircles_to_degrees(value: f64) -> f64 {
    value * 180.0 / 2_147_483_648.0
}

#[derive(Clone, Debug)]
struct TrailStats {
    bounds: Option<TrailBounds>,
    distance_m: f64,
    ascent_m: f64,
    descent_m: f64,
    min_elevation_m: Option<f64>,
    max_elevation_m: Option<f64>,
    start_time: Option<String>,
    end_time: Option<String>,
}

fn compute_stats(points: &[TrailPoint]) -> TrailStats {
    let mut bounds: Option<TrailBounds> = None;
    let mut distance_m = 0.0;
    let mut ascent_m = 0.0;
    let mut descent_m = 0.0;
    let mut min_elevation_m: Option<f64> = None;
    let mut max_elevation_m: Option<f64> = None;
    let mut previous: Option<&TrailPoint> = None;

    for point in points {
        bounds = Some(match bounds {
            Some(bounds) => TrailBounds {
                min_lng: bounds.min_lng.min(point.lng),
                min_lat: bounds.min_lat.min(point.lat),
                max_lng: bounds.max_lng.max(point.lng),
                max_lat: bounds.max_lat.max(point.lat),
            },
            None => TrailBounds {
                min_lng: point.lng,
                min_lat: point.lat,
                max_lng: point.lng,
                max_lat: point.lat,
            },
        });
        if let Some(elevation) = point.elevation_m {
            min_elevation_m =
                Some(min_elevation_m.map_or(elevation, |current| current.min(elevation)));
            max_elevation_m =
                Some(max_elevation_m.map_or(elevation, |current| current.max(elevation)));
        }
        if let Some(previous) = previous {
            distance_m += haversine_m(previous, point);
            if let (Some(previous_elevation), Some(elevation)) =
                (previous.elevation_m, point.elevation_m)
            {
                let delta = elevation - previous_elevation;
                if delta > 0.0 {
                    ascent_m += delta;
                } else {
                    descent_m += -delta;
                }
            }
        }
        previous = Some(point);
    }

    TrailStats {
        bounds,
        distance_m,
        ascent_m,
        descent_m,
        min_elevation_m,
        max_elevation_m,
        start_time: points.iter().find_map(|point| point.time.clone()),
        end_time: points.iter().rev().find_map(|point| point.time.clone()),
    }
}

fn haversine_m(a: &TrailPoint, b: &TrailPoint) -> f64 {
    let radius_m = 6_371_000.0_f64;
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlng = (b.lng - a.lng).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
    2.0 * radius_m * h.sqrt().asin()
}

fn simplify_points(points: &[TrailPoint], max_points: usize) -> Vec<TrailPoint> {
    simplify_items(points, max_points)
}

fn simplify_items<T: Clone>(items: &[T], max_points: usize) -> Vec<T> {
    if items.len() <= max_points || max_points < 2 {
        return items.to_vec();
    }
    let last = items.len() - 1;
    let target_last = max_points - 1;
    let mut simplified = Vec::with_capacity(max_points);
    let mut previous_index = None;
    for output_index in 0..=target_last {
        let index = (output_index * last + target_last / 2) / target_last;
        if previous_index != Some(index) {
            simplified.push(items[index].clone());
            previous_index = Some(index);
        }
    }
    simplified
}

fn simplify_coordinates(coordinates: &[Vec<f64>], max_points: usize) -> Vec<Vec<f64>> {
    simplify_items(coordinates, max_points)
}

fn trail_geojson(points: &[TrailPoint]) -> Result<JsonValue, ApiError> {
    let coordinates = points
        .iter()
        .map(|point| {
            let mut coordinate = vec![point.lng, point.lat];
            if let Some(elevation) = point.elevation_m {
                coordinate.push(elevation);
            }
            coordinate
        })
        .collect::<Vec<_>>();
    let feature = Feature::from(GeoJsonGeometry::new(GeoJsonValue::LineString(coordinates)));
    serde_json::to_value(feature).map_err(ApiError::internal)
}

/// Downsamples a stored map LineString feature for high-density overview rendering.
pub fn overview_geojson(
    source: &JsonValue,
    max_points: usize,
) -> Result<(JsonValue, usize, bool), ApiError> {
    let mut feature =
        serde_json::from_value::<Feature>(source.clone()).map_err(ApiError::internal)?;
    let Some(geometry) = feature.geometry.take() else {
        return Err(ApiError::internal(anyhow::anyhow!(
            "trail simplified_geojson must contain geometry"
        )));
    };
    let GeoJsonValue::LineString(coordinates) = geometry.value else {
        return Err(ApiError::internal(anyhow::anyhow!(
            "trail simplified_geojson must be a LineString feature"
        )));
    };
    let original_count = coordinates.len();
    let rendered_coordinates = simplify_coordinates(&coordinates, max_points);
    let rendered_count = rendered_coordinates.len();
    feature.geometry = Some(GeoJsonGeometry::new(GeoJsonValue::LineString(
        rendered_coordinates,
    )));
    let simplified = serde_json::to_value(feature).map_err(ApiError::internal)?;
    Ok((simplified, rendered_count, rendered_count < original_count))
}

fn valid_point(point: &TrailPoint) -> bool {
    point.lng.is_finite()
        && point.lat.is_finite()
        && (-180.0..=180.0).contains(&point.lng)
        && (-90.0..=90.0).contains(&point.lat)
        && point.elevation_m.is_none_or(f64::is_finite)
}

fn xml_root_tag(bytes: &[u8]) -> Result<Option<String>, ApiError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ApiError::BadRequest("XML trail file must be valid UTF-8".to_owned()))?;
    let mut rest = text.trim_start_matches('\u{feff}').trim_start();
    loop {
        let Some(after_open) = rest.strip_prefix('<') else {
            return Ok(None);
        };
        if let Some(after_pi) = after_open.strip_prefix("?xml") {
            let Some(index) = after_pi.find("?>") else {
                return Ok(None);
            };
            rest = after_pi[index + 2..].trim_start();
            continue;
        }
        if let Some(after_comment) = after_open.strip_prefix("!--") {
            let Some(index) = after_comment.find("-->") else {
                return Ok(None);
            };
            rest = after_comment[index + 3..].trim_start();
            continue;
        }
        if after_open.starts_with('!') {
            let Some(index) = after_open.find('>') else {
                return Ok(None);
            };
            rest = after_open[index + 1..].trim_start();
            continue;
        }
        let tag = after_open
            .chars()
            .take_while(|ch| !ch.is_whitespace() && *ch != '>' && *ch != '/')
            .collect::<String>();
        let local_name = tag.rsplit(':').next().unwrap_or(&tag).to_ascii_lowercase();
        return Ok(Some(local_name));
    }
}

fn safe_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '/' && *ch != '\\')
        .collect::<String>()
}

fn display_name_from_filename(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Trail")
        .to_owned()
}

fn validation_error(field: &str, message: impl Into<String>) -> ApiError {
    ApiError::Validation(vec![FieldViolation::new(field, message)])
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    #[test]
    fn parses_gpx_track_and_route_fallback() {
        let gpx = br#"<?xml version="1.0"?>
<gpx version="1.1" creator="test">
  <trk><trkseg>
    <trkpt lat="30.0" lon="120.0"><ele>10</ele><time>2026-01-01T00:00:00Z</time></trkpt>
    <trkpt lat="30.001" lon="120.001"><ele>15</ele><time>2026-01-01T00:10:00Z</time></trkpt>
  </trkseg></trk>
</gpx>"#;
        let parsed = parse_trail_upload(
            Some("demo.gpx"),
            Some("application/gpx+xml"),
            gpx,
            1_000_000,
            10,
            10,
        )
        .unwrap();
        assert_eq!(parsed.points.len(), 2);
        assert!(parsed.distance_m > 100.0);
        assert_eq!(parsed.ascent_m, 5.0);

        let route = br#"<?xml version="1.0"?><gpx version="1.1" creator="test">
  <rte><rtept lat="31.0" lon="121.0" /><rtept lat="31.1" lon="121.1" /></rte>
</gpx>"#;
        let parsed = parse_trail_upload(Some("route.gpx"), None, route, 1_000_000, 10, 10).unwrap();
        assert_eq!(parsed.points.len(), 2);
    }

    #[test]
    fn parses_kml_linestring_and_multigeometry() {
        let kml = br#"<?xml version="1.0"?>
<kml xmlns="http://www.opengis.net/kml/2.2"><Document>
  <Placemark><LineString><coordinates>120.0,30.0,10 120.1,30.1,20</coordinates></LineString></Placemark>
  <Placemark><MultiGeometry>
    <LineString><coordinates>121.0,31.0 121.1,31.1</coordinates></LineString>
  </MultiGeometry></Placemark>
</Document></kml>"#;
        let parsed = parse_trail_upload(
            Some("demo.kml"),
            Some("application/vnd.google-earth.kml+xml"),
            kml,
            1_000_000,
            10,
            10,
        )
        .unwrap();
        assert_eq!(parsed.points.len(), 4);
        assert_eq!(parsed.points[0].elevation_m, Some(10.0));
    }

    #[test]
    fn parses_kmz_doc_kml_as_kml_source() {
        let kml = br#"<?xml version="1.0"?>
<kml xmlns="http://www.opengis.net/kml/2.2"><Document>
  <Placemark><LineString><coordinates>120.0,30.0,10 120.1,30.1,20</coordinates></LineString></Placemark>
</Document></kml>"#;
        let kmz = kmz_fixture(&[("doc.kml", kml)]);

        let parsed = parse_trail_upload(
            Some("demo.kmz"),
            Some("application/zip"),
            &kmz,
            1_000_000,
            10,
            10,
        )
        .unwrap();

        assert_eq!(parsed.source_format, TrailSourceFormat::Kml);
        assert_eq!(parsed.storage_extension, "kmz");
        assert_eq!(parsed.content_type, "application/vnd.google-earth.kmz");
        assert_eq!(parsed.original_filename, "demo.kmz");
        assert_eq!(parsed.points.len(), 2);
        assert_eq!(parsed.points[0].elevation_m, Some(10.0));
    }

    #[test]
    fn parses_kmz_first_kml_when_doc_kml_is_missing() {
        let kml = br#"<?xml version="1.0"?>
<kml xmlns="http://www.opengis.net/kml/2.2"><Document>
  <Placemark><LineString><coordinates>121.0,31.0 121.1,31.1</coordinates></LineString></Placemark>
</Document></kml>"#;
        let kmz = kmz_fixture(&[("assets/icon.txt", b"ignored"), ("routes/ridge.kml", kml)]);

        let parsed = parse_trail_upload(
            Some("ridge.kmz"),
            Some("application/vnd.google-earth.kmz"),
            &kmz,
            1_000_000,
            10,
            10,
        )
        .unwrap();

        assert_eq!(parsed.source_format, TrailSourceFormat::Kml);
        assert_eq!(parsed.points.len(), 2);
        assert_eq!(parsed.points[0].lng, 121.0);
    }

    #[test]
    fn parses_fit_gps_records() {
        let fit = tiny_fit_fixture(&[(120.0, 30.0), (120.001, 30.001)]);
        let parsed = parse_trail_upload(
            Some("activity.fit"),
            Some("application/vnd.ant.fit"),
            &fit,
            1_000_000,
            10,
            10,
        )
        .unwrap();
        assert_eq!(parsed.points.len(), 2);
        assert!((parsed.points[0].lng - 120.0).abs() < 0.000_001);
        assert!((parsed.points[0].lat - 30.0).abs() < 0.000_001);
        assert!(parsed.distance_m > 100.0);
    }

    #[test]
    fn rejects_empty_unsupported_and_point_limit() {
        assert!(parse_trail_upload(Some("empty.gpx"), None, b"", 10, 10, 10).is_err());
        assert!(parse_trail_upload(Some("bad.kmz"), None, b"PK", 10, 10, 10).is_err());
        let gpx = br#"<?xml version="1.0"?><gpx version="1.1" creator="test">
  <trk><trkseg><trkpt lat="1" lon="1"/><trkpt lat="2" lon="2"/></trkseg></trk>
</gpx>"#;
        assert!(parse_trail_upload(Some("too-many.gpx"), None, gpx, 1_000_000, 1, 1).is_err());
    }

    #[test]
    fn rejects_invalid_kmz_archives() {
        assert!(
            parse_trail_upload(
                Some("invalid.kmz"),
                Some("application/vnd.google-earth.kmz"),
                b"PK\x03\x04invalid",
                1_000_000,
                10,
                10,
            )
            .is_err()
        );

        let no_kml = kmz_fixture(&[("readme.txt", b"no route here")]);
        assert!(parse_trail_upload(Some("missing.kmz"), None, &no_kml, 1_000_000, 10, 10).is_err());

        let empty_kml = kmz_fixture(&[("doc.kml", b"")]);
        assert!(
            parse_trail_upload(Some("empty.kmz"), None, &empty_kml, 1_000_000, 10, 10).is_err()
        );

        let oversized = kmz_fixture(&[("doc.kml", b"<kml></kml>")]);
        assert!(extract_kmz_kml(&oversized, 4).is_err());
    }

    #[test]
    fn simplifies_to_configured_point_limit() {
        let points = (0..50)
            .map(|index| TrailPoint {
                lng: index as f64,
                lat: index as f64 / 2.0,
                elevation_m: None,
                time: None,
            })
            .collect::<Vec<_>>();
        let simplified = simplify_points(&points, 5);
        assert_eq!(simplified.len(), 5);
        assert_eq!(simplified.first().unwrap().lng, 0.0);
        assert_eq!(simplified.last().unwrap().lng, 49.0);
    }

    fn kmz_fixture(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, bytes) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn tiny_fit_fixture(points: &[(f64, f64)]) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(0x40);
        data.push(0);
        data.push(0);
        data.extend_from_slice(&20_u16.to_le_bytes());
        data.push(2);
        data.extend_from_slice(&[0, 4, 0x85]);
        data.extend_from_slice(&[1, 4, 0x85]);
        for (lng, lat) in points {
            data.push(0);
            data.extend_from_slice(&degrees_to_semicircles(*lat).to_le_bytes());
            data.extend_from_slice(&degrees_to_semicircles(*lng).to_le_bytes());
        }

        let mut file = vec![14, 16];
        file.extend_from_slice(&100_u16.to_le_bytes());
        file.extend_from_slice(&(data.len() as u32).to_le_bytes());
        file.extend_from_slice(b".FIT");
        let header_crc = fit_crc(&file);
        file.extend_from_slice(&header_crc.to_le_bytes());
        file.extend_from_slice(&data);
        let file_crc = fit_crc(&file);
        file.extend_from_slice(&file_crc.to_le_bytes());
        file
    }

    fn degrees_to_semicircles(value: f64) -> i32 {
        (value * 2_147_483_648.0 / 180.0).round() as i32
    }

    fn fit_crc(bytes: &[u8]) -> u16 {
        const CRC_TABLE: [u16; 16] = [
            0x0000, 0xcc01, 0xd801, 0x1400, 0xf001, 0x3c00, 0x2800, 0xe401, 0xa001, 0x6c00, 0x7800,
            0xb401, 0x5000, 0x9c01, 0x8801, 0x4400,
        ];
        bytes.iter().fold(0_u16, |mut crc, byte| {
            let tmp = CRC_TABLE[((crc ^ u16::from(*byte)) & 0x0f) as usize];
            crc = (crc >> 4) ^ tmp;
            let tmp = CRC_TABLE[((crc ^ (u16::from(*byte) >> 4)) & 0x0f) as usize];
            (crc >> 4) ^ tmp
        })
    }
}
