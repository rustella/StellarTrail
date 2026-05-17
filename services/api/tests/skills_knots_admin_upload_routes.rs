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
        AdminConfig, ApiConfig, CorsConfig, KnotsMediaStorageConfig, ObjectStorageConfig,
        PublicApiConfig, RedisCacheConfig, UploadConfig,
    },
    migrate_database,
    object_store::InMemoryObjectStore,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotSeed, KnotTypeSeed, Locale,
};
use tempfile::TempDir;
use tower::ServiceExt;

const WEBP_BYTES: &[u8] = b"RIFF\x10\x00\x00\x00WEBPVP8 test-webp";
const NOT_GIF_BYTES: &[u8] = b"<html>not an image</html>";

struct TestApp {
    router: Router,
    object_store: InMemoryObjectStore,
    _temp_dir: TempDir,
}

fn sample_knot() -> KnotSeed {
    KnotSeed {
        id: "adjustable-grip-hitch-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/knots/en_us/adjustable-grip-hitch-knot".to_owned()),
        source_slug_en: "adjustable-grip-hitch-knot".to_owned(),
        source_slug_zh: Some("ke-tiao-jie-sheng-jie".to_owned()),
        difficulty: Some("beginner".to_owned()),
        localizations: vec![KnotLocalizationSeed {
            locale: Locale::En,
            slug: "adjustable-grip-hitch-knot".to_owned(),
            title: "Adjustable Grip Hitch".to_owned(),
            summary: "Adjust tension on a line.".to_owned(),
            description: Some("A practical hitch for tensioning guylines.".to_owned()),
            steps: vec!["Wrap the working end around the standing part.".to_owned()],
        }],
        categories: vec![KnotCategorySeed {
            id: "camping-knots".to_owned(),
            localizations: vec![(
                Locale::En,
                "camping-knots".to_owned(),
                "Camping Knots".to_owned(),
            )],
        }],
        types: vec![KnotTypeSeed {
            id: "hitch-knots".to_owned(),
            localizations: vec![(Locale::En, "hitch-knots".to_owned(), "Hitches".to_owned())],
        }],
        media: Vec::new(),
        raw_metadata: serde_json::json!({"english_slug":"adjustable-grip-hitch-knot"}),
    }
}

async fn test_app() -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database = DatabaseConfig::new(format!("sqlite://{}?mode=rwc", db_path.display())).unwrap();
    let db = connect_database(&database).await.unwrap();
    migrate_database(&db).await.unwrap();
    KnotRepository::new(db.clone(), "/assets")
        .replace_all_knots("admin-upload-test", &[sample_knot()])
        .await
        .unwrap();

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
        content_dir: temp_dir.path().join("content"),
        content_assets_dir: temp_dir.path().join("assets"),
        media_base_url: "/assets".to_owned(),
        redis_cache: RedisCacheConfig::disabled(),
        upload: UploadConfig::default(),
        object_storage: ObjectStorageConfig {
            endpoint: "http://127.0.0.1:19000".to_owned(),
            region: "us-east-1".to_owned(),
            bucket: "test-feedback-uploads".to_owned(),
            access_key_id: "test-access".to_owned(),
            secret_access_key: "test-secret".to_owned(),
            force_path_style: true,
        },
        knots_media_storage: KnotsMediaStorageConfig {
            storage_profile: "knots-public".to_owned(),
            endpoint: "http://127.0.0.1:19000".to_owned(),
            region: "us-east-1".to_owned(),
            bucket: "stellartrail-knots-media".to_owned(),
            access_key_id: "test-access".to_owned(),
            secret_access_key: "test-secret".to_owned(),
            force_path_style: true,
            public_base_url: "https://media.example.test/stellartrail-knots-media".to_owned(),
            max_image_bytes: 8_000_000,
            max_video_bytes: 50_000_000,
        },
        admin: AdminConfig {
            user_ids: Vec::new(),
            emails: Vec::new(),
            usernames: vec!["admin_upload".to_owned()],
        },
        public_api: PublicApiConfig::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    let state = AppState::new_with_cache_and_object_store(
        config,
        db,
        Cache::with_store_for_tests(cache_store, "test-stellartrail", Duration::from_secs(300)),
        Arc::new(object_store.clone()),
    );
    TestApp {
        router: build_router(state),
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

async fn json_get(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn register_password_user(app: &Router, username: &str) -> String {
    let email = format!("{username}@example.test");
    let password = "OutdoorPass123!";
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
    register_value["access_token"].as_str().unwrap().to_owned()
}

async fn upload_knot_media(
    app: &Router,
    token: Option<&str>,
    asset_id: &str,
    media_type: &str,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) -> (StatusCode, Value) {
    let boundary = "stellartrail-admin-media-boundary";
    let mut body = Vec::new();
    for (name, value) in [
        ("media_type", media_type),
        ("attribution", "Knots3D"),
        ("license_note", "Use only after authorization is confirmed."),
        ("source_name", "knots3d"),
        ("source_path", "media/adjustable-grip-hitch/thumbnail.webp"),
    ] {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
                .as_bytes(),
        );
    }
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let mut builder = Request::builder()
        .method("PUT")
        .uri(format!(
            "/api/admin/skills/knots/adjustable-grip-hitch-knot/media/{asset_id}"
        ))
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
async fn admin_upload_requires_authenticated_allowlisted_user() {
    let app = test_app().await;
    let (missing_status, missing_body) = upload_knot_media(
        &app.router,
        None,
        "thumbnail",
        "thumbnail",
        "thumbnail.webp",
        "image/webp",
        WEBP_BYTES,
    )
    .await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED, "{missing_body}");

    let normal_token = register_password_user(&app.router, "normal_upload").await;
    let (normal_status, normal_body) = upload_knot_media(
        &app.router,
        Some(&normal_token),
        "thumbnail",
        "thumbnail",
        "thumbnail.webp",
        "image/webp",
        WEBP_BYTES,
    )
    .await;
    assert_eq!(normal_status, StatusCode::FORBIDDEN, "{normal_body}");
}

#[tokio::test]
async fn admin_can_upload_knot_media_to_object_store_db_and_public_read_api() {
    let app = test_app().await;
    let admin_token = register_password_user(&app.router, "admin_upload").await;

    let (status, body) = upload_knot_media(
        &app.router,
        Some(&admin_token),
        "thumbnail",
        "thumbnail",
        "thumbnail.webp",
        "image/webp",
        WEBP_BYTES,
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["status"], "uploaded");
    assert_eq!(body["media"]["id"], "thumbnail");
    assert_eq!(body["media"]["media_type"], "thumbnail");
    assert_eq!(body["media"]["mime_type"], "image/webp");
    assert_eq!(body["media"]["size_bytes"], WEBP_BYTES.len());
    let uploaded_url = body["media"]["url"].as_str().unwrap();
    assert!(uploaded_url.starts_with("https://media.example.test/stellartrail-knots-media/skills/knots/adjustable-grip-hitch-knot/thumbnail/"));
    assert!(uploaded_url.ends_with(".webp"));
    assert!(!body.to_string().contains("object_key"));
    assert_eq!(app.object_store.object_count(), 1);
    let stored = app.object_store.only_object().unwrap();
    assert!(
        stored
            .object_key
            .starts_with("skills/knots/adjustable-grip-hitch-knot/thumbnail/")
    );
    assert_eq!(stored.content_type, "image/webp");
    assert_eq!(stored.bytes, WEBP_BYTES);

    let (public_status, public_body) = json_get(
        &app.router,
        "/api/skills/knots/detail/adjustable-grip-hitch-knot",
    )
    .await;
    assert_eq!(public_status, StatusCode::OK, "{public_body}");
    assert_eq!(public_body["media"][0]["url"], uploaded_url);
    assert!(!public_body.to_string().contains("/assets/"));
}

#[tokio::test]
async fn admin_upload_rejects_mismatched_type_and_invalid_magic() {
    let app = test_app().await;
    let admin_token = register_password_user(&app.router, "admin_upload").await;

    let (mismatch_status, mismatch_body) = upload_knot_media(
        &app.router,
        Some(&admin_token),
        "thumbnail",
        "draw_mp4",
        "thumbnail.webp",
        "image/webp",
        WEBP_BYTES,
    )
    .await;
    assert_eq!(mismatch_status, StatusCode::BAD_REQUEST, "{mismatch_body}");

    let (magic_status, magic_body) = upload_knot_media(
        &app.router,
        Some(&admin_token),
        "draw_gif",
        "draw_gif",
        "draw.gif",
        "image/gif",
        NOT_GIF_BYTES,
    )
    .await;
    assert_eq!(
        magic_status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "{magic_body}"
    );
}
