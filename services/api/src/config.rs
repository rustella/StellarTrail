//! API service configuration module that merges optional YAML files with environment variables for database, WeChat login, Redis cache, uploads, object storage, and other runtime settings.

use std::{
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
    minio: FileMinioConfig,
    object_storage: FileObjectStorageConfig,
    avatar_storage: FileAvatarStorageConfig,
    knots_media_storage: FileKnotsMediaStorageConfig,
    rate_limit: FileRateLimitConfig,
    public_api: FilePublicApiConfig,
    cors: FileCorsConfig,
    mail: FileMailConfig,
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
    pub minio: MinioConfig,
    pub object_storage: ObjectStorageConfig,
    pub avatar_storage: AvatarStorageConfig,
    pub knots_media_storage: KnotsMediaStorageConfig,
    pub public_api: PublicApiConfig,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub mail: MailConfig,
}

impl ApiConfig {
    /// Builds runtime configuration from `config.yaml` (or `CONFIG_PATH`) plus environment overrides.
    pub fn from_env() -> anyhow::Result<Self> {
        let FileConfig {
            app,
            database,
            wechat,
            redis,
            upload: file_upload,
            minio: file_minio,
            object_storage: file_object_storage,
            avatar_storage: file_avatar_storage,
            knots_media_storage: file_knots_media_storage,
            rate_limit: file_rate_limit,
            public_api: file_public_api,
            cors: file_cors,
            mail: file_mail,
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
            minio,
            object_storage,
            avatar_storage,
            knots_media_storage,
            public_api,
            rate_limit,
            cors,
            mail,
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
