use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::Value;
use stellartrail_api::{
    config::{ApiConfig, ObjectStorageConfig, PublicApiConfig, RedisCacheConfig, UploadConfig},
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
async fn knot_detail_returns_resolved_locale_media_urls_and_no_language_specific_slugs() {
    let app = seeded_app().await;
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
        "/assets/skills/knots/adjustable-grip-hitch-knot/adjustable-grip-hitch-demo.gif"
    );
    let serialized = body.to_string();
    assert!(!serialized.contains("zh_slug"));
    assert!(!serialized.contains("english_slug"));
    assert!(!serialized.contains("source_slug_zh"));
    assert!(!serialized.contains("source_slug_en"));
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

    let app = seeded_app().await;
    let (status, _headers, body) = json_response(
        &app.router,
        Request::builder()
            .uri("/api/skills?category=knot")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "unsupported_query_parameter");
    assert_eq!(body["parameter"], "category");
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

#[tokio::test]
async fn public_knots_are_available_without_authorization_and_send_cache_headers() {
    let app = seeded_app().await;
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/skills/knots/list?offset=0&limit=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(header::ETAG));
    assert!(response.headers().contains_key(header::CACHE_CONTROL));
    assert_eq!(
        response.headers().get("X-Content-Type-Options").unwrap(),
        "nosniff"
    );
}

#[tokio::test]
async fn public_knots_support_conditional_get_with_etag() {
    let app = seeded_app().await;
    let first = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/skills/knots/detail/adjustable-grip-hitch-knot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("first response");
    assert_eq!(first.status(), StatusCode::OK);
    let etag = first
        .headers()
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let second = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/skills/knots/detail/adjustable-grip-hitch-knot")
                .header(header::IF_NONE_MATCH, etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("second response");
    assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn public_knots_rate_limit_before_expensive_reads() {
    let app = seeded_app().await;
    for attempt in 1..=2 {
        let response = app
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/skills/knots/list")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK, "attempt {attempt}");
    }
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/skills/knots/list")
                .header("X-Forwarded-For", "203.0.113.99")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("limited response");
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().contains_key(header::RETRY_AFTER));
}
