use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use std::time::Duration;
use stellartrail_api::{
    cache::{Cache, InMemoryCacheStore},
    config::{
        ApiConfig, CorsConfig, ObjectStorageConfig, PublicApiConfig, RedisCacheConfig, UploadConfig,
    },
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AuthRepository, KnotRepository, hash_token},
};
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
};
use tempfile::TempDir;
use time::{Duration as TimeDuration, OffsetDateTime};
use tower::ServiceExt;

struct TestApp {
    router: Router,
    db: sea_orm::DatabaseConnection,
    _temp_dir: TempDir,
}

fn sample_knot() -> KnotSeed {
    KnotSeed {
        id: "adjustable-grip-hitch-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/knots/en_us/adjustable-grip-hitch-knot".to_owned()),
        source_slug_en: "adjustable-grip-hitch-knot".to_owned(),
        source_slug_zh: Some("ke-tiao-jie-sheng-jie".to_owned()),
        localizations: vec![
            KnotLocalizationSeed {
                locale: Locale::En,
                slug: "adjustable-grip-hitch-knot".to_owned(),
                title: "Adjustable Grip Hitch".to_owned(),
                summary: "Adjust tension on a line.".to_owned(),
                aliases: vec![
                    "Adjustable Loop".to_owned(),
                    "Cawley Adjustable Hitch".to_owned(),
                ],
                description: Some("A practical hitch for tensioning guylines.".to_owned()),
                steps: vec!["Wrap the working end around the standing part.".to_owned()],
            },
            KnotLocalizationSeed {
                locale: Locale::ZhCn,
                slug: "ke-tiao-jie-sheng-jie".to_owned(),
                title: "可调节绳结".to_owned(),
                summary: "调节绳索上的张力。".to_owned(),
                aliases: vec!["可调节活结".to_owned(), "考利可调节套结".to_owned()],
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

fn technical_knot() -> KnotSeed {
    KnotSeed {
        id: "figure-eight-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/knots/en_us/figure-eight-knot".to_owned()),
        source_slug_en: "figure-eight-knot".to_owned(),
        source_slug_zh: Some("ba-zi-jie".to_owned()),
        localizations: vec![
            KnotLocalizationSeed {
                locale: Locale::En,
                slug: "figure-eight-knot".to_owned(),
                title: "Figure Eight Knot".to_owned(),
                summary: "Stopper knot for rope ends.".to_owned(),
                aliases: vec!["Flemish Knot".to_owned()],
                description: Some("A compact stopper for rope ends.".to_owned()),
                steps: vec!["Form an eight shape.".to_owned()],
            },
            KnotLocalizationSeed {
                locale: Locale::ZhCn,
                slug: "ba-zi-jie".to_owned(),
                title: "八字结".to_owned(),
                summary: "用于绳端防脱的基础结。".to_owned(),
                aliases: vec!["八字扣".to_owned()],
                description: Some("适合作为绳端止动。".to_owned()),
                steps: vec!["绕出八字形。".to_owned()],
            },
        ],
        categories: vec![KnotCategorySeed {
            id: "basic-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "basic-knots".to_owned(),
                    "Basic Knots".to_owned(),
                ),
                (
                    Locale::ZhCn,
                    "ji-chu-sheng-jie".to_owned(),
                    "基础绳结".to_owned(),
                ),
            ],
        }],
        types: vec![KnotTypeSeed {
            id: "stopper-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "stopper-knots".to_owned(),
                    "Stopper Knots".to_owned(),
                ),
                (Locale::ZhCn, "zhi-dong-jie".to_owned(), "止动结".to_owned()),
            ],
        }],
        media: vec![],
        raw_metadata: serde_json::json!({
            "english_slug": "figure-eight-knot",
            "zh_slug": "ba-zi-jie"
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
    seeded_app_with_uploaded_media_cache(Cache::disabled()).await
}

async fn seeded_app_with_uploaded_media_cache(cache: Cache) -> TestApp {
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
        object_storage: ObjectStorageConfig::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
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
        rate_limit: Default::default(),
        request_signature: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
    };
    let repository = KnotRepository::new(db.clone());
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
        router: build_router(AppState::new_with_cache(config, db.clone(), cache)),
        db,
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
        commit_hash: None,
        database,
        wechat_mock_login: true,
        wechat_app_id: None,
        wechat_app_secret: None,
        redis_cache: RedisCacheConfig::disabled(),
        upload: UploadConfig::default(),
        minio: Default::default(),
        object_storage: ObjectStorageConfig::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
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
        rate_limit: Default::default(),
        request_signature: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
    };
    let repository = KnotRepository::new(db.clone());
    repository
        .replace_all_knots("test-fixture", &[sample_knot()])
        .await
        .expect("seed knots");
    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
        _temp_dir: temp_dir,
    }
}

async fn seeded_filter_app() -> TestApp {
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
        object_storage: ObjectStorageConfig::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        public_api: PublicApiConfig {
            rate_limit_enabled: true,
            rate_limit_window_seconds: 60,
            rate_limit_max_requests_per_ip: 20,
            cache_ttl_seconds: 300,
            cache_stale_seconds: 600,
            max_list_limit: 100,
            max_search_query_chars: 64,
            max_offset: 10_000,
            trust_proxy_headers: false,
            trusted_proxy_cidrs: Vec::new(),
        },
        rate_limit: Default::default(),
        request_signature: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
    };
    let repository = KnotRepository::new(db.clone());
    repository
        .replace_all_knots("test-fixture", &[sample_knot(), technical_knot()])
        .await
        .expect("seed knots");
    TestApp {
        router: build_router(AppState::new(config, db.clone())),
        db,
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

async fn request_json(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let (status, _headers, body) = json_response(app, builder.body(Body::empty()).unwrap()).await;
    (status, body)
}

async fn request_json_body(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let (status, _headers, body) = json_response(
        app,
        builder
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap(),
    )
    .await;
    (status, body)
}

async fn create_test_session(app: &TestApp, username: &str) -> String {
    let repository = AuthRepository::new(app.db.clone());
    let user = repository
        .create_password_user(
            username,
            &format!("{username}@example.test"),
            "test-password-hash",
        )
        .await
        .expect("create user");
    let access_token = format!("access-token-{username}");
    let refresh_token = format!("refresh-token-{username}");
    repository
        .create_session(
            &user.id,
            &hash_token(&access_token),
            OffsetDateTime::now_utc() + TimeDuration::days(1),
            &hash_token(&refresh_token),
            OffsetDateTime::now_utc() + TimeDuration::days(30),
        )
        .await
        .expect("create session");
    access_token
}

#[tokio::test]
async fn knot_disclaimer_requires_authenticated_user() {
    let app = seeded_app().await;

    let (status, body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/knots/disclaimer",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn knot_disclaimer_can_be_accepted_and_read_back() {
    let app = seeded_app().await;
    let token = create_test_session(&app, "knot-disclaimer-user").await;

    let (initial_status, initial_body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/knots/disclaimer",
        Some(&token),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK, "{initial_body}");
    assert_eq!(initial_body["key"], "knot_tutorial_disclaimer");
    assert_eq!(initial_body["version"], "v1");
    assert_eq!(initial_body["accepted"], false);
    assert!(
        initial_body["content"]
            .as_str()
            .unwrap()
            .contains("仅用于一般绳结知识学习和非承重练习")
    );
    assert!(
        initial_body["content"]
            .as_str()
            .unwrap()
            .contains("法律另有规定或因本程序、开发者依法应承担责任的除外")
    );
    assert!(
        !initial_body["content"]
            .as_str()
            .unwrap()
            .contains("本程序及其开发者、运营者不承担责任")
    );

    let (accepted_status, accepted_body) = request_json_body(
        &app.router,
        "POST",
        "/api/v1/me/skills/knots/disclaimer/acceptance",
        Some(&token),
        json!({
            "client_platform": "wechat_miniprogram",
            "client_version": "1.0.0",
            "device_model": "iPhone 15"
        }),
    )
    .await;
    assert_eq!(accepted_status, StatusCode::OK, "{accepted_body}");
    assert_eq!(accepted_body["accepted"], true);
    assert!(accepted_body["accepted_at"].as_str().is_some());

    let (read_status, read_body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/knots/disclaimer",
        Some(&token),
    )
    .await;
    assert_eq!(read_status, StatusCode::OK, "{read_body}");
    assert_eq!(read_body["accepted"], true);
    assert_eq!(read_body["accepted_at"], accepted_body["accepted_at"]);
}

#[tokio::test]
async fn knot_disclaimer_acceptance_is_idempotent_and_refreshes_client_metadata() {
    let app = seeded_app().await;
    let token = create_test_session(&app, "knot-disclaimer-repeat").await;

    for version in ["1.0.0", "1.0.1"] {
        let (status, body) = request_json_body(
            &app.router,
            "POST",
            "/api/v1/me/skills/knots/disclaimer/acceptance",
            Some(&token),
            json!({
                "client_platform": "wechat_miniprogram",
                "client_version": version,
                "device_model": "iPhone 15"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["accepted"], true);
    }

    let row = app
        .db
        .query_one(Statement::from_string(
            app.db.get_database_backend(),
            "SELECT COUNT(*) AS count, MAX(client_version) AS client_version \
             FROM user_disclaimer_acceptances \
             WHERE disclaimer_key = 'knot_tutorial_disclaimer' AND version = 'v1'",
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.try_get::<i64>("", "count").unwrap(), 1);
    assert_eq!(
        row.try_get::<String>("", "client_version").unwrap(),
        "1.0.1"
    );
}

#[tokio::test]
async fn skills_returns_locale_resolved_category_without_parallel_language_fields() {
    let app = seeded_app().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "zh-CN");
    assert!(headers.get(header::CACHE_CONTROL).is_some());
    assert!(headers.get(header::ETAG).is_some());
    assert_eq!(body["items"][0]["id"], "knots");
    assert_eq!(body["items"][0]["title"], "绳结");
    assert_eq!(body["items"][0]["item_count"], 1);
    assert!(!body.to_string().contains("zh_slug"));
    assert!(!body.to_string().contains("english_slug"));
    assert!(!body.to_string().contains("title_en"));
}

#[tokio::test]
async fn knot_filters_return_locale_resolved_categories_with_counts() {
    let app = seeded_filter_app().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/filters")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "zh-CN");
    assert_eq!(body["locale"], "zh-CN");
    assert!(body["categories"].as_array().unwrap().iter().any(|option| {
        option["id"] == "camping-knots"
            && option["slug"] == "lu-ying-sheng-jie"
            && option["title"] == "露营绳结"
            && option["count"] == 1
    }));
    assert!(body.get("difficulties").is_none());
}

#[tokio::test]
async fn knots_list_combines_category_and_keyword_filters() {
    let app = seeded_filter_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=20&category=camping-knots&q=%E8%B0%83%E8%8A%82")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["id"], "adjustable-grip-hitch-knot");
    assert_eq!(body["items"][0]["aliases"][0], "可调节活结");
    assert_eq!(body["page"]["next_offset"], Value::Null);

    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=20&category=camping-knots&q=%E8%80%83%E5%88%A9")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["id"], "adjustable-grip-hitch-knot");

    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=20&category=camping-knots&q=Cawley")
            .header("X-StellarTrail-Locale", "en")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["aliases"][1], "Cawley Adjustable Hitch");

    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=20&category=camping-knots&q=%E5%85%AB%E5%AD%97")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["items"].as_array().unwrap().is_empty());
    assert_eq!(body["page"]["next_offset"], Value::Null);
}

#[tokio::test]
async fn knots_list_rejects_removed_difficulty_filter() {
    let app = seeded_filter_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?difficulty=technical")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "unsupported_query_parameter");
    assert_eq!(body["parameter"], "difficulty");
}

#[tokio::test]
async fn favorite_skill_routes_require_authentication() {
    let app = seeded_filter_app().await;

    for (method, uri) in [
        ("GET", "/api/v1/me/skills/favorites"),
        (
            "GET",
            "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        ),
        (
            "PUT",
            "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        ),
        (
            "DELETE",
            "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        ),
    ] {
        let (status, body) = request_json(&app.router, method, uri, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        assert_eq!(body["code"], "unauthorized", "{method} {uri}");
    }
}

#[tokio::test]
async fn favorite_skill_routes_soft_delete_and_restore_user_knot_favorites() {
    let app = seeded_filter_app().await;
    let user_a_token = create_test_session(&app, "trail-alice").await;
    let user_b_token = create_test_session(&app, "trail-bob").await;

    let (status, body) = request_json(
        &app.router,
        "PUT",
        "/api/v1/me/skills/favorites/knots/figure-eight-knot",
        Some(&user_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_favorited"], true);
    assert_eq!(body["skill_category"], "knots");

    let (status, body) = request_json(
        &app.router,
        "PUT",
        "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        Some(&user_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_favorited"], true);

    let (status, body) = request_json(
        &app.router,
        "DELETE",
        "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        Some(&user_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_favorited"], false);
    assert_eq!(body["favorited_at"], Value::Null);

    let row = app
        .db
        .query_one(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "SELECT is_deleted FROM user_knot_favorites WHERE knot_id = ?",
            vec!["adjustable-grip-hitch-knot".into()],
        ))
        .await
        .unwrap()
        .unwrap();
    assert!(row.try_get::<bool>("", "is_deleted").unwrap());

    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "UPDATE user_knot_favorites SET favorited_at = ? WHERE knot_id = ?",
            vec![
                "2000-01-01T00:00:00Z".into(),
                "adjustable-grip-hitch-knot".into(),
            ],
        ))
        .await
        .unwrap();
    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "UPDATE user_knot_favorites SET favorited_at = ? WHERE knot_id = ?",
            vec!["2001-01-01T00:00:00Z".into(), "figure-eight-knot".into()],
        ))
        .await
        .unwrap();

    let (status, body) = request_json(
        &app.router,
        "PUT",
        "/api/v1/me/skills/favorites/knots/adjustable-grip-hitch-knot",
        Some(&user_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_favorited"], true);

    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .method("GET")
            .uri("/api/v1/me/skills/favorites?skill_category=knots&offset=0&limit=10")
            .header(header::AUTHORIZATION, format!("Bearer {user_a_token}"))
            .header("X-StellarTrail-Locale", "en")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["locale"], "en");
    assert_eq!(body["filters"][0]["id"], "all");
    assert_eq!(body["filters"][0]["count"], 2);
    assert_eq!(body["filters"][1]["id"], "knots");
    assert_eq!(body["filters"][1]["count"], 2);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(body["items"][0]["knot"]["id"], "adjustable-grip-hitch-knot");
    assert_eq!(
        body["items"][0]["knot"]["aliases"][1],
        "Cawley Adjustable Hitch"
    );
    assert_eq!(body["items"][0]["skill_category"], "knots");
    assert!(body["items"][0]["favorited_at"].is_string());
    assert_eq!(body["page"]["next_offset"], Value::Null);

    let (status, body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/favorites",
        Some(&user_b_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn favorite_skill_routes_report_status_and_validate_knot_ids() {
    let app = seeded_filter_app().await;
    let token = create_test_session(&app, "trail-cora").await;

    let (status, body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/favorites/knots/figure-eight-knot",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_favorited"], false);
    assert_eq!(body["favorited_at"], Value::Null);

    for method in ["GET", "PUT", "DELETE"] {
        let (status, body) = request_json(
            &app.router,
            method,
            "/api/v1/me/skills/favorites/knots/missing-knot",
            Some(&token),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method}: {body}");
        assert_eq!(body["code"], "not_found");
    }

    let (status, body) = request_json(
        &app.router,
        "GET",
        "/api/v1/me/skills/favorites?skill_category=routes",
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_query_parameter");
    assert_eq!(body["parameter"], "skill_category");
}

#[tokio::test]
async fn public_skills_etag_supports_not_modified_responses() {
    let app = seeded_app().await;
    let (status, headers, _body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=1")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let etag = headers
        .get(header::ETAG)
        .expect("etag header")
        .to_str()
        .expect("etag value")
        .to_owned();
    assert!(headers.get(header::CACHE_CONTROL).is_some());

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills/knots/list?offset=0&limit=1")
                .header("X-StellarTrail-Locale", "zh-CN")
                .header(header::IF_NONE_MATCH, etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        response.headers().get(header::CONTENT_LANGUAGE).unwrap(),
        "zh-CN"
    );
}

#[tokio::test]
async fn public_knot_list_detail_and_filters_use_response_cache() {
    let cache_store = InMemoryCacheStore::default();
    let app = seeded_app_with_uploaded_media_cache(Cache::with_store_for_tests(
        cache_store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;

    for path in [
        "/api/v1/skills/knots/list?offset=0&limit=1",
        "/api/v1/skills/knots/detail/adjustable-grip-hitch-knot",
        "/api/v1/skills/knots/filters",
    ] {
        let (first_status, _first_headers, _first_body) = json_response(
            &app.router,
            Request::builder()
                .uri(path)
                .header("X-StellarTrail-Locale", "zh-CN")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(first_status, StatusCode::OK, "{path}");
        let after_first = cache_store.stats();
        let (second_status, _second_headers, _second_body) = json_response(
            &app.router,
            Request::builder()
                .uri(path)
                .header("X-StellarTrail-Locale", "zh-CN")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(second_status, StatusCode::OK, "{path}");
        let after_second = cache_store.stats();
        assert!(
            after_second.hit_count > after_first.hit_count,
            "{path} should hit public response cache: before={after_first:?} after={after_second:?}",
        );
    }
}

#[tokio::test]
async fn knots_list_uses_offset_pagination_and_hides_source_slugs() {
    let app = seeded_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=1")
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
    assert_eq!(body["items"][0]["aliases"][0], "Adjustable Loop");
    let serialized = body.to_string();
    assert!(!serialized.contains("cursor"));
    assert!(!serialized.contains("next_cursor"));
    assert!(!serialized.contains("zh_slug"));
    assert!(!serialized.contains("english_slug"));
    assert!(!serialized.contains("source_slug_zh"));
    assert!(!serialized.contains("source_slug_en"));
    assert!(!serialized.contains("is_favorited"));
}

#[tokio::test]
async fn knot_detail_returns_media_resource_urls_from_db_and_no_language_specific_slugs() {
    let app = seeded_app_with_uploaded_media().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/detail/adjustable-grip-hitch-knot")
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
    assert_eq!(body["aliases"][1], "考利可调节套结");
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
async fn knot_offline_manifest_returns_complete_payload_with_deduped_media_estimate() {
    let app = seeded_app_with_uploaded_media().await;
    let (status, headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/offline-manifest")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "zh-CN");
    assert!(headers.get(header::CACHE_CONTROL).is_some());
    assert!(headers.get(header::ETAG).is_some());
    assert_eq!(body["locale"], "zh-CN");
    assert_eq!(body["item_count"], 1);
    assert_eq!(body["media_count"], 2);
    assert_eq!(body["estimated_bytes"], 746348);
    assert_eq!(body["items"][0]["id"], "adjustable-grip-hitch-knot");
    assert_eq!(body["items"][0]["title"], "可调节绳结");
    assert_eq!(body["items"][0]["aliases"][0], "可调节活结");
    assert_eq!(body["items"][0]["steps"][0], "将绳头绕过主绳。");
    assert_eq!(
        body["items"][0]["media"][0]["url"],
        "https://minio-a.example.com/stellartrail-knots-media/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash-a.webp"
    );
    assert_eq!(
        body["items"][0]["media"][1]["url"],
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
            "offline manifest leaked {forbidden}: {serialized}"
        );
    }
}

#[tokio::test]
async fn knot_offline_manifest_uses_public_response_cache_and_etag() {
    let cache_store = InMemoryCacheStore::default();
    let app = seeded_app_with_uploaded_media_cache(Cache::with_store_for_tests(
        cache_store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;

    let (status, headers, _body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/offline-manifest")
            .header("X-StellarTrail-Locale", "zh-CN")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let etag = headers
        .get(header::ETAG)
        .expect("etag header")
        .to_str()
        .expect("etag value")
        .to_owned();
    let after_first = cache_store.stats();
    assert!(after_first.set_count >= 1);

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/skills/knots/offline-manifest")
                .header("X-StellarTrail-Locale", "zh-CN")
                .header(header::IF_NONE_MATCH, etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        response.headers().get(header::CONTENT_LANGUAGE).unwrap(),
        "zh-CN"
    );
    let after_second = cache_store.stats();
    assert!(after_second.hit_count > after_first.hit_count);
}

#[tokio::test]
async fn knots_list_uses_media_resource_urls_from_different_public_domains() {
    let app = seeded_app_with_uploaded_media().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/v1/skills/knots/list?offset=0&limit=1")
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
        "/api/v1/skills/knots/list?locale=en",
        "/api/v1/skills/knots/list?l%6fcale=en",
        "/api/v1/skills/knots/offline-manifest?locale=en",
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
        ("/api/v1/skills/knots/list?cursor=abc", "cursor"),
        ("/api/v1/skills/knots/list?next_cursor=abc", "next_cursor"),
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
async fn offline_manifest_rejects_filter_and_pagination_query_parameters() {
    for (uri, parameter) in [
        ("/api/v1/skills/knots/offline-manifest?offset=0", "offset"),
        ("/api/v1/skills/knots/offline-manifest?limit=10", "limit"),
        (
            "/api/v1/skills/knots/offline-manifest?category=camping-knots",
            "category",
        ),
        ("/api/v1/skills/knots/offline-manifest?q=grip", "q"),
        ("/api/v1/skills/knots/offline-manifest?cursor=abc", "cursor"),
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
                .uri("/api/v1/skills/knots/list")
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
            .uri("/api/v1/skills/knots/list?offset=0&limit=1")
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
        "/api/v1/skills/knots",
        "/api/v1/skills/knots/adjustable-grip-hitch-knot",
        "/api/v1/knots",
        "/api/v1/skills?category=knot",
        "/api/v1/skills/adjustable-grip-hitch-knot",
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
async fn legacy_assets_path_is_not_registered() {
    let app = seeded_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/assets/skills/knots/adjustable-grip-hitch-knot/adjustable-grip-hitch-demo.gif")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}
