//! Public outdoor skill routes for DB-backed skill categories and knots.

use std::collections::HashMap;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::get,
};
use stellartrail_domain::skill::SkillCategoriesResponse;

use crate::{error::ApiError, state::AppState};

use super::localization::{
    cached_localized_json_with, parse_u32_query, reject_all_query_parameters, reject_query_locale,
    resolve_locale,
};

/// Builds all DB-backed outdoor skill routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(skill_categories))
        .route("/api/skills/knots/list", get(knot_list))
        .route("/api/skills/knots/filters", get(knot_filters))
        .route(
            "/api/skills/knots/offline-manifest",
            get(knot_offline_manifest),
        )
        .route("/api/skills/knots/detail/:id", get(knot_detail))
}

async fn skill_categories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    if query.contains_key("category") {
        return Err(ApiError::NotFound);
    }
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-categories",
        &format!("v1|{}", locale.as_str()),
        || async {
            let items = state
                .knot_repository()
                .list_skill_categories(locale)
                .await?;
            Ok(SkillCategoriesResponse { items })
        },
    )
    .await
}

async fn knot_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    if query.contains_key("cursor") {
        return Err(ApiError::unsupported_query_parameter("cursor"));
    }
    if query.contains_key("next_cursor") {
        return Err(ApiError::unsupported_query_parameter("next_cursor"));
    }
    let locale = resolve_locale(&headers)?;
    let offset = parse_u32_query(&query, "offset", 0)?;
    let limit = parse_u32_query(&query, "limit", 20)?.clamp(1, 100);
    let category = query.get("category").map(String::as_str);
    let difficulty = query.get("difficulty").map(String::as_str);
    let q = query.get("q").map(String::as_str);
    let normalized_input = format!(
        "v1|{}|offset={offset}|limit={limit}|category={}|difficulty={}|q={}",
        locale.as_str(),
        category.unwrap_or_default(),
        difficulty.unwrap_or_default(),
        q.unwrap_or_default()
    );
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-list",
        &normalized_input,
        || async {
            state
                .knot_repository()
                .list_knots(locale, offset, limit, category, difficulty, q)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_filters(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-filters",
        &format!("v1|{}", locale.as_str()),
        || async {
            state
                .knot_repository()
                .list_knot_filters(locale)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_offline_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    reject_all_query_parameters(&query)?;
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-offline-manifest",
        &format!("v1|{}", locale.as_str()),
        || async {
            state
                .knot_repository()
                .offline_manifest(locale)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let normalized_input = format!("v1|{}|id={id}", locale.as_str());
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-detail",
        &normalized_input,
        || async {
            state
                .knot_repository()
                .get_knot_detail(&id, locale)
                .await?
                .ok_or(ApiError::NotFound)
        },
    )
    .await
}
