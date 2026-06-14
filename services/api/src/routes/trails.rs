//! Authenticated trail library, map state, trail link, and annotation routes.

use std::collections::BTreeSet;

use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use stellartrail_db::repositories::TrailRepository;
use stellartrail_domain::{
    trail::{MapAnnotation, MapTrailLink, TrailBounds},
    validation::FieldViolation,
};

use crate::{
    dto::trail::{
        ListTrailsResponse, MapAnnotationRequest, OutdoorExperienceMapStateResponse,
        TrailLinkRequest, TrailUploadResponse, TripMapStateResponse, TripOverviewMapTrail,
        TripsMapOverviewResponse, TripsMapOverviewStats, UpdateMapAnnotationRequest,
        UpdateTrailRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    routes::map::map_config_response,
    services::trail_service,
    state::AppState,
};

/// Builds authenticated trail and map routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/trails", get(list_trails).post(upload_trail))
        .route("/me/trails/:trail_id/file", get(download_trail_file))
        .route(
            "/me/trails/:trail_id",
            get(get_trail).patch(update_trail).delete(delete_trail),
        )
        .route("/me/trips/map-overview", get(get_trips_map_overview))
        .route("/me/trips/:id/map", get(get_trip_map))
        .route(
            "/me/trips/:id/trails",
            axum::routing::post(upload_trip_trail),
        )
        .route(
            "/me/trips/:id/trail-links",
            axum::routing::post(link_trip_trail),
        )
        .route(
            "/me/trips/:id/trail-links/:trail_id",
            axum::routing::delete(unlink_trip_trail),
        )
        .route(
            "/me/trips/:id/map-annotations",
            get(list_trip_annotations).post(create_trip_annotation),
        )
        .route(
            "/me/trips/:id/map-annotations/:annotation_id",
            axum::routing::patch(update_trip_annotation).delete(delete_trip_annotation),
        )
        .route(
            "/me/outdoor-experiences/:id/map",
            get(get_outdoor_experience_map),
        )
        .route(
            "/me/outdoor-experiences/:id/trail-links",
            axum::routing::post(link_outdoor_experience_trail),
        )
        .route(
            "/me/outdoor-experiences/:id/trail-links/:trail_id",
            axum::routing::delete(unlink_outdoor_experience_trail),
        )
        .route(
            "/me/outdoor-experiences/:id/map-annotations",
            get(list_outdoor_experience_annotations).post(create_outdoor_experience_annotation),
        )
        .route(
            "/me/outdoor-experiences/:id/map-annotations/:annotation_id",
            axum::routing::patch(update_outdoor_experience_annotation)
                .delete(delete_outdoor_experience_annotation),
        )
}

async fn list_trails(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<ListTrailsResponse>, ApiError> {
    let items = TrailRepository::new(state.db().clone())
        .list_owned(&user.id)
        .await?;
    Ok(Json(ListTrailsResponse { items }))
}

async fn upload_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    multipart: Multipart,
) -> Result<(StatusCode, Json<TrailUploadResponse>), ApiError> {
    let upload = read_multipart_file(multipart).await?;
    let trail = trail_service::upload_trail_to_library(
        &state,
        &user.id,
        upload.file_name.as_deref(),
        upload.content_type.as_deref(),
        upload.bytes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(trail)))
}

async fn get_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(trail_id): Path<String>,
) -> Result<Json<TrailUploadResponse>, ApiError> {
    let trail = TrailRepository::new(state.db().clone())
        .get_owned(&user.id, &trail_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(trail))
}

async fn update_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(trail_id): Path<String>,
    Json(payload): Json<UpdateTrailRequest>,
) -> Result<Json<TrailUploadResponse>, ApiError> {
    let mut patch = payload.into_patch();
    patch.validate_and_normalize()?;
    let trail = TrailRepository::new(state.db().clone())
        .update_owned_metadata(&user.id, &trail_id, &patch)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(trail))
}

async fn delete_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(trail_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let deleted = TrailRepository::new(state.db().clone())
        .delete_owned(&user.id, &trail_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn download_trail_file(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(trail_id): Path<String>,
) -> Result<Response, ApiError> {
    let trail = TrailRepository::new(state.db().clone())
        .get_owned(&user.id, &trail_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let object = state
        .object_store()
        .get_object(&trail.object_key)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;
    let disposition = format!(
        "attachment; filename=\"{}\"",
        safe_header_filename(&trail.original_filename)
    );
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, object.content_type),
            (header::CONTENT_DISPOSITION, disposition),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        Body::from(object.bytes),
    )
        .into_response())
}

async fn get_trip_map(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<TripMapStateResponse>, ApiError> {
    let (trails, annotations) = TrailRepository::new(state.db().clone())
        .trip_map_state(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(TripMapStateResponse {
        map: map_config_response(&state, &headers),
        trails,
        annotations,
    }))
}

async fn upload_trip_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<MapTrailLink>), ApiError> {
    let upload = read_multipart_file(multipart).await?;
    let link = trail_service::upload_trail_to_trip(
        &state,
        &user.id,
        &id,
        upload.file_name.as_deref(),
        upload.content_type.as_deref(),
        upload.bytes,
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(link)))
}

async fn link_trip_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<TrailLinkRequest>,
) -> Result<(StatusCode, Json<MapTrailLink>), ApiError> {
    let link = TrailRepository::new(state.db().clone())
        .link_trail_to_trip(
            &user.id,
            &id,
            &payload.trail_id,
            state.config().trail.max_trails_per_trip,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(link)))
}

async fn unlink_trip_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, trail_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let deleted = TrailRepository::new(state.db().clone())
        .unlink_trail_from_trip(&user.id, &id, &trail_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn get_trips_map_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<TripsMapOverviewResponse>, ApiError> {
    let trail_config = &state.config().trail;
    let fetch_limit = trail_config.overview_max_trails.saturating_add(1);
    let rows = TrailRepository::new(state.db().clone())
        .trips_map_overview(&user.id, trail_config.overview_max_trips, fetch_limit)
        .await?;
    let mut truncated = rows.len() as u64 > trail_config.overview_max_trails;
    let mut trails = Vec::new();
    let mut bounds: Option<TrailBounds> = None;
    let mut trip_ids = BTreeSet::new();
    let mut rendered_trail_ids = BTreeSet::new();
    let mut rendered_point_count = 0usize;
    let mut total_distance_m = 0.0;
    let mut total_ascent_m = 0.0;
    let mut total_descent_m = 0.0;

    for row in rows
        .into_iter()
        .take(trail_config.overview_max_trails as usize)
    {
        let remaining = trail_config
            .overview_max_points
            .saturating_sub(rendered_point_count as u64) as usize;
        if remaining < 2 {
            truncated = true;
            break;
        }
        let trail_id = row.link.trail_id.clone();
        if rendered_trail_ids.contains(&trail_id) {
            continue;
        }
        let per_trail_limit = (trail_config.overview_max_points_per_trail as usize).min(remaining);
        let (simplified_geojson, rendered_count, simplified) =
            trail_service::overview_geojson(&row.link.simplified_geojson, per_trail_limit)?;
        if rendered_count < 2 {
            continue;
        }
        truncated |= simplified || row.link.trail.point_count as usize > rendered_count;
        rendered_point_count += rendered_count;
        trip_ids.insert(row.trip_id.clone());
        total_distance_m += row.link.trail.distance_m;
        total_ascent_m += row.link.trail.ascent_m;
        total_descent_m += row.link.trail.descent_m;
        if let Some(next_bounds) = &row.link.trail.bounds {
            bounds = Some(merge_bounds(bounds, next_bounds));
        }
        rendered_trail_ids.insert(trail_id);
        trails.push(TripOverviewMapTrail::from_trip(row, simplified_geojson));
    }

    let remaining_trail_slots =
        (trail_config.overview_max_trails as usize).saturating_sub(trails.len());
    if remaining_trail_slots > 0 {
        let library_fetch_limit = remaining_trail_slots as u64 + 1;
        let library_trails = TrailRepository::new(state.db().clone())
            .list_owned_unlinked_to_trips(&user.id, library_fetch_limit)
            .await?;
        truncated |= library_trails.len() > remaining_trail_slots;
        for trail in library_trails.into_iter().take(remaining_trail_slots) {
            if rendered_trail_ids.contains(&trail.id) {
                continue;
            }
            let remaining = trail_config
                .overview_max_points
                .saturating_sub(rendered_point_count as u64) as usize;
            if remaining < 2 {
                truncated = true;
                break;
            }
            let per_trail_limit =
                (trail_config.overview_max_points_per_trail as usize).min(remaining);
            let (simplified_geojson, rendered_count, simplified) =
                trail_service::overview_geojson(&trail.simplified_geojson, per_trail_limit)?;
            if rendered_count < 2 {
                continue;
            }
            truncated |= simplified || trail.point_count as usize > rendered_count;
            rendered_point_count += rendered_count;
            total_distance_m += trail.distance_m;
            total_ascent_m += trail.ascent_m;
            total_descent_m += trail.descent_m;
            if let Some(next_bounds) = &trail.bounds {
                bounds = Some(merge_bounds(bounds, next_bounds));
            }
            rendered_trail_ids.insert(trail.id.clone());
            trails.push(TripOverviewMapTrail::from_library(
                trail,
                simplified_geojson,
            ));
        }
    }

    let trail_count = trails.len();
    Ok(Json(TripsMapOverviewResponse {
        map: map_config_response(&state, &headers),
        trails,
        bounds,
        stats: TripsMapOverviewStats {
            trip_count: trip_ids.len(),
            trail_count,
            rendered_point_count,
            total_distance_m,
            total_ascent_m,
            total_descent_m,
        },
        truncated,
    }))
}

async fn list_trip_annotations(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<MapAnnotation>>, ApiError> {
    let (_, annotations) = TrailRepository::new(state.db().clone())
        .trip_map_state(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(annotations))
}

async fn create_trip_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<MapAnnotationRequest>,
) -> Result<(StatusCode, Json<MapAnnotation>), ApiError> {
    let mut draft = payload.into_draft();
    draft.validate_and_normalize()?;
    let annotation = TrailRepository::new(state.db().clone())
        .create_trip_annotation(
            &user.id,
            &id,
            &draft,
            state.config().trail.max_annotations_per_context,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(annotation)))
}

async fn update_trip_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, annotation_id)): Path<(String, String)>,
    Json(payload): Json<UpdateMapAnnotationRequest>,
) -> Result<Json<MapAnnotation>, ApiError> {
    let (mut patch, base_versions, force_fields) = payload.into_parts();
    patch.validate_and_normalize()?;
    let annotation = TrailRepository::new(state.db().clone())
        .update_trip_annotation(
            &user.id,
            &id,
            &annotation_id,
            &patch,
            base_versions,
            force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(annotation))
}

async fn delete_trip_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, annotation_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let deleted = TrailRepository::new(state.db().clone())
        .delete_trip_annotation(&user.id, &id, &annotation_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn get_outdoor_experience_map(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<OutdoorExperienceMapStateResponse>, ApiError> {
    let (trails, annotations) = TrailRepository::new(state.db().clone())
        .outdoor_experience_map_state(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(OutdoorExperienceMapStateResponse {
        map: map_config_response(&state, &headers),
        trails,
        annotations,
    }))
}

async fn link_outdoor_experience_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<TrailLinkRequest>,
) -> Result<(StatusCode, Json<MapTrailLink>), ApiError> {
    let link = TrailRepository::new(state.db().clone())
        .link_trail_to_outdoor_experience(&user.id, &id, &payload.trail_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(link)))
}

async fn unlink_outdoor_experience_trail(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, trail_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let deleted = TrailRepository::new(state.db().clone())
        .unlink_trail_from_outdoor_experience(&user.id, &id, &trail_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn list_outdoor_experience_annotations(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<MapAnnotation>>, ApiError> {
    let (_, annotations) = TrailRepository::new(state.db().clone())
        .outdoor_experience_map_state(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(annotations))
}

async fn create_outdoor_experience_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<MapAnnotationRequest>,
) -> Result<(StatusCode, Json<MapAnnotation>), ApiError> {
    let mut draft = payload.into_draft();
    draft.validate_and_normalize()?;
    let annotation = TrailRepository::new(state.db().clone())
        .create_outdoor_experience_annotation(
            &user.id,
            &id,
            &draft,
            state.config().trail.max_annotations_per_context,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(annotation)))
}

async fn update_outdoor_experience_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, annotation_id)): Path<(String, String)>,
    Json(payload): Json<UpdateMapAnnotationRequest>,
) -> Result<Json<MapAnnotation>, ApiError> {
    let (mut patch, base_versions, force_fields) = payload.into_parts();
    patch.validate_and_normalize()?;
    let annotation = TrailRepository::new(state.db().clone())
        .update_outdoor_experience_annotation(
            &user.id,
            &id,
            &annotation_id,
            &patch,
            base_versions,
            force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(annotation))
}

async fn delete_outdoor_experience_annotation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, annotation_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let deleted = TrailRepository::new(state.db().clone())
        .delete_outdoor_experience_annotation(&user.id, &id, &annotation_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

struct MultipartFile {
    file_name: Option<String>,
    content_type: Option<String>,
    bytes: Vec<u8>,
}

async fn read_multipart_file(mut multipart: Multipart) -> Result<MultipartFile, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::BadRequest(format!("invalid multipart body: {error}")))?
    {
        if field.name() == Some("file") {
            let file_name = field.file_name().map(ToOwned::to_owned);
            let content_type = field.content_type().map(ToOwned::to_owned);
            let bytes = field.bytes().await.map_err(|error| {
                ApiError::BadRequest(format!("invalid multipart file: {error}"))
            })?;
            return Ok(MultipartFile {
                file_name,
                content_type,
                bytes: bytes.to_vec(),
            });
        }
    }
    Err(ApiError::Validation(vec![FieldViolation::new(
        "file",
        "is required",
    )]))
}

fn safe_header_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '"' && *ch != '\\')
        .collect::<String>()
}

fn merge_bounds(current: Option<TrailBounds>, next: &TrailBounds) -> TrailBounds {
    current.map_or_else(
        || next.clone(),
        |current| TrailBounds {
            min_lng: current.min_lng.min(next.min_lng),
            min_lat: current.min_lat.min(next.min_lat),
            max_lng: current.max_lng.max(next.max_lng),
            max_lat: current.max_lat.max(next.max_lat),
        },
    )
}
