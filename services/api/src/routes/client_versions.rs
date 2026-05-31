//! Public and administrator routes for client release versions.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};

use crate::{
    dto::client_version::{
        ClientVersionAdminQuery, ClientVersionPublicQuery, ClientVersionRequest,
        ClientVersionResponse, ListClientVersionsResponse,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::{admin_service, client_version_service},
    state::AppState,
};

/// Builds client version routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/client-versions", get(list_public_versions))
        .route("/client-versions/current", get(current_public_version))
        .route(
            "/admin/client-versions",
            get(list_admin_versions).post(create_admin_version),
        )
        .route("/admin/client-versions/:id", patch(update_admin_version))
}

async fn list_public_versions(
    State(state): State<AppState>,
    Query(query): Query<ClientVersionPublicQuery>,
) -> Result<Json<ListClientVersionsResponse>, ApiError> {
    let (records, next_cursor) =
        client_version_service::list_public(&state, query.client_key, query.limit, query.cursor)
            .await?;
    Ok(Json(ListClientVersionsResponse {
        items: records
            .iter()
            .map(ClientVersionResponse::from_record_public)
            .collect(),
        next_cursor,
    }))
}

async fn current_public_version(
    State(state): State<AppState>,
    Query(query): Query<ClientVersionPublicQuery>,
) -> Result<Json<ClientVersionResponse>, ApiError> {
    let record = client_version_service::current_public(&state, query.client_key).await?;
    Ok(Json(ClientVersionResponse::from_record_public(&record)))
}

async fn list_admin_versions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ClientVersionAdminQuery>,
) -> Result<Json<ListClientVersionsResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let (records, next_cursor) = client_version_service::list_admin(
        &state,
        query.client_key,
        query.status,
        query.limit,
        query.cursor,
    )
    .await?;
    Ok(Json(ListClientVersionsResponse {
        items: records
            .iter()
            .map(ClientVersionResponse::from_record_admin)
            .collect(),
        next_cursor,
    }))
}

async fn create_admin_version(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<ClientVersionRequest>,
) -> Result<(StatusCode, Json<ClientVersionResponse>), ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let record = client_version_service::create_admin(&state, &user.id, payload.into()).await?;
    Ok((
        StatusCode::CREATED,
        Json(ClientVersionResponse::from_record_admin(&record)),
    ))
}

async fn update_admin_version(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<ClientVersionRequest>,
) -> Result<Json<ClientVersionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let record =
        client_version_service::update_admin(&state, &user.id, &id, payload.into()).await?;
    Ok(Json(ClientVersionResponse::from_record_admin(&record)))
}
