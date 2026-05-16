//! API service configuration module that parses environment variables for database, WeChat login, Redis cache, uploads, object storage, and other runtime settings.

use std::{env, net::SocketAddr, path::PathBuf};

use stellartrail_db::{DatabaseConfig, DatabaseKind};

/// Redis read-through cache configuration.
#[derive(Clone, Debug)]
pub struct RedisCacheConfig {
    pub url: Option<String>,
    pub key_prefix: String,
    pub gear_ttl_seconds: u64,
}

/// Public unauthenticated read API protection and cache configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicApiConfig {
    pub rate_limit_enabled: bool,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_max_requests_per_ip: u64,
    pub cache_ttl_seconds: u64,
    pub cache_stale_seconds: u64,
    pub max_list_limit: u32,
    pub max_search_query_chars: usize,
    pub max_offset: u32,
    pub trust_proxy_headers: bool,
    pub trusted_proxy_cidrs: Vec<String>,
}

impl Default for PublicApiConfig {
    fn default() -> Self {
        Self {
            rate_limit_enabled: true,
            rate_limit_window_seconds: 60,
            rate_limit_max_requests_per_ip: 120,
            cache_ttl_seconds: 300,
            cache_stale_seconds: 600,
            max_list_limit: 100,
            max_search_query_chars: 64,
            max_offset: 10_000,
            trust_proxy_headers: false,
            trusted_proxy_cidrs: Vec::new(),
        }
    }
}

impl RedisCacheConfig {
    /// Returns a disabled cache configuration for tests and local defaults.
    pub fn disabled() -> Self {
        Self {
            url: None,
            key_prefix: "stellartrail".to_owned(),
            gear_ttl_seconds: 30,
        }
    }
}

/// Feedback image upload limits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadConfig {
    pub max_image_bytes: u64,
    pub rate_limit_window_seconds: u64,
    pub max_images_per_window: u64,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_image_bytes: 8_000_000,
            rate_limit_window_seconds: 3600,
            max_images_per_window: 30,
        }
    }
}

/// S3-compatible object storage configuration. MinIO is the local integration-test implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

impl Default for ObjectStorageConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:19000".to_owned(),
            region: "us-east-1".to_owned(),
            bucket: "stellartrail-uploads".to_owned(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            force_path_style: true,
        }
    }
}

/// Runtime API configuration containing environment, bind address, database, auth providers, cache, upload, and content directory settings.
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
    pub content_assets_dir: PathBuf,
    pub media_base_url: String,
    pub redis_cache: RedisCacheConfig,
    pub upload: UploadConfig,
    pub object_storage: ObjectStorageConfig,
    pub public_api: PublicApiConfig,
}

impl ApiConfig {
    /// Builds runtime configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_owned());
        let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()?;
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://stellartrail.db".to_owned());
        let wechat_mock_login =
            optional_bool_env("WECHAT_MOCK_LOGIN")?.unwrap_or(app_env == "local");
        let wechat_app_id = optional_env("WECHAT_APP_ID");
        let wechat_app_secret = optional_env("WECHAT_APP_SECRET");
        let content_dir = env::var("CONTENT_DIR").unwrap_or_else(|_| "content".to_owned());
        let content_assets_dir = env::var("CONTENT_ASSETS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&content_dir).join("assets"));
        let media_base_url = optional_env("MEDIA_BASE_URL").unwrap_or_else(|| "/assets".to_owned());
        let redis_cache = RedisCacheConfig {
            url: optional_env("REDIS_URL"),
            key_prefix: optional_env("REDIS_KEY_PREFIX")
                .unwrap_or_else(|| "stellartrail".to_owned()),
            gear_ttl_seconds: optional_u64_env("REDIS_GEAR_CACHE_TTL_SECONDS", 30)?,
        };
        let upload = UploadConfig {
            max_image_bytes: optional_u64_env("UPLOAD_MAX_IMAGE_BYTES", 8_000_000)?,
            rate_limit_window_seconds: optional_u64_env("UPLOAD_RATE_LIMIT_WINDOW_SECONDS", 3600)?,
            max_images_per_window: optional_u64_env("UPLOAD_MAX_IMAGES_PER_WINDOW", 30)?,
        };
        if upload.max_image_bytes == 0 {
            anyhow::bail!("UPLOAD_MAX_IMAGE_BYTES must be greater than 0");
        }
        if upload.rate_limit_window_seconds == 0 {
            anyhow::bail!("UPLOAD_RATE_LIMIT_WINDOW_SECONDS must be greater than 0");
        }
        if upload.max_images_per_window == 0 {
            anyhow::bail!("UPLOAD_MAX_IMAGES_PER_WINDOW must be greater than 0");
        }
        let default_storage = ObjectStorageConfig::default();
        let object_storage = ObjectStorageConfig {
            endpoint: optional_env("OBJECT_STORAGE_ENDPOINT").unwrap_or(default_storage.endpoint),
            region: optional_env("OBJECT_STORAGE_REGION").unwrap_or(default_storage.region),
            bucket: optional_env("OBJECT_STORAGE_BUCKET").unwrap_or(default_storage.bucket),
            access_key_id: optional_env("OBJECT_STORAGE_ACCESS_KEY_ID")
                .unwrap_or(default_storage.access_key_id),
            secret_access_key: optional_env("OBJECT_STORAGE_SECRET_ACCESS_KEY")
                .unwrap_or(default_storage.secret_access_key),
            force_path_style: optional_bool_env("OBJECT_STORAGE_FORCE_PATH_STYLE")?.unwrap_or(true),
        };

        let public_api = PublicApiConfig {
            rate_limit_enabled: optional_bool_env("PUBLIC_API_RATE_LIMIT_ENABLED")?.unwrap_or(true),
            rate_limit_window_seconds: optional_u64_env(
                "PUBLIC_API_RATE_LIMIT_WINDOW_SECONDS",
                60,
            )?,
            rate_limit_max_requests_per_ip: optional_u64_env(
                "PUBLIC_API_RATE_LIMIT_MAX_REQUESTS_PER_IP",
                120,
            )?,
            cache_ttl_seconds: optional_u64_env("PUBLIC_API_CACHE_TTL_SECONDS", 300)?,
            cache_stale_seconds: optional_u64_env("PUBLIC_API_CACHE_STALE_SECONDS", 600)?,
            max_list_limit: optional_u32_env("PUBLIC_API_MAX_LIST_LIMIT", 100)?,
            max_search_query_chars: optional_usize_env("PUBLIC_API_MAX_SEARCH_QUERY_CHARS", 64)?,
            max_offset: optional_u32_env("PUBLIC_API_MAX_OFFSET", 10_000)?,
            trust_proxy_headers: optional_bool_env("TRUST_PROXY_HEADERS")?.unwrap_or(false),
            trusted_proxy_cidrs: optional_env("TRUSTED_PROXY_CIDRS")
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        };
        validate_public_api_config(&public_api)?;

        Ok(Self {
            app_env,
            host,
            port,
            database: DatabaseConfig::new(database_url)?,
            wechat_mock_login,
            wechat_app_id,
            wechat_app_secret,
            content_dir: PathBuf::from(content_dir),
            content_assets_dir,
            media_base_url,
            redis_cache,
            upload,
            object_storage,
            public_api,
        })
    }

    /// Returns the API socket bind address.
    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("validated socket address")
    }

    /// Returns the configured database kind.
    pub fn database_kind(&self) -> DatabaseKind {
        self.database.kind
    }
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn optional_u64_env(name: &str, default: u64) -> anyhow::Result<u64> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u64>()?),
        None => Ok(default),
    }
}

fn optional_u32_env(name: &str, default: u32) -> anyhow::Result<u32> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u32>()?),
        None => Ok(default),
    }
}

fn optional_usize_env(name: &str, default: usize) -> anyhow::Result<usize> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<usize>()?),
        None => Ok(default),
    }
}

fn validate_public_api_config(config: &PublicApiConfig) -> anyhow::Result<()> {
    if config.rate_limit_window_seconds == 0 {
        anyhow::bail!("PUBLIC_API_RATE_LIMIT_WINDOW_SECONDS must be greater than 0");
    }
    if config.rate_limit_max_requests_per_ip == 0 {
        anyhow::bail!("PUBLIC_API_RATE_LIMIT_MAX_REQUESTS_PER_IP must be greater than 0");
    }
    if config.cache_ttl_seconds == 0 {
        anyhow::bail!("PUBLIC_API_CACHE_TTL_SECONDS must be greater than 0");
    }
    if config.max_list_limit == 0 {
        anyhow::bail!("PUBLIC_API_MAX_LIST_LIMIT must be greater than 0");
    }
    if config.max_search_query_chars == 0 || config.max_search_query_chars > 256 {
        anyhow::bail!("PUBLIC_API_MAX_SEARCH_QUERY_CHARS must be in 1..=256");
    }
    if config.trust_proxy_headers && config.trusted_proxy_cidrs.is_empty() {
        anyhow::bail!("TRUSTED_PROXY_CIDRS must be set when TRUST_PROXY_HEADERS=true");
    }
    Ok(())
}

fn optional_bool_env(name: &str) -> anyhow::Result<Option<bool>> {
    Ok(match optional_env(name) {
        Some(value) => Some(match value.as_str() {
            "1" | "true" | "TRUE" | "yes" | "YES" => true,
            "0" | "false" | "FALSE" | "no" | "NO" => false,
            other => anyhow::bail!("{name} must be a boolean value, got {other}"),
        }),
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn from_env_reads_wechat_credentials() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("APP_ENV", "production");
            env::set_var("APP_HOST", "127.0.0.1");
            env::set_var("APP_PORT", "8080");
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("WECHAT_MOCK_LOGIN", "false");
            env::set_var("WECHAT_APP_ID", " wx-app-id ");
            env::set_var("WECHAT_APP_SECRET", " wx-secret ");
            env::set_var("CONTENT_DIR", "content");
            env::set_var("CONTENT_ASSETS_DIR", "content/assets");
            env::set_var("MEDIA_BASE_URL", "/assets");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert!(!config.wechat_mock_login);
        assert_eq!(config.wechat_app_id.as_deref(), Some("wx-app-id"));
        assert_eq!(config.wechat_app_secret.as_deref(), Some("wx-secret"));
        assert_eq!(config.content_assets_dir, PathBuf::from("content/assets"));
        assert_eq!(config.media_base_url, "/assets");
        assert_eq!(config.upload, UploadConfig::default());

        restore_env(saved);
    }

    #[test]
    fn from_env_reads_redis_cache_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("APP_ENV", "local");
            env::set_var("APP_HOST", "127.0.0.1");
            env::set_var("APP_PORT", "8080");
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("WECHAT_MOCK_LOGIN", "true");
            env::set_var("CONTENT_DIR", "content");
            env::remove_var("CONTENT_ASSETS_DIR");
            env::remove_var("MEDIA_BASE_URL");
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
        assert_eq!(config.content_assets_dir, PathBuf::from("content/assets"));
        assert_eq!(config.media_base_url, "/assets");

        restore_env(saved);
    }

    #[test]
    fn from_env_reads_upload_and_object_storage_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_IMAGE_BYTES", "123456");
            env::set_var("UPLOAD_RATE_LIMIT_WINDOW_SECONDS", "60");
            env::set_var("UPLOAD_MAX_IMAGES_PER_WINDOW", "7");
            env::set_var("OBJECT_STORAGE_ENDPOINT", " http://minio:9000 ");
            env::set_var("OBJECT_STORAGE_REGION", " us-east-1 ");
            env::set_var("OBJECT_STORAGE_BUCKET", " stellartrail-test ");
            env::set_var("OBJECT_STORAGE_ACCESS_KEY_ID", " local-key ");
            env::set_var("OBJECT_STORAGE_SECRET_ACCESS_KEY", " local-secret ");
            env::set_var("OBJECT_STORAGE_FORCE_PATH_STYLE", "true");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(
            config.upload,
            UploadConfig {
                max_image_bytes: 123456,
                rate_limit_window_seconds: 60,
                max_images_per_window: 7,
            },
        );
        assert_eq!(config.object_storage.endpoint, "http://minio:9000");
        assert_eq!(config.object_storage.bucket, "stellartrail-test");
        assert_eq!(config.object_storage.access_key_id, "local-key");
        assert_eq!(config.object_storage.secret_access_key, "local-secret");
        assert!(config.object_storage.force_path_style);

        restore_env(saved);
    }

    #[test]
    fn from_env_reads_public_api_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("PUBLIC_API_RATE_LIMIT_ENABLED", "true");
            env::set_var("PUBLIC_API_RATE_LIMIT_WINDOW_SECONDS", "30");
            env::set_var("PUBLIC_API_RATE_LIMIT_MAX_REQUESTS_PER_IP", "9");
            env::set_var("PUBLIC_API_CACHE_TTL_SECONDS", "120");
            env::set_var("PUBLIC_API_CACHE_STALE_SECONDS", "240");
            env::set_var("PUBLIC_API_MAX_LIST_LIMIT", "50");
            env::set_var("PUBLIC_API_MAX_SEARCH_QUERY_CHARS", "24");
            env::set_var("PUBLIC_API_MAX_OFFSET", "500");
            env::set_var("TRUST_PROXY_HEADERS", "true");
            env::set_var("TRUSTED_PROXY_CIDRS", "10.0.0.0/8,127.0.0.1/32");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.public_api.rate_limit_window_seconds, 30);
        assert_eq!(config.public_api.rate_limit_max_requests_per_ip, 9);
        assert_eq!(config.public_api.cache_ttl_seconds, 120);
        assert_eq!(config.public_api.cache_stale_seconds, 240);
        assert_eq!(config.public_api.max_list_limit, 50);
        assert_eq!(config.public_api.max_search_query_chars, 24);
        assert_eq!(config.public_api.max_offset, 500);
        assert!(config.public_api.trust_proxy_headers);
        assert_eq!(
            config.public_api.trusted_proxy_cidrs,
            vec!["10.0.0.0/8".to_owned(), "127.0.0.1/32".to_owned()],
        );

        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_trusting_proxy_headers_without_trusted_cidrs() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("TRUST_PROXY_HEADERS", "true");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("TRUSTED_PROXY_CIDRS"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_zero_upload_limits() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(&CONFIG_KEYS);
        unsafe {
            clear_env(&CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_IMAGE_BYTES", "0");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("UPLOAD_MAX_IMAGE_BYTES"), "{error}");
        restore_env(saved);
    }

    const CONFIG_KEYS: [&str; 33] = [
        "APP_ENV",
        "APP_HOST",
        "APP_PORT",
        "DATABASE_URL",
        "WECHAT_MOCK_LOGIN",
        "WECHAT_APP_ID",
        "WECHAT_APP_SECRET",
        "CONTENT_DIR",
        "CONTENT_ASSETS_DIR",
        "MEDIA_BASE_URL",
        "REDIS_URL",
        "REDIS_KEY_PREFIX",
        "REDIS_GEAR_CACHE_TTL_SECONDS",
        "UPLOAD_MAX_IMAGE_BYTES",
        "UPLOAD_RATE_LIMIT_WINDOW_SECONDS",
        "UPLOAD_MAX_IMAGES_PER_WINDOW",
        "OBJECT_STORAGE_ENDPOINT",
        "OBJECT_STORAGE_REGION",
        "OBJECT_STORAGE_BUCKET",
        "OBJECT_STORAGE_ACCESS_KEY_ID",
        "OBJECT_STORAGE_SECRET_ACCESS_KEY",
        "OBJECT_STORAGE_FORCE_PATH_STYLE",
        "PUBLIC_API_RATE_LIMIT_ENABLED",
        "PUBLIC_API_RATE_LIMIT_WINDOW_SECONDS",
        "PUBLIC_API_RATE_LIMIT_MAX_REQUESTS_PER_IP",
        "PUBLIC_API_CACHE_TTL_SECONDS",
        "PUBLIC_API_CACHE_STALE_SECONDS",
        "PUBLIC_API_MAX_LIST_LIMIT",
        "PUBLIC_API_MAX_SEARCH_QUERY_CHARS",
        "PUBLIC_API_MAX_OFFSET",
        "TRUST_PROXY_HEADERS",
        "TRUSTED_PROXY_CIDRS",
        "MINIO_ROOT_USER",
    ];

    fn snapshot_env(keys: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
        keys.iter().map(|key| (*key, env::var(key).ok())).collect()
    }

    unsafe fn clear_env(keys: &[&'static str]) {
        for key in keys {
            unsafe { env::remove_var(key) };
        }
    }

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
