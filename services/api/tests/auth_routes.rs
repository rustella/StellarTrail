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
    http::{Request, StatusCode, header},
};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    services::wechat::{WechatCodeSession, WechatCodeSessionClient},
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database};
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    db: sea_orm::DatabaseConnection,
    _temp_dir: TempDir,
}

async fn test_app() -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "local".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        database,
        wechat_mock_login: true,
        wechat_app_id: None,
        wechat_app_secret: None,
        content_dir: temp_dir.path().join("content"),
        content_assets_dir: temp_dir.path().join("assets"),
        media_base_url: "/assets".to_owned(),
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        object_storage: Default::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: Default::default(),
        cors: CorsConfig::default(),
    };
    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
        _temp_dir: temp_dir,
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

async fn register_password_user(
    app: &Router,
    username: &str,
    email: &str,
    password: &str,
) -> Value {
    let (code_status, code_value) = send_json(
        app,
        "POST",
        "/api/auth/email-verification-code",
        None,
        json!({"email": email}),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let verification_code = code_value["debug_code"].as_str().unwrap();

    let (register_status, register_value) = send_json(
        app,
        "POST",
        "/api/auth/register",
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

async fn create_captcha(app: &Router, account: &str) -> (String, String) {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/auth/captcha",
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

#[tokio::test]
async fn local_mock_login_returns_token_and_user() {
    let app = test_app().await;

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/auth/wechat-login",
        None,
        json!({"code": "local-dev-user", "profile": {"nickname": "测试用户"}}),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    assert!(value["access_token"].as_str().unwrap().len() > 20);
    assert_eq!(value["user"]["nickname"], "测试用户");
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
        "/api/auth/refresh",
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
        "/api/me/gears/stats",
        Some(new_access_token),
    )
    .await;
    assert_eq!(gear_status, StatusCode::OK, "{gear_stats}");

    let (old_access_status, old_access_value) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
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
        "/api/auth/refresh",
        None,
        json!({"refresh_token": old_refresh_token}),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED, "{replay_value}");
    assert_eq!(replay_value["code"], "unauthorized");
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
        database,
        wechat_mock_login: false,
        wechat_app_id: Some("wx-test-app-id".to_owned()),
        wechat_app_secret: Some("wx-test-secret".to_owned()),
        content_dir: temp_dir.path().join("content"),
        content_assets_dir: temp_dir.path().join("assets"),
        media_base_url: "/assets".to_owned(),
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        object_storage: Default::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: Default::default(),
        cors: CorsConfig::default(),
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
        "/api/auth/wechat-login",
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
        send_empty(&router, "GET", "/api/me/gears/stats", Some(access_token)).await;
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
        "/api/auth/login",
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
        "/api/auth/login",
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
            "/api/auth/login",
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
        "/api/auth/login",
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
    assert_eq!(captcha_value["captcha"]["endpoint"], "/api/auth/captcha");

    let (captcha_ticket, captcha_answer) = create_captcha(&app.router, "trail_bob").await;
    let (verified_status, verified_value) = send_json(
        &app.router,
        "POST",
        "/api/auth/login",
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
        "/api/auth/login",
        None,
        json!({"account": "bob@example.com", "password": "OutdoorPass123!"}),
    )
    .await;
    assert_eq!(reset_status, StatusCode::OK, "{reset_value}");
}

#[tokio::test]
async fn protected_gear_routes_reject_missing_token() {
    let app = test_app().await;

    let (status, value) = send_empty(&app.router, "GET", "/api/me/gears/stats", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
    assert_eq!(value["code"], "unauthorized");
}
