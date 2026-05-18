use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
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
        admin: Default::default(),
        public_api: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    let db = connect_database(&config.database).await.unwrap();
    migrate_database(&db).await.unwrap();
    GearTemplateRepository::new(db.clone())
        .replace_system_templates("content-route-test", &default_system_gear_templates())
        .await
        .unwrap();
    let state = AppState::new(config, db);
    TestApp {
        router: build_router(state),
        _temp_dir: temp_dir,
    }
}

async fn get_json(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
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
async fn gear_template_routes_return_db_seed_without_content_directory() {
    let app = test_app().await;

    let (status, templates) = get_json(&app.router, "/api/gear-templates").await;
    assert_eq!(status, StatusCode::OK, "{templates}");
    assert_eq!(templates["items"].as_array().unwrap().len(), 1);
    assert_eq!(templates["items"][0]["id"], "backpacking-basic");
    assert_eq!(
        templates["items"][0]["categories"][0]["items"][0],
        "雨衣或硬壳"
    );

    let (status, template) = get_json(&app.router, "/api/gear-templates/backpacking-basic").await;
    assert_eq!(status, StatusCode::OK, "{template}");
    assert_eq!(template["title"], "入门徒步基础装备模板");
    assert_eq!(template["categories"][0]["id"], "rain_protection");
}

#[tokio::test]
async fn removed_content_routes_are_not_registered() {
    let app = test_app().await;

    for uri in [
        "/api/mountains",
        "/api/mountains/wugongshan",
        "/api/routes",
        "/api/routes/wugongshan-classic-2d1n",
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

    let (status, body) = get_json(&app.router, "/api/gear-templates/missing-template").await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["code"], "not_found");
}
