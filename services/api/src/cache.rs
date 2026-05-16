//! Optional cache layer that wraps Redis and test in-memory stores to provide degradable read-through caching for high-traffic gear endpoints.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::config::RedisCacheConfig;

/// Cache facade that hides Redis/in-memory implementation differences and lets callers fall back to the database when caching is unavailable.
#[derive(Clone)]
pub struct Cache {
    store: Option<Arc<dyn CacheStore>>,
    key_prefix: Arc<str>,
    gear_ttl: Duration,
}

impl Cache {
    /// Runs the `disabled` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn disabled() -> Self {
        Self {
            store: None,
            key_prefix: Arc::from("stellartrail"),
            gear_ttl: Duration::from_secs(DEFAULT_GEAR_CACHE_TTL_SECONDS),
        }
    }

    /// Runs the `with store for tests` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn with_store_for_tests(
        store: InMemoryCacheStore,
        key_prefix: impl Into<String>,
        gear_ttl: Duration,
    ) -> Self {
        Self {
            store: Some(Arc::new(store)),
            key_prefix: Arc::from(key_prefix.into()),
            gear_ttl,
        }
    }

    /// Runs the `from config` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn from_config(config: &RedisCacheConfig) -> anyhow::Result<Self> {
        // An empty REDIS_URL explicitly disables caching, so local and test environments do not need Redis running.
        let Some(url) = config.url.as_deref() else {
            return Ok(Self::disabled_with_config(config));
        };
        // Probe Redis during startup first; failures only degrade with a warning and must not block the core API.
        match RedisCacheStore::connect(url).await {
            Ok(store) => Ok(Self {
                store: Some(Arc::new(store)),
                key_prefix: Arc::from(config.key_prefix.clone()),
                gear_ttl: Duration::from_secs(config.gear_ttl_seconds),
            }),
            Err(error) => {
                warn!(error = %error, "redis cache unavailable at startup; continuing without cache");
                Ok(Self::disabled_with_config(config))
            }
        }
    }

    /// Runs the `is enabled` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn is_enabled(&self) -> bool {
        self.store.is_some()
    }

    /// Builds a response cache key for gear read endpoints and includes the per-user version to support invalidation after writes.
    pub async fn gear_response_key(
        &self,
        user_id: &str,
        scope: &str,
        payload: &str,
    ) -> Option<String> {
        let store = self.store.as_ref()?;
        // The per-user version is part of the response cache key, so writes only need to increment it to age out old cache entries.
        let version_key = self.gear_version_key(user_id);
        let version = match store.get(&version_key).await {
            Ok(Some(value)) => value.parse::<u64>().unwrap_or(0),
            Ok(None) => 0,
            Err(error) => {
                warn!(error = %error, "redis gear cache version read failed; bypassing cache");
                return None;
            }
        };
        Some(format!(
            "{}:gear:{user_id}:v{version}:{scope}:{}",
            self.key_prefix,
            digest(payload),
        ))
    }

    /// Reads JSON from cache and deserializes it into the target response type; failures return None to trigger a database fallback.
    pub async fn get_json<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let store = self.store.as_ref()?;
        match store.get(key).await {
            // The cache stores complete response JSON; deserialization failures fall back to the database to keep responses correct.
            Ok(Some(value)) => match serde_json::from_str(&value) {
                Ok(value) => Some(value),
                Err(error) => {
                    warn!(key, error = %error, "redis cache payload decode failed; bypassing cache");
                    None
                }
            },
            Ok(None) => None,
            Err(error) => {
                warn!(key, error = %error, "redis cache read failed; bypassing cache");
                None
            }
        }
    }

    /// Serializes the response to JSON for caching; write failures are warning-only and do not affect the HTTP response.
    pub async fn set_json<T>(&self, key: &str, value: &T)
    where
        T: Serialize,
    {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(value) else {
            warn!(
                key,
                "redis cache payload encode failed; skipping cache write"
            );
            return;
        };
        if let Err(error) = store.set(key, &payload, self.gear_ttl).await {
            warn!(key, error = %error, "redis cache write failed; continuing without cache");
        }
    }

    /// Increments the user gear cache version so keys from older versions expire naturally.
    pub async fn invalidate_user_gear(&self, user_id: &str) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let key = self.gear_version_key(user_id);
        if let Err(error) = store.increment(&key).await {
            warn!(key, error = %error, "redis gear cache invalidation failed");
        }
    }

    /// Increments a key and applies a TTL on first use. Returns None when the cache is disabled or unavailable.
    pub async fn increment_with_ttl(&self, key: &str, ttl: Duration) -> Option<u64> {
        let store = self.store.as_ref()?;
        match store.increment_with_ttl(key, ttl).await {
            Ok(value) => Some(value),
            Err(error) => {
                warn!(key, error = %error, "redis increment-with-ttl failed; falling back when possible");
                None
            }
        }
    }

    /// Runs the `disabled with config` server-side flow while preserving input validation, error propagation, and state invariants.
    fn disabled_with_config(config: &RedisCacheConfig) -> Self {
        Self {
            store: None,
            key_prefix: Arc::from(config.key_prefix.clone()),
            gear_ttl: Duration::from_secs(config.gear_ttl_seconds),
        }
    }

    /// Runs the `gear version key` server-side flow while preserving input validation, error propagation, and state invariants.
    fn gear_version_key(&self, user_id: &str) -> String {
        format!("{}:gear:{user_id}:version", self.key_prefix)
    }
}

const DEFAULT_GEAR_CACHE_TTL_SECONDS: u64 = 30;

/// Cache store abstraction requiring both Redis and in-memory test stores to support read, write, and version increment operations.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Runs the `get` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
    /// Runs the `set` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()>;
    /// Runs the `increment` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn increment(&self, key: &str) -> anyhow::Result<u64>;
    /// Increments a key and sets a TTL on first use.
    async fn increment_with_ttl(&self, key: &str, ttl: Duration) -> anyhow::Result<u64>;
}

/// Stable data boundary for `InMemoryCacheStore`, exposed by or reused within this module.
#[derive(Clone, Default)]
pub struct InMemoryCacheStore {
    inner: Arc<Mutex<InMemoryCacheInner>>,
}

/// Stable data boundary for `InMemoryCacheInner`, exposed by or reused within this module.
#[derive(Default)]
struct InMemoryCacheInner {
    entries: HashMap<String, InMemoryCacheEntry>,
    stats: InMemoryCacheStats,
}

/// Stable data boundary for `InMemoryCacheEntry`, exposed by or reused within this module.
struct InMemoryCacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

/// Stable data boundary for `InMemoryCacheStats`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Default)]
pub struct InMemoryCacheStats {
    pub get_count: usize,
    pub hit_count: usize,
    pub set_count: usize,
    pub increment_count: usize,
}

impl InMemoryCacheStore {
    /// Runs the `stats` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn stats(&self) -> InMemoryCacheStats {
        self.inner.lock().unwrap().stats
    }
}

#[async_trait]
impl CacheStore for InMemoryCacheStore {
    /// Runs the `get` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.get_count += 1;
        let expired = inner
            .entries
            .get(key)
            .and_then(|entry| entry.expires_at)
            .is_some_and(|expires_at| expires_at <= Instant::now());
        if expired {
            inner.entries.remove(key);
            return Ok(None);
        }
        let value = inner.entries.get(key).map(|entry| entry.value.clone());
        if value.is_some() {
            inner.stats.hit_count += 1;
        }
        Ok(value)
    }

    /// Runs the `set` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.set_count += 1;
        inner.entries.insert(
            key.to_owned(),
            InMemoryCacheEntry {
                value: value.to_owned(),
                expires_at: Some(Instant::now() + ttl),
            },
        );
        Ok(())
    }

    /// Runs the `increment` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn increment(&self, key: &str) -> anyhow::Result<u64> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.increment_count += 1;
        let current = inner
            .entries
            .get(key)
            .and_then(|entry| entry.value.parse::<u64>().ok())
            .unwrap_or(0);
        let next = current + 1;
        inner.entries.insert(
            key.to_owned(),
            InMemoryCacheEntry {
                value: next.to_string(),
                expires_at: None,
            },
        );
        Ok(next)
    }

    async fn increment_with_ttl(&self, key: &str, ttl: Duration) -> anyhow::Result<u64> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.increment_count += 1;
        let now = Instant::now();
        let expired = inner
            .entries
            .get(key)
            .and_then(|entry| entry.expires_at)
            .is_some_and(|expires_at| expires_at <= now);
        if expired {
            inner.entries.remove(key);
        }
        let current = inner
            .entries
            .get(key)
            .and_then(|entry| entry.value.parse::<u64>().ok())
            .unwrap_or(0);
        let next = current + 1;
        inner.entries.insert(
            key.to_owned(),
            InMemoryCacheEntry {
                value: next.to_string(),
                expires_at: Some(now + ttl),
            },
        );
        Ok(next)
    }
}

/// Stable data boundary for `RedisCacheStore`, exposed by or reused within this module.
#[derive(Clone)]
struct RedisCacheStore {
    connection: redis::aio::MultiplexedConnection,
}

impl RedisCacheStore {
    /// Runs the `connect` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let mut connection = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut connection).await?;
        Ok(Self { connection })
    }
}

#[async_trait]
impl CacheStore for RedisCacheStore {
    /// Runs the `get` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut connection = self.connection.clone();
        Ok(connection.get(key).await?)
    }

    /// Runs the `set` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let _: () = connection.set_ex(key, value, ttl.as_secs()).await?;
        Ok(())
    }

    /// Runs the `increment` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn increment(&self, key: &str) -> anyhow::Result<u64> {
        let mut connection = self.connection.clone();
        let value: i64 = connection.incr(key, 1).await?;
        Ok(value.max(0) as u64)
    }

    async fn increment_with_ttl(&self, key: &str, ttl: Duration) -> anyhow::Result<u64> {
        let mut connection = self.connection.clone();
        let value: i64 = connection.incr(key, 1).await?;
        if value == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(key)
                .arg(ttl.as_secs())
                .query_async(&mut connection)
                .await?;
        }
        Ok(value.max(0) as u64)
    }
}

/// Runs the `digest` server-side flow while preserving input validation, error propagation, and state invariants.
fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
