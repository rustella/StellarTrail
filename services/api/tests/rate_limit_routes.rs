use std::net::SocketAddr;

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, RateLimitConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database};
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    _temp_dir: TempDir,
}

async fn test_app(rate_limit: RateLimitConfig) -> TestApp {
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
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: Default::default(),
        rate_limit,
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    TestApp {
        router: build_router(AppState::new(config, db)),
        _temp_dir: temp_dir,
    }
}

fn rate_limit_config(max_requests_per_ip: u64, max_requests_per_user: u64) -> RateLimitConfig {
    RateLimitConfig {
        enabled: true,
        window_seconds: 60,
        max_requests_per_ip,
        max_requests_per_user,
        trust_proxy_headers: false,
        trusted_proxy_cidrs: Vec::new(),
    }
}

async fn send_empty(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    ip: [u8; 4],
) -> (StatusCode, Value, axum::http::HeaderMap) {
    let addr = SocketAddr::from((ip, 12345));
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let mut request = builder.body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo(addr));
    let response = app.clone().oneshot(request).await.unwrap();
    response_parts(response).await
}

async fn send_empty_with_xff(
    app: &Router,
    path: &str,
    direct_ip: [u8; 4],
    x_forwarded_for: &str,
) -> (StatusCode, Value, axum::http::HeaderMap) {
    let addr = SocketAddr::from((direct_ip, 12345));
    let mut request = Request::builder()
        .uri(path)
        .header("x-forwarded-for", x_forwarded_for)
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(ConnectInfo(addr));
    let response = app.clone().oneshot(request).await.unwrap();
    response_parts(response).await
}

async fn send_json(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Value,
    ip: [u8; 4],
) -> (StatusCode, Value, axum::http::HeaderMap) {
    let addr = SocketAddr::from((ip, 12345));
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let mut request = builder.body(Body::from(body.to_string())).unwrap();
    request.extensions_mut().insert(ConnectInfo(addr));
    let response = app.clone().oneshot(request).await.unwrap();
    response_parts(response).await
}

async fn response_parts(
    response: axum::response::Response,
) -> (StatusCode, Value, axum::http::HeaderMap) {
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
    ip: [u8; 4],
) -> Value {
    let (code_status, code_value, _) = send_json(
        app,
        "POST",
        "/api/auth/email-verification-code",
        None,
        json!({"email": email}),
        ip,
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_value}");
    let verification_code = code_value["debug_code"].as_str().unwrap();

    let (register_status, register_value, _) = send_json(
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
        ip,
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_value}");
    register_value
}

#[tokio::test]
async fn unauthenticated_requests_are_rate_limited_by_ip() {
    let app = test_app(rate_limit_config(2, 100)).await;

    let (first_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 10]).await;
    let (second_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 10]).await;
    let (third_status, third_body, third_headers) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 10]).await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(third_status, StatusCode::TOO_MANY_REQUESTS, "{third_body}");
    assert_eq!(third_body["code"], "rate_limited");
    assert!(third_headers.contains_key(header::RETRY_AFTER));
}

#[tokio::test]
async fn ip_limit_is_shared_across_routes_and_fallback() {
    let app = test_app(rate_limit_config(2, 100)).await;

    let (health_status, _, _) =
        send_empty(&app.router, "GET", "/healthz", None, [203, 0, 113, 11]).await;
    let (meta_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 11]).await;
    let (missing_status, body, _) =
        send_empty(&app.router, "GET", "/api/missing", None, [203, 0, 113, 11]).await;

    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(meta_status, StatusCode::OK);
    assert_eq!(missing_status, StatusCode::TOO_MANY_REQUESTS, "{body}");
}

#[tokio::test]
async fn different_ips_have_separate_buckets() {
    let app = test_app(rate_limit_config(1, 100)).await;

    let (first_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 12]).await;
    let (second_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 12]).await;
    let (other_ip_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [198, 51, 100, 12]).await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(other_ip_status, StatusCode::OK);
}

#[tokio::test]
async fn trusted_proxy_headers_choose_forwarded_client_ip() {
    let app = test_app(RateLimitConfig {
        trust_proxy_headers: true,
        trusted_proxy_cidrs: vec!["172.16.0.0/12".to_owned()],
        ..rate_limit_config(1, 100)
    })
    .await;

    let (first_status, _, _) = send_empty_with_xff(
        &app.router,
        "/api/meta",
        [172, 16, 0, 2],
        "203.0.113.13, 198.51.100.1",
    )
    .await;
    let (forwarded_limited_status, _, _) =
        send_empty_with_xff(&app.router, "/api/meta", [172, 16, 0, 3], "203.0.113.13").await;
    let (other_forwarded_status, _, _) =
        send_empty_with_xff(&app.router, "/api/meta", [172, 16, 0, 3], "203.0.113.14").await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(forwarded_limited_status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(other_forwarded_status, StatusCode::OK);
}

#[tokio::test]
async fn untrusted_proxy_headers_are_ignored() {
    let app = test_app(RateLimitConfig {
        trust_proxy_headers: true,
        trusted_proxy_cidrs: vec!["172.16.0.0/12".to_owned()],
        ..rate_limit_config(1, 100)
    })
    .await;

    let (first_status, _, _) =
        send_empty_with_xff(&app.router, "/api/meta", [198, 51, 100, 20], "203.0.113.20").await;
    let (direct_ip_limited_status, _, _) =
        send_empty_with_xff(&app.router, "/api/meta", [198, 51, 100, 20], "203.0.113.21").await;
    let (same_forwarded_different_direct_status, _, _) =
        send_empty_with_xff(&app.router, "/api/meta", [198, 51, 100, 21], "203.0.113.20").await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(direct_ip_limited_status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(same_forwarded_different_direct_status, StatusCode::OK);
}

#[tokio::test]
async fn authenticated_requests_are_rate_limited_by_user_even_when_ip_has_capacity() {
    let app = test_app(rate_limit_config(100, 2)).await;
    let register_value = register_password_user(
        &app.router,
        "limit_user_one",
        "limit-user-one@example.test",
        "OutdoorPass123!",
        [203, 0, 113, 30],
    )
    .await;
    let token = register_value["access_token"].as_str().unwrap();

    let (first_status, _, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(token),
        [203, 0, 113, 30],
    )
    .await;
    let (second_status, _, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(token),
        [203, 0, 113, 31],
    )
    .await;
    let (third_status, body, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(token),
        [203, 0, 113, 32],
    )
    .await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(third_status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert_eq!(body["code"], "rate_limited");
}

#[tokio::test]
async fn authenticated_requests_share_ip_limit_across_users() {
    let app = test_app(rate_limit_config(6, 100)).await;
    let first = register_password_user(
        &app.router,
        "ip_shared_one",
        "ip-shared-one@example.test",
        "OutdoorPass123!",
        [203, 0, 113, 40],
    )
    .await;
    let second = register_password_user(
        &app.router,
        "ip_shared_two",
        "ip-shared-two@example.test",
        "OutdoorPass123!",
        [203, 0, 113, 40],
    )
    .await;
    let first_token = first["access_token"].as_str().unwrap();
    let second_token = second["access_token"].as_str().unwrap();

    let (first_status, _, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(first_token),
        [203, 0, 113, 40],
    )
    .await;
    let (second_status, _, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(second_token),
        [203, 0, 113, 40],
    )
    .await;
    let (third_status, body, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some(first_token),
        [203, 0, 113, 40],
    )
    .await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(third_status, StatusCode::TOO_MANY_REQUESTS, "{body}");
}

#[tokio::test]
async fn disabling_global_rate_limit_preserves_original_route_behavior() {
    let app = test_app(RateLimitConfig {
        enabled: false,
        ..rate_limit_config(1, 1)
    })
    .await;

    let (first_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 50]).await;
    let (second_status, _, _) =
        send_empty(&app.router, "GET", "/api/meta", None, [203, 0, 113, 50]).await;

    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);
}

#[tokio::test]
async fn invalid_bearer_token_still_uses_original_auth_failure() {
    let app = test_app(rate_limit_config(100, 1)).await;

    let (status, body, _) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/stats",
        Some("not-a-real-token"),
        [203, 0, 113, 60],
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["code"], "unauthorized");
}
