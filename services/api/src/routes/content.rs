//! Public DB-backed gear template routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use stellartrail_domain::gear_template::GearTemplate;

use crate::{error::ApiError, state::AppState};

/// Stable data boundary for `ListResponse`, exposed by or reused within this module.
#[derive(Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
}

/// Builds DB-backed public content routes that remain in scope for the initial MVP.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/gear-templates", get(list_gear_templates))
        .route("/api/gear-templates/:id", get(get_gear_template))
}

/// Lists active DB-backed gear templates.
async fn list_gear_templates(
    State(state): State<AppState>,
) -> Result<Json<ListResponse<GearTemplate>>, ApiError> {
    Ok(Json(ListResponse {
        items: state.gear_template_repository().list_templates().await?,
    }))
}

/// Returns one active DB-backed gear template by id.
async fn get_gear_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GearTemplate>, ApiError> {
    state
        .gear_template_repository()
        .get_template(&id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}
