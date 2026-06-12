//! API service configuration module that merges optional YAML files with environment variables for database, WeChat login, Redis cache, uploads, object storage, and other runtime settings.
//!
//! SMS verification credentials are intentionally loaded only from the optional
//! YAML config file so deploys can mount one secret-bearing config document
//! instead of passing SMS secrets through the process environment.

use std::{
    collections::HashSet,
    env, fmt, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use axum::http::HeaderValue;
use serde::Deserialize;
use stellartrail_db::{DatabaseConfig, DatabaseKind};

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    app: FileAppConfig,
    database: FileDatabaseConfig,
    wechat: FileWechatConfig,
    redis: FileRedisCacheConfig,
    upload: FileUploadConfig,
    trail: FileTrailConfig,
    map: FileMapConfig,
    minio: FileMinioConfig,
    object_storage: FileObjectStorageConfig,
    avatar_storage: FileAvatarStorageConfig,
    knots_media_storage: FileKnotsMediaStorageConfig,
    rate_limit: FileRateLimitConfig,
    request_signature: FileRequestSignatureConfig,
    public_api: FilePublicApiConfig,
    cors: FileCorsConfig,
    mail: FileMailConfig,
    sms: FileSmsConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileAppConfig {
    env: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    commit_hash: Option<String>,
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
    max_total_images_per_user: Option<u64>,
    max_total_bytes_per_user: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileTrailConfig {
    upload_max_bytes: Option<u64>,
    upload_max_points: Option<u64>,
    max_simplified_points: Option<u64>,
    max_trails_per_trip: Option<u64>,
    max_annotations_per_context: Option<u64>,
    overview_max_trips: Option<u64>,
    overview_max_trails: Option<u64>,
    overview_max_points: Option<u64>,
    overview_max_points_per_trail: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileMapConfig {
    provider: Option<String>,
    style_url: Option<String>,
    public_key: Option<String>,
    styles: Option<Vec<FileMapStyleConfig>>,
    default_style_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileMapStyleConfig {
    id: Option<String>,
    label: Option<String>,
    style_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileMinioConfig {
    endpoint: Option<String>,
    region: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    force_path_style: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileObjectStorageConfig {
    bucket: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileAvatarStorageConfig {
    bucket: Option<String>,
    public_base_url: Option<String>,
    max_image_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileKnotsMediaStorageConfig {
    storage_profile: Option<String>,
    bucket: Option<String>,
    public_base_url: Option<String>,
    max_image_bytes: Option<u64>,
    max_video_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileRateLimitConfig {
    enabled: Option<bool>,
    window_seconds: Option<u64>,
    max_requests_per_ip: Option<u64>,
    max_requests_per_user: Option<u64>,
    trust_proxy_headers: Option<bool>,
    trusted_proxy_cidrs: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileRequestSignatureConfig {
    enabled: Option<bool>,
    nonce_ttl_seconds: Option<u64>,
    clients: Option<Vec<FileRequestSignatureClientConfig>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileRequestSignatureClientConfig {
    app_id: Option<String>,
    app_secret: Option<String>,
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileCorsConfig {
    allowed_origins: Option<Vec<String>>,
    allow_credentials: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileMailConfig {
    enabled: Option<bool>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_tls: Option<MailSmtpTls>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    from: Option<String>,
    verification_subject: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileSmsConfig {
    enabled: Option<bool>,
    endpoint: Option<String>,
    access_key_id: Option<String>,
    access_key_secret: Option<String>,
    sign_name: Option<String>,
    scheme_name: Option<String>,
    valid_time_seconds: Option<u64>,
    interval_seconds: Option<u64>,
    login_register_template_code: Option<String>,
    change_bound_phone_template_code: Option<String>,
    password_reset_template_code: Option<String>,
    bind_new_phone_template_code: Option<String>,
    verify_bound_phone_template_code: Option<String>,
    phone_rate_limit: FileSmsPhoneRateLimitConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileSmsPhoneRateLimitConfig {
    enabled: Option<bool>,
    cooldown_seconds: Option<u64>,
    window_seconds: Option<u64>,
    max_sends_per_window: Option<u64>,
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

/// Browser CORS allowlist for public web origins routed through Traefik.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
}

/// Global application route rate limit configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub window_seconds: u64,
    pub max_requests_per_ip: u64,
    pub max_requests_per_user: u64,
    pub trust_proxy_headers: bool,
    pub trusted_proxy_cidrs: Vec<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_seconds: 60,
            max_requests_per_ip: 120,
            max_requests_per_user: 240,
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

/// Request signature validation configuration loaded only from ignored YAML config files.
#[derive(Clone, Eq, PartialEq)]
pub struct RequestSignatureConfig {
    pub enabled: bool,
    pub nonce_ttl_seconds: u64,
    pub clients: Vec<RequestSignatureClientConfig>,
}

impl fmt::Debug for RequestSignatureConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestSignatureConfig")
            .field("enabled", &self.enabled)
            .field("nonce_ttl_seconds", &self.nonce_ttl_seconds)
            .field("clients", &self.clients)
            .finish()
    }
}

impl Default for RequestSignatureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            nonce_ttl_seconds: 300,
            clients: Vec::new(),
        }
    }
}

/// One request-signing client credential pair.
#[derive(Clone, Eq, PartialEq)]
pub struct RequestSignatureClientConfig {
    pub app_id: String,
    pub app_secret: String,
}

impl fmt::Debug for RequestSignatureClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestSignatureClientConfig")
            .field("app_id", &self.app_id)
            .field("app_secret", &"<redacted>")
            .finish()
    }
}

/// Feedback image upload limits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadConfig {
    pub max_image_bytes: u64,
    pub rate_limit_window_seconds: u64,
    pub max_images_per_window: u64,
    pub max_total_images_per_user: u64,
    pub max_total_bytes_per_user: u64,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_image_bytes: 8_000_000,
            rate_limit_window_seconds: 3600,
            max_images_per_window: 30,
            max_total_images_per_user: 100,
            max_total_bytes_per_user: 200_000_000,
        }
    }
}

/// Trail upload parsing and map-context limits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrailConfig {
    pub upload_max_bytes: u64,
    pub upload_max_points: u64,
    pub max_simplified_points: u64,
    pub max_trails_per_trip: u64,
    pub max_annotations_per_context: u64,
    pub overview_max_trips: u64,
    pub overview_max_trails: u64,
    pub overview_max_points: u64,
    pub overview_max_points_per_trail: u64,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            upload_max_bytes: 25_000_000,
            upload_max_points: 50_000,
            max_simplified_points: 2_000,
            max_trails_per_trip: 20,
            max_annotations_per_context: 500,
            overview_max_trips: 100,
            overview_max_trails: 200,
            overview_max_points: 5_000,
            overview_max_points_per_trail: 160,
        }
    }
}

/// Client-visible map provider configuration. Service tokens are intentionally excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MapStyleConfig {
    pub id: String,
    pub label: String,
    pub style_url: String,
}

/// Client-visible map provider configuration. Service tokens are intentionally excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MapConfig {
    pub provider: String,
    pub style_url: String,
    pub public_key: Option<String>,
    pub styles: Vec<MapStyleConfig>,
    pub default_style_id: String,
}

impl Default for MapConfig {
    fn default() -> Self {
        let styles = default_map_styles();
        Self {
            provider: "maptiler".to_owned(),
            style_url: "https://api.maptiler.com/maps/outdoor-v2/style.json".to_owned(),
            public_key: None,
            styles,
            default_style_id: "outdoor".to_owned(),
        }
    }
}

fn default_map_styles() -> Vec<MapStyleConfig> {
    vec![
        MapStyleConfig {
            id: "outdoor".to_owned(),
            label: "户外".to_owned(),
            style_url: "https://api.maptiler.com/maps/outdoor-v2/style.json".to_owned(),
        },
        MapStyleConfig {
            id: "streets".to_owned(),
            label: "街道".to_owned(),
            style_url: "https://api.maptiler.com/maps/streets-v2/style.json".to_owned(),
        },
        MapStyleConfig {
            id: "satellite".to_owned(),
            label: "卫星".to_owned(),
            style_url: "https://api.maptiler.com/maps/satellite/style.json".to_owned(),
        },
    ]
}

/// Shared S3-compatible MinIO connection configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinioConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

impl Default for MinioConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:19000".to_owned(),
            region: "us-east-1".to_owned(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            force_path_style: true,
        }
    }
}

/// Private feedback image object bucket configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectStorageConfig {
    pub bucket: String,
}

impl Default for ObjectStorageConfig {
    fn default() -> Self {
        Self {
            bucket: "stellartrail-uploads".to_owned(),
        }
    }
}

/// Public profile avatar bucket configuration. Public URLs can point at a different MinIO/CDN domain than the API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvatarStorageConfig {
    pub bucket: String,
    pub public_base_url: String,
    pub max_image_bytes: u64,
}

impl Default for AvatarStorageConfig {
    fn default() -> Self {
        Self {
            bucket: "stellartrail-avatars".to_owned(),
            public_base_url: "http://127.0.0.1:19000/stellartrail-avatars".to_owned(),
            max_image_bytes: 2_000_000,
        }
    }
}

/// Public Knots3D media bucket configuration. Public URLs can point at a different MinIO/CDN domain than the API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnotsMediaStorageConfig {
    pub storage_profile: String,
    pub bucket: String,
    pub public_base_url: String,
    pub max_image_bytes: u64,
    pub max_video_bytes: u64,
}

impl Default for KnotsMediaStorageConfig {
    fn default() -> Self {
        Self {
            storage_profile: "knots-public".to_owned(),
            bucket: "stellartrail-knots-media".to_owned(),
            public_base_url: "http://127.0.0.1:19000/stellartrail-knots-media".to_owned(),
            max_image_bytes: 32_000_000,
            max_video_bytes: 80_000_000,
        }
    }
}

/// SMTP transport security mode for transactional email delivery.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MailSmtpTls {
    /// Use TLS from the initial SMTP connection, commonly port 465.
    #[default]
    Implicit,
    /// Upgrade a plaintext connection with STARTTLS, commonly port 587.
    StartTls,
    /// Disable SMTP TLS. This is only intended for local development relays.
    None,
}

/// Transactional email configuration used for registration verification codes.
#[derive(Clone, Eq, PartialEq)]
pub struct MailConfig {
    pub enabled: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: MailSmtpTls,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from: String,
    pub verification_subject: String,
}

impl fmt::Debug for MailConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MailConfig")
            .field("enabled", &self.enabled)
            .field("smtp_host", &self.smtp_host)
            .field("smtp_port", &self.smtp_port)
            .field("smtp_tls", &self.smtp_tls)
            .field("smtp_username", &self.smtp_username)
            .field("smtp_password", &"<redacted>")
            .field("from", &self.from)
            .field("verification_subject", &self.verification_subject)
            .finish()
    }
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            smtp_host: "smtp.example.invalid".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: String::new(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        }
    }
}

/// Aliyun SMS verification configuration used for phone login, registration, password reset, and phone binding.
#[derive(Clone, Eq, PartialEq)]
pub struct SmsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sign_name: String,
    pub scheme_name: String,
    pub valid_time_seconds: u64,
    pub interval_seconds: u64,
    pub login_register_template_code: String,
    pub change_bound_phone_template_code: String,
    pub password_reset_template_code: String,
    pub bind_new_phone_template_code: String,
    pub verify_bound_phone_template_code: String,
    pub phone_rate_limit: SmsPhoneRateLimitConfig,
}

/// Per-phone SMS send quota used before calling the SMS provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmsPhoneRateLimitConfig {
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub window_seconds: u64,
    pub max_sends_per_window: u64,
}

impl Default for SmsPhoneRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_seconds: 60,
            window_seconds: 86_400,
            max_sends_per_window: 20,
        }
    }
}

impl fmt::Debug for SmsConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SmsConfig")
            .field("enabled", &self.enabled)
            .field("endpoint", &self.endpoint)
            .field("access_key_id", &redacted_presence(&self.access_key_id))
            .field("access_key_secret", &"<redacted>")
            .field("sign_name", &self.sign_name)
            .field("scheme_name", &self.scheme_name)
            .field("valid_time_seconds", &self.valid_time_seconds)
            .field("interval_seconds", &self.interval_seconds)
            .field(
                "login_register_template_code",
                &self.login_register_template_code,
            )
            .field(
                "change_bound_phone_template_code",
                &self.change_bound_phone_template_code,
            )
            .field(
                "password_reset_template_code",
                &self.password_reset_template_code,
            )
            .field(
                "bind_new_phone_template_code",
                &self.bind_new_phone_template_code,
            )
            .field(
                "verify_bound_phone_template_code",
                &self.verify_bound_phone_template_code,
            )
            .field("phone_rate_limit", &self.phone_rate_limit)
            .finish()
    }
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "dypnsapi.aliyuncs.com".to_owned(),
            access_key_id: String::new(),
            access_key_secret: String::new(),
            sign_name: String::new(),
            scheme_name: String::new(),
            valid_time_seconds: 300,
            interval_seconds: 60,
            login_register_template_code: "100001".to_owned(),
            change_bound_phone_template_code: "100002".to_owned(),
            password_reset_template_code: "100003".to_owned(),
            bind_new_phone_template_code: "100004".to_owned(),
            verify_bound_phone_template_code: "100005".to_owned(),
            phone_rate_limit: SmsPhoneRateLimitConfig::default(),
        }
    }
}

/// Runtime API configuration containing environment, bind address, database, auth providers, cache, upload, content directory, and email settings.
#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub commit_hash: Option<String>,
    pub database: DatabaseConfig,
    pub wechat_mock_login: bool,
    pub wechat_app_id: Option<String>,
    pub wechat_app_secret: Option<String>,
    pub redis_cache: RedisCacheConfig,
    pub upload: UploadConfig,
    pub trail: TrailConfig,
    pub map: MapConfig,
    pub minio: MinioConfig,
    pub object_storage: ObjectStorageConfig,
    pub avatar_storage: AvatarStorageConfig,
    pub knots_media_storage: KnotsMediaStorageConfig,
    pub public_api: PublicApiConfig,
    pub rate_limit: RateLimitConfig,
    pub request_signature: RequestSignatureConfig,
    pub cors: CorsConfig,
    pub mail: MailConfig,
    pub sms: SmsConfig,
}

impl ApiConfig {
    /// Builds runtime configuration from `config.yaml` (or `CONFIG_PATH`) plus environment overrides.
    ///
    /// SMS verification config is file-only and is not overridden by `SMS_*`
    /// environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let FileConfig {
            app,
            database,
            wechat,
            redis,
            upload: file_upload,
            trail: file_trail,
            map: file_map,
            minio: file_minio,
            object_storage: file_object_storage,
            avatar_storage: file_avatar_storage,
            knots_media_storage: file_knots_media_storage,
            rate_limit: file_rate_limit,
            request_signature: file_request_signature,
            public_api: file_public_api,
            cors: file_cors,
            mail: file_mail,
            sms: file_sms,
        } = FileConfig::load_from_env()?;

        let app_env = config_string_env("APP_ENV", app.env, "local");
        let host = config_string_env("APP_HOST", app.host, "127.0.0.1");
        let port = config_u16_env("APP_PORT", app.port, 8080)?;
        let commit_hash = normalize_config_commit_hash(config_optional_string_env(
            "APP_COMMIT_HASH",
            app.commit_hash,
        ))?;
        let database_url =
            config_string_env("DATABASE_URL", database.url, "sqlite://stellartrail.db");
        let wechat_mock_login =
            config_bool_env("WECHAT_MOCK_LOGIN", wechat.mock_login, app_env == "local")?;
        let wechat_app_id = config_optional_string_env("WECHAT_APP_ID", wechat.app_id);
        let wechat_app_secret = config_optional_string_env("WECHAT_APP_SECRET", wechat.app_secret);
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
            max_total_images_per_user: config_u64_env(
                "UPLOAD_MAX_TOTAL_IMAGES_PER_USER",
                file_upload.max_total_images_per_user,
                100,
            )?,
            max_total_bytes_per_user: config_u64_env(
                "UPLOAD_MAX_TOTAL_BYTES_PER_USER",
                file_upload.max_total_bytes_per_user,
                200_000_000,
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
        if upload.max_total_images_per_user == 0 {
            anyhow::bail!("UPLOAD_MAX_TOTAL_IMAGES_PER_USER must be greater than 0");
        }
        if upload.max_total_bytes_per_user == 0 {
            anyhow::bail!("UPLOAD_MAX_TOTAL_BYTES_PER_USER must be greater than 0");
        }
        let default_trail = TrailConfig::default();
        let trail = TrailConfig {
            upload_max_bytes: config_u64_env(
                "TRAIL_UPLOAD_MAX_BYTES",
                file_trail.upload_max_bytes,
                default_trail.upload_max_bytes,
            )?,
            upload_max_points: config_u64_env(
                "TRAIL_UPLOAD_MAX_POINTS",
                file_trail.upload_max_points,
                default_trail.upload_max_points,
            )?,
            max_simplified_points: config_u64_env(
                "TRAIL_MAX_SIMPLIFIED_POINTS",
                file_trail.max_simplified_points,
                default_trail.max_simplified_points,
            )?,
            max_trails_per_trip: config_u64_env(
                "TRAIL_MAX_TRAILS_PER_TRIP",
                file_trail.max_trails_per_trip,
                default_trail.max_trails_per_trip,
            )?,
            max_annotations_per_context: config_u64_env(
                "TRAIL_MAX_ANNOTATIONS_PER_CONTEXT",
                file_trail.max_annotations_per_context,
                default_trail.max_annotations_per_context,
            )?,
            overview_max_trips: config_u64_env(
                "TRAIL_OVERVIEW_MAX_TRIPS",
                file_trail.overview_max_trips,
                default_trail.overview_max_trips,
            )?,
            overview_max_trails: config_u64_env(
                "TRAIL_OVERVIEW_MAX_TRAILS",
                file_trail.overview_max_trails,
                default_trail.overview_max_trails,
            )?,
            overview_max_points: config_u64_env(
                "TRAIL_OVERVIEW_MAX_POINTS",
                file_trail.overview_max_points,
                default_trail.overview_max_points,
            )?,
            overview_max_points_per_trail: config_u64_env(
                "TRAIL_OVERVIEW_MAX_POINTS_PER_TRAIL",
                file_trail.overview_max_points_per_trail,
                default_trail.overview_max_points_per_trail,
            )?,
        };
        validate_trail_config(&trail)?;

        let default_map = MapConfig::default();
        let legacy_style_url = optional_env_alias("MAP_STYLE_URL", &["MAPTILER_STYLE_URL"])
            .or_else(|| normalize_file_string(file_map.style_url))
            .unwrap_or_else(|| default_map.style_url.clone());
        let styles = map_styles_from_file(file_map.styles, &default_map.styles, &legacy_style_url);
        let default_style_id = normalize_file_string(file_map.default_style_id)
            .unwrap_or_else(|| default_map.default_style_id.clone());
        let style_url = styles
            .iter()
            .find(|style| style.id == default_style_id)
            .map(|style| style.style_url.clone())
            .unwrap_or_else(|| legacy_style_url.clone());
        let map = MapConfig {
            provider: config_string_env("MAP_PROVIDER", file_map.provider, &default_map.provider),
            style_url,
            public_key: optional_env_alias("MAP_PUBLIC_KEY", &["MAPTILER_PUBLIC_KEY"])
                .or_else(|| normalize_file_string(file_map.public_key)),
            styles,
            default_style_id,
        };
        validate_map_config(&map)?;

        let default_minio = MinioConfig::default();
        let minio = MinioConfig {
            endpoint: config_string_env_alias(
                "MINIO_ENDPOINT",
                &["OBJECT_STORAGE_ENDPOINT"],
                file_minio.endpoint,
                &default_minio.endpoint,
            ),
            region: config_string_env_alias(
                "MINIO_REGION",
                &["OBJECT_STORAGE_REGION"],
                file_minio.region,
                &default_minio.region,
            ),
            access_key_id: config_string_env_alias(
                "MINIO_ACCESS_KEY_ID",
                &["OBJECT_STORAGE_ACCESS_KEY_ID"],
                file_minio.access_key_id,
                &default_minio.access_key_id,
            ),
            secret_access_key: config_string_env_alias(
                "MINIO_SECRET_ACCESS_KEY",
                &["OBJECT_STORAGE_SECRET_ACCESS_KEY"],
                file_minio.secret_access_key,
                &default_minio.secret_access_key,
            ),
            force_path_style: config_bool_env_alias(
                "MINIO_FORCE_PATH_STYLE",
                &["OBJECT_STORAGE_FORCE_PATH_STYLE"],
                file_minio.force_path_style,
                true,
            )?,
        };
        validate_minio_config(&minio)?;

        let default_storage = ObjectStorageConfig::default();
        let object_storage = ObjectStorageConfig {
            bucket: config_string_env(
                "OBJECT_STORAGE_BUCKET",
                file_object_storage.bucket,
                &default_storage.bucket,
            ),
        };
        validate_object_storage_config(&object_storage)?;

        let default_avatar_storage = AvatarStorageConfig::default();
        let avatar_bucket = config_string_env(
            "AVATAR_STORAGE_BUCKET",
            file_avatar_storage.bucket,
            &default_avatar_storage.bucket,
        );
        let avatar_public_base_url = config_optional_string_env(
            "AVATAR_STORAGE_PUBLIC_BASE_URL",
            file_avatar_storage.public_base_url,
        )
        .unwrap_or_else(|| format!("{}/{}", minio.endpoint.trim_end_matches('/'), avatar_bucket));
        let avatar_storage = AvatarStorageConfig {
            bucket: avatar_bucket,
            public_base_url: avatar_public_base_url.trim_end_matches('/').to_owned(),
            max_image_bytes: config_u64_env(
                "AVATAR_STORAGE_MAX_IMAGE_BYTES",
                file_avatar_storage.max_image_bytes,
                default_avatar_storage.max_image_bytes,
            )?,
        };
        validate_avatar_storage_config(&avatar_storage)?;

        let default_knots_storage = KnotsMediaStorageConfig::default();
        let knots_media_bucket = config_string_env(
            "KNOTS_MEDIA_BUCKET",
            file_knots_media_storage.bucket,
            &default_knots_storage.bucket,
        );
        let knots_media_public_base_url = config_optional_string_env(
            "KNOTS_MEDIA_PUBLIC_BASE_URL",
            file_knots_media_storage.public_base_url,
        )
        .unwrap_or_else(|| {
            format!(
                "{}/{}",
                minio.endpoint.trim_end_matches('/'),
                knots_media_bucket
            )
        });
        let knots_media_storage = KnotsMediaStorageConfig {
            storage_profile: config_string_env(
                "KNOTS_MEDIA_STORAGE_PROFILE",
                file_knots_media_storage.storage_profile,
                &default_knots_storage.storage_profile,
            ),
            bucket: knots_media_bucket,
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

        let rate_limit = RateLimitConfig {
            enabled: config_bool_env("RATE_LIMIT_ENABLED", file_rate_limit.enabled, true)?,
            window_seconds: config_u64_env(
                "RATE_LIMIT_WINDOW_SECONDS",
                file_rate_limit.window_seconds,
                60,
            )?,
            max_requests_per_ip: config_u64_env(
                "RATE_LIMIT_MAX_REQUESTS_PER_IP",
                file_rate_limit.max_requests_per_ip,
                120,
            )?,
            max_requests_per_user: config_u64_env(
                "RATE_LIMIT_MAX_REQUESTS_PER_USER",
                file_rate_limit.max_requests_per_user,
                240,
            )?,
            trust_proxy_headers: config_bool_env(
                "RATE_LIMIT_TRUST_PROXY_HEADERS",
                file_rate_limit.trust_proxy_headers,
                false,
            )?,
            trusted_proxy_cidrs: config_list_env(
                "RATE_LIMIT_TRUSTED_PROXY_CIDRS",
                file_rate_limit.trusted_proxy_cidrs,
            ),
        };
        validate_rate_limit_config(&rate_limit)?;

        let request_signature = request_signature_config_from_file(file_request_signature)?;
        validate_request_signature_config(&request_signature)?;

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
        let default_mail = MailConfig::default();
        let mail = MailConfig {
            enabled: config_bool_env("MAIL_ENABLED", file_mail.enabled, default_mail.enabled)?,
            smtp_host: config_string_env(
                "MAIL_SMTP_HOST",
                file_mail.smtp_host,
                &default_mail.smtp_host,
            ),
            smtp_port: config_u16_env(
                "MAIL_SMTP_PORT",
                file_mail.smtp_port,
                default_mail.smtp_port,
            )?,
            smtp_tls: config_mail_tls_env(
                "MAIL_SMTP_TLS",
                file_mail.smtp_tls,
                default_mail.smtp_tls,
            )?,
            smtp_username: config_string_env(
                "MAIL_SMTP_USERNAME",
                file_mail.smtp_username,
                &default_mail.smtp_username,
            ),
            smtp_password: config_string_env(
                "MAIL_SMTP_PASSWORD",
                file_mail.smtp_password,
                &default_mail.smtp_password,
            ),
            from: config_string_env("MAIL_FROM", file_mail.from, &default_mail.from),
            verification_subject: config_string_env(
                "MAIL_VERIFICATION_SUBJECT",
                file_mail.verification_subject,
                &default_mail.verification_subject,
            ),
        };
        validate_mail_config(&mail)?;

        let default_sms = SmsConfig::default();
        let sms = SmsConfig {
            enabled: file_sms.enabled.unwrap_or(default_sms.enabled),
            endpoint: normalize_sms_endpoint(&config_file_string(
                file_sms.endpoint,
                &default_sms.endpoint,
            )),
            access_key_id: config_file_string(file_sms.access_key_id, &default_sms.access_key_id),
            access_key_secret: config_file_string(
                file_sms.access_key_secret,
                &default_sms.access_key_secret,
            ),
            sign_name: config_file_string(file_sms.sign_name, &default_sms.sign_name),
            scheme_name: config_file_string(file_sms.scheme_name, &default_sms.scheme_name),
            valid_time_seconds: file_sms
                .valid_time_seconds
                .unwrap_or(default_sms.valid_time_seconds),
            interval_seconds: file_sms
                .interval_seconds
                .unwrap_or(default_sms.interval_seconds),
            login_register_template_code: config_file_string(
                file_sms.login_register_template_code,
                &default_sms.login_register_template_code,
            ),
            change_bound_phone_template_code: config_file_string(
                file_sms.change_bound_phone_template_code,
                &default_sms.change_bound_phone_template_code,
            ),
            password_reset_template_code: config_file_string(
                file_sms.password_reset_template_code,
                &default_sms.password_reset_template_code,
            ),
            bind_new_phone_template_code: config_file_string(
                file_sms.bind_new_phone_template_code,
                &default_sms.bind_new_phone_template_code,
            ),
            verify_bound_phone_template_code: config_file_string(
                file_sms.verify_bound_phone_template_code,
                &default_sms.verify_bound_phone_template_code,
            ),
            phone_rate_limit: SmsPhoneRateLimitConfig {
                enabled: file_sms
                    .phone_rate_limit
                    .enabled
                    .unwrap_or(default_sms.phone_rate_limit.enabled),
                cooldown_seconds: file_sms
                    .phone_rate_limit
                    .cooldown_seconds
                    .unwrap_or(default_sms.phone_rate_limit.cooldown_seconds),
                window_seconds: file_sms
                    .phone_rate_limit
                    .window_seconds
                    .unwrap_or(default_sms.phone_rate_limit.window_seconds),
                max_sends_per_window: file_sms
                    .phone_rate_limit
                    .max_sends_per_window
                    .unwrap_or(default_sms.phone_rate_limit.max_sends_per_window),
            },
        };
        validate_sms_config(&sms)?;

        let cors = CorsConfig {
            allowed_origins: config_list_env("CORS_ALLOWED_ORIGINS", file_cors.allowed_origins),
            allow_credentials: config_bool_env(
                "CORS_ALLOW_CREDENTIALS",
                file_cors.allow_credentials,
                false,
            )?,
        };
        validate_cors_config(&cors)?;

        Ok(Self {
            app_env,
            host,
            port,
            commit_hash,
            database: DatabaseConfig::new(database_url)?,
            wechat_mock_login,
            wechat_app_id,
            wechat_app_secret,
            redis_cache,
            upload,
            trail,
            map,
            minio,
            object_storage,
            avatar_storage,
            knots_media_storage,
            public_api,
            rate_limit,
            request_signature,
            cors,
            mail,
            sms,
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

fn map_styles_from_file(
    file_styles: Option<Vec<FileMapStyleConfig>>,
    default_styles: &[MapStyleConfig],
    legacy_style_url: &str,
) -> Vec<MapStyleConfig> {
    match file_styles {
        Some(styles) => styles
            .into_iter()
            .map(|style| MapStyleConfig {
                id: normalize_file_string(style.id).unwrap_or_default(),
                label: normalize_file_string(style.label).unwrap_or_default(),
                style_url: normalize_file_string(style.style_url).unwrap_or_default(),
            })
            .collect(),
        None => default_styles
            .iter()
            .cloned()
            .map(|mut style| {
                if style.id == "outdoor" {
                    style.style_url = legacy_style_url.to_owned();
                }
                style
            })
            .collect(),
    }
}

fn config_file_string(file_value: Option<String>, default: &str) -> String {
    normalize_file_string(file_value).unwrap_or_else(|| default.to_owned())
}

fn config_string_env(name: &str, file_value: Option<String>, default: &str) -> String {
    optional_env(name)
        .or_else(|| normalize_file_string(file_value))
        .unwrap_or_else(|| default.to_owned())
}

fn config_string_env_alias(
    name: &str,
    aliases: &[&str],
    file_value: Option<String>,
    default: &str,
) -> String {
    optional_env_alias(name, aliases)
        .or_else(|| normalize_file_string(file_value))
        .unwrap_or_else(|| default.to_owned())
}

fn config_optional_string_env(name: &str, file_value: Option<String>) -> Option<String> {
    optional_env(name).or_else(|| normalize_file_string(file_value))
}

fn normalize_config_commit_hash(value: Option<String>) -> anyhow::Result<Option<String>> {
    let Some(value) = value.map(|value| value.trim().to_ascii_lowercase()) else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    if !(7..=40).contains(&value.len()) || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        anyhow::bail!("APP_COMMIT_HASH must be a 7 to 40 character hexadecimal Git commit hash");
    }
    Ok(Some(value))
}

fn optional_env_alias(name: &str, aliases: &[&str]) -> Option<String> {
    optional_env(name).or_else(|| aliases.iter().find_map(|alias| optional_env(alias)))
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

fn config_bool_env_alias(
    name: &str,
    aliases: &[&str],
    file_value: Option<bool>,
    default: bool,
) -> anyhow::Result<bool> {
    if let Some(value) = optional_env_alias(name, aliases) {
        Ok(value.parse::<bool>()?)
    } else {
        Ok(file_value.unwrap_or(default))
    }
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

fn validate_minio_config(config: &MinioConfig) -> anyhow::Result<()> {
    if config.endpoint.trim().is_empty() {
        anyhow::bail!("MINIO_ENDPOINT must not be empty");
    }
    if config.region.trim().is_empty() {
        anyhow::bail!("MINIO_REGION must not be empty");
    }
    Ok(())
}

fn validate_object_storage_config(config: &ObjectStorageConfig) -> anyhow::Result<()> {
    if config.bucket.trim().is_empty() {
        anyhow::bail!("OBJECT_STORAGE_BUCKET must not be empty");
    }
    Ok(())
}

fn validate_trail_config(config: &TrailConfig) -> anyhow::Result<()> {
    if config.upload_max_bytes == 0 {
        anyhow::bail!("TRAIL_UPLOAD_MAX_BYTES must be greater than 0");
    }
    if config.upload_max_points == 0 {
        anyhow::bail!("TRAIL_UPLOAD_MAX_POINTS must be greater than 0");
    }
    if config.max_simplified_points == 0 {
        anyhow::bail!("TRAIL_MAX_SIMPLIFIED_POINTS must be greater than 0");
    }
    if config.max_simplified_points > config.upload_max_points {
        anyhow::bail!("TRAIL_MAX_SIMPLIFIED_POINTS must not exceed TRAIL_UPLOAD_MAX_POINTS");
    }
    if config.max_trails_per_trip == 0 {
        anyhow::bail!("TRAIL_MAX_TRAILS_PER_TRIP must be greater than 0");
    }
    if config.max_annotations_per_context == 0 {
        anyhow::bail!("TRAIL_MAX_ANNOTATIONS_PER_CONTEXT must be greater than 0");
    }
    if config.overview_max_trips == 0 {
        anyhow::bail!("TRAIL_OVERVIEW_MAX_TRIPS must be greater than 0");
    }
    if config.overview_max_trails == 0 {
        anyhow::bail!("TRAIL_OVERVIEW_MAX_TRAILS must be greater than 0");
    }
    if config.overview_max_points < 2 {
        anyhow::bail!("TRAIL_OVERVIEW_MAX_POINTS must be at least 2");
    }
    if config.overview_max_points_per_trail < 2 {
        anyhow::bail!("TRAIL_OVERVIEW_MAX_POINTS_PER_TRAIL must be at least 2");
    }
    if config.overview_max_points_per_trail > config.overview_max_points {
        anyhow::bail!(
            "TRAIL_OVERVIEW_MAX_POINTS_PER_TRAIL must not exceed TRAIL_OVERVIEW_MAX_POINTS"
        );
    }
    Ok(())
}

fn validate_map_config(config: &MapConfig) -> anyhow::Result<()> {
    if config.provider.trim().is_empty() {
        anyhow::bail!("MAP_PROVIDER must not be empty");
    }
    if config.provider.trim() != config.provider {
        anyhow::bail!("MAP_PROVIDER must not be padded");
    }
    if config.style_url.trim().is_empty() {
        anyhow::bail!("MAP_STYLE_URL must not be empty");
    }
    if config.style_url.trim() != config.style_url {
        anyhow::bail!("MAP_STYLE_URL must not be padded");
    }
    if config.styles.is_empty() {
        anyhow::bail!("MAP_STYLES must not be empty");
    }
    if config.default_style_id.trim().is_empty() {
        anyhow::bail!("MAP_DEFAULT_STYLE_ID must not be empty");
    }
    if config.default_style_id.trim() != config.default_style_id {
        anyhow::bail!("MAP_DEFAULT_STYLE_ID must not be padded");
    }
    let mut style_ids = HashSet::new();
    let mut has_default_style = false;
    for style in &config.styles {
        if style.id.trim().is_empty() {
            anyhow::bail!("MAP_STYLE_ID must not be empty");
        }
        if style.id.trim() != style.id {
            anyhow::bail!("MAP_STYLE_ID must not be padded");
        }
        if !style_ids.insert(style.id.as_str()) {
            anyhow::bail!("MAP_STYLE_ID must be unique");
        }
        if style.label.trim().is_empty() {
            anyhow::bail!("MAP_STYLE_LABEL must not be empty");
        }
        if style.label.trim() != style.label {
            anyhow::bail!("MAP_STYLE_LABEL must not be padded");
        }
        if style.style_url.trim().is_empty() {
            anyhow::bail!("MAP_STYLE_URL must not be empty");
        }
        if style.style_url.trim() != style.style_url {
            anyhow::bail!("MAP_STYLE_URL must not be padded");
        }
        has_default_style |= style.id == config.default_style_id;
    }
    if !has_default_style {
        anyhow::bail!("MAP_DEFAULT_STYLE_ID must match one configured style");
    }
    Ok(())
}

fn validate_avatar_storage_config(config: &AvatarStorageConfig) -> anyhow::Result<()> {
    if config.bucket.trim().is_empty() {
        anyhow::bail!("AVATAR_STORAGE_BUCKET must not be empty");
    }
    if config.public_base_url.trim().is_empty() {
        anyhow::bail!("AVATAR_STORAGE_PUBLIC_BASE_URL must not be empty");
    }
    if config.max_image_bytes == 0 {
        anyhow::bail!("AVATAR_STORAGE_MAX_IMAGE_BYTES must be greater than 0");
    }
    Ok(())
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

fn validate_cors_config(config: &CorsConfig) -> anyhow::Result<()> {
    for origin in &config.allowed_origins {
        if origin.trim() != origin || origin.is_empty() {
            anyhow::bail!("CORS_ALLOWED_ORIGINS entries must not be empty or padded");
        }
        let Some(origin_host) = origin
            .strip_prefix("https://")
            .or_else(|| origin.strip_prefix("http://"))
        else {
            anyhow::bail!("CORS_ALLOWED_ORIGINS entries must start with http:// or https://");
        };
        if origin_host.is_empty() || origin_host.contains('/') {
            anyhow::bail!(
                "CORS_ALLOWED_ORIGINS entries must be origins without paths, queries, or fragments"
            );
        }
        HeaderValue::from_str(origin)
            .with_context(|| format!("CORS origin {origin} must be a valid header value"))?;
    }
    Ok(())
}

fn config_mail_tls_env(
    name: &str,
    file_value: Option<MailSmtpTls>,
    default: MailSmtpTls,
) -> anyhow::Result<MailSmtpTls> {
    match optional_env(name) {
        Some(value) => parse_mail_tls(name, &value),
        None => Ok(file_value.unwrap_or(default)),
    }
}

fn parse_mail_tls(name: &str, value: &str) -> anyhow::Result<MailSmtpTls> {
    match value {
        "implicit" | "implicit_tls" | "tls" | "wrapper" => Ok(MailSmtpTls::Implicit),
        "starttls" | "start_tls" | "required" => Ok(MailSmtpTls::StartTls),
        "none" | "plain" | "disabled" => Ok(MailSmtpTls::None),
        other => anyhow::bail!("{name} must be one of implicit, starttls, or none; got {other}"),
    }
}

fn validate_rate_limit_config(config: &RateLimitConfig) -> anyhow::Result<()> {
    if config.window_seconds == 0 {
        anyhow::bail!("RATE_LIMIT_WINDOW_SECONDS must be greater than 0");
    }
    if config.max_requests_per_ip == 0 {
        anyhow::bail!("RATE_LIMIT_MAX_REQUESTS_PER_IP must be greater than 0");
    }
    if config.max_requests_per_user == 0 {
        anyhow::bail!("RATE_LIMIT_MAX_REQUESTS_PER_USER must be greater than 0");
    }
    if config.trust_proxy_headers && config.trusted_proxy_cidrs.is_empty() {
        anyhow::bail!(
            "RATE_LIMIT_TRUSTED_PROXY_CIDRS must be set when RATE_LIMIT_TRUST_PROXY_HEADERS=true"
        );
    }
    Ok(())
}

fn request_signature_config_from_file(
    file_config: FileRequestSignatureConfig,
) -> anyhow::Result<RequestSignatureConfig> {
    let default = RequestSignatureConfig::default();
    let clients = file_config
        .clients
        .unwrap_or_default()
        .into_iter()
        .map(|client| {
            let app_id = normalize_file_string(client.app_id).unwrap_or_default();
            let app_secret = normalize_file_string(client.app_secret).unwrap_or_default();
            Ok(RequestSignatureClientConfig { app_id, app_secret })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(RequestSignatureConfig {
        enabled: file_config.enabled.unwrap_or(default.enabled),
        nonce_ttl_seconds: file_config
            .nonce_ttl_seconds
            .unwrap_or(default.nonce_ttl_seconds),
        clients,
    })
}

fn validate_request_signature_config(config: &RequestSignatureConfig) -> anyhow::Result<()> {
    if config.nonce_ttl_seconds == 0 {
        anyhow::bail!("request_signature.nonce_ttl_seconds must be greater than 0");
    }
    if !config.enabled {
        return Ok(());
    }
    if config.clients.is_empty() {
        anyhow::bail!("request_signature.clients must not be empty when enabled=true");
    }
    let mut seen_app_ids = HashSet::new();
    for client in &config.clients {
        if client.app_id.trim().is_empty() {
            anyhow::bail!("request_signature.clients[].app_id must not be empty");
        }
        if client.app_secret.trim().is_empty() {
            anyhow::bail!("request_signature.clients[].app_secret must not be empty");
        }
        if !seen_app_ids.insert(client.app_id.as_str()) {
            anyhow::bail!("request_signature.clients[].app_id must be unique");
        }
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

fn validate_mail_config(config: &MailConfig) -> anyhow::Result<()> {
    if !config.enabled {
        return Ok(());
    }
    if config.smtp_host.trim().is_empty() {
        anyhow::bail!("MAIL_SMTP_HOST must not be empty when MAIL_ENABLED=true");
    }
    if config.smtp_port == 0 {
        anyhow::bail!("MAIL_SMTP_PORT must be greater than 0 when MAIL_ENABLED=true");
    }
    if config.smtp_username.trim().is_empty() {
        anyhow::bail!("MAIL_SMTP_USERNAME must not be empty when MAIL_ENABLED=true");
    }
    if config.smtp_password.trim().is_empty() {
        anyhow::bail!("MAIL_SMTP_PASSWORD must not be empty when MAIL_ENABLED=true");
    }
    if config.from.trim().is_empty() {
        anyhow::bail!("MAIL_FROM must not be empty when MAIL_ENABLED=true");
    }
    if config.verification_subject.trim().is_empty() {
        anyhow::bail!("MAIL_VERIFICATION_SUBJECT must not be empty when MAIL_ENABLED=true");
    }
    Ok(())
}

fn validate_sms_config(config: &SmsConfig) -> anyhow::Result<()> {
    if config.valid_time_seconds == 0 {
        anyhow::bail!("sms.valid_time_seconds must be greater than 0");
    }
    if config.interval_seconds == 0 {
        anyhow::bail!("sms.interval_seconds must be greater than 0");
    }
    if config.phone_rate_limit.cooldown_seconds == 0 {
        anyhow::bail!("sms.phone_rate_limit.cooldown_seconds must be greater than 0");
    }
    if config.phone_rate_limit.window_seconds == 0 {
        anyhow::bail!("sms.phone_rate_limit.window_seconds must be greater than 0");
    }
    if config.phone_rate_limit.max_sends_per_window == 0 {
        anyhow::bail!("sms.phone_rate_limit.max_sends_per_window must be greater than 0");
    }
    if config.phone_rate_limit.cooldown_seconds > config.phone_rate_limit.window_seconds {
        anyhow::bail!(
            "sms.phone_rate_limit.cooldown_seconds must not exceed sms.phone_rate_limit.window_seconds"
        );
    }
    for (name, value) in [
        ("sms.endpoint", &config.endpoint),
        (
            "sms.login_register_template_code",
            &config.login_register_template_code,
        ),
        (
            "sms.change_bound_phone_template_code",
            &config.change_bound_phone_template_code,
        ),
        (
            "sms.password_reset_template_code",
            &config.password_reset_template_code,
        ),
        (
            "sms.bind_new_phone_template_code",
            &config.bind_new_phone_template_code,
        ),
        (
            "sms.verify_bound_phone_template_code",
            &config.verify_bound_phone_template_code,
        ),
    ] {
        if value.trim().is_empty() {
            anyhow::bail!("{name} must not be empty");
        }
    }
    if config.enabled {
        if config.access_key_id.trim().is_empty() {
            anyhow::bail!("sms.access_key_id must not be empty when sms.enabled=true");
        }
        if config.access_key_secret.trim().is_empty() {
            anyhow::bail!("sms.access_key_secret must not be empty when sms.enabled=true");
        }
        if config.sign_name.trim().is_empty() {
            anyhow::bail!("sms.sign_name must not be empty when sms.enabled=true");
        }
    }
    Ok(())
}

fn normalize_sms_endpoint(endpoint: &str) -> String {
    endpoint
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_matches('/')
        .to_owned()
}

fn redacted_presence(value: &str) -> &'static str {
    if value.trim().is_empty() {
        "<empty>"
    } else {
        "<redacted>"
    }
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
            env::set_var(
                "APP_COMMIT_HASH",
                " 376FD6C1EF08636477D5257AB720BC783BEEB358 ",
            );
            env::set_var("WECHAT_APP_ID", " wx-app-id ");
            env::set_var("WECHAT_APP_SECRET", " wx-secret ");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert!(!config.wechat_mock_login);
        assert_eq!(
            config.commit_hash.as_deref(),
            Some("376fd6c1ef08636477d5257ab720bc783beeb358"),
        );
        assert_eq!(config.wechat_app_id.as_deref(), Some("wx-app-id"));
        assert_eq!(config.wechat_app_secret.as_deref(), Some("wx-secret"));
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
redis:
  url: redis://127.0.0.1:6379/4
  key_prefix: yaml-prefix
  gear_cache_ttl_seconds: 90
upload:
  max_image_bytes: 111111
  rate_limit_window_seconds: 120
  max_images_per_window: 8
  max_total_images_per_user: 88
  max_total_bytes_per_user: 999999
trail:
  upload_max_bytes: 2222222
  upload_max_points: 1234
  max_simplified_points: 123
  max_trails_per_trip: 9
  max_annotations_per_context: 321
  overview_max_trips: 7
  overview_max_trails: 8
  overview_max_points: 456
  overview_max_points_per_trail: 45
map:
  provider: maptiler
  style_url: https://maps.example.invalid/style.json
  public_key: yaml-public-key
  default_style_id: streets
  styles:
    - id: outdoor
      label: 户外
      style_url: https://maps.example.invalid/outdoor.json
    - id: streets
      label: 街道
      style_url: https://maps.example.invalid/streets.json
    - id: satellite
      label: 卫星
      style_url: https://maps.example.invalid/satellite.json
minio:
  endpoint: http://minio.example.invalid
  region: us-east-1
  access_key_id: x
  secret_access_key: x
  force_path_style: true
object_storage:
  bucket: yaml-uploads
avatar_storage:
  bucket: yaml-avatars
  public_base_url: https://cdn.example.invalid/avatars
  max_image_bytes: 444444
knots_media_storage:
  storage_profile: yaml-knots
  bucket: yaml-knots-media
  public_base_url: https://cdn.example.invalid/knots
  max_image_bytes: 222222
  max_video_bytes: 333333
rate_limit:
  enabled: true
  window_seconds: 20
  max_requests_per_ip: 6
  max_requests_per_user: 12
  trust_proxy_headers: true
  trusted_proxy_cidrs:
    - 172.16.0.0/12
request_signature:
  enabled: true
  nonce_ttl_seconds: 180
  clients:
    - app_id: yaml-client
      app_secret: yaml-signing-secret
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
cors:
  allowed_origins:
    - https://app.example.invalid
    - https://www.example.invalid
  allow_credentials: true
mail:
  enabled: true
  smtp_host: smtp.example.invalid
  smtp_port: 465
  smtp_tls: implicit
  smtp_username: sender@example.test
  smtp_password: example-mail-password
  from: "StellarTrail <sender@example.test>"
  verification_subject: 寻径星野邮箱验证码
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
        assert_eq!(config.redis_cache.key_prefix, "yaml-prefix");
        assert_eq!(config.upload.max_image_bytes, 111111);
        assert_eq!(config.upload.max_total_images_per_user, 88);
        assert_eq!(config.upload.max_total_bytes_per_user, 999999);
        assert_eq!(config.trail.upload_max_bytes, 2222222);
        assert_eq!(config.trail.upload_max_points, 1234);
        assert_eq!(config.trail.max_simplified_points, 123);
        assert_eq!(config.trail.max_trails_per_trip, 9);
        assert_eq!(config.trail.max_annotations_per_context, 321);
        assert_eq!(config.trail.overview_max_trips, 7);
        assert_eq!(config.trail.overview_max_trails, 8);
        assert_eq!(config.trail.overview_max_points, 456);
        assert_eq!(config.trail.overview_max_points_per_trail, 45);
        assert_eq!(config.map.provider, "maptiler");
        assert_eq!(
            config.map.style_url,
            "https://maps.example.invalid/streets.json"
        );
        assert_eq!(config.map.public_key.as_deref(), Some("yaml-public-key"));
        assert_eq!(config.map.default_style_id, "streets");
        assert_eq!(config.map.styles.len(), 3);
        assert_eq!(config.map.styles[0].id, "outdoor");
        assert_eq!(
            config.map.styles[0].style_url,
            "https://maps.example.invalid/outdoor.json"
        );
        assert_eq!(config.map.styles[1].label, "街道");
        assert_eq!(config.minio.endpoint, "http://minio.example.invalid");
        assert_eq!(config.object_storage.bucket, "yaml-uploads");
        assert_eq!(config.avatar_storage.bucket, "yaml-avatars");
        assert_eq!(
            config.avatar_storage.public_base_url,
            "https://cdn.example.invalid/avatars"
        );
        assert_eq!(config.avatar_storage.max_image_bytes, 444444);
        assert_eq!(config.knots_media_storage.storage_profile, "yaml-knots");
        assert_eq!(config.knots_media_storage.max_video_bytes, 333333);
        assert_eq!(config.rate_limit.window_seconds, 20);
        assert_eq!(config.rate_limit.max_requests_per_ip, 6);
        assert_eq!(config.rate_limit.max_requests_per_user, 12);
        assert!(config.rate_limit.trust_proxy_headers);
        assert_eq!(
            config.rate_limit.trusted_proxy_cidrs,
            vec!["172.16.0.0/12".to_owned()]
        );
        assert!(config.request_signature.enabled);
        assert_eq!(config.request_signature.nonce_ttl_seconds, 180);
        assert_eq!(config.request_signature.clients.len(), 1);
        assert_eq!(config.request_signature.clients[0].app_id, "yaml-client");
        assert_eq!(
            config.request_signature.clients[0].app_secret,
            "yaml-signing-secret"
        );
        assert_eq!(config.public_api.rate_limit_window_seconds, 15);
        assert_eq!(
            config.public_api.trusted_proxy_cidrs,
            vec!["10.0.0.0/8".to_owned()]
        );
        assert_eq!(
            config.cors.allowed_origins,
            vec![
                "https://app.example.invalid".to_owned(),
                "https://www.example.invalid".to_owned(),
            ]
        );
        assert!(config.cors.allow_credentials);
        assert!(config.mail.enabled);
        assert_eq!(config.mail.smtp_host, "smtp.example.invalid");
        assert_eq!(config.mail.smtp_port, 465);
        assert_eq!(config.mail.smtp_tls, MailSmtpTls::Implicit);
        assert_eq!(config.mail.smtp_username, "sender@example.test");
        assert_eq!(config.mail.smtp_password, "example-mail-password");
        assert_eq!(config.mail.from, "StellarTrail <sender@example.test>");
        assert_eq!(config.mail.verification_subject, "寻径星野邮箱验证码");

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
            env::set_var(
                "CORS_ALLOWED_ORIGINS",
                "https://app.example.invalid, https://site.example.invalid, https://www.example.invalid",
            );
            env::set_var("CORS_ALLOW_CREDENTIALS", "true");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(config.app_env, "production");
        assert_eq!(config.port, 7070);
        assert_eq!(config.database.url, "sqlite://env-config.db");
        assert!(!config.wechat_mock_login);
        assert_eq!(config.public_api.max_list_limit, 40);
        assert_eq!(
            config.cors.allowed_origins,
            vec![
                "https://app.example.invalid".to_owned(),
                "https://site.example.invalid".to_owned(),
                "https://www.example.invalid".to_owned(),
            ]
        );
        assert!(config.cors.allow_credentials);

        restore_env(saved);
    }
    #[test]
    fn from_env_reads_upload_minio_and_bucket_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_IMAGE_BYTES", "123456");
            env::set_var("UPLOAD_RATE_LIMIT_WINDOW_SECONDS", "60");
            env::set_var("UPLOAD_MAX_IMAGES_PER_WINDOW", "7");
            env::set_var("UPLOAD_MAX_TOTAL_IMAGES_PER_USER", "11");
            env::set_var("UPLOAD_MAX_TOTAL_BYTES_PER_USER", "654321");
            env::set_var("TRAIL_UPLOAD_MAX_BYTES", "3333333");
            env::set_var("TRAIL_UPLOAD_MAX_POINTS", "3333");
            env::set_var("TRAIL_MAX_SIMPLIFIED_POINTS", "333");
            env::set_var("TRAIL_MAX_TRAILS_PER_TRIP", "13");
            env::set_var("TRAIL_MAX_ANNOTATIONS_PER_CONTEXT", "444");
            env::set_var("TRAIL_OVERVIEW_MAX_TRIPS", "77");
            env::set_var("TRAIL_OVERVIEW_MAX_TRAILS", "88");
            env::set_var("TRAIL_OVERVIEW_MAX_POINTS", "999");
            env::set_var("TRAIL_OVERVIEW_MAX_POINTS_PER_TRAIL", "99");
            env::set_var("MAP_PROVIDER", "maptiler");
            env::set_var("MAP_STYLE_URL", " https://maps.example.test/outdoor.json ");
            env::set_var("MAP_PUBLIC_KEY", " public-map-key ");
            env::set_var("MINIO_ENDPOINT", " http://minio:9000 ");
            env::set_var("MINIO_REGION", " us-east-1 ");
            env::set_var("MINIO_ACCESS_KEY_ID", " local-key ");
            env::set_var("MINIO_SECRET_ACCESS_KEY", " local-secret ");
            env::set_var("MINIO_FORCE_PATH_STYLE", "true");
            env::set_var("OBJECT_STORAGE_BUCKET", " stellartrail-test ");
            env::set_var("AVATAR_STORAGE_BUCKET", " stellartrail-avatar-test ");
            env::set_var(
                "AVATAR_STORAGE_PUBLIC_BASE_URL",
                " https://assets.example.test/avatars ",
            );
            env::set_var("AVATAR_STORAGE_MAX_IMAGE_BYTES", "234567");
        }

        let config = ApiConfig::from_env().unwrap();

        assert_eq!(
            config.upload,
            UploadConfig {
                max_image_bytes: 123456,
                rate_limit_window_seconds: 60,
                max_images_per_window: 7,
                max_total_images_per_user: 11,
                max_total_bytes_per_user: 654321,
            },
        );
        assert_eq!(config.minio.endpoint, "http://minio:9000");
        assert_eq!(config.minio.access_key_id, "local-key");
        assert_eq!(config.minio.secret_access_key, "local-secret");
        assert!(config.minio.force_path_style);
        assert_eq!(config.object_storage.bucket, "stellartrail-test");
        assert_eq!(config.trail.upload_max_bytes, 3333333);
        assert_eq!(config.trail.upload_max_points, 3333);
        assert_eq!(config.trail.max_simplified_points, 333);
        assert_eq!(config.trail.max_trails_per_trip, 13);
        assert_eq!(config.trail.max_annotations_per_context, 444);
        assert_eq!(config.trail.overview_max_trips, 77);
        assert_eq!(config.trail.overview_max_trails, 88);
        assert_eq!(config.trail.overview_max_points, 999);
        assert_eq!(config.trail.overview_max_points_per_trail, 99);
        assert_eq!(config.map.provider, "maptiler");
        assert_eq!(
            config.map.style_url,
            "https://maps.example.test/outdoor.json"
        );
        assert_eq!(config.map.public_key.as_deref(), Some("public-map-key"));
        assert_eq!(config.map.default_style_id, "outdoor");
        assert_eq!(
            config.map.styles[0].style_url,
            "https://maps.example.test/outdoor.json"
        );
        assert_eq!(config.map.styles[1].id, "streets");
        assert_eq!(config.map.styles[2].id, "satellite");
        assert_eq!(config.avatar_storage.bucket, "stellartrail-avatar-test");
        assert_eq!(
            config.avatar_storage.public_base_url,
            "https://assets.example.test/avatars"
        );
        assert_eq!(config.avatar_storage.max_image_bytes, 234567);

        restore_env(saved);
    }

    #[test]
    fn from_env_reads_global_rate_limit_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("RATE_LIMIT_ENABLED", "true");
            env::set_var("RATE_LIMIT_WINDOW_SECONDS", "45");
            env::set_var("RATE_LIMIT_MAX_REQUESTS_PER_IP", "11");
            env::set_var("RATE_LIMIT_MAX_REQUESTS_PER_USER", "22");
            env::set_var("RATE_LIMIT_TRUST_PROXY_HEADERS", "true");
            env::set_var(
                "RATE_LIMIT_TRUSTED_PROXY_CIDRS",
                "172.16.0.0/12,127.0.0.1/32",
            );
        }

        let config = ApiConfig::from_env().unwrap();

        assert!(config.rate_limit.enabled);
        assert_eq!(config.rate_limit.window_seconds, 45);
        assert_eq!(config.rate_limit.max_requests_per_ip, 11);
        assert_eq!(config.rate_limit.max_requests_per_user, 22);
        assert!(config.rate_limit.trust_proxy_headers);
        assert_eq!(
            config.rate_limit.trusted_proxy_cidrs,
            vec!["172.16.0.0/12".to_owned(), "127.0.0.1/32".to_owned()],
        );

        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_invalid_global_rate_limit_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("RATE_LIMIT_WINDOW_SECONDS", "0");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("RATE_LIMIT_WINDOW_SECONDS"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_enabled_request_signature_without_clients() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
database:
  url: sqlite://stellartrail.db
request_signature:
  enabled: true
  clients: []
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("request_signature.clients"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_duplicate_request_signature_app_id() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
database:
  url: sqlite://stellartrail.db
request_signature:
  enabled: true
  clients:
    - app_id: duplicate-client
      app_secret: first-secret
    - app_id: duplicate-client
      app_secret: second-secret
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(
            error.contains("request_signature.clients[].app_id"),
            "{error}"
        );
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
    fn from_env_rejects_cors_origins_with_paths() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("CORS_ALLOWED_ORIGINS", "https://app.example.invalid/path");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("CORS_ALLOWED_ORIGINS"), "{error}");
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

    #[test]
    fn from_env_rejects_zero_upload_total_quota() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_TOTAL_IMAGES_PER_USER", "0");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(
            error.contains("UPLOAD_MAX_TOTAL_IMAGES_PER_USER"),
            "{error}"
        );
        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_zero_upload_total_byte_quota() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("UPLOAD_MAX_TOTAL_BYTES_PER_USER", "0");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("UPLOAD_MAX_TOTAL_BYTES_PER_USER"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_reads_mail_config_from_environment() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("MAIL_ENABLED", "true");
            env::set_var("MAIL_SMTP_HOST", " smtp.example.invalid ");
            env::set_var("MAIL_SMTP_PORT", "465");
            env::set_var("MAIL_SMTP_TLS", "implicit");
            env::set_var("MAIL_SMTP_USERNAME", " sender@example.test ");
            env::set_var("MAIL_SMTP_PASSWORD", " example-mail-password ");
            env::set_var("MAIL_FROM", " StellarTrail <sender@example.test> ");
            env::set_var("MAIL_VERIFICATION_SUBJECT", " 寻径星野邮箱验证码 ");
        }

        let config = ApiConfig::from_env().unwrap();

        assert!(config.mail.enabled);
        assert_eq!(config.mail.smtp_host, "smtp.example.invalid");
        assert_eq!(config.mail.smtp_port, 465);
        assert_eq!(config.mail.smtp_tls, MailSmtpTls::Implicit);
        assert_eq!(config.mail.smtp_username, "sender@example.test");
        assert_eq!(config.mail.smtp_password, "example-mail-password");
        assert_eq!(config.mail.from, "StellarTrail <sender@example.test>");
        assert_eq!(config.mail.verification_subject, "寻径星野邮箱验证码");

        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_enabled_mail_without_password() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("MAIL_ENABLED", "true");
            env::set_var("MAIL_SMTP_HOST", "smtp.example.invalid");
            env::set_var("MAIL_SMTP_PORT", "465");
            env::set_var("MAIL_SMTP_USERNAME", "sender@example.test");
            env::set_var("MAIL_SMTP_PASSWORD", "");
            env::set_var("MAIL_FROM", "StellarTrail <sender@example.test>");
            env::set_var("MAIL_VERIFICATION_SUBJECT", "寻径星野邮箱验证码");
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("MAIL_SMTP_PASSWORD"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_reads_sms_config_from_yaml_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
database:
  url: sqlite://stellartrail.db
sms:
  enabled: true
  endpoint: " https://dypnsapi.aliyuncs.com/ "
  access_key_id: " sms-access-key-id "
  access_key_secret: " sms-access-key-secret "
  sign_name: " example-sms-sign-name "
  scheme_name: " stellartrail "
  valid_time_seconds: 600
  interval_seconds: 90
  login_register_template_code: "100001"
  change_bound_phone_template_code: "100002"
  password_reset_template_code: "100003"
  bind_new_phone_template_code: "100004"
  verify_bound_phone_template_code: "100005"
  phone_rate_limit:
    enabled: true
    cooldown_seconds: 75
    window_seconds: 3600
    max_sends_per_window: 8
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let config = ApiConfig::from_env().unwrap();

        assert!(config.sms.enabled);
        assert_eq!(config.sms.endpoint, "dypnsapi.aliyuncs.com");
        assert_eq!(config.sms.access_key_id, "sms-access-key-id");
        assert_eq!(config.sms.access_key_secret, "sms-access-key-secret");
        assert_eq!(config.sms.sign_name, "example-sms-sign-name");
        assert_eq!(config.sms.scheme_name, "stellartrail");
        assert_eq!(config.sms.valid_time_seconds, 600);
        assert_eq!(config.sms.interval_seconds, 90);
        assert_eq!(config.sms.login_register_template_code, "100001");
        assert_eq!(config.sms.change_bound_phone_template_code, "100002");
        assert_eq!(config.sms.password_reset_template_code, "100003");
        assert_eq!(config.sms.bind_new_phone_template_code, "100004");
        assert_eq!(config.sms.verify_bound_phone_template_code, "100005");
        assert!(config.sms.phone_rate_limit.enabled);
        assert_eq!(config.sms.phone_rate_limit.cooldown_seconds, 75);
        assert_eq!(config.sms.phone_rate_limit.window_seconds, 3600);
        assert_eq!(config.sms.phone_rate_limit.max_sends_per_window, 8);

        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_invalid_sms_phone_rate_limit_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
database:
  url: sqlite://stellartrail.db
sms:
  phone_rate_limit:
    enabled: true
    cooldown_seconds: 120
    window_seconds: 60
    max_sends_per_window: 20
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("cooldown_seconds"), "{error}");
        restore_env(saved);
    }

    #[test]
    fn from_env_ignores_sms_environment_values() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("DATABASE_URL", "sqlite://stellartrail.db");
            env::set_var("SMS_ENABLED", "true");
            env::set_var("SMS_ACCESS_KEY_ID", "sms-access-key-id");
            env::set_var("SMS_ACCESS_KEY_SECRET", "sms-access-key-secret");
            env::set_var("SMS_SCHEME_NAME", "stellartrail");
        }

        let config = ApiConfig::from_env().unwrap();

        assert!(!config.sms.enabled);
        assert!(config.sms.access_key_id.is_empty());
        assert!(config.sms.access_key_secret.is_empty());
        assert!(config.sms.scheme_name.is_empty());
        restore_env(saved);
    }

    #[test]
    fn from_env_rejects_enabled_sms_without_secret_in_yaml_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = snapshot_env(CONFIG_KEYS);
        let config_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            config_file.path(),
            r#"
database:
  url: sqlite://stellartrail.db
sms:
  enabled: true
  access_key_id: sms-access-key-id
  access_key_secret: ""
  scheme_name: stellartrail
"#,
        )
        .unwrap();
        unsafe {
            clear_env(CONFIG_KEYS);
            env::set_var("CONFIG_PATH", config_file.path());
        }

        let error = ApiConfig::from_env().unwrap_err().to_string();

        assert!(error.contains("sms.access_key_secret"), "{error}");
        restore_env(saved);
    }

    const CONFIG_KEYS: &[&str] = &[
        "CONFIG_PATH",
        "APP_ENV",
        "APP_HOST",
        "APP_PORT",
        "APP_COMMIT_HASH",
        "DATABASE_URL",
        "WECHAT_MOCK_LOGIN",
        "WECHAT_APP_ID",
        "WECHAT_APP_SECRET",
        "REDIS_URL",
        "REDIS_KEY_PREFIX",
        "REDIS_GEAR_CACHE_TTL_SECONDS",
        "UPLOAD_MAX_IMAGE_BYTES",
        "UPLOAD_RATE_LIMIT_WINDOW_SECONDS",
        "UPLOAD_MAX_IMAGES_PER_WINDOW",
        "UPLOAD_MAX_TOTAL_IMAGES_PER_USER",
        "UPLOAD_MAX_TOTAL_BYTES_PER_USER",
        "TRAIL_UPLOAD_MAX_BYTES",
        "TRAIL_UPLOAD_MAX_POINTS",
        "TRAIL_MAX_SIMPLIFIED_POINTS",
        "TRAIL_MAX_TRAILS_PER_TRIP",
        "TRAIL_MAX_ANNOTATIONS_PER_CONTEXT",
        "TRAIL_OVERVIEW_MAX_TRIPS",
        "TRAIL_OVERVIEW_MAX_TRAILS",
        "TRAIL_OVERVIEW_MAX_POINTS",
        "TRAIL_OVERVIEW_MAX_POINTS_PER_TRAIL",
        "MAP_PROVIDER",
        "MAP_STYLE_URL",
        "MAP_PUBLIC_KEY",
        "MAPTILER_STYLE_URL",
        "MAPTILER_PUBLIC_KEY",
        "MINIO_ENDPOINT",
        "MINIO_REGION",
        "MINIO_ACCESS_KEY_ID",
        "MINIO_SECRET_ACCESS_KEY",
        "MINIO_FORCE_PATH_STYLE",
        "OBJECT_STORAGE_BUCKET",
        "OBJECT_STORAGE_ENDPOINT",
        "OBJECT_STORAGE_REGION",
        "OBJECT_STORAGE_ACCESS_KEY_ID",
        "OBJECT_STORAGE_SECRET_ACCESS_KEY",
        "OBJECT_STORAGE_FORCE_PATH_STYLE",
        "AVATAR_STORAGE_BUCKET",
        "AVATAR_STORAGE_PUBLIC_BASE_URL",
        "AVATAR_STORAGE_MAX_IMAGE_BYTES",
        "RATE_LIMIT_ENABLED",
        "RATE_LIMIT_WINDOW_SECONDS",
        "RATE_LIMIT_MAX_REQUESTS_PER_IP",
        "RATE_LIMIT_MAX_REQUESTS_PER_USER",
        "RATE_LIMIT_TRUST_PROXY_HEADERS",
        "RATE_LIMIT_TRUSTED_PROXY_CIDRS",
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
        "CORS_ALLOWED_ORIGINS",
        "CORS_ALLOW_CREDENTIALS",
        "MINIO_ROOT_USER",
        "KNOTS_MEDIA_STORAGE_PROFILE",
        "KNOTS_MEDIA_BUCKET",
        "KNOTS_MEDIA_PUBLIC_BASE_URL",
        "KNOTS_MEDIA_MAX_IMAGE_BYTES",
        "KNOTS_MEDIA_MAX_VIDEO_BYTES",
        "MAIL_ENABLED",
        "MAIL_SMTP_HOST",
        "MAIL_SMTP_PORT",
        "MAIL_SMTP_TLS",
        "MAIL_SMTP_USERNAME",
        "MAIL_SMTP_PASSWORD",
        "MAIL_FROM",
        "MAIL_VERIFICATION_SUBJECT",
        "SMS_ENABLED",
        "SMS_ENDPOINT",
        "SMS_ACCESS_KEY_ID",
        "SMS_ACCESS_KEY_SECRET",
        "ALIBABA_CLOUD_ACCESS_KEY_ID",
        "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
        "SMS_SIGN_NAME",
        "SMS_SCHEME_NAME",
        "SMS_VALID_TIME_SECONDS",
        "SMS_INTERVAL_SECONDS",
        "SMS_TEMPLATE_LOGIN_REGISTER",
        "SMS_TEMPLATE_CHANGE_BOUND_PHONE",
        "SMS_TEMPLATE_PASSWORD_RESET",
        "SMS_TEMPLATE_BIND_NEW_PHONE",
        "SMS_TEMPLATE_VERIFY_BOUND_PHONE",
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
