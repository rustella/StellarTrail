use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    cache::{Cache, InMemoryCacheStore},
    config::{
        ApiConfig, AvatarStorageConfig, CorsConfig, ObjectStorageConfig, PublicApiConfig,
        RedisCacheConfig, UploadConfig,
    },
    migrate_database,
    object_store::InMemoryObjectStore,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AdminRoleRepository, AuthRepository},
};
use tempfile::TempDir;
use tower::ServiceExt;

const PNG_1X1: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1, 13, 10,
    45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

struct TestApp {
    router: Router,
    db: sea_orm::DatabaseConnection,
    object_store: InMemoryObjectStore,
    _temp_dir: TempDir,
}

async fn test_app(max_images_per_window: u64) -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    let object_store = InMemoryObjectStore::default();
    let cache_store = InMemoryCacheStore::default();
    let config = ApiConfig {
        app_env: "local".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        database,
        wechat_mock_login: true,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: UploadConfig {
            max_image_bytes: 8_000_000,
            rate_limit_window_seconds: 3600,
            max_images_per_window,
        },
        minio: Default::default(),
        object_storage: ObjectStorageConfig {
            bucket: "test-uploads".to_owned(),
        },
        avatar_storage: AvatarStorageConfig {
            bucket: "test-avatars".to_owned(),
            public_base_url: "https://assets.example.test/test-avatars".to_owned(),
            max_image_bytes: 2_000_000,
        },
        knots_media_storage: Default::default(),
        public_api: PublicApiConfig::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    let state = AppState::new_with_cache_and_object_store(
        config,
        db.clone(),
        Cache::with_store_for_tests(cache_store, "test-stellartrail", Duration::from_secs(300)),
        Arc::new(object_store.clone()),
    );
    TestApp {
        router: build_router(state),
        db,
        object_store,
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
) -> (StatusCode, Vec<u8>) {
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
    (status, bytes.to_vec())
}

async fn send_empty_json(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let (status, bytes) = send_empty(app, method, path, token).await;
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn register_password_user(app: &Router, suffix: &str) -> String {
    let username = format!("upload_{suffix}");
    let email = format!("upload_{suffix}@example.test");
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
    let username = format!("upload_{suffix}");
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

async fn upload_image(
    app: &Router,
    token: Option<&str>,
    filename: &str,
    declared_content_type: &str,
    bytes: &[u8],
) -> (StatusCode, Value) {
    let boundary = "stellartrail-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"purpose\"\r\n\r\nfeedback\r\n");
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {declared_content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/me/uploads")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        );
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body)).unwrap())
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

async fn upload_avatar(
    app: &Router,
    token: Option<&str>,
    filename: &str,
    declared_content_type: &str,
    bytes: &[u8],
) -> (StatusCode, Value) {
    let boundary = "stellartrail-avatar-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {declared_content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let mut builder = Request::builder()
        .method("PUT")
        .uri("/api/v1/me/profile/avatar")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        );
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body)).unwrap())
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
async fn authenticated_user_can_upload_png_feedback_image_to_object_store() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "happy").await;

    let (status, value) = upload_image(
        &app.router,
        Some(&token),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{value}");
    assert_eq!(value["purpose"], "feedback");
    assert_eq!(value["image_type"], "png");
    assert_eq!(value["content_type"], "image/png");
    assert_eq!(value["size_bytes"], PNG_1X1.len());
    assert_eq!(value["sha256"].as_str().unwrap().len(), 64);
    assert!(
        value["download_url"]
            .as_str()
            .unwrap()
            .starts_with("/api/v1/me/uploads/")
    );
    assert_eq!(app.object_store.object_count(), 1);
    let stored = app.object_store.only_object().unwrap();
    assert!(stored.object_key.starts_with("feedback-images/"));
    assert!(stored.object_key.ends_with(".png"));
    assert_eq!(stored.content_type, "image/png");
}

#[tokio::test]
async fn authenticated_user_can_upload_avatar_to_public_bucket_and_update_profile() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "avatar").await;

    let (status, value) = upload_avatar(
        &app.router,
        Some(&token),
        "avatar.png",
        "image/png",
        PNG_1X1,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["user"]["username"], "upload_avatar");
    let avatar_url = value["user"]["avatar_url"].as_str().unwrap();
    assert!(
        avatar_url.starts_with("https://assets.example.test/test-avatars/users/"),
        "{avatar_url}"
    );
    assert!(avatar_url.ends_with(".png"), "{avatar_url}");
    assert_eq!(app.object_store.object_count(), 1);
    let stored = app.object_store.only_object().unwrap();
    assert_eq!(stored.bucket.as_deref(), Some("test-avatars"));
    assert!(stored.object_key.starts_with("users/"));
    assert!(stored.object_key.ends_with(".png"));
    assert_eq!(stored.content_type, "image/png");
    assert_eq!(
        stored.cache_control.as_deref(),
        Some("public, max-age=31536000, immutable")
    );
}

#[tokio::test]
async fn avatar_upload_accepts_wechat_temp_file_without_extension() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "wechat_avatar").await;

    let (status, value) = upload_avatar(
        &app.router,
        Some(&token),
        "tmp_avatar",
        "application/octet-stream",
        PNG_1X1,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{value}");
    let avatar_url = value["user"]["avatar_url"].as_str().unwrap();
    assert!(
        avatar_url.starts_with("https://assets.example.test/test-avatars/users/"),
        "{avatar_url}"
    );
    assert!(avatar_url.ends_with(".png"), "{avatar_url}");
    let stored = app.object_store.only_object().unwrap();
    assert_eq!(stored.content_type, "image/png");
}

#[tokio::test]
async fn avatar_upload_requires_authentication() {
    let app = test_app(30).await;

    let (status, value) =
        upload_avatar(&app.router, None, "avatar.png", "image/png", PNG_1X1).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
    assert_eq!(value["code"], "unauthorized");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn avatar_upload_rejects_invalid_image_without_storing_object() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "avatar-invalid").await;

    let (status, value) = upload_avatar(
        &app.router,
        Some(&token),
        "avatar.png",
        "image/png",
        b"<html>not an image</html>",
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["code"], "validation_failed");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn authenticated_user_can_download_own_upload_from_object_store() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "download").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&token),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let id = upload["id"].as_str().unwrap();

    let (download_status, body) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/uploads/{id}"),
        Some(&token),
    )
    .await;

    assert_eq!(download_status, StatusCode::OK);
    assert_eq!(&body[..8], &PNG_1X1[..8]);
}

#[tokio::test]
async fn upload_rejects_extension_magic_mismatch_without_storing_object() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "mismatch").await;

    let (status, value) = upload_image(
        &app.router,
        Some(&token),
        "screen.jpg",
        "image/jpeg",
        PNG_1X1,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["code"], "validation_failed");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn upload_rejects_content_type_magic_mismatch_without_storing_object() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "mime").await;

    let (status, value) = upload_image(
        &app.router,
        Some(&token),
        "screen.png",
        "image/jpeg",
        PNG_1X1,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["code"], "validation_failed");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn upload_rejects_html_with_image_extension_without_storing_object() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "html").await;

    let (status, value) = upload_image(
        &app.router,
        Some(&token),
        "payload.png",
        "image/png",
        b"<script>alert(1)</script>",
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn upload_rejects_image_larger_than_8mb_without_storing_object() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "large").await;
    let mut bytes = vec![0_u8; 8_000_001];
    bytes[..PNG_1X1.len()].copy_from_slice(PNG_1X1);

    let (status, value) =
        upload_image(&app.router, Some(&token), "large.png", "image/png", &bytes).await;

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{value}");
    assert_eq!(value["code"], "payload_too_large");
    assert_eq!(app.object_store.object_count(), 0);
}

#[tokio::test]
async fn upload_rate_limits_same_user_within_window() {
    let app = test_app(1).await;
    let token = register_password_user(&app.router, "limited").await;

    let (first_status, first) =
        upload_image(&app.router, Some(&token), "one.png", "image/png", PNG_1X1).await;
    assert_eq!(first_status, StatusCode::CREATED, "{first}");
    let (second_status, second) =
        upload_image(&app.router, Some(&token), "two.png", "image/png", PNG_1X1).await;

    assert_eq!(second_status, StatusCode::TOO_MANY_REQUESTS, "{second}");
    assert_eq!(second["code"], "rate_limited");
}

#[tokio::test]
async fn upload_rate_limit_is_per_user() {
    let app = test_app(1).await;
    let token_a = register_password_user(&app.router, "limited_a").await;
    let token_b = register_password_user(&app.router, "limited_b").await;
    let (first_status, first) =
        upload_image(&app.router, Some(&token_a), "one.png", "image/png", PNG_1X1).await;
    assert_eq!(first_status, StatusCode::CREATED, "{first}");

    let (second_status, second) =
        upload_image(&app.router, Some(&token_b), "two.png", "image/png", PNG_1X1).await;

    assert_eq!(second_status, StatusCode::CREATED, "{second}");
}

#[tokio::test]
async fn upload_requires_authentication() {
    let app = test_app(30).await;

    let (status, value) = upload_image(&app.router, None, "screen.png", "image/png", PNG_1X1).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{value}");
}

#[tokio::test]
async fn download_rejects_cross_user_access() {
    let app = test_app(30).await;
    let token_a = register_password_user(&app.router, "owner").await;
    let token_b = register_password_user(&app.router, "other").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&token_a),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let id = upload["id"].as_str().unwrap();

    let (download_status, _body) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/uploads/{id}"),
        Some(&token_b),
    )
    .await;

    assert_eq!(download_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn authenticated_user_can_submit_feedback_with_uploaded_images() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "feedback").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&token),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let image_id = upload["id"].as_str().unwrap();

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/feedback",
        Some(&token),
        json!({
            "category": "bug",
            "content": "路线详情页海拔显示不对",
            "contact": "feedback@example.test",
            "page": "/pages/routes/detail/index?id=wugongshan",
            "client_platform": "wechat_miniprogram",
            "client_version": "0.1.0",
            "device_model": "iPhone 15",
            "image_ids": [image_id],
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{value}");
    assert_eq!(value["status"], "open");
    assert_eq!(value["images"].as_array().unwrap().len(), 1);
    assert_eq!(value["images"][0]["id"], image_id);
}

#[tokio::test]
async fn admin_can_list_feedback_and_download_attached_images() {
    let app = test_app(30).await;
    let user_token = register_password_user(&app.router, "admin_feedback_user").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&user_token),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let image_id = upload["id"].as_str().unwrap();

    let (submit_status, submitted) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/feedback",
        Some(&user_token),
        json!({
            "category": "bug",
            "content": "装备详情页图片没有显示",
            "contact": "feedback@example.test",
            "page": "/pages/gears/detail/index?id=gear-1",
            "client_platform": "wechat_miniprogram",
            "client_version": "0.1.0",
            "device_model": "iPhone 15",
            "image_ids": [image_id],
        }),
    )
    .await;
    assert_eq!(submit_status, StatusCode::CREATED, "{submitted}");

    let (missing_status, missing) =
        send_empty_json(&app.router, "GET", "/api/v1/admin/feedback", None).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED, "{missing}");

    let (non_admin_status, non_admin) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/admin/feedback",
        Some(&user_token),
    )
    .await;
    assert_eq!(non_admin_status, StatusCode::FORBIDDEN, "{non_admin}");

    let admin_token = grant_admin_role(&app, "feedback_admin").await;
    let (list_status, list) = send_empty_json(
        &app.router,
        "GET",
        "/api/v1/admin/feedback?status=open",
        Some(&admin_token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["category"], "bug");
    assert_eq!(list["items"][0]["content"], "装备详情页图片没有显示");
    assert_eq!(list["items"][0]["contact"], "feedback@example.test");
    assert_eq!(
        list["items"][0]["user"]["username"],
        "upload_admin_feedback_user"
    );
    assert_eq!(list["items"][0]["images"][0]["id"], image_id);
    assert_eq!(
        list["items"][0]["images"][0]["download_url"],
        format!("/api/v1/admin/feedback-images/{image_id}")
    );

    let (image_status, image_bytes) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/admin/feedback-images/{image_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(image_status, StatusCode::OK);
    assert_eq!(image_bytes, PNG_1X1);
}

#[tokio::test]
async fn feedback_allows_more_than_five_images() {
    let app = test_app(20).await;
    let token = register_password_user(&app.router, "manyimages").await;
    let mut image_ids = Vec::new();
    for index in 0..6 {
        let filename = format!("screen-{index}.png");
        let (upload_status, upload) =
            upload_image(&app.router, Some(&token), &filename, "image/png", PNG_1X1).await;
        assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
        image_ids.push(upload["id"].as_str().unwrap().to_owned());
    }

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/feedback",
        Some(&token),
        json!({
            "category": "suggestion",
            "content": "一次反馈需要关联超过五张图片",
            "image_ids": image_ids,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{value}");
    assert_eq!(value["images"].as_array().unwrap().len(), 6);
}

#[tokio::test]
async fn feedback_rejects_image_from_another_user() {
    let app = test_app(30).await;
    let token_a = register_password_user(&app.router, "image_owner").await;
    let token_b = register_password_user(&app.router, "feedback_owner").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&token_a),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let image_id = upload["id"].as_str().unwrap();

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/feedback",
        Some(&token_b),
        json!({
            "category": "bug",
            "content": "引用别人图片应该失败",
            "image_ids": [image_id],
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["code"], "validation_failed");
}

#[tokio::test]
async fn feedback_rejects_duplicate_image_ids() {
    let app = test_app(30).await;
    let token = register_password_user(&app.router, "dupe").await;
    let (upload_status, upload) = upload_image(
        &app.router,
        Some(&token),
        "screen.png",
        "image/png",
        PNG_1X1,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED, "{upload}");
    let image_id = upload["id"].as_str().unwrap();

    let (status, value) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/feedback",
        Some(&token),
        json!({
            "category": "bug",
            "content": "重复图片应该失败",
            "image_ids": [image_id, image_id],
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{value}");
    assert_eq!(value["code"], "validation_failed");
}
