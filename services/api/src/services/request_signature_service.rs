//! Request-signature middleware and nonce replay protection for business API routes.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, Method, Uri, header, uri::PathAndQuery},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    config::RequestSignatureConfig,
    error::ApiError,
    routes::{API_PREFIX, API_PREFIX_WITH_SLASH, map::is_map_style_json_path},
    state::AppState,
};

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_ALGORITHM: &str = "STELLARTRAIL-HMAC-SHA256";
const SIGNING_FIELD_APP_ID: &str = "app_id";
const SIGNING_FIELD_NONCE: &str = "nonce";
const SIGNING_FIELD_SIGNATURE: &str = "signature";

/// In-memory fallback store for request-signature nonce replay protection.
#[derive(Clone, Default)]
pub struct InMemoryRequestSignatureNonceStore {
    inner: Arc<Mutex<HashMap<String, Instant>>>,
}

impl InMemoryRequestSignatureNonceStore {
    /// Records a nonce key for the TTL window and returns false when it already exists.
    pub fn record_once(&self, key: &str, ttl: Duration) -> bool {
        let now = Instant::now();
        let mut inner = self.inner.lock().expect("signature nonce mutex poisoned");
        inner.retain(|_, expires_at| *expires_at > now);
        if inner.contains_key(key) {
            return false;
        }
        inner.insert(key.to_owned(), now + ttl);
        true
    }
}

#[derive(Debug)]
struct SignatureFields {
    app_id: String,
    nonce: String,
    signature: String,
}

/// Validates request signatures for protected API routes before handlers read request bodies.
pub async fn enforce_request_signature(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !should_verify_request(&state, request.method(), request.uri().path()) {
        return next.run(request).await;
    }

    match verify_request_signature(state, request).await {
        Ok(request) => next.run(request).await,
        Err(error) => error.into_response(),
    }
}

fn should_verify_request(state: &AppState, method: &Method, path: &str) -> bool {
    state.config().request_signature.enabled
        && method != Method::OPTIONS
        && is_versioned_api_path(path)
        && !is_signature_exempt_path(path)
}

fn is_versioned_api_path(path: &str) -> bool {
    path == API_PREFIX || path.starts_with(API_PREFIX_WITH_SLASH)
}

fn is_signature_exempt_path(path: &str) -> bool {
    is_map_style_json_path(path)
        || matches!(
            path,
            "/healthz" | "/ping" | "/echo" | "/api/v1/ping" | "/api/v1/echo"
        )
}

async fn verify_request_signature(state: AppState, request: Request) -> Result<Request, ApiError> {
    let (mut parts, body) = request.into_parts();
    let body_limit = signature_body_limit(&state);
    let body_bytes = to_bytes(body, body_limit)
        .await
        .map_err(|_| ApiError::PayloadTooLarge {
            max_bytes: body_limit as u64,
        })?;
    let query = parts.uri.query().unwrap_or_default().to_owned();
    let is_json_body = is_json_request(&parts.headers) && !body_bytes.is_empty();
    let fields = if is_json_body {
        signature_fields_from_json(&body_bytes)?
    } else {
        signature_fields_from_query(&query)?
    };
    let app_secret = app_secret_for(&state.config().request_signature, &fields.app_id)?;
    let body_hash = if is_json_body {
        signed_json_body_sha256_hex(&body_bytes)?
    } else {
        sha256_hex(&body_bytes)
    };
    let canonical_request = canonical_request(
        parts.method.as_str(),
        parts.uri.path(),
        &canonical_query(&query),
        &body_hash,
        &fields.app_id,
        &fields.nonce,
    );
    if !verify_signature(app_secret, &canonical_request, &fields.signature) {
        return Err(ApiError::InvalidRequestSignature);
    }
    record_nonce_once(&state, &fields.app_id, &fields.nonce).await?;
    if !is_json_body {
        parts.uri = uri_without_signing_query_fields(&parts.uri)?;
    }
    Ok(Request::from_parts(parts, Body::from(body_bytes)))
}

fn signature_body_limit(state: &AppState) -> usize {
    state
        .config()
        .upload
        .max_image_bytes
        .max(state.config().avatar_storage.max_image_bytes)
        .max(state.config().knots_media_storage.max_video_bytes)
        .saturating_add(1_000_000) as usize
}

fn is_json_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let media_type = value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            media_type == "application/json" || media_type.ends_with("+json")
        })
        .unwrap_or(false)
}

fn signature_fields_from_json(body: &Bytes) -> Result<SignatureFields, ApiError> {
    let value: Value =
        serde_json::from_slice(body).map_err(|_| ApiError::InvalidRequestSignature)?;
    let Value::Object(object) = value else {
        return Err(ApiError::InvalidRequestSignature);
    };
    let field = |name: &str| -> Result<String, ApiError> {
        object
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or(ApiError::InvalidRequestSignature)
    };
    Ok(SignatureFields {
        app_id: field(SIGNING_FIELD_APP_ID)?,
        nonce: field(SIGNING_FIELD_NONCE)?,
        signature: field(SIGNING_FIELD_SIGNATURE)?,
    })
}

fn signature_fields_from_query(query: &str) -> Result<SignatureFields, ApiError> {
    Ok(SignatureFields {
        app_id: single_query_value(query, SIGNING_FIELD_APP_ID)?,
        nonce: single_query_value(query, SIGNING_FIELD_NONCE)?,
        signature: single_query_value(query, SIGNING_FIELD_SIGNATURE)?,
    })
}

fn single_query_value(query: &str, name: &str) -> Result<String, ApiError> {
    let mut value = None;
    for (key, candidate) in query_pairs(query) {
        if key != name {
            continue;
        }
        if value.is_some() {
            return Err(ApiError::InvalidRequestSignature);
        }
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return Err(ApiError::InvalidRequestSignature);
        }
        value = Some(candidate.to_owned());
    }
    value.ok_or(ApiError::InvalidRequestSignature)
}

fn app_secret_for<'a>(
    config: &'a RequestSignatureConfig,
    app_id: &str,
) -> Result<&'a str, ApiError> {
    config
        .clients
        .iter()
        .find(|client| client.app_id == app_id)
        .map(|client| client.app_secret.as_str())
        .ok_or(ApiError::InvalidRequestSignature)
}

fn signed_json_body_sha256_hex(body: &Bytes) -> Result<String, ApiError> {
    let mut value: Value =
        serde_json::from_slice(body).map_err(|_| ApiError::InvalidRequestSignature)?;
    let Value::Object(object) = &mut value else {
        return Err(ApiError::InvalidRequestSignature);
    };
    object.remove(SIGNING_FIELD_APP_ID);
    object.remove(SIGNING_FIELD_NONCE);
    object.remove(SIGNING_FIELD_SIGNATURE);
    let mut canonical = String::new();
    write_canonical_json(&value, &mut canonical)?;
    Ok(sha256_hex(canonical.as_bytes()))
}

fn write_canonical_json(value: &Value, output: &mut String) -> Result<(), ApiError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            output.push_str(&serde_json::to_string(value).map_err(ApiError::internal)?);
        }
        Value::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(item, output)?;
            }
            output.push(']');
        }
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            output.push('{');
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).map_err(ApiError::internal)?);
                output.push(':');
                write_canonical_json(item, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

fn canonical_query(query: &str) -> String {
    let mut pairs = query_pairs(query)
        .into_iter()
        .filter(|(key, _)| *key != SIGNING_FIELD_SIGNATURE)
        .collect::<Vec<_>>();
    pairs.sort_by(|(left_key, left_value), (right_key, right_value)| {
        left_key
            .cmp(right_key)
            .then_with(|| left_value.cmp(right_value))
    });
    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn query_pairs(query: &str) -> Vec<(&str, &str)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
        .collect()
}

fn uri_without_signing_query_fields(uri: &Uri) -> Result<Uri, ApiError> {
    let Some(query) = uri.query() else {
        return Ok(uri.clone());
    };
    let forwarded_query = query_pairs(query)
        .into_iter()
        .filter(|(key, _)| !is_signing_query_field(key))
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    let path_and_query = if forwarded_query.is_empty() {
        uri.path().to_owned()
    } else {
        format!("{}?{forwarded_query}", uri.path())
    };
    let mut parts = uri.clone().into_parts();
    parts.path_and_query =
        Some(PathAndQuery::from_maybe_shared(path_and_query).map_err(ApiError::internal)?);
    Uri::from_parts(parts).map_err(ApiError::internal)
}

fn is_signing_query_field(key: &str) -> bool {
    matches!(
        key,
        SIGNING_FIELD_APP_ID | SIGNING_FIELD_NONCE | SIGNING_FIELD_SIGNATURE
    )
}

fn canonical_request(
    method: &str,
    path: &str,
    canonical_query: &str,
    body_hash_hex: &str,
    app_id: &str,
    nonce: &str,
) -> String {
    [
        SIGNATURE_ALGORITHM,
        method,
        path,
        canonical_query,
        body_hash_hex,
        app_id,
        nonce,
    ]
    .join("\n")
}

fn verify_signature(app_secret: &str, canonical_request: &str, signature: &str) -> bool {
    let Ok(signature_bytes) = hex::decode(signature) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(app_secret.as_bytes()) else {
        return false;
    };
    mac.update(canonical_request.as_bytes());
    mac.verify_slice(&signature_bytes).is_ok()
}

async fn record_nonce_once(state: &AppState, app_id: &str, nonce: &str) -> Result<(), ApiError> {
    let ttl = Duration::from_secs(state.config().request_signature.nonce_ttl_seconds);
    let key = format!(
        "{}:request-signature:nonce:{}:{}",
        state.config().redis_cache.key_prefix,
        sha256_hex(app_id.as_bytes()),
        sha256_hex(nonce.as_bytes())
    );
    let accepted = match state.cache().increment_with_ttl(&key, ttl).await {
        Some(count) => count == 1,
        None => state.request_signature_nonce_store().record_once(&key, ttl),
    };
    if accepted {
        Ok(())
    } else {
        Err(ApiError::InvalidRequestSignature)
    }
}

fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(secret: &str, canonical_request: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(canonical_request.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn canonical_query_sorts_pairs_and_removes_signature() {
        assert_eq!(
            canonical_query("z=3&signature=abc&app_id=client&nonce=n1&a=2&a=1"),
            "a=1&a=2&app_id=client&nonce=n1&z=3"
        );
    }

    #[test]
    fn query_signing_fields_are_removed_before_handlers_read_query() {
        let uri = "/api/v1/roadmap?client_key=wechat_miniprogram&app_id=client&nonce=n1&limit=50&signature=sig"
            .parse::<Uri>()
            .unwrap();

        let cleaned = uri_without_signing_query_fields(&uri).unwrap();

        assert_eq!(
            cleaned.path_and_query().unwrap().as_str(),
            "/api/v1/roadmap?client_key=wechat_miniprogram&limit=50"
        );
    }

    #[test]
    fn query_signing_field_removal_does_not_leave_empty_query() {
        let uri = "/api/v1/meta?app_id=client&nonce=n1&signature=sig"
            .parse::<Uri>()
            .unwrap();

        let cleaned = uri_without_signing_query_fields(&uri).unwrap();

        assert_eq!(cleaned.path_and_query().unwrap().as_str(), "/api/v1/meta");
    }

    #[test]
    fn json_body_hash_removes_signing_fields_and_sorts_keys() {
        let left = Bytes::from_static(
            br#"{"b":2,"app_id":"client","nonce":"n1","signature":"sig","a":1}"#,
        );
        let right = Bytes::from_static(br#"{"a":1,"b":2}"#);

        assert_eq!(
            signed_json_body_sha256_hex(&left).unwrap(),
            signed_json_body_sha256_hex(&right).unwrap()
        );
    }

    #[test]
    fn hmac_signature_verifies_and_rejects_bad_values() {
        let canonical = canonical_request(
            "GET",
            "/api/v1/meta",
            "app_id=client&nonce=n1",
            &sha256_hex(b""),
            "client",
            "n1",
        );
        let valid = signature("secret", &canonical);

        assert!(verify_signature("secret", &canonical, &valid));
        assert!(!verify_signature("other-secret", &canonical, &valid));
        assert!(!verify_signature("secret", &canonical, "not-hex"));
    }

    #[test]
    fn in_memory_nonce_store_rejects_replay() {
        let store = InMemoryRequestSignatureNonceStore::default();

        assert!(store.record_once("key", Duration::from_secs(300)));
        assert!(!store.record_once("key", Duration::from_secs(300)));
        assert!(store.record_once("other", Duration::from_secs(300)));
    }
}
