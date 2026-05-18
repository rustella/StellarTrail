use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AdminRoleRepository, ApiUsageQuery, ApiUsageRecord, ApiUsageRepository},
};
use tempfile::TempDir;
use time::OffsetDateTime;
use tokio::time::sleep;
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
    };
    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
        _temp_dir: temp_dir,
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
    json_response(response).await
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
    json_response(response).await
}

async fn json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn register_password_user(app: &Router, suffix: &str, email: &str) -> Value {
    let username = format!("api_usage_{suffix}");
    let passphrase = "OutdoorPass123!";
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
            "password": passphrase,
            "confirm_password": passphrase,
            "email_verification_code": verification_code,
        }),
    )
    .await;
    assert_eq!(register_status, StatusCode::OK, "{register_value}");
    register_value
}

async fn grant_admin_role(app: &TestApp, target_user_id: &str, granted_by_user_id: &str) {
    let result = AdminRoleRepository::new(app.db.clone())
        .grant_admin(target_user_id, granted_by_user_id)
        .await
        .unwrap();
    assert!(result.record.role.can_administer());
}

async fn wait_for_usage(
    db: &sea_orm::DatabaseConnection,
    route_pattern: &str,
    method: &str,
    status_code: i32,
) -> Vec<ApiUsageRecord> {
    let today = OffsetDateTime::now_utc().date().to_string();
    let repo = ApiUsageRepository::new(db.clone());
    for _ in 0..50 {
        let rows = repo
            .list(&ApiUsageQuery {
                from_date: today.clone(),
                to_date: today.clone(),
                user_id: None,
                method: Some(method.to_owned()),
                route_pattern: Some(route_pattern.to_owned()),
                limit: 100,
                offset: 0,
            })
            .await
            .unwrap();
        if rows.iter().any(|row| row.status_code == status_code) {
            return rows;
        }
        sleep(Duration::from_millis(20)).await;
    }
    panic!("usage row for {method} {route_pattern} {status_code} was not persisted");
}

#[tokio::test]
async fn middleware_records_authenticated_user_id_and_route_template_without_query() {
    let admin_email = "api-usage-admin@example.test";
    let app = test_app().await;
    let login = register_password_user(&app.router, "admin", admin_email).await;
    let token = login["access_token"].as_str().unwrap().to_owned();
    let user_id = login["user"]["id"].as_str().unwrap().to_owned();
    grant_admin_role(&app, &user_id, &user_id).await;

    let (status, body) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears?limit=20&token=secret-token",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let rows = wait_for_usage(&app.db, "/api/me/gears", "GET", 200).await;
    let row = rows
        .iter()
        .find(|row| row.user_id.as_deref() == Some(user_id.as_str()) && row.status_code == 200)
        .expect("authenticated usage row should include trusted user id");
    assert_eq!(row.route_pattern, "/api/me/gears");
    assert_eq!(row.call_count, 1);

    let today = OffsetDateTime::now_utc().date().to_string();
    let admin_path = format!(
        "/api/admin/api-usage?from={today}&to={today}&method=GET&route=%2Fapi%2Fme%2Fgears"
    );
    let (admin_status, admin_body) =
        send_empty(&app.router, "GET", &admin_path, Some(&token)).await;
    assert_eq!(admin_status, StatusCode::OK, "{admin_body}");
    let item = admin_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["user_id"] == user_id && item["status_code"] == 200)
        .expect("admin response should expose the matching aggregate");
    assert_eq!(item["route_pattern"], "/api/me/gears");
    let serialized = admin_body.to_string();
    assert!(!serialized.contains("secret-token"));
    assert!(!serialized.contains("limit=20"));
}

#[tokio::test]
async fn middleware_records_failed_auth_as_anonymous_template_only() {
    let app = test_app().await;

    let (status, body) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/private-object-id?access_token=secret-token",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");

    let rows = wait_for_usage(&app.db, "/api/me/gears/:id", "GET", 401).await;
    let row = rows
        .iter()
        .find(|row| row.status_code == 401)
        .expect("failed authentication should still be counted anonymously");
    assert!(row.user_id.is_none());
    assert_eq!(row.route_pattern, "/api/me/gears/:id");
    let serialized = serde_json::to_string(&json!({
        "route_pattern": row.route_pattern,
        "user_id": row.user_id,
    }))
    .unwrap();
    assert!(!serialized.contains("private-object-id"));
    assert!(!serialized.contains("secret-token"));
}

#[tokio::test]
async fn admin_usage_endpoint_requires_database_admin_role() {
    let app = test_app().await;
    let login =
        register_password_user(&app.router, "non_admin", "api-usage-non-admin@example.test").await;
    let token = login["access_token"].as_str().unwrap();
    let today = OffsetDateTime::now_utc().date().to_string();

    let (status, body) = send_empty(
        &app.router,
        "GET",
        &format!("/api/admin/api-usage?from={today}&to={today}"),
        Some(token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}
