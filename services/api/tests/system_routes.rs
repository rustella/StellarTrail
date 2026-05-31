use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::Value;
use stellartrail_api::{
    config::{ApiConfig, CorsConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{DatabaseConfig, connect_database};
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
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    TestApp {
        router: build_router(AppState::new(config, db)),
        _temp_dir: temp_dir,
    }
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
async fn system_routes_return_health_and_meta() {
    let app = test_app().await;

    let (health_status, health) = send_empty(&app.router, "GET", "/healthz", None).await;
    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(health["status"], "ok");

    let (meta_status, meta) = send_empty(&app.router, "GET", "/api/v1/meta", None).await;
    assert_eq!(meta_status, StatusCode::OK);
    assert_eq!(meta["name"], "StellarTrail");
    assert_eq!(meta["database_kind"], "sqlite");
}

#[tokio::test]
async fn old_api_prefix_is_not_registered() {
    let app = test_app().await;

    let (status, body) = send_empty(&app.router, "GET", "/api/meta", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}
