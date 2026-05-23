//! Roadmap routes for public planning content, authenticated user interest, and administrator maintenance.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, put},
};

use crate::{
    dto::roadmap::{
        ListRoadmapResponse, RoadmapInteractionStatusResponse, RoadmapItemRequest,
        RoadmapItemResponse, RoadmapListQuery,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::{admin_service, roadmap_service},
    state::AppState,
};

/// Builds every Roadmap route.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/roadmap", get(list_public))
        .route("/me/roadmap", get(list_me))
        .route("/me/roadmap/:id/vote", put(vote).delete(unvote))
        .route(
            "/me/roadmap/:id/subscription",
            put(subscribe).delete(unsubscribe),
        )
        .route("/admin/roadmap", get(list_admin).post(create_admin))
        .route(
            "/admin/roadmap/:id",
            patch(update_admin).delete(delete_admin),
        )
}

async fn list_public(
    State(state): State<AppState>,
    Query(query): Query<RoadmapListQuery>,
) -> Result<Json<ListRoadmapResponse>, ApiError> {
    let (entries, next_cursor) = roadmap_service::list_public(&state, query.into()).await?;
    Ok(Json(ListRoadmapResponse::from_entries(
        &entries,
        next_cursor,
    )))
}

async fn list_me(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<RoadmapListQuery>,
) -> Result<Json<ListRoadmapResponse>, ApiError> {
    let (entries, next_cursor) =
        roadmap_service::list_for_user(&state, &user.id, query.into()).await?;
    Ok(Json(ListRoadmapResponse::from_entries(
        &entries,
        next_cursor,
    )))
}

async fn vote(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<RoadmapInteractionStatusResponse>, ApiError> {
    Ok(Json(RoadmapItemResponse::from_entry(
        &roadmap_service::vote(&state, &user.id, &id).await?,
    )))
}

async fn unvote(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<RoadmapInteractionStatusResponse>, ApiError> {
    Ok(Json(RoadmapItemResponse::from_entry(
        &roadmap_service::unvote(&state, &user.id, &id).await?,
    )))
}

async fn subscribe(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<RoadmapInteractionStatusResponse>, ApiError> {
    Ok(Json(RoadmapItemResponse::from_entry(
        &roadmap_service::subscribe(&state, &user.id, &id).await?,
    )))
}

async fn unsubscribe(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<RoadmapInteractionStatusResponse>, ApiError> {
    Ok(Json(RoadmapItemResponse::from_entry(
        &roadmap_service::unsubscribe(&state, &user.id, &id).await?,
    )))
}

async fn list_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<RoadmapListQuery>,
) -> Result<Json<ListRoadmapResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let (entries, next_cursor) = roadmap_service::list_admin(&state, query.into()).await?;
    Ok(Json(ListRoadmapResponse::from_entries(
        &entries,
        next_cursor,
    )))
}

async fn create_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<RoadmapItemRequest>,
) -> Result<(StatusCode, Json<RoadmapItemResponse>), ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let entry = roadmap_service::create_admin(&state, &user.id, payload.into()).await?;
    Ok((
        StatusCode::CREATED,
        Json(RoadmapItemResponse::from_entry(&entry)),
    ))
}

async fn update_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<RoadmapItemRequest>,
) -> Result<Json<RoadmapItemResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let entry = roadmap_service::update_admin(&state, &user.id, &id, payload.into()).await?;
    Ok(Json(RoadmapItemResponse::from_entry(&entry)))
}

async fn delete_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    roadmap_service::delete_admin(&state, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
