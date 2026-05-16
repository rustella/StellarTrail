//! 可选缓存层，封装 Redis 与测试内存实现，为装备库高频读接口提供可降级的 read-through cache。

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

/// 缓存门面对象，隐藏 Redis/内存实现差异，并在缓存不可用时让调用方自然回源数据库。
#[derive(Clone)]
pub struct Cache {
    store: Option<Arc<dyn CacheStore>>,
    key_prefix: Arc<str>,
    gear_ttl: Duration,
}

impl Cache {
    /// 执行 `disabled` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn disabled() -> Self {
        Self {
            store: None,
            key_prefix: Arc::from("stellartrail"),
            gear_ttl: Duration::from_secs(DEFAULT_GEAR_CACHE_TTL_SECONDS),
        }
    }

    /// 执行 `with store for tests` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `from config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn from_config(config: &RedisCacheConfig) -> anyhow::Result<Self> {
        // REDIS_URL 为空表示显式关闭缓存，本地和测试环境无需额外启动 Redis。
        let Some(url) = config.url.as_deref() else {
            return Ok(Self::disabled_with_config(config));
        };
        // 启动阶段先探测 Redis；失败时只降级并告警，不能阻断核心 API 可用性。
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

    /// 执行 `is enabled` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn is_enabled(&self) -> bool {
        self.store.is_some()
    }

    /// 生成装备读接口响应缓存 key，并把用户级版本号纳入 key 以支持写后失效。
    pub async fn gear_response_key(
        &self,
        user_id: &str,
        scope: &str,
        payload: &str,
    ) -> Option<String> {
        let store = self.store.as_ref()?;
        // 用户级版本号参与响应缓存 key，写操作递增版本即可让旧缓存自然失效。
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

    /// 从缓存读取 JSON 并反序列化为目标响应类型；失败时返回 None 触发数据库回源。
    pub async fn get_json<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let store = self.store.as_ref()?;
        match store.get(key).await {
            // 缓存保存的是完整响应 JSON，反序列化失败时回源数据库以保证响应正确。
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

    /// 将响应序列化为 JSON 写入缓存；写入失败仅记录告警，不影响 HTTP 响应。
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

    /// 递增用户装备缓存版本号，让旧版本 key 自然失效。
    pub async fn invalidate_user_gear(&self, user_id: &str) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let key = self.gear_version_key(user_id);
        if let Err(error) = store.increment(&key).await {
            warn!(key, error = %error, "redis gear cache invalidation failed");
        }
    }

    /// 执行 `disabled with config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn disabled_with_config(config: &RedisCacheConfig) -> Self {
        Self {
            store: None,
            key_prefix: Arc::from(config.key_prefix.clone()),
            gear_ttl: Duration::from_secs(config.gear_ttl_seconds),
        }
    }

    /// 执行 `gear version key` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn gear_version_key(&self, user_id: &str) -> String {
        format!("{}:gear:{user_id}:version", self.key_prefix)
    }
}

const DEFAULT_GEAR_CACHE_TTL_SECONDS: u64 = 30;

/// 缓存存储抽象，约束 Redis 与测试内存实现必须提供读取、写入和版本递增能力。
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// 执行 `get` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
    /// 执行 `set` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()>;
    /// 执行 `increment` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn increment(&self, key: &str) -> anyhow::Result<u64>;
}

/// InMemoryCacheStore 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Default)]
pub struct InMemoryCacheStore {
    inner: Arc<Mutex<InMemoryCacheInner>>,
}

/// InMemoryCacheInner 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Default)]
struct InMemoryCacheInner {
    entries: HashMap<String, InMemoryCacheEntry>,
    stats: InMemoryCacheStats,
}

/// InMemoryCacheEntry 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
struct InMemoryCacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

/// InMemoryCacheStats 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Copy, Debug, Default)]
pub struct InMemoryCacheStats {
    pub get_count: usize,
    pub hit_count: usize,
    pub set_count: usize,
    pub increment_count: usize,
}

impl InMemoryCacheStore {
    /// 执行 `stats` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn stats(&self) -> InMemoryCacheStats {
        self.inner.lock().unwrap().stats
    }
}

#[async_trait]
impl CacheStore for InMemoryCacheStore {
    /// 执行 `get` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `set` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `increment` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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
}

/// RedisCacheStore 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone)]
struct RedisCacheStore {
    connection: redis::aio::MultiplexedConnection,
}

impl RedisCacheStore {
    /// 执行 `connect` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let mut connection = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut connection).await?;
        Ok(Self { connection })
    }
}

#[async_trait]
impl CacheStore for RedisCacheStore {
    /// 执行 `get` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut connection = self.connection.clone();
        Ok(connection.get(key).await?)
    }

    /// 执行 `set` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()> {
        let mut connection = self.connection.clone();
        let _: () = connection.set_ex(key, value, ttl.as_secs()).await?;
        Ok(())
    }

    /// 执行 `increment` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn increment(&self, key: &str) -> anyhow::Result<u64> {
        let mut connection = self.connection.clone();
        let value: i64 = connection.incr(key, 1).await?;
        Ok(value.max(0) as u64)
    }
}

/// 执行 `digest` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
