//! Integration tests for authentication routes and session renewal behavior.
//!
//! These tests exercise the public HTTP boundary instead of calling service
//! functions directly. That keeps coverage close to real clients: JSON payloads
//! are serialized through Axum, access tokens are sent as bearer headers, and
//! refresh-token replay is verified through the same route that Web, WeChat, and
//! Android clients use.

use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode, header},
};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use stellartrail_api::{
    config::{
        ApiConfig, CorsConfig, MailConfig, MailSmtpTls, RedisCacheConfig, SmsConfig,
        SmsPhoneRateLimitConfig,
    },
    email::{EmailSender, VerificationEmail},
    migrate_database,
    routes::build_router,
    services::wechat::{WechatCodeSession, WechatCodeSessionClient},
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AuthRepository, hash_token},
};
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    db: sea_orm::DatabaseConnection,
    _temp_dir: TempDir,
}

async fn test_app() -> TestApp {
    test_app_with_sms_config(sms_config_with_phone_rate_limit(false, 60, 86_400, 20)).await
}

async fn test_app_with_sms_config(sms: SmsConfig) -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "local".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: true,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms,
    };
    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
        _temp_dir: temp_dir,
    }
}

fn sms_config_with_phone_rate_limit(
    enabled: bool,
    cooldown_seconds: u64,
    window_seconds: u64,
    max_sends_per_window: u64,
) -> SmsConfig {
    SmsConfig {
        phone_rate_limit: SmsPhoneRateLimitConfig {
            enabled,
            cooldown_seconds,
            window_seconds,
            max_sends_per_window,
        },
        ..Default::default()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WechatCode2SessionCall {
    app_id: String,
    app_secret: String,
    code: String,
}

#[derive(Clone, Default)]
struct RecordingWechatCodeSessionClient {
    calls: Arc<Mutex<Vec<WechatCode2SessionCall>>>,
}

#[derive(Clone, Default)]
struct RecordingEmailSender {
    sent: Arc<Mutex<Vec<VerificationEmail>>>,
    fail: bool,
}

#[async_trait::async_trait]
impl EmailSender for RecordingEmailSender {
    async fn send_verification_code(&self, email: VerificationEmail) -> anyhow::Result<()> {
        if self.fail {
            anyhow::bail!("smtp password rejected by upstream relay");
        }
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

impl WechatCodeSessionClient for RecordingWechatCodeSessionClient {
    fn code2session(
        &self,
        app_id: &str,
        app_secret: &str,
        code: &str,
    ) -> anyhow::Result<WechatCodeSession> {
        self.calls.lock().unwrap().push(WechatCode2SessionCall {
            app_id: app_id.to_owned(),
            app_secret: app_secret.to_owned(),
            code: code.to_owned(),
        });
        Ok(WechatCodeSession {
            openid: "wechat-openid-real-user".to_owned(),
            unionid: Some("wechat-unionid-real-user".to_owned()),
        })
    }
}

async fn send_json(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn send_json_with_headers(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value, HeaderMap) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value, headers)
}

async fn register_password_user(
    app: &Router,
    username: &str,
    email: &str,
    password: &str,
) -> Value {
    let (code_status, code_value) = send_json(
        app,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({"email": email}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let verification_code = code_value["debug_code"].as_str().unwrap();

    let (register_status, register_value) = send_json(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        json!({
            "username": username,
            "email": email,
            "password": password,
            "confirm_password": password,
            "email_verification_code": verification_code,
        }),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_value}");
    register_value
}

async fn register_sms_user(
    app: &Router,
    username: &str,
    nickname: &str,
    phone: &str,
    password: &str,
) -> Value {
    let (code_status, code_value) = send_json(
        app,
        "POST",
        "/api/v1/auth/sms-registration-code",
        None,
        json!({"phone": phone}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let verification_code = code_value["debug_code"].as_str().unwrap();
    let sms_ticket = code_value["sms_ticket"].as_str().unwrap();

    let (register_status, register_value) = send_json(
        app,
        "POST",
        "/api/v1/auth/sms-register",
        None,
        json!({
            "username": username,
            "nickname": nickname,
            "phone": phone,
            "password": password,
            "confirm_password": password,
            "sms_ticket": sms_ticket,
            "sms_verification_code": verification_code,
        }),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_value}");
    register_value
}

async fn create_captcha(app: &Router, account: &str) -> (String, String) {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/captcha",
        None,
        json!({"account": account}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["captcha_type"], "image");
    assert!(value["image_svg"].as_str().unwrap().contains("<svg"));
    (
        value["captcha_ticket"].as_str().unwrap().to_owned(),
        value["debug_answer"].as_str().unwrap().to_owned(),
    )
}

async fn send_empty(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn insert_sms_challenge(app: &TestApp, phone: &str, id_suffix: usize) {
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap();
    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            r#"INSERT INTO sms_verification_challenges (
                id, phone, purpose, out_id, expires_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)"#,
            vec![
                format!("sms-limit-{phone}-{id_suffix}").into(),
                phone.to_owned().into(),
                "fixture".into(),
                format!("out-{phone}-{id_suffix}").into(),
                "2099-01-01T00:00:00Z".into(),
                now.into(),
            ],
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn local_mock_login_returns_token_and_user() {
    let app = test_app().await;

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "local-dev-user", "profile": {"nickname": "测试用户"}}),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    assert!(value["access_token"].as_str().unwrap().len() > 20);
    assert_eq!(value["user"]["nickname"], "测试用户");
}

#[tokio::test]
async fn current_profile_returns_authenticated_user_snapshot() {
    let app = test_app().await;

    let (login_status, login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({
            "code": "profile-current-user",
            "profile": {
                "nickname": "微信昵称",
                "avatar_url": "https://assets.example.test/avatar.png"
            }
        }),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_value}");
    let access_token = login_value["access_token"].as_str().unwrap();

    let (unauthorized_status, unauthorized_value) =
        send_empty(&app.router, "GET", "/api/v1/me/profile", None).await;
    assert_eq!(
        unauthorized_status,
        StatusCode::UNAUTHORIZED,
        "{unauthorized_value}"
    );

    let (profile_status, profile_value) =
        send_empty(&app.router, "GET", "/api/v1/me/profile", Some(access_token)).await;

    assert_eq!(profile_status, StatusCode::OK, "{profile_value}");
    assert_eq!(profile_value["user"]["nickname"], "微信昵称");
    assert_eq!(
        profile_value["user"]["avatar_url"],
        "https://assets.example.test/avatar.png"
    );
}

#[tokio::test]
async fn wechat_login_without_profile_preserves_existing_nickname_and_avatar() {
    let app = test_app().await;

    let (first_status, first_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({
            "code": "profile-preserve-user",
            "profile": {
                "nickname": "微信昵称",
                "avatar_url": "https://assets.example.test/avatar.png"
            }
        }),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK, "{first_value}");
    assert_eq!(first_value["user"]["nickname"], "微信昵称");

    let (second_status, second_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "profile-preserve-user"}),
    )
    .await;

    assert_eq!(second_status, StatusCode::OK, "{second_value}");
    assert_eq!(second_value["user"]["id"], first_value["user"]["id"]);
    assert_eq!(second_value["user"]["nickname"], "微信昵称");
    assert_eq!(
        second_value["user"]["avatar_url"],
        "https://assets.example.test/avatar.png"
    );
}

/// Verifies every password-login response includes both access and refresh credentials.
#[tokio::test]
async fn login_response_includes_refresh_token() {
    let app = test_app().await;

    // Registration returns the same login response shape as password and WeChat
    // login, so it is the shortest path for asserting the token contract.
    let value = register_password_user(
        &app.router,
        "refresh_alice",
        "refresh-alice@example.com",
        "OutdoorPass123!",
    )
    .await;

    assert!(value["access_token"].as_str().unwrap().len() > 20);
    assert!(value["refresh_token"].as_str().unwrap().len() > 20);
    assert_ne!(
        value["access_token"].as_str().unwrap(),
        value["refresh_token"].as_str().unwrap(),
    );
    assert!(value["expires_at"].as_str().unwrap().contains('T'));
    assert!(value["refresh_expires_at"].as_str().unwrap().contains('T'));
}

/// Verifies refresh-token rotation returns new credentials and rejects old-token replay.
#[tokio::test]
async fn refresh_token_rotates_session_and_rejects_replay() {
    let app = test_app().await;
    let login_value = register_password_user(
        &app.router,
        "refresh_bob",
        "refresh-bob@example.com",
        "OutdoorPass123!",
    )
    .await;
    let old_access_token = login_value["access_token"].as_str().unwrap().to_owned();
    let old_refresh_token = login_value["refresh_token"].as_str().unwrap().to_owned();

    // The refresh endpoint does not require an Authorization header; the refresh
    // token itself is the credential and is rotated on success.
    let (refresh_status, refresh_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/refresh",
        None,
        json!({"refresh_token": old_refresh_token}),
    )
    .await;
    assert_eq!(refresh_status, StatusCode::OK, "{refresh_value}");
    let new_access_token = refresh_value["access_token"].as_str().unwrap();
    let new_refresh_token = refresh_value["refresh_token"].as_str().unwrap();
    assert_ne!(new_access_token, old_access_token);
    assert_ne!(new_refresh_token, old_refresh_token);
    assert_eq!(refresh_value["user"]["username"], "refresh_bob");

    // The new access token must be immediately usable for private APIs that
    // still require bearer authentication.
    let (gear_status, gear_stats) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/stats",
        Some(new_access_token),
    )
    .await;
    assert_eq!(gear_status, StatusCode::OK, "{gear_stats}");

    let (old_access_status, old_access_value) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/stats",
        Some(&old_access_token),
    )
    .await;
    assert_eq!(
        old_access_status,
        StatusCode::UNAUTHORIZED,
        "{old_access_value}",
    );

    // Reusing the old refresh token proves the conditional rotation guard: the
    // stored refresh hash no longer matches after the first successful refresh.
    let (replay_status, replay_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/refresh",
        None,
        json!({"refresh_token": old_refresh_token}),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED, "{replay_value}");
    assert_eq!(replay_value["code"], "unauthorized");
}

#[tokio::test]
async fn production_email_verification_sends_mail_and_hides_debug_code() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "production".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: false,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: MailConfig {
            enabled: true,
            smtp_host: "smtp.example.invalid".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: "x".to_owned(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        },
        sms: Default::default(),
    };
    let sender = RecordingEmailSender::default();
    let sent = Arc::clone(&sender.sent);
    let router = build_router(AppState::new_with_email_sender(
        config,
        db,
        Arc::new(sender),
    ));

    let (status, value) = send_json(
        &router,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({"email": "Trail.User@Example.COM"}),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["email"], "trail.user@example.com");
    assert!(value.get("debug_code").is_none(), "{value}");
    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "trail.user@example.com");
    assert_eq!(sent[0].from, "StellarTrail <sender@example.test>");
    assert_eq!(sent[0].subject, "寻径星野邮箱验证码");
    assert_eq!(sent[0].expires_minutes, 10);
    assert_eq!(sent[0].code.len(), 6);
    assert!(sent[0].code.chars().all(|ch| ch.is_ascii_digit()));
}

#[tokio::test]
async fn production_email_verification_delivery_failure_returns_safe_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "production".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: false,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: MailConfig {
            enabled: true,
            smtp_host: "smtp.example.invalid".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: "x".to_owned(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        },
        sms: Default::default(),
    };
    let router = build_router(AppState::new_with_email_sender(
        config,
        db,
        Arc::new(RecordingEmailSender {
            sent: Arc::new(Mutex::new(Vec::new())),
            fail: true,
        }),
    ));

    let (status, value) = send_json(
        &router,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({"email": "trail.user@example.com"}),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_GATEWAY, "{value}");
    assert_eq!(value["code"], "email_delivery_failed");
    assert!(!value.to_string().contains("smtp password"), "{value}");
}

#[tokio::test]
async fn production_wechat_login_uses_code2session_client() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "production".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: false,
        wechat_app_id: Some("wx-test-app-id".to_owned()),
        wechat_app_secret: Some("wx-test-secret".to_owned()),
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
    };
    let wechat_client = RecordingWechatCodeSessionClient::default();
    let calls = Arc::clone(&wechat_client.calls);
    let router = build_router(AppState::new_with_wechat_client(
        config,
        db,
        Arc::new(wechat_client),
    ));

    let (status, value) = send_json(
        &router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "wx-js-code", "profile": {"nickname": "微信用户", "avatar_url": null}}),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    let access_token = value["access_token"].as_str().unwrap();
    assert!(access_token.len() > 20);
    assert_eq!(value["user"]["nickname"], "微信用户");
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[WechatCode2SessionCall {
            app_id: "wx-test-app-id".to_owned(),
            app_secret: "wx-test-secret".to_owned(),
            code: "wx-js-code".to_owned(),
        }],
    );

    let (gear_status, gear_stats) =
        send_empty(&router, "GET", "/api/v1/me/gears/stats", Some(access_token)).await;
    assert_eq!(gear_status, StatusCode::OK, "{gear_stats}");
}

#[tokio::test]
async fn email_registration_and_password_login_flow_uses_sha256_password_hash() {
    let app = test_app().await;

    let register_value = register_password_user(
        &app.router,
        "trail_alice",
        "Alice@Example.COM",
        "OutdoorPass123!",
    )
    .await;
    let registered_user_id = register_value["user"]["id"].as_str().unwrap().to_owned();
    assert!(register_value["access_token"].as_str().unwrap().len() > 20);
    assert_eq!(register_value["user"]["username"], "trail_alice");
    assert_eq!(register_value["user"]["email"], "alice@example.com");

    let row = app
        .db
        .query_one(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "SELECT password_hash FROM users WHERE email = ?",
            vec!["alice@example.com".into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let password_hash: String = row.try_get("", "password_hash").unwrap();
    assert_eq!(
        password_hash,
        "153dcd2b66f0ccc59397d949893b9c20ac562ef7e6eda2bc9203f2b53dffbc9e",
    );
    assert_ne!(password_hash, "OutdoorPass123!");

    let (username_login_status, username_login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "trail_alice", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(
        username_login_status,
        StatusCode::OK,
        "{username_login_value}"
    );
    assert_eq!(
        username_login_value["user"]["id"].as_str().unwrap(),
        registered_user_id,
    );

    let (email_login_status, email_login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "alice@example.com", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(email_login_status, StatusCode::OK, "{email_login_value}");
    assert_eq!(
        email_login_value["user"]["id"].as_str().unwrap(),
        registered_user_id,
    );
}

#[tokio::test]
async fn sms_registration_sets_phone_and_password_login_accepts_phone() {
    let app = test_app().await;

    let (code_status, code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-registration-code",
        None,
        json!({"phone": "+86 138-0013-8000"}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    assert_eq!(code_value["phone"], "13800138000");
    let debug_code = code_value["debug_code"].as_str().unwrap();

    let (register_status, register_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-register",
        None,
        json!({
            "username": "sms_alice",
            "nickname": "短信 Alice",
            "phone": "13800138000",
            "password": "OutdoorPass123!",
            "confirm_password": "OutdoorPass123!",
            "sms_ticket": code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": debug_code,
        }),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_value}");
    assert_eq!(register_value["user"]["username"], "sms_alice");
    assert_eq!(register_value["user"]["nickname"], "短信 Alice");
    assert_eq!(register_value["user"]["phone"], "13800138000");
    assert!(register_value["user"]["email"].is_null());

    let row = app
        .db
        .query_one(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "SELECT phone, phone_bound_at FROM users WHERE username = ?",
            vec!["sms_alice".into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let stored_phone: String = row.try_get("", "phone").unwrap();
    let phone_bound_at: Option<String> = row.try_get("", "phone_bound_at").unwrap();
    assert_eq!(stored_phone, "13800138000");
    assert!(phone_bound_at.is_some());

    let (phone_login_status, phone_login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "13800138000", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(phone_login_status, StatusCode::OK, "{phone_login_value}");
    assert_eq!(
        phone_login_value["user"]["id"].as_str().unwrap(),
        register_value["user"]["id"].as_str().unwrap(),
    );
}

#[tokio::test]
async fn sms_send_rate_limit_blocks_same_phone_across_purposes() {
    let app =
        test_app_with_sms_config(sms_config_with_phone_rate_limit(true, 60, 86_400, 20)).await;

    let (first_status, first_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-registration-code",
        None,
        json!({"phone": "13800138110"}),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK, "{first_value}");

    let (second_status, second_value, second_headers) = send_json_with_headers(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login-code",
        None,
        json!({"phone": "13800138110"}),
    )
    .await;

    assert_eq!(
        second_status,
        StatusCode::TOO_MANY_REQUESTS,
        "{second_value}"
    );
    assert_eq!(second_value["code"], "rate_limited");
    assert!(second_value["retry_after_seconds"].as_u64().unwrap() > 0);
    assert!(second_headers.get(header::RETRY_AFTER).is_some());
}

#[tokio::test]
async fn sms_send_rate_limit_blocks_more_than_window_quota() {
    let app = test_app_with_sms_config(sms_config_with_phone_rate_limit(true, 1, 86_400, 20)).await;
    for index in 0..20 {
        insert_sms_challenge(&app, "13800138111", index).await;
    }

    let (status, value, headers) = send_json_with_headers(
        &app.router,
        "POST",
        "/api/v1/auth/sms-registration-code",
        None,
        json!({"phone": "13800138111"}),
    )
    .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{value}");
    assert_eq!(value["code"], "rate_limited");
    assert!(value["retry_after_seconds"].as_u64().unwrap() > 60);
    assert!(headers.get(header::RETRY_AFTER).is_some());
}

#[tokio::test]
async fn phantom_sms_send_for_missing_phone_counts_toward_rate_limit() {
    let app =
        test_app_with_sms_config(sms_config_with_phone_rate_limit(true, 60, 86_400, 20)).await;

    let (first_status, first_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login-code",
        None,
        json!({"phone": "13800138112"}),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK, "{first_value}");
    assert!(first_value.get("debug_code").is_none(), "{first_value}");

    let (second_status, second_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-password-reset-code",
        None,
        json!({"phone": "13800138112"}),
    )
    .await;

    assert_eq!(
        second_status,
        StatusCode::TOO_MANY_REQUESTS,
        "{second_value}"
    );
    assert_eq!(second_value["code"], "rate_limited");
}

#[tokio::test]
async fn sms_login_uses_ticket_once_and_missing_phone_send_does_not_debug() {
    let app = test_app().await;
    register_sms_user(
        &app.router,
        "sms_login_alice",
        "短信登录",
        "13800138001",
        "OutdoorPass123!",
    )
    .await;

    let (missing_status, missing_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login-code",
        None,
        json!({"phone": "13800138099"}),
    )
    .await;
    assert_eq!(missing_status, StatusCode::OK, "{missing_value}");
    assert!(missing_value.get("debug_code").is_none());

    let (code_status, code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login-code",
        None,
        json!({"phone": "13800138001"}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let login_code = code_value["debug_code"].as_str().unwrap().to_owned();

    let (login_status, login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login",
        None,
        json!({
            "phone": "13800138001",
            "sms_ticket": code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": login_code,
        }),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_value}");
    assert_eq!(login_value["user"]["phone"], "13800138001");

    let (replay_status, replay_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login",
        None,
        json!({
            "phone": "13800138001",
            "sms_ticket": code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": code_value["debug_code"].as_str().unwrap(),
        }),
    )
    .await;
    assert_eq!(
        replay_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{replay_value}",
    );

    let (wrong_purpose_status, wrong_purpose_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-login",
        None,
        json!({
            "phone": "13800138099",
            "sms_ticket": missing_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": "123456",
        }),
    )
    .await;
    assert_eq!(
        wrong_purpose_status,
        StatusCode::UNAUTHORIZED,
        "{wrong_purpose_value}",
    );
}

#[tokio::test]
async fn sms_password_reset_revokes_old_sessions() {
    let app = test_app().await;
    let initial_login = register_sms_user(
        &app.router,
        "sms_reset_alice",
        "短信重置",
        "13800138002",
        "OutdoorPass123!",
    )
    .await;
    let old_access_token = initial_login["access_token"].as_str().unwrap().to_owned();

    let (code_status, code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-password-reset-code",
        None,
        json!({"phone": "13800138002"}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");

    let (reset_status, reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/sms-password-reset",
        None,
        json!({
            "phone": "13800138002",
            "sms_ticket": code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": code_value["debug_code"].as_str().unwrap(),
            "password": "NewOutdoorPass123!",
            "confirm_password": "NewOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");

    let (old_access_status, old_access_value) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/stats",
        Some(&old_access_token),
    )
    .await;
    assert_eq!(
        old_access_status,
        StatusCode::UNAUTHORIZED,
        "{old_access_value}",
    );

    let (login_status, login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "13800138002", "password": "NewOutdoorPass123!"}),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_value}");
}

#[tokio::test]
async fn phone_binding_and_rebinding_requires_new_and_current_sms_codes() {
    let app = test_app().await;

    let wechat_login = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "phone-binding-user", "profile": {"nickname": "微信用户"}}),
    )
    .await
    .1;
    let access_token = wechat_login["access_token"].as_str().unwrap();

    let (bind_code_status, bind_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/phone-binding-code",
        Some(access_token),
        json!({"phone": "13800138003"}),
    )
    .await;
    assert_eq!(bind_code_status, StatusCode::OK, "{bind_code_value}");

    let (bind_status, bind_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/phone-binding",
        Some(access_token),
        json!({
            "phone": "13800138003",
            "sms_ticket": bind_code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": bind_code_value["debug_code"].as_str().unwrap(),
        }),
    )
    .await;
    assert_eq!(bind_status, StatusCode::OK, "{bind_value}");
    assert_eq!(bind_value["user"]["phone"], "13800138003");

    let (current_code_status, current_code_value) = send_empty(
        &app.router,
        "POST",
        "/api/v1/me/phone-rebinding-current-code",
        Some(access_token),
    )
    .await;
    assert_eq!(current_code_status, StatusCode::OK, "{current_code_value}");
    assert_eq!(current_code_value["phone"], "13800138003");

    let (new_code_status, new_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/phone-binding-code",
        Some(access_token),
        json!({"phone": "13800138004"}),
    )
    .await;
    assert_eq!(new_code_status, StatusCode::OK, "{new_code_value}");

    let (rebind_missing_current_status, rebind_missing_current_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/phone-binding",
        Some(access_token),
        json!({
            "phone": "13800138004",
            "sms_ticket": new_code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": new_code_value["debug_code"].as_str().unwrap(),
        }),
    )
    .await;
    assert_eq!(
        rebind_missing_current_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{rebind_missing_current_value}",
    );

    let (rebind_status, rebind_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/phone-binding",
        Some(access_token),
        json!({
            "phone": "13800138004",
            "sms_ticket": new_code_value["sms_ticket"].as_str().unwrap(),
            "sms_verification_code": new_code_value["debug_code"].as_str().unwrap(),
            "current_sms_ticket": current_code_value["sms_ticket"].as_str().unwrap(),
            "current_sms_verification_code": current_code_value["debug_code"].as_str().unwrap(),
        }),
    )
    .await;
    assert_eq!(rebind_status, StatusCode::OK, "{rebind_value}");
    assert_eq!(rebind_value["user"]["phone"], "13800138004");
}

#[tokio::test]
async fn password_login_requires_captcha_after_repeated_failures() {
    let app = test_app().await;
    register_password_user(
        &app.router,
        "trail_bob",
        "bob@example.com",
        "OutdoorPass123!",
    )
    .await;

    for _ in 0..3 {
        let (status, value) = send_json(
            &app.router,
            "POST",
            "/api/v1/auth/login",
            None,
            json!({"account": "trail_bob", "password": "wrong-password"}),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
        assert_eq!(value["code"], "invalid_credentials");
    }

    let (captcha_status, captcha_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "trail_bob", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(
        captcha_status,
        StatusCode::PRECONDITION_REQUIRED,
        "{captcha_value}",
    );
    assert_eq!(captcha_value["code"], "captcha_required");
    assert_eq!(captcha_value["captcha"]["type"], "image");
    assert_eq!(captcha_value["captcha"]["endpoint"], "/api/v1/auth/captcha");

    let (captcha_ticket, captcha_answer) = create_captcha(&app.router, "trail_bob").await;
    let (verified_status, verified_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({
            "account": "trail_bob",
            "password": "OutdoorPass123!",
            "captcha_ticket": captcha_ticket,
            "captcha_answer": captcha_answer
        }),
    )
    .await;
    assert_eq!(verified_status, StatusCode::OK, "{verified_value}");

    let (reset_status, reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "bob@example.com", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");
}

#[tokio::test]
async fn protected_gear_routes_reject_missing_token() {
    let app = test_app().await;

    let (status, value) = send_empty(&app.router, "GET", "/api/v1/me/gears/stats", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
    assert_eq!(value["code"], "unauthorized");

    let (overview_status, overview_value) =
        send_empty(&app.router, "GET", "/api/v1/me/gears/overview", None).await;
    assert_eq!(
        overview_status,
        StatusCode::UNAUTHORIZED,
        "{overview_value}"
    );
    assert_eq!(overview_value["code"], "unauthorized");
}

#[tokio::test]
async fn email_code_login_issues_tokens_and_rejects_replay_or_wrong_purpose() {
    let app = test_app().await;
    register_password_user(
        &app.router,
        "mail_login_alice",
        "mail-login-alice@example.com",
        "OutdoorPass123!",
    )
    .await;

    let (code_status, code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login-code",
        None,
        json!({"email": "Mail-Login-Alice@Example.COM"}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    assert_eq!(code_value["email"], "mail-login-alice@example.com");
    let login_code = code_value["debug_code"].as_str().unwrap().to_owned();

    let (login_status, login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login",
        None,
        json!({"email": "mail-login-alice@example.com", "email_verification_code": login_code}),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_value}");
    assert!(login_value["access_token"].as_str().unwrap().len() > 20);
    assert_eq!(login_value["user"]["email"], "mail-login-alice@example.com");

    let (replay_status, replay_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login",
        None,
        json!({"email": "mail-login-alice@example.com", "email_verification_code": code_value["debug_code"].as_str().unwrap()}),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED, "{replay_value}");

    let (register_code_status, register_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({"email": "mail-login-alice@example.com"}),
    )
    .await;
    assert_eq!(
        register_code_status,
        StatusCode::OK,
        "{register_code_value}"
    );
    let (wrong_purpose_status, wrong_purpose_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login",
        None,
        json!({
            "email": "mail-login-alice@example.com",
            "email_verification_code": register_code_value["debug_code"].as_str().unwrap()
        }),
    )
    .await;
    assert_eq!(
        wrong_purpose_status,
        StatusCode::UNAUTHORIZED,
        "{wrong_purpose_value}",
    );
}

#[tokio::test]
async fn email_codes_lock_after_repeated_wrong_guesses() {
    let app = test_app().await;
    register_password_user(
        &app.router,
        "mail_lock_alice",
        "mail-lock-alice@example.com",
        "OutdoorPass123!",
    )
    .await;

    let (login_code_status, login_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login-code",
        None,
        json!({"email": "mail-lock-alice@example.com"}),
    )
    .await;
    assert_eq!(login_code_status, StatusCode::OK, "{login_code_value}");
    let login_code = login_code_value["debug_code"].as_str().unwrap().to_owned();

    for _ in 0..5 {
        let (guess_status, guess_value) = send_json(
            &app.router,
            "POST",
            "/api/v1/auth/email-login",
            None,
            json!({"email": "mail-lock-alice@example.com", "email_verification_code": "000000"}),
        )
        .await;
        assert_eq!(guess_status, StatusCode::UNAUTHORIZED, "{guess_value}");
    }

    let (locked_login_status, locked_login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login",
        None,
        json!({"email": "mail-lock-alice@example.com", "email_verification_code": login_code}),
    )
    .await;
    assert_eq!(
        locked_login_status,
        StatusCode::UNAUTHORIZED,
        "{locked_login_value}",
    );

    let (reset_code_status, reset_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset-code",
        None,
        json!({"email": "mail-lock-alice@example.com"}),
    )
    .await;
    assert_eq!(reset_code_status, StatusCode::OK, "{reset_code_value}");
    let reset_code = reset_code_value["debug_code"].as_str().unwrap().to_owned();

    for _ in 0..5 {
        let (guess_status, guess_value) = send_json(
            &app.router,
            "POST",
            "/api/v1/auth/password-reset",
            None,
            json!({
                "email": "mail-lock-alice@example.com",
                "email_verification_code": "000000",
                "password": "NewOutdoorPass123!",
                "confirm_password": "NewOutdoorPass123!"
            }),
        )
        .await;
        assert_eq!(guess_status, StatusCode::UNAUTHORIZED, "{guess_value}");
    }

    let (locked_reset_status, locked_reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset",
        None,
        json!({
            "email": "mail-lock-alice@example.com",
            "email_verification_code": reset_code,
            "password": "NewOutdoorPass123!",
            "confirm_password": "NewOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(
        locked_reset_status,
        StatusCode::UNAUTHORIZED,
        "{locked_reset_value}",
    );
}

#[tokio::test]
async fn password_reset_updates_password_revokes_old_sessions_and_rejects_replay() {
    let app = test_app().await;
    let initial_login = register_password_user(
        &app.router,
        "reset_alice",
        "reset-alice@example.com",
        "OutdoorPass123!",
    )
    .await;
    let old_access_token = initial_login["access_token"].as_str().unwrap().to_owned();
    let old_refresh_token = initial_login["refresh_token"].as_str().unwrap().to_owned();

    let (code_status, code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset-code",
        None,
        json!({"email": "reset-alice@example.com"}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let reset_code = code_value["debug_code"].as_str().unwrap().to_owned();

    let (reset_status, reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset",
        None,
        json!({
            "email": "reset-alice@example.com",
            "email_verification_code": reset_code,
            "password": "NewOutdoorPass123!",
            "confirm_password": "NewOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");
    assert_eq!(reset_value["user"]["email"], "reset-alice@example.com");
    let new_access_token = reset_value["access_token"].as_str().unwrap();
    assert_ne!(new_access_token, old_access_token);

    let (old_access_status, old_access_value) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/stats",
        Some(&old_access_token),
    )
    .await;
    assert_eq!(
        old_access_status,
        StatusCode::UNAUTHORIZED,
        "{old_access_value}"
    );

    let (old_refresh_status, old_refresh_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/refresh",
        None,
        json!({"refresh_token": old_refresh_token}),
    )
    .await;
    assert_eq!(
        old_refresh_status,
        StatusCode::UNAUTHORIZED,
        "{old_refresh_value}"
    );

    let (old_password_status, old_password_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "reset-alice@example.com", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(
        old_password_status,
        StatusCode::UNAUTHORIZED,
        "{old_password_value}"
    );

    let (new_password_status, new_password_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "reset-alice@example.com", "password": "NewOutdoorPass123!"}),
    )
    .await;
    assert_eq!(new_password_status, StatusCode::OK, "{new_password_value}");

    let (replay_status, replay_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset",
        None,
        json!({
            "email": "reset-alice@example.com",
            "email_verification_code": code_value["debug_code"].as_str().unwrap(),
            "password": "AnotherOutdoorPass123!",
            "confirm_password": "AnotherOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED, "{replay_value}");
}

#[tokio::test]
async fn wechat_user_can_bind_email_then_reset_password() {
    let app = test_app().await;

    let (wechat_status, wechat_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "bind-email-user", "profile": {"nickname": "微信用户"}}),
    )
    .await;
    assert_eq!(wechat_status, StatusCode::OK, "{wechat_value}");
    assert!(wechat_value["user"]["email"].is_null());
    let user_id = wechat_value["user"]["id"].as_str().unwrap().to_owned();
    let access_token = wechat_value["access_token"].as_str().unwrap().to_owned();

    let (missing_reset_code_status, missing_reset_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset-code",
        None,
        json!({"email": "bound-wechat@example.com"}),
    )
    .await;
    assert_eq!(
        missing_reset_code_status,
        StatusCode::OK,
        "{missing_reset_code_value}"
    );
    assert!(missing_reset_code_value.get("debug_code").is_none());

    let (register_code_status, register_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({"email": "bound-wechat@example.com"}),
    )
    .await;
    assert_eq!(
        register_code_status,
        StatusCode::OK,
        "{register_code_value}"
    );
    let register_code = register_code_value["debug_code"].as_str().unwrap();
    let (wrong_purpose_status, wrong_purpose_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding",
        Some(&access_token),
        json!({
            "email": "bound-wechat@example.com",
            "email_verification_code": register_code
        }),
    )
    .await;
    assert_eq!(
        wrong_purpose_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{wrong_purpose_value}"
    );

    let (bind_code_status, bind_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding-code",
        Some(&access_token),
        json!({"email": "Bound-WeChat@Example.COM"}),
    )
    .await;
    assert_eq!(bind_code_status, StatusCode::OK, "{bind_code_value}");
    assert_eq!(bind_code_value["email"], "bound-wechat@example.com");
    let bind_code = bind_code_value["debug_code"].as_str().unwrap().to_owned();

    let (bind_status, bind_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding",
        Some(&access_token),
        json!({
            "email": "bound-wechat@example.com",
            "email_verification_code": bind_code
        }),
    )
    .await;
    assert_eq!(bind_status, StatusCode::OK, "{bind_value}");
    assert_eq!(bind_value["user"]["id"], user_id);
    assert_eq!(bind_value["user"]["email"], "bound-wechat@example.com");

    let (already_bound_status, already_bound_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding-code",
        Some(&access_token),
        json!({"email": "bound-wechat@example.com"}),
    )
    .await;
    assert_eq!(
        already_bound_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{already_bound_value}"
    );

    let (reset_code_status, reset_code_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset-code",
        None,
        json!({"email": "bound-wechat@example.com"}),
    )
    .await;
    assert_eq!(reset_code_status, StatusCode::OK, "{reset_code_value}");
    let reset_code = reset_code_value["debug_code"].as_str().unwrap().to_owned();

    let (reset_status, reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset",
        None,
        json!({
            "email": "bound-wechat@example.com",
            "email_verification_code": reset_code,
            "password": "BoundOutdoorPass123!",
            "confirm_password": "BoundOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");
    assert_eq!(reset_value["user"]["id"], user_id);
    assert_eq!(reset_value["user"]["email"], "bound-wechat@example.com");

    let (password_login_status, password_login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/login",
        None,
        json!({"account": "bound-wechat@example.com", "password": "BoundOutdoorPass123!"}),
    )
    .await;
    assert_eq!(
        password_login_status,
        StatusCode::OK,
        "{password_login_value}"
    );
    assert_eq!(password_login_value["user"]["id"], user_id);
}

#[tokio::test]
async fn bind_email_requires_auth_and_rejects_taken_email() {
    let app = test_app().await;
    register_password_user(
        &app.router,
        "taken_owner",
        "taken@example.com",
        "OutdoorPass123!",
    )
    .await;

    let (unauthorized_status, unauthorized_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding-code",
        None,
        json!({"email": "new-wechat@example.com"}),
    )
    .await;
    assert_eq!(
        unauthorized_status,
        StatusCode::UNAUTHORIZED,
        "{unauthorized_value}"
    );

    let (wechat_status, wechat_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": "bind-email-taken", "profile": {"nickname": "微信用户"}}),
    )
    .await;
    assert_eq!(wechat_status, StatusCode::OK, "{wechat_value}");
    let access_token = wechat_value["access_token"].as_str().unwrap().to_owned();

    let (taken_status, taken_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/email-binding-code",
        Some(&access_token),
        json!({"email": "taken@example.com"}),
    )
    .await;
    assert_eq!(
        taken_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{taken_value}"
    );
    assert_eq!(taken_value["code"], "validation_failed");
}

#[tokio::test]
async fn login_and_reset_code_requests_do_not_reveal_missing_email() {
    let app = test_app().await;

    for path in [
        "/api/v1/auth/email-login-code",
        "/api/v1/auth/password-reset-code",
    ] {
        let (status, value) = send_json(
            &app.router,
            "POST",
            path,
            None,
            json!({"email": "missing@example.com"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        assert_eq!(value["email"], "missing@example.com");
        assert!(value.get("debug_code").is_none(), "{value}");
    }

    let (login_status, login_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/email-login",
        None,
        json!({"email": "missing@example.com", "email_verification_code": "000000"}),
    )
    .await;
    assert_eq!(login_status, StatusCode::UNAUTHORIZED, "{login_value}");

    let (reset_status, reset_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/auth/password-reset",
        None,
        json!({
            "email": "missing@example.com",
            "email_verification_code": "000000",
            "password": "NewOutdoorPass123!",
            "confirm_password": "NewOutdoorPass123!"
        }),
    )
    .await;
    assert_eq!(reset_status, StatusCode::UNAUTHORIZED, "{reset_value}");
}

#[tokio::test]
async fn production_email_login_and_reset_codes_send_mail_and_hide_debug_code() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    AuthRepository::new(db.clone())
        .create_password_user(
            "prod_mail_user",
            "prod-mail-user@example.com",
            &hash_token("OutdoorPass123!"),
        )
        .await
        .unwrap();
    let config = ApiConfig {
        app_env: "production".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: false,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: MailConfig {
            enabled: true,
            smtp_host: "smtp.example.invalid".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: "x".to_owned(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        },
        sms: Default::default(),
    };
    let sender = RecordingEmailSender::default();
    let sent = Arc::clone(&sender.sent);
    let router = build_router(AppState::new_with_email_sender(
        config,
        db,
        Arc::new(sender),
    ));

    let (login_status, login_value) = send_json(
        &router,
        "POST",
        "/api/v1/auth/email-login-code",
        None,
        json!({"email": "prod-mail-user@example.com"}),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "{login_value}");
    assert!(login_value.get("debug_code").is_none(), "{login_value}");

    let (reset_status, reset_value) = send_json(
        &router,
        "POST",
        "/api/v1/auth/password-reset-code",
        None,
        json!({"email": "prod-mail-user@example.com"}),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");
    assert!(reset_value.get("debug_code").is_none(), "{reset_value}");

    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].subject, "寻径星野登录验证码");
    assert_eq!(sent[1].subject, "寻径星野找回密码验证码");
    assert!(sent.iter().all(|message| message.code.len() == 6));
}

#[tokio::test]
async fn production_bind_email_code_sends_mail_and_hides_debug_code() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let repo = AuthRepository::new(db.clone());
    let user = repo
        .upsert_mock_user("mock:prod-bind-email", Some("微信用户".to_owned()), None)
        .await
        .unwrap();
    let access_token = "x";
    repo.create_session(
        &user.id,
        &hash_token(access_token),
        time::OffsetDateTime::now_utc() + time::Duration::hours(2),
        &hash_token("r"),
        time::OffsetDateTime::now_utc() + time::Duration::days(30),
    )
    .await
    .unwrap();
    let config = ApiConfig {
        app_env: "production".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: None,
        database,
        wechat_mock_login: false,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: MailConfig {
            enabled: true,
            smtp_host: "smtp.example.invalid".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: "x".to_owned(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        },
        sms: Default::default(),
    };
    let sender = RecordingEmailSender::default();
    let sent = Arc::clone(&sender.sent);
    let router = build_router(AppState::new_with_email_sender(
        config,
        db,
        Arc::new(sender),
    ));

    let (status, value) = send_json(
        &router,
        "POST",
        "/api/v1/me/email-binding-code",
        Some(access_token),
        json!({"email": "prod-bind-wechat@example.com"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["email"], "prod-bind-wechat@example.com");
    assert!(value.get("debug_code").is_none(), "{value}");

    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].subject, "寻径星野绑定邮箱验证码");
    assert_eq!(sent[0].to, "prod-bind-wechat@example.com");
    assert_eq!(sent[0].code.len(), 6);
}
