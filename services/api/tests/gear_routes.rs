use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    cache::{Cache, InMemoryCacheStore},
    config::{ApiConfig, ObjectStorageConfig, RedisCacheConfig, UploadConfig},
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
    test_app_with_cache(Cache::disabled()).await
}

async fn test_app_with_cache(cache: Cache) -> TestApp {
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
    };
    TestApp {
        router: build_router(AppState::new_with_cache(config, db, cache)),
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

async fn login(app: &Router, code: &str) -> String {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/auth/wechat-login",
        None,
        json!({"code": code, "profile": {"nickname": "测试用户", "avatar_url": null}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["access_token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn gear_stats_reads_are_cached_and_mutations_invalidate_cache_version() {
    let store = InMemoryCacheStore::default();
    let app = test_app_with_cache(Cache::with_store_for_tests(
        store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;
    let token = login(&app.router, "gear-cache-user").await;

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({"category": "lighting_system", "name": "缓存头灯"}),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");

    let (first_status, first_stats) =
        send_empty(&app.router, "GET", "/api/me/gears/stats", Some(&token)).await;
    assert_eq!(first_status, StatusCode::OK, "{first_stats}");
    assert_eq!(first_stats["current_count"], 1);
    let after_first_read = store.stats();
    assert!(
        after_first_read.set_count >= 1,
        "first read should populate Redis-compatible cache: {after_first_read:?}",
    );

    let (second_status, second_stats) =
        send_empty(&app.router, "GET", "/api/me/gears/stats", Some(&token)).await;
    assert_eq!(second_status, StatusCode::OK, "{second_stats}");
    assert_eq!(second_stats["current_count"], 1);
    let after_second_read = store.stats();
    assert!(
        after_second_read.hit_count > after_first_read.hit_count,
        "second read should hit Redis-compatible cache: before={after_first_read:?} after={after_second_read:?}",
    );

    let (second_create_status, second_created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({"category": "lighting_system", "name": "缓存营灯"}),
    )
    .await;
    assert_eq!(
        second_create_status,
        StatusCode::CREATED,
        "{second_created}",
    );
    let after_mutation = store.stats();
    assert!(
        after_mutation.increment_count > after_second_read.increment_count,
        "gear mutation should bump cache version instead of serving stale read cache: before={after_second_read:?} after={after_mutation:?}",
    );

    let (fresh_status, fresh_stats) =
        send_empty(&app.router, "GET", "/api/me/gears/stats", Some(&token)).await;
    assert_eq!(fresh_status, StatusCode::OK, "{fresh_stats}");
    assert_eq!(fresh_stats["current_count"], 2);
}

#[tokio::test]
async fn gear_inventory_full_flow_matches_phase_one_requirements() {
    let app = test_app().await;
    let token = login(&app.router, "gear-flow-user").await;

    let create_body = json!({
        "category": "electronics_system",
        "name": " NITECORE奈特科尔SUMMIT 20000超薄充电宝 ",
        "brand": "NITECORE奈特科尔",
        "model": "SUMMIT 20000",
        "capacity": "20000mAh",
        "description": "冬季徒步备用电源",
        "weight_g": 315,
        "purchase_date": "2026-01-22",
        "purchase_price_cents": 63900,
        "purchase_location": "京东",
        "status": "available",
        "storage_location": "装备柜 A1",
        "tags": ["冬季", "电子", "电子"],
        "share_enabled": true,
        "notes": "充满电后入库"
    });
    let (status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        create_body,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let gear_id = created["id"].as_str().unwrap();
    assert_eq!(created["name"], "NITECORE奈特科尔SUMMIT 20000超薄充电宝");
    assert_eq!(created["tags"], json!(["冬季", "电子"]));
    assert_eq!(created["share_status"], "pending");

    let (stats_status, stats) =
        send_empty(&app.router, "GET", "/api/me/gears/stats", Some(&token)).await;
    assert_eq!(stats_status, StatusCode::OK, "{stats}");
    assert_eq!(stats["current_count"], 1);
    assert_eq!(stats["total_value_cents"], 63900);
    assert_eq!(stats["total_weight_g"], 315);

    let (category_status, categories) =
        send_empty(&app.router, "GET", "/api/me/gears/categories", Some(&token)).await;
    assert_eq!(category_status, StatusCode::OK, "{categories}");
    assert_eq!(categories["items"][0]["id"], "all");
    assert_eq!(categories["items"][0]["count"], 1);

    let (list_status, list) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears?category=electronics_system&q=nitecore&sort=created_at_desc",
        Some(&token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["category_label"], "电子系统");
    assert_eq!(list["items"][0]["status_label"], "可用");

    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/me/gears/{gear_id}"),
        Some(&token),
        json!({"status": "maintenance", "storage_location": "维修箱"}),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert_eq!(updated["status"], "maintenance");
    assert_eq!(updated["storage_location"], "维修箱");

    let (delete_status, delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/me/gears/{gear_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (available_status, available) =
        send_empty(&app.router, "GET", "/api/me/gears", Some(&token)).await;
    assert_eq!(available_status, StatusCode::OK, "{available}");
    assert_eq!(available["items"].as_array().unwrap().len(), 0);

    let (history_status, history) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears?tab=history",
        Some(&token),
    )
    .await;
    assert_eq!(history_status, StatusCode::OK, "{history}");
    assert_eq!(history["items"].as_array().unwrap().len(), 1);

    let (restore_status, restored) = send_empty(
        &app.router,
        "POST",
        &format!("/api/me/gears/{gear_id}/restore"),
        Some(&token),
    )
    .await;
    assert_eq!(restore_status, StatusCode::OK, "{restored}");
    assert!(restored["archived_at"].is_null());
}

#[tokio::test]
async fn gear_import_dry_run_and_export_csv_are_supported() {
    let app = test_app().await;
    let token = login(&app.router, "import-export-user").await;

    let (dry_status, dry) = send_json(
        &app.router,
        "POST",
        "/api/me/gears/import",
        Some(&token),
        json!({
            "dry_run": true,
            "items": [{"category": "lighting_system", "name": "头灯", "status": "available"}]
        }),
    )
    .await;
    assert_eq!(dry_status, StatusCode::OK, "{dry}");
    assert_eq!(dry["created_count"], 0);

    let (import_status, imported) = send_json(
        &app.router,
        "POST",
        "/api/me/gears/import",
        Some(&token),
        json!({
            "items": [{"category": "lighting_system", "name": "头灯", "status": "available"}]
        }),
    )
    .await;
    assert_eq!(import_status, StatusCode::OK, "{imported}");
    assert_eq!(imported["created_count"], 1);

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/me/gears/export?format=csv")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/csv; charset=utf-8"
    );
    let csv = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(csv.contains("category,name,brand,model"));
    assert!(csv.contains("lighting_system,头灯"));
}

#[tokio::test]
async fn users_cannot_read_each_others_gear() {
    let app = test_app().await;
    let token_a = login(&app.router, "user-a").await;
    let token_b = login(&app.router, "user-b").await;
    let (status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token_a),
        json!({"category": "lighting_system", "name": "A 的头灯"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let gear_id = created["id"].as_str().unwrap();

    let (read_status, value) = send_empty(
        &app.router,
        "GET",
        &format!("/api/me/gears/{gear_id}"),
        Some(&token_b),
    )
    .await;

    assert_eq!(read_status, StatusCode::NOT_FOUND, "{value}");
}
