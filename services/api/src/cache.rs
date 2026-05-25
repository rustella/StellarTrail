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
use stellartrail_domain::gear::{GearCategory, GearSpecs, allowed_spec_keys};
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
        self.set_json_with_ttl(key, value, self.gear_ttl).await;
    }

    /// Serializes the response to JSON with a caller-provided TTL.
    pub async fn set_json_with_ttl<T>(&self, key: &str, value: &T, ttl: Duration)
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
        if let Err(error) = store.set(key, &payload, ttl).await {
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

    /// Reads the version used in public gear-atlas response cache keys.
    pub async fn public_gear_atlas_version(&self) -> u64 {
        let Some(store) = self.store.as_ref() else {
            return 0;
        };
        let key = self.public_gear_atlas_version_key();
        match store.get(&key).await {
            Ok(Some(value)) => value.parse::<u64>().unwrap_or(0),
            Ok(None) => 0,
            Err(error) => {
                warn!(key, error = %error, "redis gear-atlas public cache version read failed");
                0
            }
        }
    }

    /// Increments the public gear-atlas version so old public response keys age out naturally.
    pub async fn invalidate_public_gear_atlas(&self) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let key = self.public_gear_atlas_version_key();
        if let Err(error) = store.increment(&key).await {
            warn!(key, error = %error, "redis gear-atlas public cache invalidation failed");
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

    /// Records which normalized spec fields the current user submitted for a category without storing any spec values.
    pub async fn record_gear_spec_keys(
        &self,
        user_id: &str,
        category: GearCategory,
        specs: &GearSpecs,
    ) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let allowed_keys = allowed_spec_keys(category);
        let key = self.gear_spec_keys_key(user_id, category);
        for spec_key in specs
            .iter()
            .filter(|(spec_key, value)| {
                allowed_keys.contains(&spec_key.as_str()) && !value.trim().is_empty()
            })
            .map(|(spec_key, _)| spec_key)
        {
            if let Err(error) = store.sorted_set_increment(&key, spec_key, 1.0).await {
                warn!(key, spec_key, error = %error, "redis gear spec-key frequency write failed");
                return;
            }
        }
    }

    /// Reads the user's high-frequency spec fields for a category, returning only fields supported by the current category config.
    pub async fn gear_spec_key_rankings(
        &self,
        user_id: &str,
        category: GearCategory,
    ) -> Vec<String> {
        let Some(store) = self.store.as_ref() else {
            return Vec::new();
        };
        let allowed_keys = allowed_spec_keys(category);
        let key = self.gear_spec_keys_key(user_id, category);
        let ranked_keys = match store
            .sorted_set_rev_range(&key, GEAR_SPEC_KEY_RANKING_SCAN_LIMIT)
            .await
        {
            Ok(keys) => keys,
            Err(error) => {
                warn!(key, error = %error, "redis gear spec-key frequency read failed");
                return Vec::new();
            }
        };
        ranked_keys
            .into_iter()
            .filter(|spec_key| allowed_keys.contains(&spec_key.as_str()))
            .take(allowed_keys.len())
            .collect()
    }

    /// Records normalized gear tags and optional user color preferences in Redis-only per-user stores.
    pub async fn record_gear_tags(
        &self,
        user_id: &str,
        tags: &[String],
        tag_colors: &HashMap<String, String>,
    ) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let ranking_key = self.gear_tags_key(user_id);
        let colors_key = self.gear_tag_colors_key(user_id);
        for tag in tags
            .iter()
            .map(|value| value.trim())
            .filter(|tag| !tag.is_empty())
        {
            if let Err(error) = store.sorted_set_increment(&ranking_key, tag, 1.0).await {
                warn!(key = ranking_key, tag, error = %error, "redis gear tag frequency write failed");
                return;
            }
            if let Some(color) = tag_colors
                .get(tag)
                .map(|value| value.trim())
                .filter(|color| is_supported_gear_tag_color(color))
            {
                match store.hash_set(&colors_key, tag, color).await {
                    Ok(()) => {}
                    Err(error) => {
                        warn!(key = colors_key, tag, error = %error, "redis gear tag color write failed");
                        return;
                    }
                }
            }
        }
    }

    /// Reads high-frequency tag suggestions with their current user-level color preference.
    pub async fn gear_tag_suggestions(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Vec<(String, Option<String>)> {
        if limit == 0 {
            return Vec::new();
        }
        let Some(store) = self.store.as_ref() else {
            return Vec::new();
        };
        let ranking_key = self.gear_tags_key(user_id);
        let colors_key = self.gear_tag_colors_key(user_id);
        let tags = match store.sorted_set_rev_range(&ranking_key, limit).await {
            Ok(tags) => tags,
            Err(error) => {
                warn!(key = ranking_key, error = %error, "redis gear tag frequency read failed");
                return Vec::new();
            }
        };
        let mut suggestions = Vec::with_capacity(tags.len());
        for tag in tags {
            let color = match store.hash_get(&colors_key, &tag).await {
                Ok(value) => value.filter(|color| is_supported_gear_tag_color(color)),
                Err(error) => {
                    warn!(key = colors_key, tag, error = %error, "redis gear tag color read failed");
                    None
                }
            };
            suggestions.push((tag, color));
        }
        suggestions
    }

    /// Returns the current user-level tag color mapping for the requested tag names.
    pub async fn gear_tag_colors(&self, user_id: &str, tags: &[String]) -> HashMap<String, String> {
        let Some(store) = self.store.as_ref() else {
            return HashMap::new();
        };
        let key = self.gear_tag_colors_key(user_id);
        let mut colors = HashMap::new();
        for tag in tags
            .iter()
            .map(|value| value.trim())
            .filter(|tag| !tag.is_empty())
        {
            if colors.contains_key(tag) {
                continue;
            }
            match store.hash_get(&key, tag).await {
                Ok(Some(color)) if is_supported_gear_tag_color(&color) => {
                    colors.insert(tag.to_owned(), color);
                }
                Ok(_) => {}
                Err(error) => {
                    warn!(key, tag, error = %error, "redis gear tag color read failed");
                    return colors;
                }
            }
        }
        colors
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

    /// Builds the Redis sorted-set key for user/category spec-field frequency counts.
    fn gear_spec_keys_key(&self, user_id: &str, category: GearCategory) -> String {
        format!(
            "{}:gear:{user_id}:spec-keys:{}",
            self.key_prefix,
            category.as_str()
        )
    }

    /// Builds the Redis sorted-set key for user tag frequency counts.
    fn gear_tags_key(&self, user_id: &str) -> String {
        format!("{}:gear:{user_id}:tags", self.key_prefix)
    }

    /// Builds the Redis hash key for user tag color preferences.
    fn gear_tag_colors_key(&self, user_id: &str) -> String {
        format!("{}:gear:{user_id}:tag-colors", self.key_prefix)
    }

    /// Builds the global public atlas version key shared by list and detail response caches.
    fn public_gear_atlas_version_key(&self) -> String {
        format!("{}:gear-atlas:public-response-version", self.key_prefix)
    }
}

const DEFAULT_GEAR_CACHE_TTL_SECONDS: u64 = 30;
const GEAR_SPEC_KEY_RANKING_SCAN_LIMIT: usize = 64;
const GEAR_TAG_COLOR_TOKENS: [&str; 8] = [
    "teal", "blue", "violet", "rose", "orange", "amber", "green", "slate",
];

/// Checks whether a user-supplied tag color token is supported by the Mini Program tag palette.
pub fn is_supported_gear_tag_color(value: &str) -> bool {
    GEAR_TAG_COLOR_TOKENS.contains(&value)
}

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
    /// Increments a sorted-set member and returns the resulting score.
    async fn sorted_set_increment(
        &self,
        key: &str,
        member: &str,
        amount: f64,
    ) -> anyhow::Result<f64>;
    /// Returns sorted-set members ordered by score descending.
    async fn sorted_set_rev_range(&self, key: &str, limit: usize) -> anyhow::Result<Vec<String>>;
    /// Sets a hash field to a string value.
    async fn hash_set(&self, key: &str, field: &str, value: &str) -> anyhow::Result<()>;
    /// Reads a hash field as a string value.
    async fn hash_get(&self, key: &str, field: &str) -> anyhow::Result<Option<String>>;
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
    sorted_sets: HashMap<String, HashMap<String, f64>>,
    hashes: HashMap<String, HashMap<String, String>>,
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
    pub sorted_set_increment_count: usize,
    pub sorted_set_read_count: usize,
    pub hash_set_count: usize,
    pub hash_get_count: usize,
}

impl InMemoryCacheStore {
    /// Runs the `stats` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn stats(&self) -> InMemoryCacheStats {
        self.inner.lock().unwrap().stats
    }

    /// Returns sorted-set members for route tests without exposing any stored values.
    pub fn sorted_set_members(&self, key: &str) -> Vec<String> {
        self.inner
            .lock()
            .unwrap()
            .sorted_sets
            .get(key)
            .map(|items| items.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns hash fields for route tests without exposing Redis implementation details.
    pub fn hash_entries(&self, key: &str) -> HashMap<String, String> {
        self.inner
            .lock()
            .unwrap()
            .hashes
            .get(key)
            .cloned()
            .unwrap_or_default()
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

    async fn sorted_set_increment(
        &self,
        key: &str,
        member: &str,
        amount: f64,
    ) -> anyhow::Result<f64> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.sorted_set_increment_count += 1;
        let members = inner.sorted_sets.entry(key.to_owned()).or_default();
        let score = members.entry(member.to_owned()).or_insert(0.0);
        *score += amount;
        Ok(*score)
    }

    async fn sorted_set_rev_range(&self, key: &str, limit: usize) -> anyhow::Result<Vec<String>> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.sorted_set_read_count += 1;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let Some(members) = inner.sorted_sets.get(key) else {
            return Ok(Vec::new());
        };
        let mut ranked_members: Vec<_> = members.iter().collect();
        ranked_members.sort_by(|(left_key, left_score), (right_key, right_score)| {
            right_score
                .partial_cmp(left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left_key.cmp(right_key))
        });
        Ok(ranked_members
            .into_iter()
            .take(limit)
            .map(|(member, _)| member.clone())
            .collect())
    }

    async fn hash_set(&self, key: &str, field: &str, value: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.hash_set_count += 1;
        inner
            .hashes
            .entry(key.to_owned())
            .or_default()
            .insert(field.to_owned(), value.to_owned());
        Ok(())
    }

    async fn hash_get(&self, key: &str, field: &str) -> anyhow::Result<Option<String>> {
        let mut inner = self.inner.lock().unwrap();
        inner.stats.hash_get_count += 1;
        Ok(inner
            .hashes
            .get(key)
            .and_then(|fields| fields.get(field).cloned()))
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

    async fn sorted_set_increment(
        &self,
        key: &str,
        member: &str,
        amount: f64,
    ) -> anyhow::Result<f64> {
        let mut connection = self.connection.clone();
        let value: String = redis::cmd("ZINCRBY")
            .arg(key)
            .arg(amount)
            .arg(member)
            .query_async(&mut connection)
            .await?;
        Ok(value.parse::<f64>().unwrap_or(0.0))
    }

    async fn sorted_set_rev_range(&self, key: &str, limit: usize) -> anyhow::Result<Vec<String>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut connection = self.connection.clone();
        let stop = limit.saturating_sub(1) as isize;
        Ok(redis::cmd("ZREVRANGE")
            .arg(key)
            .arg(0)
            .arg(stop)
            .query_async(&mut connection)
            .await?)
    }

    async fn hash_set(&self, key: &str, field: &str, value: &str) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let _: () = connection.hset(key, field, value).await?;
        Ok(())
    }

    async fn hash_get(&self, key: &str, field: &str) -> anyhow::Result<Option<String>> {
        let mut connection = self.connection.clone();
        Ok(connection.hget(key, field).await?)
    }
}

/// Runs the `digest` server-side flow while preserving input validation, error propagation, and state invariants.
fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
