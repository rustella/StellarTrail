use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::Value;
use stellartrail_api::{
    config::{
        ApiConfig, CorsConfig, ObjectStorageConfig, PublicApiConfig, RedisCacheConfig, UploadConfig,
    },
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
};
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
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
        localizations: vec![
            KnotLocalizationSeed {
                locale: Locale::En,
                slug: "adjustable-grip-hitch-knot".to_owned(),
                title: "Adjustable Grip Hitch".to_owned(),
                summary: "Adjust tension on a line.".to_owned(),
                description: Some("A practical hitch for tensioning guylines.".to_owned()),
                steps: vec!["Wrap the working end around the standing part.".to_owned()],
            },
            KnotLocalizationSeed {
                locale: Locale::ZhCn,
                slug: "ke-tiao-jie-sheng-jie".to_owned(),
                title: "可调节绳结".to_owned(),
                summary: "调节绳索上的张力。".to_owned(),
                description: Some("适合风绳和营绳张力调节。".to_owned()),
                steps: vec!["将绳头绕过主绳。".to_owned()],
            },
        ],
        categories: vec![KnotCategorySeed {
            id: "camping-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "camping-knots".to_owned(),
                    "Camping Knots".to_owned(),
                ),
                (
                    Locale::ZhCn,
                    "lu-ying-sheng-jie".to_owned(),
                    "露营绳结".to_owned(),
                ),
            ],
        }],
        types: vec![KnotTypeSeed {
            id: "hitch-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "hitch-knots".to_owned(),
                    "Hitch Knots".to_owned(),
                ),
                (Locale::ZhCn, "jie-sheng".to_owned(), "系结".to_owned()),
            ],
        }],
        media: vec![KnotMediaAssetSeed {
            id: "adjustable-grip-hitch-demo".to_owned(),
            media_type: "gif".to_owned(),
            path: "skills/knots/adjustable-grip-hitch-knot/adjustable-grip-hitch-demo.gif"
                .to_owned(),
            mime_type: "image/gif".to_owned(),
            width: Some(640),
            height: Some(360),
            attribution: Some("Knots 3D".to_owned()),
            license_note: Some("Use only after authorization is confirmed.".to_owned()),
        }],
        raw_metadata: serde_json::json!({
            "english_slug": "adjustable-grip-hitch-knot",
            "zh_slug": "ke-tiao-jie-sheng-jie"
        }),
    }
}

struct UploadedKnotMediaFixture<'a> {
    knot_id: &'a str,
    asset_id: &'a str,
    media_type: &'a str,
    public_url: &'a str,
    mime_type: &'a str,
    size_bytes: i64,
    sha256_hex: &'a str,
}

async fn insert_uploaded_knot_media(
    db: &sea_orm::DatabaseConnection,
    media: UploadedKnotMediaFixture<'_>,
) {
    let media_resource_id = format!("media-{}-{}", media.knot_id, media.asset_id);
    let object_key = format!(
        "skills/knots/{}/{}/{}.bin",
        media.knot_id, media.asset_id, media.sha256_hex
    );
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO media_resources (
            id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
            mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
            status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
        ) VALUES (?, 'minio', 'knots-public', 'stellartrail-knots-media', ?, 'https://unused.example.com', ?, ?, 'bin', ?, ?, NULL, NULL, NULL, NULL, 'active', 'test', 'relative/source.bin', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![
            media_resource_id.clone().into(),
            object_key.into(),
            media.public_url.to_owned().into(),
            media.mime_type.to_owned().into(),
            media.size_bytes.into(),
            media.sha256_hex.to_owned().into(),
        ],
    ))
    .await
    .expect("insert media resource");
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO knot_media_resources (
            knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, 0, 'Knots3D', 'Use only after authorization is confirmed.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![
            media.knot_id.to_owned().into(),
            media.asset_id.to_owned().into(),
            media.media_type.to_owned().into(),
            media_resource_id.into(),
        ],
    ))
    .await
    .expect("insert knot media resource");
}

async fn seeded_app_with_uploaded_media() -> TestApp {
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
        upload: UploadConfig::default(),
        object_storage: ObjectStorageConfig::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: PublicApiConfig {
            rate_limit_enabled: true,
            rate_limit_window_seconds: 60,
            rate_limit_max_requests_per_ip: 2,
            cache_ttl_seconds: 300,
            cache_stale_seconds: 600,
            max_list_limit: 100,
            max_search_query_chars: 64,
            max_offset: 10_000,
            trust_proxy_headers: false,
            trusted_proxy_cidrs: Vec::new(),
        },
        cors: CorsConfig::default(),
    };
    let repository = KnotRepository::new(db.clone(), config.media_base_url.clone());
    repository
        .replace_all_knots("test-fixture", &[sample_knot()])
        .await
        .expect("seed knots");
    insert_uploaded_knot_media(
        &db,
        UploadedKnotMediaFixture {
            knot_id: "adjustable-grip-hitch-knot",
            asset_id: "thumbnail",
            media_type: "thumbnail",
            public_url: "https://minio-a.example.com/stellartrail-knots-media/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash-a.webp",
            mime_type: "image/webp",
            size_bytes: 12345,
            sha256_hex: "hash-a",
        },
    )
    .await;
    insert_uploaded_knot_media(
        &db,
        UploadedKnotMediaFixture {
            knot_id: "adjustable-grip-hitch-knot",
            asset_id: "draw_mp4",
            media_type: "draw_mp4",
            public_url: "https://cdn-b.example.com/knots/skills/knots/adjustable-grip-hitch-knot/draw_mp4/hash-b.mp4",
            mime_type: "video/mp4",
            size_bytes: 734003,
            sha256_hex: "hash-b",
        },
    )
    .await;
    TestApp {
        router: build_router(AppState::new(config, db)),
        _temp_dir: temp_dir,
    }
}

async fn seeded_app() -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let asset_path = temp_dir
        .path()
        .join("assets/skills/knots/adjustable-grip-hitch-knot");
    std::fs::create_dir_all(&asset_path).expect("asset dir");
    std::fs::write(asset_path.join("adjustable-grip-hitch-demo.gif"), b"GIF89a")
        .expect("asset file");

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
        upload: UploadConfig::default(),
        object_storage: ObjectStorageConfig::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: PublicApiConfig {
            rate_limit_enabled: true,
            rate_limit_window_seconds: 60,
            rate_limit_max_requests_per_ip: 2,
            cache_ttl_seconds: 300,
            cache_stale_seconds: 600,
            max_list_limit: 100,
            max_search_query_chars: 64,
            max_offset: 10_000,
            trust_proxy_headers: false,
            trusted_proxy_cidrs: Vec::new(),
        },
        cors: CorsConfig::default(),
    };
    let repository = KnotRepository::new(db.clone(), config.media_base_url.clone());
    repository
        .replace_all_knots("test-fixture", &[sample_knot()])
        .await
        .expect("seed knots");
    TestApp {
        router: build_router(AppState::new(config, db)),
        _temp_dir: temp_dir,
    }
}

async fn json_response(
    app: &Router,
    request: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let value = serde_json::from_slice::<Value>(&body).expect("json body");
    (status, headers, value)
}

#[tokio::test]
async fn skills_returns_locale_resolved_category_without_parallel_language_fields() {
    let app = seeded_app().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "zh-CN");
    assert_eq!(body["items"][0]["id"], "knots");
    assert_eq!(body["items"][0]["title"], "绳结");
    assert_eq!(body["items"][0]["item_count"], 1);
    assert!(!body.to_string().contains("zh_slug"));
    assert!(!body.to_string().contains("english_slug"));
    assert!(!body.to_string().contains("title_en"));
}

#[tokio::test]
async fn knots_list_uses_offset_pagination_and_hides_source_slugs() {
    let app = seeded_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills/knots/list?offset=0&limit=1")
            .header("X-StellarTrail-Locale", "en")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["page"]["offset"], 0);
    assert_eq!(body["page"]["next_offset"], Value::Null);
    assert_eq!(body["items"][0]["id"], "adjustable-grip-hitch-knot");
    assert_eq!(body["items"][0]["title"], "Adjustable Grip Hitch");
    let serialized = body.to_string();
    assert!(!serialized.contains("cursor"));
    assert!(!serialized.contains("next_cursor"));
    assert!(!serialized.contains("zh_slug"));
    assert!(!serialized.contains("english_slug"));
    assert!(!serialized.contains("source_slug_zh"));
    assert!(!serialized.contains("source_slug_en"));
}

#[tokio::test]
async fn knot_detail_returns_media_resource_urls_from_db_and_no_language_specific_slugs() {
    let app = seeded_app_with_uploaded_media().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills/knots/detail/adjustable-grip-hitch-knot")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "zh-CN");
    assert_eq!(body["id"], "adjustable-grip-hitch-knot");
    assert_eq!(body["title"], "可调节绳结");
    assert_eq!(body["slug"], "ke-tiao-jie-sheng-jie");
    assert_eq!(
        body["media"][0]["url"],
        "https://minio-a.example.com/stellartrail-knots-media/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash-a.webp"
    );
    assert_eq!(body["media"][0]["size_bytes"], 12345);
    assert_eq!(
        body["media"][1]["url"],
        "https://cdn-b.example.com/knots/skills/knots/adjustable-grip-hitch-knot/draw_mp4/hash-b.mp4"
    );
    let serialized = body.to_string();
    for forbidden in [
        "zh_slug",
        "english_slug",
        "source_slug_zh",
        "source_slug_en",
        "/assets/",
        "content/assets",
        ".hermes/local",
        "bucket",
        "object_key",
        "storage_profile",
        "storage_endpoint",
        "source_path",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "public response leaked {forbidden}: {serialized}"
        );
    }
}

#[tokio::test]
async fn knots_list_uses_media_resource_urls_from_different_public_domains() {
    let app = seeded_app_with_uploaded_media().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills/knots/list?offset=0&limit=1")
            .header("X-StellarTrail-Locale", "en")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["items"][0]["media"][0]["url"],
        "https://minio-a.example.com/stellartrail-knots-media/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash-a.webp"
    );
    assert_eq!(
        body["items"][0]["media"][1]["url"],
        "https://cdn-b.example.com/knots/skills/knots/adjustable-grip-hitch-knot/draw_mp4/hash-b.mp4"
    );
    assert!(!body.to_string().contains("/assets/"));
}

#[tokio::test]
async fn locale_query_parameter_is_rejected_globally() {
    for uri in [
        "/api/skills/knots/list?locale=en",
        "/api/skills/knots/list?l%6fcale=en",
    ] {
        let app = seeded_app().await;
        let (status, _headers, body) = json_response(
            &app.router,
            Request::builder().uri(uri).body(Body::empty()).unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(body["code"], "unsupported_query_parameter");
        assert_eq!(body["parameter"], "locale");
    }
}

#[tokio::test]
async fn cursor_and_next_cursor_are_rejected() {
    for (uri, parameter) in [
        ("/api/skills/knots/list?cursor=abc", "cursor"),
        ("/api/skills/knots/list?next_cursor=abc", "next_cursor"),
    ] {
        let app = seeded_app().await;
        let (status, _headers, body) = json_response(
            &app.router,
            Request::builder().uri(uri).body(Body::empty()).unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(body["code"], "unsupported_query_parameter");
        assert_eq!(body["parameter"], parameter);
    }
}

#[tokio::test]
async fn invalid_locale_header_is_rejected() {
    for raw in ["de-DE", "zh%2DCN"] {
        let app = seeded_app().await;
        let (status, _headers, body) = json_response(
            &app.router,
            Request::builder()
                .uri("/api/skills/knots/list")
                .header("X-StellarTrail-Locale", raw)
                .body(Body::empty())
                .unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{raw}");
        assert_eq!(body["code"], "invalid_header");
        assert_eq!(body["parameter"], "X-StellarTrail-Locale");
    }
}

#[tokio::test]
async fn accept_language_header_is_header_fallback_after_stellartrail_locale() {
    let app = seeded_app().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills/knots/list?offset=0&limit=1")
            .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.8")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_LANGUAGE], "en");
    assert_eq!(body["locale"], "en");
    assert_eq!(body["items"][0]["title"], "Adjustable Grip Hitch");
}

#[tokio::test]
async fn old_knots_routes_are_not_kept_as_compatibility_aliases() {
    for uri in [
        "/api/skills/knots",
        "/api/skills/knots/adjustable-grip-hitch-knot",
        "/api/knots",
        "/api/skills?category=knot",
        "/api/skills/adjustable-grip-hitch-knot",
    ] {
        let app = seeded_app().await;
        let (status, _headers, body) = json_response(
            &app.router,
            Request::builder().uri(uri).body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body["code"], "not_found", "{uri}");
    }
}

#[tokio::test]
async fn assets_are_served_from_content_assets_only() {
    let app = seeded_app().await;
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/assets/skills/knots/adjustable-grip-hitch-knot/adjustable-grip-hitch-demo.gif")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "image/gif"
    );
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert_eq!(&body[..6], b"GIF89a");
}
