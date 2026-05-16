use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    config::ApiConfig,
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
    };
    TestApp {
        router: build_router(AppState::new(config, db)),
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
async fn protected_gear_routes_reject_missing_token() {
    let app = test_app().await;

    let (status, value) = send_empty(&app.router, "GET", "/api/me/gears/stats", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
    assert_eq!(value["code"], "unauthorized");
}
