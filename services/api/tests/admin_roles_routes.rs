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
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database};
use tempfile::TempDir;
use time::{OffsetDateTime, format_description::well_known::Iso8601};
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
        request_signature: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
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

async fn register_password_user(app: &Router, username: &str) -> Value {
    let email = format!("{username}@example.test");
    let (code_status, code_body) = send_json(
        app,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({ "email": email }),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_body}");
    let code = code_body["debug_code"].as_str().unwrap();

    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        json!({
            "username": username,
            "email": email,
            "password": "Password1",
            "confirm_password": "Password1",
            "email_verification_code": code
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}

async fn insert_role(app: &TestApp, user_id: &str, role: &str) {
    let now = OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap();
    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            r#"INSERT INTO admin_roles (
                user_id, role, granted_by_user_id, created_at, updated_at
            ) VALUES (?, ?, NULL, ?, ?)
            ON CONFLICT (user_id) DO UPDATE SET
                role = excluded.role,
                updated_at = excluded.updated_at"#,
            vec![
                user_id.to_owned().into(),
                role.to_owned().into(),
                now.clone().into(),
                now.into(),
            ],
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn admin_management_requires_authenticated_super_admin() {
    let app = test_app().await;
    let normal_login = register_password_user(&app.router, "normal_admin_request").await;
    let normal_token = normal_login["access_token"].as_str().unwrap();
    let regular_admin_login = register_password_user(&app.router, "regular_admin_request").await;
    let regular_admin_token = regular_admin_login["access_token"].as_str().unwrap();
    let regular_admin_id = regular_admin_login["user"]["id"].as_str().unwrap();
    insert_role(&app, regular_admin_id, "admin").await;

    let (missing_status, missing) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        None,
        json!({"username": "target_admin"}),
    )
    .await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED, "{missing}");

    let (normal_status, normal) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(normal_token),
        json!({"username": "target_admin"}),
    )
    .await;
    assert_eq!(normal_status, StatusCode::FORBIDDEN, "{normal}");

    let (admin_status, admin) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(regular_admin_token),
        json!({"username": "target_admin"}),
    )
    .await;
    assert_eq!(admin_status, StatusCode::FORBIDDEN, "{admin}");

    let (admin_delete_status, admin_delete) = send_empty(
        &app.router,
        "DELETE",
        "/api/v1/admin/admins?username=target_admin",
        Some(regular_admin_token),
    )
    .await;
    assert_eq!(admin_delete_status, StatusCode::FORBIDDEN, "{admin_delete}");
}

#[tokio::test]
async fn super_admin_grants_and_revokes_admin_by_username_and_user_id() {
    let app = test_app().await;
    let owner_login = register_password_user(&app.router, "owner_super_admin").await;
    let owner_token = owner_login["access_token"].as_str().unwrap();
    let owner_id = owner_login["user"]["id"].as_str().unwrap();
    insert_role(&app, owner_id, "super_admin").await;
    let target_login = register_password_user(&app.router, "trail_admin").await;
    let target_token = target_login["access_token"].as_str().unwrap();
    let target_id = target_login["user"]["id"].as_str().unwrap();

    let (grant_status, grant) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"username": "Trail_Admin"}),
    )
    .await;
    assert_eq!(grant_status, StatusCode::CREATED, "{grant}");
    assert_eq!(grant["user_id"], target_id);
    assert_eq!(grant["role"], "admin");

    let today = OffsetDateTime::now_utc().date().to_string();
    let (admin_access_status, admin_access) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/admin/api-usage?from={today}&to={today}"),
        Some(target_token),
    )
    .await;
    assert_eq!(admin_access_status, StatusCode::OK, "{admin_access}");

    let (repeat_status, repeat) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"username": "trail_admin"}),
    )
    .await;
    assert_eq!(repeat_status, StatusCode::OK, "{repeat}");
    assert_eq!(repeat["role"], "admin");

    let (delete_status, delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/admin/admins?user_id={target_id}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (removed_status, removed) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/admin/api-usage?from={today}&to={today}"),
        Some(target_token),
    )
    .await;
    assert_eq!(removed_status, StatusCode::FORBIDDEN, "{removed}");

    let (grant_by_id_status, grant_by_id) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"user_id": target_id}),
    )
    .await;
    assert_eq!(grant_by_id_status, StatusCode::CREATED, "{grant_by_id}");

    let (delete_by_name_status, delete_by_name_body) = send_empty(
        &app.router,
        "DELETE",
        "/api/v1/admin/admins?username=trail_admin",
        Some(owner_token),
    )
    .await;
    assert_eq!(
        delete_by_name_status,
        StatusCode::NO_CONTENT,
        "{delete_by_name_body}"
    );
}

#[tokio::test]
async fn grant_does_not_downgrade_super_admin_and_delete_rejects_super_admin() {
    let app = test_app().await;
    let owner_login = register_password_user(&app.router, "owner_super_admin_delete").await;
    let owner_token = owner_login["access_token"].as_str().unwrap();
    let owner_id = owner_login["user"]["id"].as_str().unwrap();
    insert_role(&app, owner_id, "super_admin").await;
    let target_login = register_password_user(&app.router, "second_super_admin").await;
    let target_id = target_login["user"]["id"].as_str().unwrap();
    insert_role(&app, target_id, "super_admin").await;

    let (grant_status, grant) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"user_id": target_id}),
    )
    .await;
    assert_eq!(grant_status, StatusCode::OK, "{grant}");
    assert_eq!(grant["role"], "super_admin");

    let (delete_status, delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/admin/admins?user_id={target_id}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{delete_body}"
    );
    assert_eq!(delete_body["code"], "validation_failed");
}

#[tokio::test]
async fn admin_selector_validation_and_missing_targets_are_stable() {
    let app = test_app().await;
    let owner_login = register_password_user(&app.router, "owner_selector_admin").await;
    let owner_token = owner_login["access_token"].as_str().unwrap();
    let owner_id = owner_login["user"]["id"].as_str().unwrap();
    insert_role(&app, owner_id, "super_admin").await;

    let (empty_status, empty) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({}),
    )
    .await;
    assert_eq!(empty_status, StatusCode::UNPROCESSABLE_ENTITY, "{empty}");

    let (both_status, both) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"username": "missing_admin", "user_id": "missing-id"}),
    )
    .await;
    assert_eq!(both_status, StatusCode::UNPROCESSABLE_ENTITY, "{both}");

    let (missing_grant_status, missing_grant) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/admins",
        Some(owner_token),
        json!({"username": "missing_admin"}),
    )
    .await;
    assert_eq!(
        missing_grant_status,
        StatusCode::NOT_FOUND,
        "{missing_grant}"
    );

    let (missing_delete_status, missing_delete) = send_empty(
        &app.router,
        "DELETE",
        "/api/v1/admin/admins?username=missing_admin",
        Some(owner_token),
    )
    .await;
    assert_eq!(
        missing_delete_status,
        StatusCode::NOT_FOUND,
        "{missing_delete}"
    );

    let no_role_login = register_password_user(&app.router, "no_role_admin_delete").await;
    let no_role_id = no_role_login["user"]["id"].as_str().unwrap();
    let (no_role_delete_status, no_role_delete) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/admin/admins?user_id={no_role_id}"),
        Some(owner_token),
    )
    .await;
    assert_eq!(
        no_role_delete_status,
        StatusCode::NOT_FOUND,
        "{no_role_delete}"
    );
}
