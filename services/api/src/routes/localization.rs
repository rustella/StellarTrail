//! Shared helpers for locale-resolved public JSON responses.

use std::{collections::HashMap, future::Future};

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use stellartrail_domain::locale::Locale;

use crate::{
    error::ApiError,
    services::public_response_cache::{
        CachedPublicResponse, get_public_response, public_cache_key, set_public_response,
    },
    state::AppState,
};

/// Resolves a localized public JSON response through the shared public response cache.
pub(crate) async fn cached_localized_json_with<T, F, Fut>(
    state: &AppState,
    request_headers: &HeaderMap,
    locale: Locale,
    endpoint_class: &str,
    normalized_input: &str,
    load: F,
) -> Result<Response, ApiError>
where
    T: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let key = public_cache_key(endpoint_class, normalized_input);
    if let Some(cached) = get_public_response(state, &key).await {
        return cached_localized_json(state, request_headers, locale, cached);
    }

    let body = load().await?;
    let cached = set_public_response(state, &key, &body)
        .await
        .map_err(ApiError::internal)?;
    cached_localized_json(state, request_headers, locale, cached)
}

pub(crate) fn cached_localized_json(
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

pub(crate) fn localized_json<T: Serialize>(
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

pub(crate) fn resolve_locale(headers: &HeaderMap) -> Result<Locale, ApiError> {
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

pub(crate) fn reject_query_locale(query: &HashMap<String, String>) -> Result<(), ApiError> {
    if query.contains_key("locale") {
        Err(ApiError::unsupported_query_parameter("locale"))
    } else {
        Ok(())
    }
}

pub(crate) fn reject_all_query_parameters(query: &HashMap<String, String>) -> Result<(), ApiError> {
    if let Some(parameter) = query.keys().min() {
        return Err(ApiError::unsupported_query_parameter(parameter.clone()));
    }
    Ok(())
}

pub(crate) fn parse_u32_query(
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
