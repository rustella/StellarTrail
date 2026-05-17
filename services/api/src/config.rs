//! API service configuration module that merges optional YAML files with environment variables for database, WeChat login, Redis cache, uploads, object storage, and other runtime settings.

use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::Deserialize;
use stellartrail_db::{DatabaseConfig, DatabaseKind};

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    app: FileAppConfig,
    database: FileDatabaseConfig,
    wechat: FileWechatConfig,
    content: FileContentConfig,
    redis: FileRedisCacheConfig,
    upload: FileUploadConfig,
    object_storage: FileObjectStorageConfig,
    knots_media_storage: FileKnotsMediaStorageConfig,
    admin: FileAdminConfig,
    public_api: FilePublicApiConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileAppConfig {
    env: Option<String>,
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileDatabaseConfig {
    url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileWechatConfig {
    mock_login: Option<bool>,
    app_id: Option<String>,
    app_secret: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileContentConfig {
    dir: Option<String>,
    assets_dir: Option<String>,
    media_base_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileRedisCacheConfig {
    url: Option<String>,
    key_prefix: Option<String>,
    gear_cache_ttl_seconds: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileUploadConfig {
    max_image_bytes: Option<u64>,
    rate_limit_window_seconds: Option<u64>,
    max_images_per_window: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileObjectStorageConfig {
    endpoint: Option<String>,
    region: Option<String>,
    bucket: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    force_path_style: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileKnotsMediaStorageConfig {
    storage_profile: Option<String>,
    endpoint: Option<String>,
    region: Option<String>,
    bucket: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    force_path_style: Option<bool>,
    public_base_url: Option<String>,
    max_image_bytes: Option<u64>,
    max_video_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileAdminConfig {
    user_ids: Option<Vec<String>>,
    emails: Option<Vec<String>>,
    usernames: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FilePublicApiConfig {
    rate_limit_enabled: Option<bool>,
    rate_limit_window_seconds: Option<u64>,
    rate_limit_max_requests_per_ip: Option<u64>,
    cache_ttl_seconds: Option<u64>,
    cache_stale_seconds: Option<u64>,
    max_list_limit: Option<u32>,
    max_search_query_chars: Option<usize>,
    max_offset: Option<u32>,
    trust_proxy_headers: Option<bool>,
    trusted_proxy_cidrs: Option<Vec<String>>,
}

impl FileConfig {
    fn load_from_env() -> anyhow::Result<Self> {
        let Some(path) = config_path_from_env()? else {
            return Ok(Self::default());
        };
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_yaml::from_str(&raw)
            .with_context(|| format!("failed to parse config file {}", path.display()))
    }
}

fn config_path_from_env() -> anyhow::Result<Option<PathBuf>> {
    match env::var("CONFIG_PATH") {
        Ok(value) if value.trim().is_empty() => return Ok(None),
        Ok(value) => return Ok(Some(PathBuf::from(value.trim()))),
        Err(env::VarError::NotPresent) => {}
        Err(error) => return Err(error).context("CONFIG_PATH must be valid Unicode"),
    }

    let default_path = Path::new(DEFAULT_CONFIG_PATH);
    Ok(default_path.exists().then(|| default_path.to_path_buf()))
}

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

/// Public Knots3D media object storage configuration. Public URLs can point at a different MinIO/CDN domain than the API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnotsMediaStorageConfig {
    pub storage_profile: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
    pub public_base_url: String,
    pub max_image_bytes: u64,
    pub max_video_bytes: u64,
}

impl Default for KnotsMediaStorageConfig {
    fn default() -> Self {
        Self {
            storage_profile: "knots-public".to_owned(),
            endpoint: "http://127.0.0.1:19000".to_owned(),
            region: "us-east-1".to_owned(),
            bucket: "stellartrail-knots-media".to_owned(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            force_path_style: true,
            public_base_url: "http://127.0.0.1:19000/stellartrail-knots-media".to_owned(),
            max_image_bytes: 32_000_000,
            max_video_bytes: 80_000_000,
        }
    }
}

/// Allowlist used by administrator-only routes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdminConfig {
    pub user_ids: Vec<String>,
    pub emails: Vec<String>,
    pub usernames: Vec<String>,
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
    pub knots_media_storage: KnotsMediaStorageConfig,
    pub admin: AdminConfig,
    pub public_api: PublicApiConfig,
}

impl ApiConfig {
    /// Builds runtime configuration from `config.yaml` (or `CONFIG_PATH`) plus environment overrides.
    pub fn from_env() -> anyhow::Result<Self> {
        let FileConfig {
            app,
            database,
            wechat,
            content,
            redis,
            upload: file_upload,
            object_storage: file_object_storage,
            knots_media_storage: file_knots_media_storage,
            admin: file_admin,
            public_api: file_public_api,
        } = FileConfig::load_from_env()?;

        let app_env = config_string_env("APP_ENV", app.env, "local");
        let host = config_string_env("APP_HOST", app.host, "127.0.0.1");
        let port = config_u16_env("APP_PORT", app.port, 8080)?;
        let database_url =
            config_string_env("DATABASE_URL", database.url, "sqlite://stellartrail.db");
        let wechat_mock_login =
            config_bool_env("WECHAT_MOCK_LOGIN", wechat.mock_login, app_env == "local")?;
        let wechat_app_id = config_optional_string_env("WECHAT_APP_ID", wechat.app_id);
        let wechat_app_secret = config_optional_string_env("WECHAT_APP_SECRET", wechat.app_secret);
        let content_dir = config_string_env("CONTENT_DIR", content.dir, "content");
        let content_assets_dir =
            config_optional_string_env("CONTENT_ASSETS_DIR", content.assets_dir)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(&content_dir).join("assets"));
        let media_base_url = config_string_env("MEDIA_BASE_URL", content.media_base_url, "/assets");
        let redis_cache = RedisCacheConfig {
            url: config_optional_string_env("REDIS_URL", redis.url),
            key_prefix: config_string_env("REDIS_KEY_PREFIX", redis.key_prefix, "stellartrail"),
            gear_ttl_seconds: config_u64_env(
                "REDIS_GEAR_CACHE_TTL_SECONDS",
                redis.gear_cache_ttl_seconds,
                30,
            )?,
        };
        let upload = UploadConfig {
            max_image_bytes: config_u64_env(
                "UPLOAD_MAX_IMAGE_BYTES",
                file_upload.max_image_bytes,
                8_000_000,
            )?,
            rate_limit_window_seconds: config_u64_env(
                "UPLOAD_RATE_LIMIT_WINDOW_SECONDS",
                file_upload.rate_limit_window_seconds,
                3600,
            )?,
            max_images_per_window: config_u64_env(
                "UPLOAD_MAX_IMAGES_PER_WINDOW",
                file_upload.max_images_per_window,
                30,
            )?,
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
            endpoint: config_string_env(
                "OBJECT_STORAGE_ENDPOINT",
                file_object_storage.endpoint,
                &default_storage.endpoint,
            ),
            region: config_string_env(
                "OBJECT_STORAGE_REGION",
                file_object_storage.region,
                &default_storage.region,
            ),
            bucket: config_string_env(
                "OBJECT_STORAGE_BUCKET",
                file_object_storage.bucket,
                &default_storage.bucket,
            ),
            access_key_id: config_string_env(
                "OBJECT_STORAGE_ACCESS_KEY_ID",
                file_object_storage.access_key_id,
                &default_storage.access_key_id,
            ),
            secret_access_key: config_string_env(
                "OBJECT_STORAGE_SECRET_ACCESS_KEY",
                file_object_storage.secret_access_key,
                &default_storage.secret_access_key,
            ),
            force_path_style: config_bool_env(
                "OBJECT_STORAGE_FORCE_PATH_STYLE",
                file_object_storage.force_path_style,
                true,
            )?,
        };
        let default_knots_storage = KnotsMediaStorageConfig::default();
        let knots_media_bucket = config_string_env(
            "KNOTS_MEDIA_BUCKET",
            file_knots_media_storage.bucket,
            &default_knots_storage.bucket,
        );
        let knots_media_endpoint =
            config_optional_string_env("KNOTS_MEDIA_ENDPOINT", file_knots_media_storage.endpoint)
                .unwrap_or_else(|| object_storage.endpoint.clone());
        let knots_media_public_base_url = config_optional_string_env(
            "KNOTS_MEDIA_PUBLIC_BASE_URL",
            file_knots_media_storage.public_base_url,
        )
        .unwrap_or_else(|| {
            format!(
                "{}/{}",
                knots_media_endpoint.trim_end_matches('/'),
                knots_media_bucket
            )
        });
        let knots_media_storage = KnotsMediaStorageConfig {
            storage_profile: config_string_env(
                "KNOTS_MEDIA_STORAGE_PROFILE",
                file_knots_media_storage.storage_profile,
                &default_knots_storage.storage_profile,
            ),
            endpoint: knots_media_endpoint,
            region: config_optional_string_env(
                "KNOTS_MEDIA_REGION",
                file_knots_media_storage.region,
            )
            .unwrap_or_else(|| object_storage.region.clone()),
            bucket: knots_media_bucket,
            access_key_id: config_optional_string_env(
                "KNOTS_MEDIA_ACCESS_KEY_ID",
                file_knots_media_storage.access_key_id,
            )
            .unwrap_or_else(|| object_storage.access_key_id.clone()),
            secret_access_key: config_optional_string_env(
                "KNOTS_MEDIA_SECRET_ACCESS_KEY",
                file_knots_media_storage.secret_access_key,
            )
            .unwrap_or_else(|| object_storage.secret_access_key.clone()),
            force_path_style: config_bool_env(
                "KNOTS_MEDIA_FORCE_PATH_STYLE",
                file_knots_media_storage.force_path_style,
                object_storage.force_path_style,
            )?,
            public_base_url: knots_media_public_base_url.trim_end_matches('/').to_owned(),
            max_image_bytes: config_u64_env(
                "KNOTS_MEDIA_MAX_IMAGE_BYTES",
                file_knots_media_storage.max_image_bytes,
                default_knots_storage.max_image_bytes,
            )?,
            max_video_bytes: config_u64_env(
                "KNOTS_MEDIA_MAX_VIDEO_BYTES",
                file_knots_media_storage.max_video_bytes,
                default_knots_storage.max_video_bytes,
            )?,
        };
        validate_knots_media_storage_config(&knots_media_storage)?;
        let admin = AdminConfig {
            user_ids: config_list_env("ADMIN_USER_IDS", file_admin.user_ids),
            emails: config_list_env("ADMIN_EMAILS", file_admin.emails)
                .into_iter()
                .map(|value| value.to_ascii_lowercase())
                .collect(),
            usernames: config_list_env("ADMIN_USERNAMES", file_admin.usernames)
                .into_iter()
                .map(|value| value.to_ascii_lowercase())
                .collect(),
        };

        let public_api = PublicApiConfig {
            rate_limit_enabled: config_bool_env(
                "PUBLIC_API_RATE_LIMIT_ENABLED",
                file_public_api.rate_limit_enabled,
                true,
            )?,
            rate_limit_window_seconds: config_u64_env(
                "PUBLIC_API_RATE_LIMIT_WINDOW_SECONDS",
                file_public_api.rate_limit_window_seconds,
                60,
            )?,
            rate_limit_max_requests_per_ip: config_u64_env(
                "PUBLIC_API_RATE_LIMIT_MAX_REQUESTS_PER_IP",
                file_public_api.rate_limit_max_requests_per_ip,
                120,
            )?,
            cache_ttl_seconds: config_u64_env(
                "PUBLIC_API_CACHE_TTL_SECONDS",
                file_public_api.cache_ttl_seconds,
                300,
            )?,
            cache_stale_seconds: config_u64_env(
                "PUBLIC_API_CACHE_STALE_SECONDS",
                file_public_api.cache_stale_seconds,
                600,
            )?,
            max_list_limit: config_u32_env(
                "PUBLIC_API_MAX_LIST_LIMIT",
                file_public_api.max_list_limit,
                100,
            )?,
            max_search_query_chars: config_usize_env(
                "PUBLIC_API_MAX_SEARCH_QUERY_CHARS",
                file_public_api.max_search_query_chars,
                64,
            )?,
            max_offset: config_u32_env(
                "PUBLIC_API_MAX_OFFSET",
                file_public_api.max_offset,
                10_000,
            )?,
            trust_proxy_headers: config_bool_env(
                "TRUST_PROXY_HEADERS",
                file_public_api.trust_proxy_headers,
                false,
            )?,
            trusted_proxy_cidrs: config_list_env(
                "TRUSTED_PROXY_CIDRS",
                file_public_api.trusted_proxy_cidrs,
            ),
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
            knots_media_storage,
            admin,
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

fn normalize_file_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn config_string_env(name: &str, file_value: Option<String>, default: &str) -> String {
    optional_env(name)
        .or_else(|| normalize_file_string(file_value))
        .unwrap_or_else(|| default.to_owned())
}

fn config_optional_string_env(name: &str, file_value: Option<String>) -> Option<String> {
    optional_env(name).or_else(|| normalize_file_string(file_value))
}

fn config_u16_env(name: &str, file_value: Option<u16>, default: u16) -> anyhow::Result<u16> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u16>()?),
        None => Ok(file_value.unwrap_or(default)),
    }
}

fn config_u64_env(name: &str, file_value: Option<u64>, default: u64) -> anyhow::Result<u64> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u64>()?),
        None => Ok(file_value.unwrap_or(default)),
    }
}

fn config_u32_env(name: &str, file_value: Option<u32>, default: u32) -> anyhow::Result<u32> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<u32>()?),
        None => Ok(file_value.unwrap_or(default)),
    }
}

fn config_usize_env(
    name: &str,
    file_value: Option<usize>,
    default: usize,
) -> anyhow::Result<usize> {
    match optional_env(name) {
        Some(value) => Ok(value.parse::<usize>()?),
        None => Ok(file_value.unwrap_or(default)),
    }
}

fn config_bool_env(name: &str, file_value: Option<bool>, default: bool) -> anyhow::Result<bool> {
    Ok(optional_bool_env(name)?.or(file_value).unwrap_or(default))
}

fn config_list_env(name: &str, file_value: Option<Vec<String>>) -> Vec<String> {
    if env::var(name).is_ok() {
        return optional_csv_env(name);
    }
    file_value
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn optional_csv_env(name: &str) -> Vec<String> {
    optional_env(name)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn validate_knots_media_storage_config(config: &KnotsMediaStorageConfig) -> anyhow::Result<()> {
    if config.storage_profile.trim().is_empty() {
        anyhow::bail!("KNOTS_MEDIA_STORAGE_PROFILE must not be empty");
    }
    if config.bucket.trim().is_empty() {
        anyhow::bail!("KNOTS_MEDIA_BUCKET must not be empty");
    }
    if config.public_base_url.trim().is_empty() {
        anyhow::bail!("KNOTS_MEDIA_PUBLIC_BASE_URL must not be empty");
    }
    if config.max_image_bytes == 0 {
        anyhow::bail!("KNOTS_MEDIA_MAX_IMAGE_BYTES must be greater than 0");
    }
    if config.max_video_bytes == 0 {
        anyhow::bail!("KNOTS_MEDIA_MAX_VIDEO_BYTES must be greater than 0");
    }
    Ok(())
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
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
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
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
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
    fn committed_example_yaml_config_file_is_parseable() {
        let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("config.example.yaml");
        let raw = std::fs::read_to_string(&example_path).unwrap();

        let _: FileConfig = serde_yaml::from_str(&raw).unwrap();
    }

    #[test]
    fn from_env_reads_yaml_config_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
app:
  env: production
  host: 0.0.0.0
  port: 9090
database:
  url: sqlite://yaml-config.db
wechat:
  mock_login: false
  app_id: yaml-app
  app_secret: x
content:
  dir: content-prod
  assets_dir: content-prod/assets
  media_base_url: https://cdn.example.invalid/assets
redis:
  url: redis://127.0.0.1:6379/4
  key_prefix: yaml-prefix
  gear_cache_ttl_seconds: 90
upload:
  max_image_bytes: 111111
  rate_limit_window_seconds: 120
  max_images_per_window: 8
object_storage:
  endpoint: http://object-store.example.invalid
  region: us-east-1
  bucket: yaml-uploads
  access_key_id: x
  secret_access_key: x
  force_path_style: true
knots_media_storage:
  storage_profile: yaml-knots
  endpoint: http://knots-store.example.invalid
  region: us-east-1
  bucket: yaml-knots-media
  access_key_id: x
  secret_access_key: x
  force_path_style: true
  public_base_url: https://cdn.example.invalid/knots
  max_image_bytes: 222222
  max_video_bytes: 333333
admin:
  user_ids:
    - user-a
  emails:
    - Admin@Example.Invalid
  usernames:
    - TrailAdmin
public_api:
  rate_limit_enabled: true
  rate_limit_window_seconds: 15
  rate_limit_max_requests_per_ip: 5
  cache_ttl_seconds: 30
  cache_stale_seconds: 60
  max_list_limit: 25
  max_search_query_chars: 32
  max_offset: 250
  trust_proxy_headers: true
  trusted_proxy_cidrs:
    - 10.0.0.0/8
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.database.url, "sqlite://yaml-config.db");
        assert!(!config.wechat_mock_login);
        assert_eq!(config.wechat_app_id.as_deref(), Some("yaml-app"));
        assert_eq!(config.wechat_app_secret.as_deref(), Some("x"));
        assert_eq!(config.content_dir, PathBuf::from("content-prod"));
        assert_eq!(
            config.content_assets_dir,
            PathBuf::from("content-prod/assets")
        );
        assert_eq!(config.media_base_url, "https://cdn.example.invalid/assets");
        assert_eq!(config.redis_cache.key_prefix, "yaml-prefix");
        assert_eq!(config.upload.max_image_bytes, 111111);
        assert_eq!(config.object_storage.bucket, "yaml-uploads");
        assert_eq!(config.knots_media_storage.storage_profile, "yaml-knots");
        assert_eq!(config.knots_media_storage.max_video_bytes, 333333);
        assert_eq!(config.admin.user_ids, vec!["user-a".to_owned()]);
        assert_eq!(
            config.admin.emails,
            vec!["admin@example.invalid".to_owned()]
        );
        assert_eq!(config.admin.usernames, vec!["trailadmin".to_owned()]);
        assert_eq!(config.public_api.rate_limit_window_seconds, 15);
        assert_eq!(
            config.public_api.trusted_proxy_cidrs,
            vec!["10.0.0.0/8".to_owned()]
        );

        restore_env(saved);
    }

    #[test]
    fn from_env_lets_environment_override_yaml_config_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
app:
  env: local
  port: 9090
database:
  url: sqlite://yaml-config.db
wechat:
  mock_login: true
public_api:
  max_list_limit: 25
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
            env::set_var("APP_ENV", "production");
            env::set_var("APP_PORT", "7070");
            env::set_var("DATABASE_URL", "sqlite://env-config.db");
            env::set_var("WECHAT_MOCK_LOGIN", "false");
            env::set_var("PUBLIC_API_MAX_LIST_LIMIT", "40");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert_eq!(config.port, 7070);
        assert_eq!(config.database.url, "sqlite://env-config.db");
        assert!(!config.wechat_mock_login);
        assert_eq!(config.public_api.max_list_limit, 40);

        restore_env(saved);
    }
    #[test]
    fn from_env_reads_upload_and_object_storage_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
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
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
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
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
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
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_IMAGE_BYTES", "0");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("UPLOAD_MAX_IMAGE_BYTES"), "{error}");
        restore_env(saved);
    }

    const CONFIG_KEYS: &[&str] = &[
        "CONFIG_PATH",
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
        "KNOTS_MEDIA_STORAGE_PROFILE",
        "KNOTS_MEDIA_ENDPOINT",
        "KNOTS_MEDIA_REGION",
        "KNOTS_MEDIA_BUCKET",
        "KNOTS_MEDIA_ACCESS_KEY_ID",
        "KNOTS_MEDIA_SECRET_ACCESS_KEY",
        "KNOTS_MEDIA_FORCE_PATH_STYLE",
        "KNOTS_MEDIA_PUBLIC_BASE_URL",
        "KNOTS_MEDIA_MAX_IMAGE_BYTES",
        "KNOTS_MEDIA_MAX_VIDEO_BYTES",
        "ADMIN_USER_IDS",
        "ADMIN_EMAILS",
        "ADMIN_USERNAMES",
    ];
    fn snapshot_env(keys: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
        keys.iter().map(|key| (*key, env::var(key).ok())).collect()
    }

    unsafe fn clear_env(keys: &[&'static str]) {
        for key in keys {
            unsafe { env::remove_var(key) };
        }
        // Keep unit tests isolated from a real local config.yaml that may exist in a developer checkout.
        unsafe { env::set_var("CONFIG_PATH", "") };
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
