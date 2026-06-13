//! Client identity middleware for business API requests.
//!
//! Every non-OPTIONS `/api/v1/*` request must declare the calling client and
//! version in `X-StellarTrail-Client` using the compact `<client>/<version>`
//! shape. The value is validated only at the HTTP boundary; it is not persisted
//! or used as an authorization signal.

use axum::{
    extract::Request,
    http::{HeaderMap, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    error::ApiError,
    routes::{API_PREFIX_WITH_SLASH, map::is_map_style_json_path},
};

/// Header carrying the compact client identity value, for example `ios/0.1.0`.
pub const CLIENT_IDENTITY_HEADER: &str = "X-StellarTrail-Client";

const MAX_CLIENT_VERSION_LEN: usize = 64;

/// Validates required client identity headers before protected API handlers run.
pub async fn enforce_client_identity(request: Request, next: Next) -> Response {
    if !should_verify_request(request.method(), request.uri().path()) {
        return next.run(request).await;
    }

    match validate_client_identity(request.headers()) {
        Ok(()) => next.run(request).await,
        Err(error) => error.into_response(),
    }
}

fn should_verify_request(method: &Method, path: &str) -> bool {
    method != Method::OPTIONS
        && path.starts_with(API_PREFIX_WITH_SLASH)
        && !is_map_style_json_path(path)
}

fn validate_client_identity(headers: &HeaderMap) -> Result<(), ApiError> {
    let raw = headers
        .get(CLIENT_IDENTITY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::invalid_header(CLIENT_IDENTITY_HEADER))?;
    let (client, version) = raw
        .split_once('/')
        .ok_or_else(|| ApiError::invalid_header(CLIENT_IDENTITY_HEADER))?;
    let client = client.trim();
    let version = version.trim();
    if !is_supported_client(client) || version.is_empty() || version.len() > MAX_CLIENT_VERSION_LEN
    {
        return Err(ApiError::invalid_header(CLIENT_IDENTITY_HEADER));
    }
    Ok(())
}

fn is_supported_client(value: &str) -> bool {
    matches!(value, "web" | "wechat" | "android" | "ios" | "mac")
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::*;

    #[test]
    fn validates_supported_client_identity() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CLIENT_IDENTITY_HEADER,
            HeaderValue::from_static("ios/0.1.0"),
        );

        assert!(validate_client_identity(&headers).is_ok());
    }

    #[test]
    fn rejects_missing_invalid_or_empty_client_identity() {
        for raw in ["", "desktop/0.1.0", "ios", "/0.1.0", "ios/", "ios/   "] {
            let mut headers = HeaderMap::new();
            headers.insert(CLIENT_IDENTITY_HEADER, HeaderValue::from_str(raw).unwrap());

            assert!(validate_client_identity(&headers).is_err(), "{raw:?}");
        }
    }

    #[test]
    fn should_verify_only_business_non_options_requests() {
        assert!(should_verify_request(&Method::GET, "/api/v1/meta"));
        assert!(should_verify_request(&Method::POST, "/api/v1/auth/login"));
        assert!(!should_verify_request(&Method::OPTIONS, "/api/v1/meta"));
        assert!(!should_verify_request(&Method::GET, "/healthz"));
    }
}
