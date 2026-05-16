//! API 服务配置模块，集中解析环境变量并暴露数据库、微信登录和 Redis 缓存等运行时参数。

use std::{env, net::SocketAddr, path::PathBuf};

use stellartrail_db::{DatabaseConfig, DatabaseKind};

/// RedisCacheConfig 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug)]
pub struct RedisCacheConfig {
    pub url: Option<String>,
    pub key_prefix: String,
    pub gear_ttl_seconds: u64,
}

impl RedisCacheConfig {
    /// 执行 `disabled` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn disabled() -> Self {
        Self {
            url: None,
            key_prefix: "stellartrail".to_owned(),
            gear_ttl_seconds: 30,
        }
    }
}

/// API 服务运行时配置，汇总环境、监听地址、数据库、微信和内容目录等参数。
#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
    pub wechat_mock_login: bool,
    pub wechat_app_id: Option<String>,
    pub wechat_app_secret: Option<String>,
    pub content_dir: PathBuf,
    pub redis_cache: RedisCacheConfig,
}

impl ApiConfig {
    /// 从环境变量构建运行配置，并对端口、数据库连接串和可选 Redis TTL 做基础解析。
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_owned());
        let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()?;
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://stellartrail.db".to_owned());
        let wechat_mock_login = env::var("WECHAT_MOCK_LOGIN")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(app_env == "local");
        let wechat_app_id = optional_env("WECHAT_APP_ID");
        let wechat_app_secret = optional_env("WECHAT_APP_SECRET");
        let content_dir = env::var("CONTENT_DIR").unwrap_or_else(|_| "content".to_owned());
        // Redis 配置保持可选，空 REDIS_URL 会在缓存层转为 disabled 状态。
        let redis_cache = RedisCacheConfig {
            url: optional_env("REDIS_URL"),
            key_prefix: optional_env("REDIS_KEY_PREFIX")
                .unwrap_or_else(|| "stellartrail".to_owned()),
            gear_ttl_seconds: optional_u64_env("REDIS_GEAR_CACHE_TTL_SECONDS", 30)?,
        };

        Ok(Self {
            app_env,
            host,
            port,
            database: DatabaseConfig::new(database_url)?,
            wechat_mock_login,
            wechat_app_id,
            wechat_app_secret,
            content_dir: PathBuf::from(content_dir),
            redis_cache,
        })
    }

    /// 执行 `bind addr` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("validated socket address")
    }

    /// 执行 `database kind` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn database_kind(&self) -> DatabaseKind {
        self.database.kind
    }
}

/// 执行 `optional env` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// 执行 `optional u64 env` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn optional_u64_env(name: &str, default: u64) -> anyhow::Result<u64> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u64>()?),
        None => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 执行 `from env reads wechat credentials` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    #[test]
    fn from_env_reads_wechat_credentials() {
        let _guard = ENV_LOCK.lock().unwrap();
        let keys = [
            "APP_ENV",
            "APP_HOST",
            "APP_PORT",
            "DATABASE_URL",
            "WECHAT_MOCK_LOGIN",
            "WECHAT_APP_ID",
            "WECHAT_APP_SECRET",
            "CONTENT_DIR",
            "REDIS_URL",
            "REDIS_KEY_PREFIX",
            "REDIS_GEAR_CACHE_TTL_SECONDS",
        ];
        let saved = snapshot_env(&keys);

        unsafe {
            env::set_var("APP_ENV", "production");
            env::set_var("APP_HOST", "127.0.0.1");
            env::set_var("APP_PORT", "8080");
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("WECHAT_MOCK_LOGIN", "false");
            env::set_var("WECHAT_APP_ID", " wx-app-id ");
            env::set_var("WECHAT_APP_SECRET", " wx-secret ");
            env::set_var("CONTENT_DIR", "content");
            env::remove_var("REDIS_URL");
            env::remove_var("REDIS_KEY_PREFIX");
            env::remove_var("REDIS_GEAR_CACHE_TTL_SECONDS");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert!(!config.wechat_mock_login);
        assert_eq!(config.wechat_app_id.as_deref(), Some("wx-app-id"));
        assert_eq!(config.wechat_app_secret.as_deref(), Some("wx-secret"));

        restore_env(saved);
    }

    /// 执行 `from env reads redis cache config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    #[test]
    fn from_env_reads_redis_cache_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let keys = [
            "APP_ENV",
            "APP_HOST",
            "APP_PORT",
            "DATABASE_URL",
            "WECHAT_MOCK_LOGIN",
            "WECHAT_APP_ID",
            "WECHAT_APP_SECRET",
            "CONTENT_DIR",
            "REDIS_URL",
            "REDIS_KEY_PREFIX",
            "REDIS_GEAR_CACHE_TTL_SECONDS",
        ];
        let saved = snapshot_env(&keys);

        unsafe {
            env::set_var("APP_ENV", "local");
            env::set_var("APP_HOST", "127.0.0.1");
            env::set_var("APP_PORT", "8080");
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("WECHAT_MOCK_LOGIN", "true");
            env::set_var("CONTENT_DIR", "content");
            env::set_var("REDIS_URL", " redis://127.0.0.1:6379/2 ");
            env::set_var("REDIS_KEY_PREFIX", " stellartrail-test ");
            env::set_var("REDIS_GEAR_CACHE_TTL_SECONDS", "45");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(
            config.redis_cache.url.as_deref(),
            Some("redis://127.0.0.1:6379/2"),
        );
        assert_eq!(config.redis_cache.key_prefix, "stellartrail-test");
        assert_eq!(config.redis_cache.gear_ttl_seconds, 45);

        restore_env(saved);
    }

    /// 执行 `snapshot env` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn snapshot_env(keys: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
        keys.iter().map(|key| (*key, env::var(key).ok())).collect()
    }

    /// 执行 `restore env` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn restore_env(saved: Vec<(&'static str, Option<String>)>) {
        for (key, value) in saved {
            unsafe {
                if let Some(value) = value {
                    env::set_var(key, value);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }
}
