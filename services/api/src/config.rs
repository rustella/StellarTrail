//! API service configuration module that parses environment variables for database, WeChat login, Redis cache, and other runtime settings.

use std::{env, net::SocketAddr, path::PathBuf};

use stellartrail_db::{DatabaseConfig, DatabaseKind};

/// Stable data boundary for `RedisCacheConfig`, exposed by or reused within this module.
#[derive(Clone, Debug)]
pub struct RedisCacheConfig {
    pub url: Option<String>,
    pub key_prefix: String,
    pub gear_ttl_seconds: u64,
}

impl RedisCacheConfig {
    /// Runs the `disabled` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn disabled() -> Self {
        Self {
            url: None,
            key_prefix: "stellartrail".to_owned(),
            gear_ttl_seconds: 30,
        }
    }
}

/// Runtime API configuration containing environment, bind address, database, WeChat, and content directory settings.
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
    /// Builds runtime configuration from environment variables and parses the port, database URL, and optional Redis TTL.
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
        // Redis remains optional; an empty REDIS_URL is converted to a disabled cache state by the cache layer.
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

    /// Runs the `bind addr` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("validated socket address")
    }

    /// Runs the `database kind` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn database_kind(&self) -> DatabaseKind {
        self.database.kind
    }
}

/// Runs the `optional env` server-side flow while preserving input validation, error propagation, and state invariants.
fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Runs the `optional u64 env` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `from env reads wechat credentials` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `from env reads redis cache config` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `snapshot env` server-side flow while preserving input validation, error propagation, and state invariants.
    fn snapshot_env(keys: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
        keys.iter().map(|key| (*key, env::var(key).ok())).collect()
    }

    /// Runs the `restore env` server-side flow while preserving input validation, error propagation, and state invariants.
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
