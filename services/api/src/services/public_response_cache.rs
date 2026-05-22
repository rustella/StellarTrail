//! Public read response cache helpers for unauthenticated APIs.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::state::AppState;

/// Cached public response stored as final JSON body plus strong ETag.
#[derive(Clone, Debug)]
pub struct CachedPublicResponse {
    pub body: String,
    pub etag: String,
}

/// In-memory fallback public response cache.
#[derive(Clone, Default)]
pub struct InMemoryPublicResponseCache {
    inner: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

#[derive(Clone)]
struct CacheEntry {
    response: CachedPublicResponse,
    expires_at: Instant,
}

impl InMemoryPublicResponseCache {
    pub fn get(&self, key: &str) -> Option<CachedPublicResponse> {
        let now = Instant::now();
        let mut inner = self
            .inner
            .lock()
            .expect("public response cache mutex poisoned");
        inner.retain(|_, entry| entry.expires_at > now);
        inner.get(key).map(|entry| entry.response.clone())
    }

    pub fn set(&self, key: &str, response: CachedPublicResponse, ttl: Duration) {
        let mut inner = self
            .inner
            .lock()
            .expect("public response cache mutex poisoned");
        inner.insert(
            key.to_owned(),
            CacheEntry {
                response,
                expires_at: Instant::now() + ttl,
            },
        );
    }

    /// Removes all in-memory fallback entries whose key starts with the given prefix.
    pub fn remove_prefix(&self, prefix: &str) {
        let mut inner = self
            .inner
            .lock()
            .expect("public response cache mutex poisoned");
        inner.retain(|key, _| !key.starts_with(prefix));
    }
}

pub async fn get_public_response(state: &AppState, key: &str) -> Option<CachedPublicResponse> {
    if let Some(response) = state
        .cache()
        .get_json::<CachedPublicResponseWire>(key)
        .await
    {
        return Some(response.into());
    }
    state.public_response_cache().get(key)
}

pub async fn set_public_response<T: Serialize>(
    state: &AppState,
    key: &str,
    value: &T,
) -> anyhow::Result<CachedPublicResponse> {
    let body = serde_json::to_string(value)?;
    let etag = etag_for(&body);
    let cached = CachedPublicResponse { body, etag };
    let ttl = Duration::from_secs(state.config().public_api.cache_ttl_seconds);
    state
        .cache()
        .set_json_with_ttl(key, &CachedPublicResponseWire::from(cached.clone()), ttl)
        .await;
    state.public_response_cache().set(key, cached.clone(), ttl);
    Ok(cached)
}

pub fn public_cache_key(endpoint_class: &str, normalized_input: &str) -> String {
    format!(
        "public-response:{endpoint_class}:{}",
        digest(normalized_input)
    )
}

/// Invalidates public gear-atlas cached responses across Redis keys and the in-process fallback.
pub async fn invalidate_gear_atlas_public_responses(state: &AppState) {
    state.cache().invalidate_public_gear_atlas().await;
    state
        .public_response_cache()
        .remove_prefix("public-response:gear-atlas-");
}

fn etag_for(body: &str) -> String {
    format!("\"{}\"", &digest(body)[..16])
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct CachedPublicResponseWire {
    body: String,
    etag: String,
}

impl From<CachedPublicResponseWire> for CachedPublicResponse {
    fn from(value: CachedPublicResponseWire) -> Self {
        Self {
            body: value.body,
            etag: value.etag,
        }
    }
}

impl From<CachedPublicResponse> for CachedPublicResponseWire {
    fn from(value: CachedPublicResponse) -> Self {
        Self {
            body: value.body,
            etag: value.etag,
        }
    }
}
