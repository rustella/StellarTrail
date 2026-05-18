//! Public gear atlas, user submission, and administrator review routes.
//!
//! User-gear submissions are always materialized server-side from the personal
//! gear row so clients cannot accidentally upload private purchase, storage, or
//! note fields into the public atlas table.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use stellartrail_db::repositories::{
    GearAtlasRepository, GearRepository, ListGearAtlasAdminOptions, ListGearAtlasOptions,
};
use stellartrail_domain::{gear_atlas::draft_from_personal_gear, validation::FieldViolation};

use crate::{
    dto::gear_atlas::{
        CreateGearAtlasSubmissionRequest, GearAtlasPublicItemResponse, GearAtlasSubmissionResponse,
        ListAdminGearAtlasSubmissionsQuery, ListGearAtlasQuery, ListGearAtlasResponse,
        ListGearAtlasSubmissionsResponse, ListMyGearAtlasSubmissionsQuery,
        RejectGearAtlasSubmissionRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::admin_service,
    state::AppState,
};

/// Builds all gear atlas routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/gear-atlas", get(list_public))
        .route("/api/gear-atlas/:id", get(get_public))
        .route(
            "/api/me/gear-atlas-submissions",
            get(list_my_submissions).post(create_manual_submission),
        )
        .route(
            "/api/me/gears/:id/atlas-submission",
            post(create_submission_from_personal_gear),
        )
        .route(
            "/api/admin/gear-atlas-submissions",
            get(list_admin_submissions),
        )
        .route(
            "/api/admin/gear-atlas-submissions/:id",
            get(get_admin_submission),
        )
        .route(
            "/api/admin/gear-atlas-submissions/:id/approve",
            post(approve_submission),
        )
        .route(
            "/api/admin/gear-atlas-submissions/:id/reject",
            post(reject_submission),
        )
}

async fn list_public(
    State(state): State<AppState>,
    Query(query): Query<ListGearAtlasQuery>,
) -> Result<Json<ListGearAtlasResponse>, ApiError> {
    let (items, next_cursor) = GearAtlasRepository::new(state.db().clone())
        .list_public(&ListGearAtlasOptions {
            category: query.category,
            q: query.q,
            sort: query.sort,
            limit: query.limit.unwrap_or(20),
            cursor: query.cursor,
        })
        .await?;
    Ok(Json(ListGearAtlasResponse {
        items: items
            .iter()
            .map(GearAtlasPublicItemResponse::from)
            .collect(),
        next_cursor,
    }))
}

async fn get_public(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasPublicItemResponse>, ApiError> {
    let item = GearAtlasRepository::new(state.db().clone())
        .get_public(&id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(GearAtlasPublicItemResponse::from(&item)))
}

async fn create_manual_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateGearAtlasSubmissionRequest>,
) -> Result<(StatusCode, Json<GearAtlasSubmissionResponse>), ApiError> {
    let mut draft = payload.into_draft(&user.id);
    draft
        .validate_and_normalize()
        .map_err(|error| ApiError::Validation(error.fields))?;
    let item = GearAtlasRepository::new(state.db().clone())
        .create_submission(&draft)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(GearAtlasSubmissionResponse::from(&item)),
    ))
}

async fn create_submission_from_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<GearAtlasSubmissionResponse>), ApiError> {
    let gear = GearRepository::new(state.db().clone())
        .get(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let atlas_repo = GearAtlasRepository::new(state.db().clone());
    if let Some(existing) = atlas_repo
        .active_source_submission(&user.id, &gear.id)
        .await?
    {
        return Ok((
            StatusCode::OK,
            Json(GearAtlasSubmissionResponse::from(&existing)),
        ));
    }
    let mut draft = draft_from_personal_gear(&user.id, &gear);
    draft
        .validate_and_normalize()
        .map_err(|error| ApiError::Validation(error.fields))?;
    let item = atlas_repo.create_submission(&draft).await?;
    Ok((
        StatusCode::CREATED,
        Json(GearAtlasSubmissionResponse::from(&item)),
    ))
}

async fn list_my_submissions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListMyGearAtlasSubmissionsQuery>,
) -> Result<Json<ListGearAtlasSubmissionsResponse>, ApiError> {
    let (items, next_cursor) = GearAtlasRepository::new(state.db().clone())
        .list_user_submissions(&user.id, query.limit.unwrap_or(20), query.cursor.as_deref())
        .await?;
    Ok(Json(ListGearAtlasSubmissionsResponse {
        items: items
            .iter()
            .map(GearAtlasSubmissionResponse::from)
            .collect(),
        next_cursor,
    }))
}

async fn list_admin_submissions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListAdminGearAtlasSubmissionsQuery>,
) -> Result<Json<ListGearAtlasSubmissionsResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let (items, next_cursor) = GearAtlasRepository::new(state.db().clone())
        .list_admin(&ListGearAtlasAdminOptions {
            status: query.status,
            category: query.category,
            q: query.q,
            limit: query.limit.unwrap_or(20),
            cursor: query.cursor,
        })
        .await?;
    Ok(Json(ListGearAtlasSubmissionsResponse {
        items: items
            .iter()
            .map(GearAtlasSubmissionResponse::from)
            .collect(),
        next_cursor,
    }))
}

async fn get_admin_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let item = GearAtlasRepository::new(state.db().clone())
        .get_any(&id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(GearAtlasSubmissionResponse::from(&item)))
}

async fn approve_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let item = GearAtlasRepository::new(state.db().clone())
        .approve(&id, &user.id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(GearAtlasSubmissionResponse::from(&item)))
}

async fn reject_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<RejectGearAtlasSubmissionRequest>,
) -> Result<Json<GearAtlasSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let reason = normalize_rejection_reason(payload.reason)?;
    let item = GearAtlasRepository::new(state.db().clone())
        .reject(&id, &user.id, reason)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(GearAtlasSubmissionResponse::from(&item)))
}

fn normalize_rejection_reason(value: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 200 {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "reason",
            "must be at most 200 characters",
        )]));
    }
    Ok(Some(trimmed.to_owned()))
}
