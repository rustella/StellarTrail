use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode, header},
};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::Value;
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database, repositories::GearTemplateRepository};
use stellartrail_domain::gear_template::default_system_gear_templates;
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
        trail: Default::default(),
        map: Default::default(),
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
    let db = connect_database(&config.database).await.unwrap();
    migrate_database(&db).await.unwrap();
    GearTemplateRepository::new(db.clone())
        .replace_system_templates("content-route-test", &default_system_gear_templates())
        .await
        .unwrap();
    let state = AppState::new(config, db.clone());
    TestApp {
        router: build_router(state),
        db,
        _temp_dir: temp_dir,
    }
}

async fn get_json(app: &Router, path: &str) -> (StatusCode, Value) {
    let (status, _, value) = get_json_with_headers(app, path, &[]).await;
    (status, value)
}

async fn get_json_with_headers(
    app: &Router,
    path: &str,
    headers: &[(&str, &str)],
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .header("X-StellarTrail-Client", "web/test")
        .uri(path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, value)
}

#[tokio::test]
async fn content_page_route_returns_seeded_profile_about_copy() {
    let app = test_app().await;
    let seeded_row = app
        .db
        .query_one(Statement::from_string(
            app.db.get_database_backend(),
            "SELECT content_json FROM app_content_pages WHERE page_key = 'profile_about' AND client_key = 'wechat_miniprogram' AND locale = 'zh-CN'",
        ))
        .await
        .unwrap();
    assert!(
        seeded_row.is_some(),
        "profile About content seed is missing"
    );

    let (status, body) = get_json(
        &app.router,
        "/api/v1/content-pages/profile_about?client_key=wechat_miniprogram&locale=zh-CN",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["page_key"], "profile_about");
    assert_eq!(body["client_key"], "wechat_miniprogram");
    assert_eq!(body["locale"], "zh-CN");
    assert_eq!(body["eyebrow"], "🏕️ 寻径星野");
    assert_eq!(body["title"], "关于寻径星野");
    assert_eq!(body["subtitle"], "把每次出发前的准备，整理得更安心。");
    assert_eq!(body["sections"].as_array().unwrap().len(), 3);
    assert_eq!(body["sections"][0]["title"], "出发准备");
    assert_eq!(body["sections"][0]["icon"], "🧭");
    assert_eq!(
        body["sections"][2]["body"],
        "这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。希望它能陪你把每次出发前的准备做得更清楚、更安心。"
    );
    assert!(
        !body["sections"][2]["body"]
            .as_str()
            .unwrap()
            .contains("广告")
    );
    assert!(
        !body["sections"][2]["body"]
            .as_str()
            .unwrap()
            .contains("商业化")
    );
    assert_eq!(body["button_text"], "知道了");
    assert_eq!(body["updated_at"], "2026-06-07T00:00:00Z");
}

#[tokio::test]
async fn content_page_route_rejects_unknown_or_invalid_selectors() {
    let app = test_app().await;

    let (missing_status, missing_body) =
        get_json(&app.router, "/api/v1/content-pages/missing_page").await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND, "{missing_body}");
    assert_eq!(missing_body["code"], "not_found");

    let (client_status, client_body) = get_json(
        &app.router,
        "/api/v1/content-pages/profile_about?client_key=unknown",
    )
    .await;
    assert_eq!(
        client_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{client_body}"
    );
    assert_eq!(client_body["code"], "validation_failed");
    assert_eq!(client_body["fields"][0]["field"], "client_key");

    let (locale_status, locale_body) = get_json(
        &app.router,
        "/api/v1/content-pages/profile_about?locale=en-US",
    )
    .await;
    assert_eq!(
        locale_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{locale_body}"
    );
    assert_eq!(locale_body["code"], "validation_failed");
    assert_eq!(locale_body["fields"][0]["field"], "locale");
}

#[tokio::test]
async fn gear_template_routes_return_db_seed_without_content_directory() {
    let app = test_app().await;

    let (status, templates) = get_json(&app.router, "/api/v1/gear-templates").await;
    assert_eq!(status, StatusCode::OK, "{templates}");
    assert_eq!(templates["items"].as_array().unwrap().len(), 1);
    assert_eq!(templates["items"][0]["id"], "backpacking-basic");
    assert_eq!(
        templates["items"][0]["categories"][0]["items"][0],
        "雨衣或硬壳"
    );

    let (status, template) =
        get_json(&app.router, "/api/v1/gear-templates/backpacking-basic").await;
    assert_eq!(status, StatusCode::OK, "{template}");
    assert_eq!(template["title"], "入门徒步基础装备模板");
    assert_eq!(template["categories"][0]["id"], "rain_protection");
}

#[tokio::test]
async fn gear_template_routes_resolve_english_locale_headers() {
    let app = test_app().await;

    let (status, headers, templates) = get_json_with_headers(
        &app.router,
        "/api/v1/gear-templates",
        &[("X-StellarTrail-Locale", "en")],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{templates}");
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "en");
    assert_eq!(
        templates["items"][0]["title"],
        "Beginner Backpacking Essentials Template"
    );
    assert_eq!(
        templates["items"][0]["categories"][0]["items"][0],
        "Rain shell or hardshell"
    );

    let (status, headers, detail) = get_json_with_headers(
        &app.router,
        "/api/v1/gear-templates/backpacking-basic",
        &[("Accept-Language", "en-US,en;q=0.8")],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(headers.get(header::CONTENT_LANGUAGE).unwrap(), "en");
    assert_eq!(detail["categories"][1]["name"], "Lighting");

    let (status, body) = get_json(&app.router, "/api/v1/gear-templates?locale=en").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "unsupported_query_parameter");
    assert_eq!(body["parameter"], "locale");

    let (status, body) = get_json(
        &app.router,
        "/api/v1/gear-templates/backpacking-basic?locale=en",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "unsupported_query_parameter");
    assert_eq!(body["parameter"], "locale");
}

#[tokio::test]
async fn removed_content_routes_are_not_registered() {
    let app = test_app().await;

    for uri in [
        "/api/v1/mountains",
        "/api/v1/mountains/wugongshan",
        "/api/v1/routes",
        "/api/v1/routes/wugongshan-classic-2d1n",
        "/assets/skills/knots/adjustable-grip-hitch-knot/demo.gif",
    ] {
        let (status, body) = get_json(&app.router, uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}: {body}");
        assert_eq!(body["code"], "not_found", "{uri}: {body}");
    }
}

#[tokio::test]
async fn gear_template_detail_returns_not_found_for_unknown_id() {
    let app = test_app().await;

    let (status, body) = get_json(&app.router, "/api/v1/gear-templates/missing-template").await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["code"], "not_found");
}
