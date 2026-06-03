use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, PublicApiConfig, RedisCacheConfig, UploadConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AdminRoleRepository, AuthRepository},
};
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
        commit_hash: None,
        database,
        wechat_mock_login: true,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: UploadConfig::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: PublicApiConfig::default(),
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
    read_json_response(response).await
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
    read_json_response(response).await
}

async fn read_json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn register_password_user(app: &Router, suffix: &str) -> String {
    let username = format!("roadmap_{suffix}");
    let email = format!("roadmap_{suffix}@example.test");
    let password = "OutdoorPass123!";
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
    register_value["access_token"].as_str().unwrap().to_owned()
}

async fn grant_admin_role(app: &TestApp, suffix: &str) -> String {
    let token = register_password_user(&app.router, suffix).await;
    let username = format!("roadmap_{suffix}");
    let user = AuthRepository::new(app.db.clone())
        .find_user_by_username(&username)
        .await
        .unwrap()
        .expect("registered test admin should exist");
    let result = AdminRoleRepository::new(app.db.clone())
        .grant_admin(&user.id, &user.id)
        .await
        .unwrap();
    assert!(result.record.role.can_administer());
    token
}

fn roadmap_payload(title: &str, is_published: bool) -> Value {
    json!({
        "client_key": "wechat_miniprogram",
        "title": title,
        "summary": "一条测试路线图",
        "details": "管理员维护的路线图条目",
        "category": "gear",
        "status": "designing",
        "priority": 55,
        "sort_order": 5,
        "is_published": is_published
    })
}

#[tokio::test]
async fn public_roadmap_returns_seeded_wechat_items_and_validates_filters() {
    let app = test_app().await;

    let (status, body) = send_empty(
        &app.router,
        "GET",
        "/api/v1/roadmap?client_key=wechat_miniprogram&limit=50",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let items = body["items"].as_array().unwrap();
    assert!(items.len() >= 8);
    assert_eq!(items[0]["id"], "smart-packing-template");
    assert_eq!(items[0]["is_voted"], false);
    assert_eq!(items[0]["is_subscribed"], false);
    assert!(items.iter().any(|item| item["id"] == "route-encyclopedia"));

    let (bad_status, bad_body) = send_empty(
        &app.router,
        "GET",
        "/api/v1/roadmap?client_key=desktop",
        None,
    )
    .await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_body}");
    assert_eq!(bad_body["fields"][0]["field"], "client_key");

    let (bad_status, bad_body) =
        send_empty(&app.router, "GET", "/api/v1/roadmap?status=live", None).await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_body}");
    assert_eq!(bad_body["fields"][0]["field"], "status");

    let (bad_status, bad_body) =
        send_empty(&app.router, "GET", "/api/v1/roadmap?cursor=next-page", None).await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_body}");
    assert_eq!(bad_body["fields"][0]["field"], "cursor");
}

#[tokio::test]
async fn current_user_can_vote_subscribe_cancel_and_is_isolated_from_other_users() {
    let app = test_app().await;
    let user_a = register_password_user(&app.router, "alice").await;
    let user_b = register_password_user(&app.router, "bob").await;

    for (method, path) in [
        ("GET", "/api/v1/me/roadmap"),
        ("PUT", "/api/v1/me/roadmap/smart-packing-template/vote"),
        (
            "PUT",
            "/api/v1/me/roadmap/smart-packing-template/subscription",
        ),
    ] {
        let (status, body) = send_empty(&app.router, method, path, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path} {body}");
    }

    let (vote_status, vote_body) = send_empty(
        &app.router,
        "PUT",
        "/api/v1/me/roadmap/smart-packing-template/vote",
        Some(&user_a),
    )
    .await;
    assert_eq!(vote_status, StatusCode::OK, "{vote_body}");
    assert_eq!(vote_body["is_voted"], true);
    assert_eq!(vote_body["vote_count"], 1);

    let (vote_status, vote_body) = send_empty(
        &app.router,
        "PUT",
        "/api/v1/me/roadmap/smart-packing-template/vote",
        Some(&user_a),
    )
    .await;
    assert_eq!(vote_status, StatusCode::OK, "{vote_body}");
    assert_eq!(vote_body["vote_count"], 1);

    let (subscribe_status, subscribe_body) = send_empty(
        &app.router,
        "PUT",
        "/api/v1/me/roadmap/smart-packing-template/subscription",
        Some(&user_a),
    )
    .await;
    assert_eq!(subscribe_status, StatusCode::OK, "{subscribe_body}");
    assert_eq!(subscribe_body["is_subscribed"], true);
    assert_eq!(subscribe_body["subscription_count"], 1);

    let (other_status, other_body) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/roadmap?client_key=wechat_miniprogram",
        Some(&user_b),
    )
    .await;
    assert_eq!(other_status, StatusCode::OK, "{other_body}");
    let other_item = other_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "smart-packing-template")
        .unwrap();
    assert_eq!(other_item["vote_count"], 1);
    assert_eq!(other_item["is_voted"], false);
    assert_eq!(other_item["is_subscribed"], false);

    let (unvote_status, unvote_body) = send_empty(
        &app.router,
        "DELETE",
        "/api/v1/me/roadmap/smart-packing-template/vote",
        Some(&user_a),
    )
    .await;
    assert_eq!(unvote_status, StatusCode::OK, "{unvote_body}");
    assert_eq!(unvote_body["is_voted"], false);
    assert_eq!(unvote_body["vote_count"], 0);

    let (unsubscribe_status, unsubscribe_body) = send_empty(
        &app.router,
        "DELETE",
        "/api/v1/me/roadmap/smart-packing-template/subscription",
        Some(&user_a),
    )
    .await;
    assert_eq!(unsubscribe_status, StatusCode::OK, "{unsubscribe_body}");
    assert_eq!(unsubscribe_body["is_subscribed"], false);
    assert_eq!(unsubscribe_body["subscription_count"], 0);
}

#[tokio::test]
async fn admin_can_create_publish_update_list_and_soft_delete_roadmap_items() {
    let app = test_app().await;
    let user_token = register_password_user(&app.router, "not_admin").await;
    let admin_token = grant_admin_role(&app, "admin").await;

    let (forbidden_status, forbidden_body) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/roadmap",
        Some(&user_token),
        roadmap_payload("普通用户不能创建", true),
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN, "{forbidden_body}");

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/roadmap",
        Some(&admin_token),
        roadmap_payload("测试 Roadmap 草稿", false),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["is_published"], false);
    assert_eq!(created["published_at"], Value::Null);
    let id = created["id"].as_str().unwrap();

    let (public_status, public_body) = send_empty(
        &app.router,
        "GET",
        "/api/v1/roadmap?client_key=wechat_miniprogram&status=designing",
        None,
    )
    .await;
    assert_eq!(public_status, StatusCode::OK, "{public_body}");
    assert_eq!(public_body["items"].as_array().unwrap().len(), 0);

    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/admin/roadmap/{id}"),
        Some(&admin_token),
        roadmap_payload("测试 Roadmap 已发布", true),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert_eq!(updated["is_published"], true);
    assert!(updated["published_at"].as_str().is_some());

    let (admin_list_status, admin_list) = send_empty(
        &app.router,
        "GET",
        "/api/v1/admin/roadmap?status=designing",
        Some(&admin_token),
    )
    .await;
    assert_eq!(admin_list_status, StatusCode::OK, "{admin_list}");
    assert!(
        admin_list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == id)
    );

    let (delete_status, delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/admin/roadmap/{id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (public_status, public_body) = send_empty(
        &app.router,
        "GET",
        "/api/v1/roadmap?client_key=wechat_miniprogram&status=designing",
        None,
    )
    .await;
    assert_eq!(public_status, StatusCode::OK, "{public_body}");
    assert_eq!(public_body["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn admin_roadmap_validates_payload_fields() {
    let app = test_app().await;
    let admin_token = grant_admin_role(&app, "validator").await;

    let (status, body) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/roadmap",
        Some(&admin_token),
        json!({
            "client_key": "desktop",
            "title": " ",
            "summary": " ",
            "details": null,
            "category": "navigation",
            "status": "live",
            "priority": 101,
            "sort_order": 100001,
            "is_published": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let fields = body["fields"].as_array().unwrap();
    for expected in [
        "client_key",
        "title",
        "summary",
        "category",
        "status",
        "priority",
        "sort_order",
    ] {
        assert!(
            fields.iter().any(|field| field["field"] == expected),
            "missing {expected}: {fields:?}"
        );
    }
}
