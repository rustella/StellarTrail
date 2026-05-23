//! Public DB-backed gear template routes.

use std::collections::HashMap;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::get,
};
use serde::Serialize;

use crate::{error::ApiError, state::AppState};

use super::localization::{localized_json, reject_query_locale, resolve_locale};

/// Stable data boundary for `ListResponse`, exposed by or reused within this module.
#[derive(Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
}

/// Builds DB-backed public content routes that remain in scope for the initial MVP.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/gear-templates", get(list_gear_templates))
        .route("/gear-templates/:id", get(get_gear_template))
}

/// Lists active DB-backed gear templates.
async fn list_gear_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let response = ListResponse {
        items: state
            .gear_template_repository()
            .list_templates(locale)
            .await?,
    };
    localized_json(&state, &headers, locale, response)
}

/// Returns one active DB-backed gear template by id.
async fn get_gear_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let template = state
        .gear_template_repository()
        .get_template(&id, locale)
        .await?
        .ok_or(ApiError::NotFound)?;
    localized_json(&state, &headers, locale, template)
}
