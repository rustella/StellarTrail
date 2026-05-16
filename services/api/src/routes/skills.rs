//! Public outdoor skill routes for DB-backed skill categories and knots.

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use stellartrail_domain::skill::{Locale, SkillCategoriesResponse};

use crate::{error::ApiError, state::AppState};

/// Builds all DB-backed outdoor skill routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(skill_categories))
        .route("/api/skills/knots/list", get(knot_list))
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
    let items = state
        .knot_repository()
        .list_skill_categories(locale)
        .await?;
    localized_json(locale, SkillCategoriesResponse { items })
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
    let q = query.get("q").map(String::as_str);
    let response = state
        .knot_repository()
        .list_knots(locale, offset, limit, category, q)
        .await?;
    localized_json(locale, response)
}

async fn knot_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let Some(detail) = state.knot_repository().get_knot_detail(&id, locale).await? else {
        return Err(ApiError::NotFound);
    };
    localized_json(locale, detail)
}

fn localized_json<T: Serialize>(locale: Locale, body: T) -> Result<Response, ApiError> {
    let mut response = (StatusCode::OK, Json(body)).into_response();
    response.headers_mut().insert(
        header::CONTENT_LANGUAGE,
        HeaderValue::from_str(locale.as_str())
            .map_err(|_| ApiError::invalid_header(header::CONTENT_LANGUAGE.as_str()))?,
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Accept-Language, X-StellarTrail-Locale"),
    );
    Ok(response)
}

fn resolve_locale(headers: &HeaderMap) -> Result<Locale, ApiError> {
    if let Some(value) = headers.get("X-StellarTrail-Locale") {
        let raw = value
            .to_str()
            .map_err(|_| ApiError::invalid_header("X-StellarTrail-Locale"))?;
        return Locale::parse(raw).ok_or_else(|| ApiError::invalid_header("X-StellarTrail-Locale"));
    }

    if let Some(value) = headers.get(header::ACCEPT_LANGUAGE) {
        let raw = value
            .to_str()
            .map_err(|_| ApiError::invalid_header(header::ACCEPT_LANGUAGE.as_str()))?;
        for candidate in raw.split(',') {
            let language = candidate.split(';').next().unwrap_or_default().trim();
            if let Some(locale) = Locale::parse(language) {
                return Ok(locale);
            }
        }
    }

    Ok(Locale::ZhCn)
}

fn reject_query_locale(query: &HashMap<String, String>) -> Result<(), ApiError> {
    if query.contains_key("locale") {
        Err(ApiError::unsupported_query_parameter("locale"))
    } else {
        Ok(())
    }
}

fn parse_u32_query(
    query: &HashMap<String, String>,
    key: &'static str,
    default: u32,
) -> Result<u32, ApiError> {
    match query.get(key) {
        Some(value) if value.trim().is_empty() => Ok(default),
        Some(value) => value.parse::<u32>().map_err(|_| {
            ApiError::invalid_query_parameter(
                key,
                format!("query parameter `{key}` must be a non-negative integer"),
            )
        }),
        None => Ok(default),
    }
}
