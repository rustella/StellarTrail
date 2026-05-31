use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use sea_orm::{ConnectionTrait, Statement};
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
    test_app_with_commit_hash(None).await
}

async fn test_app_with_commit_hash(commit_hash: Option<&str>) -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let config = ApiConfig {
        app_env: "local".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        commit_hash: commit_hash.map(str::to_owned),
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
    read_json_response(response).await
}

async fn send_empty_json(
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
    let username = format!("version_{suffix}");
    let email = format!("version_{suffix}@example.test");
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
    let username = format!("version_{suffix}");
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

#[tokio::test]
async fn public_client_versions_return_seeded_wechat_release() {
    let app = test_app().await;

    let (status, current) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions/current?client_key=wechat_miniprogram",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{current}");
    assert_eq!(current["version"], "0.2.1");
    assert_eq!(current["title"], "0.2.1 离线绳结访问修复");
    assert_eq!(current["status"], "published");
    assert!(current.get("commit_hash").is_none());
    assert_eq!(current["release_notes"].as_array().unwrap().len(), 3);
    assert_eq!(current["release_note_sections"][0]["key"], "feature");
    assert_eq!(current["release_note_sections"][0]["title"], "Feature");
    assert_eq!(current["release_note_sections"][1]["key"], "bug_fix");
    assert_eq!(
        current["release_note_sections"][1]["items"]
            .as_array()
            .unwrap()
            .len(),
        2,
    );

    let (list_status, list) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions?client_key=wechat_miniprogram",
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"][0]["version"], "0.2.1");
    assert!(list["items"][0].get("commit_hash").is_none());
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["version"] == "0.1.0")
    );
}

#[tokio::test]
async fn public_client_versions_validate_client_key_and_filter_drafts() {
    let app = test_app().await;
    let admin_token = grant_admin_role(&app, "draft_filter").await;
    let (create_status, create_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&admin_token),
        json!({
            "client_key": "web",
            "version": "0.2.0",
            "title": "Web 草稿",
            "release_notes": ["暂不公开"],
            "status": "draft"
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{create_value}");

    let (current_status, current_value) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions/current?client_key=web",
        None,
    )
    .await;
    assert_eq!(current_status, StatusCode::NOT_FOUND, "{current_value}");

    let (list_status, list_value) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions?client_key=web",
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "{list_value}");
    assert_eq!(list_value["items"].as_array().unwrap().len(), 0);

    let (bad_status, bad_value) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions?client_key=desktop",
        None,
    )
    .await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_value}");
    assert_eq!(bad_value["fields"][0]["field"], "client_key");
}

#[tokio::test]
async fn public_client_versions_map_legacy_array_notes_to_feature_section() {
    let app = test_app().await;
    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, status, published_at,
                created_by_user_id, updated_by_user_id, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            vec![
                "legacy-array-version".into(),
                "macos".into(),
                "0.2.0".into(),
                "macOS 旧格式版本".into(),
                r#"["旧格式第一条","旧格式第二条"]"#.into(),
                "published".into(),
                "2026-05-24T00:00:00Z".into(),
                sea_orm::Value::String(None),
                sea_orm::Value::String(None),
                "2026-05-24T00:00:00Z".into(),
                "2026-05-24T00:00:00Z".into(),
            ],
        ))
        .await
        .unwrap();

    let (status, value) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions/current?client_key=macos",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(
        value["release_notes"],
        json!(["旧格式第一条", "旧格式第二条"])
    );
    assert_eq!(value["release_note_sections"][0]["key"], "feature");
    assert_eq!(value["release_note_sections"][0]["title"], "Feature");
    assert_eq!(
        value["release_note_sections"][0]["items"],
        json!(["旧格式第一条", "旧格式第二条"])
    );
}

#[tokio::test]
async fn admin_can_create_publish_update_and_list_client_versions() {
    let app = test_app().await;
    let admin_token = grant_admin_role(&app, "publisher").await;

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&admin_token),
        json!({
            "client_key": "android",
            "version": "0.1.0",
            "title": "Android 初始版本",
            "release_note_sections": [
                {"key": "feature", "title": "Feature", "items": ["新增 Android 客户端"]}
            ],
            "status": "draft"
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["published_at"], Value::Null);

    let id = created["id"].as_str().unwrap();
    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/admin/client-versions/{id}"),
        Some(&admin_token),
        json!({
            "client_key": "android",
            "version": "0.1.0",
            "title": "Android 0.1.0",
            "release_note_sections": [
                {"key": "feature", "title": "Feature", "items": ["新增 Android 客户端"]},
                {"key": "bug_fix", "title": "BugFix", "items": ["支持多域名探测"]}
            ],
            "status": "published",
            "commit_hash": "ABCDEF1"
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert!(updated["published_at"].as_str().is_some());
    assert_eq!(updated["release_notes"].as_array().unwrap().len(), 2);
    assert_eq!(updated["commit_hash"], "abcdef1");
    assert_eq!(updated["release_note_sections"][0]["key"], "feature");
    assert_eq!(updated["release_note_sections"][1]["key"], "bug_fix");

    let (admin_list_status, admin_list) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/admin/client-versions?client_key=android&status=published",
        Some(&admin_token),
    )
    .await;
    assert_eq!(admin_list_status, StatusCode::OK, "{admin_list}");
    assert_eq!(admin_list["items"][0]["title"], "Android 0.1.0");
    assert_eq!(admin_list["items"][0]["commit_hash"], "abcdef1");

    let (public_status, public_list) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions?client_key=android",
        None,
    )
    .await;
    assert_eq!(public_status, StatusCode::OK, "{public_list}");
    assert_eq!(public_list["items"][0]["version"], "0.1.0");
    assert!(public_list["items"][0].get("commit_hash").is_none());
}

#[tokio::test]
async fn admin_create_defaults_commit_hash_from_config() {
    let app = test_app_with_commit_hash(Some("376fd6c1ef08636477d5257ab720bc783beeb358")).await;
    let admin_token = grant_admin_role(&app, "commit_default").await;

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&admin_token),
        json!({
            "client_key": "web",
            "version": "0.3.0",
            "title": "Web 0.3.0",
            "release_note_sections": [
                {"key": "feature", "title": "Feature", "items": ["新增后台追踪"]}
            ],
            "status": "published"
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["commit_hash"],
        "376fd6c1ef08636477d5257ab720bc783beeb358"
    );

    let (public_status, public_value) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/client-versions/current?client_key=web",
        None,
    )
    .await;
    assert_eq!(public_status, StatusCode::OK, "{public_value}");
    assert!(public_value.get("commit_hash").is_none());
}

#[tokio::test]
async fn admin_client_versions_require_admin_and_validate_payload() {
    let app = test_app().await;
    let user_token = register_password_user(&app.router, "not_admin").await;

    let payload = json!({
        "client_key": "ios",
        "version": "0.1.0",
        "title": "iOS 版本",
        "release_notes": ["新增 iOS 客户端"],
        "status": "published"
    });
    let (unauth_status, unauth_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        None,
        payload.clone(),
    )
    .await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED, "{unauth_value}");

    let (forbidden_status, forbidden_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&user_token),
        payload,
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN, "{forbidden_value}");

    let admin_token = grant_admin_role(&app, "validator").await;
    let (bad_status, bad_value) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&admin_token),
        json!({
            "client_key": "ios",
            "version": "v1",
            "title": " ",
            "release_note_sections": [
                {"key": "notes", "title": "Notes", "items": ["只有备注"]}
            ],
            "status": "live",
            "commit_hash": "not-a-hash"
        }),
    )
    .await;
    assert_eq!(bad_status, StatusCode::UNPROCESSABLE_ENTITY, "{bad_value}");
    let fields = bad_value["fields"].as_array().unwrap();
    assert!(fields.iter().any(|field| field["field"] == "version"));
    assert!(fields.iter().any(|field| field["field"] == "title"));
    assert!(
        fields
            .iter()
            .any(|field| field["field"] == "release_note_sections")
    );
    assert!(fields.iter().any(|field| field["field"] == "status"));
    assert!(fields.iter().any(|field| field["field"] == "commit_hash"));
}

#[tokio::test]
async fn admin_client_versions_reject_duplicate_client_version() {
    let app = test_app().await;
    let admin_token = grant_admin_role(&app, "duplicate").await;
    let payload = json!({
        "client_key": "wechat_miniprogram",
        "version": "0.1.0",
        "title": "重复版本",
        "release_notes": ["重复"],
        "status": "published"
    });

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/admin/client-versions",
        Some(&admin_token),
        payload,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["fields"][0]["field"], "version");
}
