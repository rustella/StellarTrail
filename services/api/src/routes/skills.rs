//! Public outdoor skill routes for unauthenticated, DB-backed skill categories and knots.

use std::{collections::HashMap, net::SocketAddr};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use serde_json::json;
use stellartrail_domain::skill::{Locale, SkillCategoriesResponse};

use crate::{
    error::ApiError,
    services::{
        public_rate_limit_service::{RateLimitDecision, check_public_rate_limit},
        public_response_cache::{
            CachedPublicResponse, get_public_response, public_cache_key, set_public_response,
        },
    },
    state::AppState,
};

const VARY_VALUE: &str = "Accept-Language, X-StellarTrail-Locale";

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
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    reject_unexpected_query(&query, &[])?;
    let locale = resolve_locale(&headers)?;
    public_json(
        &state,
        "skills-categories",
        locale,
        "skills-categories",
        &headers,
        connect_info.as_ref(),
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
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    for unsupported in ["cursor", "next_cursor"] {
        if query.contains_key(unsupported) {
            return Err(ApiError::unsupported_query_parameter(unsupported));
        }
    }
    reject_unexpected_query(&query, &["offset", "limit", "category", "q"])?;
    let locale = resolve_locale(&headers)?;
    let offset = parse_u32_query(&query, "offset", 0)?;
    if offset > state.config().public_api.max_offset {
        return Err(ApiError::invalid_query_parameter(
            "offset",
            format!(
                "query parameter `offset` must be at most {}",
                state.config().public_api.max_offset
            ),
        ));
    }
    let limit = parse_u32_query(&query, "limit", 20)?;
    if limit == 0 || limit > state.config().public_api.max_list_limit {
        return Err(ApiError::invalid_query_parameter(
            "limit",
            format!(
                "query parameter `limit` must be in 1..={}",
                state.config().public_api.max_list_limit
            ),
        ));
    }
    let category = optional_safe_slug(&query, "category")?;
    let q = optional_search_query(&query, state.config().public_api.max_search_query_chars)?;
    let normalized = format!(
        "locale={};offset={offset};limit={limit};category={};q={}",
        locale.as_str(),
        category.as_deref().unwrap_or(""),
        q.as_deref().unwrap_or("")
    );
    public_json(
        &state,
        "skills-knots-list",
        locale,
        &normalized,
        &headers,
        connect_info.as_ref(),
        || async {
            state
                .knot_repository()
                .list_knots(locale, offset, limit, category.as_deref(), q.as_deref())
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    reject_unexpected_query(&query, &[])?;
    validate_safe_slug("id", &id)?;
    let locale = resolve_locale(&headers)?;
    let normalized = format!("locale={};id={id}", locale.as_str());
    public_json(
        &state,
        "skills-knots-detail",
        locale,
        &normalized,
        &headers,
        connect_info.as_ref(),
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

async fn public_json<T, F, Fut>(
    state: &AppState,
    endpoint_class: &'static str,
    locale: Locale,
    normalized_input: &str,
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
    load: F,
) -> Result<Response, ApiError>
where
    T: Serialize,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, ApiError>>,
{
    let decision = check_public_rate_limit(state, endpoint_class, headers, connect_info).await;
    if !decision.allowed {
        return Ok(rate_limited_response(decision));
    }

    let key = public_cache_key(endpoint_class, normalized_input);
    if let Some(cached) = get_public_response(state, &key).await {
        return cached_response(state, locale, headers, cached);
    }

    let body = load().await?;
    let cached = set_public_response(state, &key, &body)
        .await
        .map_err(ApiError::internal)?;
    cached_response(state, locale, headers, cached)
}

fn cached_response(
    state: &AppState,
    locale: Locale,
    request_headers: &HeaderMap,
    cached: CachedPublicResponse,
) -> Result<Response, ApiError> {
    if request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|if_none_match| {
            if_none_match
                .split(',')
                .any(|tag| tag.trim() == cached.etag)
        })
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        add_public_headers(response.headers_mut(), state, locale, &cached.etag)?;
        return Ok(response);
    }

    let mut response = Response::new(Body::from(cached.body));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    add_public_headers(response.headers_mut(), state, locale, &cached.etag)?;
    Ok(response)
}

fn add_public_headers(
    headers: &mut HeaderMap,
    state: &AppState,
    locale: Locale,
    etag: &str,
) -> Result<(), ApiError> {
    headers.insert(
        header::CONTENT_LANGUAGE,
        HeaderValue::from_str(locale.as_str())
            .map_err(|_| ApiError::invalid_header(header::CONTENT_LANGUAGE.as_str()))?,
    );
    headers.insert(header::VARY, HeaderValue::from_static(VARY_VALUE));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(&format!(
            "public, max-age={}, stale-while-revalidate={}",
            state.config().public_api.cache_ttl_seconds.min(60),
            state.config().public_api.cache_stale_seconds
        ))
        .map_err(|_| ApiError::invalid_header(header::CACHE_CONTROL.as_str()))?,
    );
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(etag).map_err(|_| ApiError::invalid_header(header::ETAG.as_str()))?,
    );
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    Ok(())
}

fn rate_limited_response(decision: RateLimitDecision) -> Response {
    let body = json!({
        "code": "rate_limited",
        "message": "Too many requests. Please retry later."
    })
    .to_string();
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&decision.retry_after_seconds.to_string())
            .expect("retry-after is numeric"),
    );
    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&decision.limit.to_string()).expect("limit is numeric"),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&decision.remaining.to_string()).expect("remaining is numeric"),
    );
    headers.insert(
        "X-RateLimit-Reset",
        HeaderValue::from_str(&decision.reset_unix_seconds.to_string()).expect("reset is numeric"),
    );
    response
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

fn reject_unexpected_query(
    query: &HashMap<String, String>,
    allowed: &[&str],
) -> Result<(), ApiError> {
    for key in query.keys() {
        if !allowed.iter().any(|allowed| allowed == key) {
            return Err(ApiError::unsupported_query_parameter(key.clone()));
        }
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

fn optional_safe_slug(
    query: &HashMap<String, String>,
    key: &'static str,
) -> Result<Option<String>, ApiError> {
    query
        .get(key)
        .map(|value| {
            let value = value.trim().to_owned();
            validate_safe_slug(key, &value)?;
            Ok(value)
        })
        .transpose()
}

fn optional_search_query(
    query: &HashMap<String, String>,
    max_chars: usize,
) -> Result<Option<String>, ApiError> {
    query
        .get("q")
        .map(|value| {
            let value = value.trim().to_owned();
            if value.chars().count() > max_chars {
                return Err(ApiError::invalid_query_parameter(
                    "q",
                    format!("query parameter `q` must be at most {max_chars} characters"),
                ));
            }
            Ok(value)
        })
        .transpose()
        .map(|value| value.filter(|value| !value.is_empty()))
}

fn validate_safe_slug(parameter: &'static str, value: &str) -> Result<(), ApiError> {
    let valid_len = (1..=128).contains(&value.len());
    let valid_chars = value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    let valid_edges = value
        .as_bytes()
        .first()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit());
    if valid_len && valid_chars && valid_edges {
        Ok(())
    } else {
        Err(ApiError::invalid_query_parameter(
            parameter,
            format!("`{parameter}` must match [a-z0-9][a-z0-9-]{{0,127}}"),
        ))
    }
}
