//! Public outdoor skill routes for DB-backed skill categories and knots.

use std::collections::HashMap;

use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use stellartrail_domain::skill::{Locale, SkillCategoriesResponse};

use crate::{
    error::ApiError,
    services::public_response_cache::{
        CachedPublicResponse, get_public_response, public_cache_key, set_public_response,
    },
    state::AppState,
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
    let items = state
        .knot_repository()
        .list_skill_categories(locale)
        .await?;
    localized_json(&state, &headers, locale, SkillCategoriesResponse { items })
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
    let response = state
        .knot_repository()
        .list_knots(locale, offset, limit, category, difficulty, q)
        .await?;
    localized_json(&state, &headers, locale, response)
}

async fn knot_filters(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let response = state.knot_repository().list_knot_filters(locale).await?;
    localized_json(&state, &headers, locale, response)
}

async fn knot_offline_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    reject_all_query_parameters(&query)?;
    let locale = resolve_locale(&headers)?;
    let key = public_cache_key(
        "skills-knots-offline-manifest",
        &format!("v1|{}", locale.as_str()),
    );

    if let Some(cached) = get_public_response(&state, &key).await {
        return cached_localized_json(&state, &headers, locale, cached);
    }

    let manifest = state.knot_repository().offline_manifest(locale).await?;
    let cached = set_public_response(&state, &key, &manifest)
        .await
        .map_err(ApiError::internal)?;
    cached_localized_json(&state, &headers, locale, cached)
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
    localized_json(&state, &headers, locale, detail)
}

fn cached_localized_json(
    state: &AppState,
    request_headers: &HeaderMap,
    locale: Locale,
    cached: CachedPublicResponse,
) -> Result<Response, ApiError> {
    let cache_control = format!(
        "public, max-age={}, stale-while-revalidate={}",
        state.config().public_api.cache_ttl_seconds,
        state.config().public_api.cache_stale_seconds
    );

    if request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .any(|candidate| candidate.trim() == cached.etag)
        })
        .unwrap_or(false)
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        insert_public_headers(response.headers_mut(), locale, &cached.etag, &cache_control)?;
        return Ok(response);
    }

    let mut response = (StatusCode::OK, cached.body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    insert_public_headers(response.headers_mut(), locale, &cached.etag, &cache_control)?;
    Ok(response)
}

fn localized_json<T: Serialize>(
    state: &AppState,
    request_headers: &HeaderMap,
    locale: Locale,
    body: T,
) -> Result<Response, ApiError> {
    let body = serde_json::to_string(&body).map_err(ApiError::internal)?;
    let etag = format!("\"{}\"", hex::encode(Sha256::digest(body.as_bytes())));
    let cache_control = format!(
        "public, max-age={}, stale-while-revalidate={}",
        state.config().public_api.cache_ttl_seconds,
        state.config().public_api.cache_stale_seconds
    );

    if request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(',').any(|candidate| candidate.trim() == etag))
        .unwrap_or(false)
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        insert_public_headers(response.headers_mut(), locale, &etag, &cache_control)?;
        return Ok(response);
    }

    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    insert_public_headers(response.headers_mut(), locale, &etag, &cache_control)?;
    Ok(response)
}

fn insert_public_headers(
    headers: &mut HeaderMap,
    locale: Locale,
    etag: &str,
    cache_control: &str,
) -> Result<(), ApiError> {
    headers.insert(
        header::CONTENT_LANGUAGE,
        HeaderValue::from_str(locale.as_str())
            .map_err(|_| ApiError::invalid_header(header::CONTENT_LANGUAGE.as_str()))?,
    );
    headers.insert(
        header::VARY,
        HeaderValue::from_static("Accept-Language, X-StellarTrail-Locale"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(cache_control)
            .map_err(|_| ApiError::invalid_header(header::CACHE_CONTROL.as_str()))?,
    );
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(etag).map_err(|_| ApiError::invalid_header(header::ETAG.as_str()))?,
    );
    Ok(())
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

fn reject_all_query_parameters(query: &HashMap<String, String>) -> Result<(), ApiError> {
    if let Some(parameter) = query.keys().min() {
        return Err(ApiError::unsupported_query_parameter(parameter.clone()));
    }
    Ok(())
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
