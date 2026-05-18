use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    cache::{Cache, InMemoryCacheStore},
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
        redis_cache: RedisCacheConfig::disabled(),
        upload: Default::default(),
        minio: Default::default(),
        object_storage: Default::default(),
        avatar_storage: Default::default(),
        knots_media_storage: Default::default(),
        admin: Default::default(),
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
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
    let value = login_response(app, code).await;
    value["access_token"].as_str().unwrap().to_owned()
}

async fn login_response(app: &Router, code: &str) -> Value {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/auth/wechat-login",
        None,
        json!({"code": code, "profile": {"nickname": "测试用户", "avatar_url": null}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
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
async fn spec_key_rankings_track_keys_in_redis_without_values_and_scope_by_user() {
    let store = InMemoryCacheStore::default();
    let app = test_app_with_cache(Cache::with_store_for_tests(
        store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;
    let login_body = login_response(&app.router, "spec-rank-user").await;
    let token = login_body["access_token"].as_str().unwrap().to_owned();
    let user_id = login_body["user"]["id"].as_str().unwrap();

    let (initial_status, initial) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/spec-key-rankings?category=electronics_system",
        Some(&token),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK, "{initial}");
    assert_eq!(initial, json!({"keys": []}));

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "充电宝",
            "specs": {
                "battery_capacity": "20000 mAh",
                "output_power": "65 W",
                "ports": "",
                "material": "铝合金"
            }
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    let gear_id = created["id"].as_str().unwrap();

    let (second_create_status, second_created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "备用电源",
            "specs": {
                "battery_capacity": "10000 mAh"
            }
        }),
    )
    .await;
    assert_eq!(
        second_create_status,
        StatusCode::CREATED,
        "{second_created}",
    );

    let (ranking_status, ranking) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/spec-key-rankings?category=electronics_system",
        Some(&token),
    )
    .await;
    assert_eq!(ranking_status, StatusCode::OK, "{ranking}");
    assert_eq!(ranking["keys"][0], "battery_capacity");
    assert!(
        ranking["keys"]
            .as_array()
            .unwrap()
            .contains(&json!("output_power"))
    );
    assert!(
        !ranking["keys"]
            .as_array()
            .unwrap()
            .contains(&json!("material")),
        "legacy specs are not part of the current category ranking response: {ranking}",
    );

    let key = format!("test-stellartrail:gear:{user_id}:spec-keys:electronics_system");
    let members = store.sorted_set_members(&key);
    assert!(members.contains(&"battery_capacity".to_owned()));
    assert!(!members.iter().any(|member| member.contains("20000")));
    assert!(!members.iter().any(|member| member.contains("65 W")));

    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/me/gears/{gear_id}"),
        Some(&token),
        json!({
            "specs": {
                "battery_capacity": "20000 mAh",
                "rated_energy": "74 Wh"
            }
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");

    let (updated_ranking_status, updated_ranking) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/spec-key-rankings?category=electronics_system",
        Some(&token),
    )
    .await;
    assert_eq!(updated_ranking_status, StatusCode::OK, "{updated_ranking}",);
    assert_eq!(updated_ranking["keys"][0], "battery_capacity");
    assert!(
        updated_ranking["keys"]
            .as_array()
            .unwrap()
            .contains(&json!("rated_energy"))
    );

    let other_token = login(&app.router, "spec-rank-other-user").await;
    let (other_status, other_ranking) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/spec-key-rankings?category=electronics_system",
        Some(&other_token),
    )
    .await;
    assert_eq!(other_status, StatusCode::OK, "{other_ranking}");
    assert_eq!(other_ranking, json!({"keys": []}));
}

#[tokio::test]
async fn spec_key_rankings_degrade_when_cache_is_disabled() {
    let app = test_app().await;
    let token = login(&app.router, "spec-rank-disabled-cache-user").await;
    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "无 Redis 充电宝",
            "specs": {
                "battery_capacity": "20000 mAh"
            }
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");

    let (ranking_status, ranking) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/spec-key-rankings?category=electronics_system",
        Some(&token),
    )
    .await;
    assert_eq!(ranking_status, StatusCode::OK, "{ranking}");
    assert_eq!(ranking, json!({"keys": []}));
}

#[tokio::test]
async fn tag_suggestions_track_tag_frequency_and_colors_without_changing_gear_tags() {
    let store = InMemoryCacheStore::default();
    let app = test_app_with_cache(Cache::with_store_for_tests(
        store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;
    let login_body = login_response(&app.router, "tag-suggestion-user").await;
    let token = login_body["access_token"].as_str().unwrap().to_owned();
    let user_id = login_body["user"]["id"].as_str().unwrap();

    let (initial_status, initial) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/tag-suggestions",
        Some(&token),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK, "{initial}");
    assert_eq!(initial, json!({"items": []}));

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "冬季充电宝",
            "tags": ["冬季", "电子", "电子"],
            "tag_colors": {
                "冬季": "blue",
                "电子": "teal"
            }
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["tags"], json!(["冬季", "电子"]));
    assert_eq!(
        created["tag_colors"],
        json!({"冬季": "blue", "电子": "teal"})
    );

    let color_key = format!("test-stellartrail:gear:{user_id}:tag-colors");
    let colors = store.hash_entries(&color_key);
    assert_eq!(colors.get("冬季").map(String::as_str), Some("blue"));
    assert_eq!(colors.get("电子").map(String::as_str), Some("teal"));

    let (suggestion_status, suggestions) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/tag-suggestions?limit=20",
        Some(&token),
    )
    .await;
    assert_eq!(suggestion_status, StatusCode::OK, "{suggestions}");
    let suggestion_items = suggestions["items"].as_array().unwrap();
    assert!(suggestion_items.contains(&json!({"tag": "冬季", "color": "blue"})));
    assert!(suggestion_items.contains(&json!({"tag": "电子", "color": "teal"})));

    let (list_status, list) = send_empty(&app.router, "GET", "/api/me/gears", Some(&token)).await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"][0]["tags"], json!(["冬季", "电子"]));
    assert_eq!(
        list["items"][0]["tag_colors"],
        json!({"冬季": "blue", "电子": "teal"})
    );

    let other_token = login(&app.router, "tag-suggestion-other-user").await;
    let (other_status, other_suggestions) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/tag-suggestions",
        Some(&other_token),
    )
    .await;
    assert_eq!(other_status, StatusCode::OK, "{other_suggestions}");
    assert_eq!(other_suggestions, json!({"items": []}));
}

#[tokio::test]
async fn tag_color_validation_and_disabled_cache_degrade_cleanly() {
    let app = test_app().await;
    let token = login(&app.router, "tag-color-disabled-cache-user").await;

    let (invalid_status, invalid) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "错误标签色装备",
            "tags": ["冬季"],
            "tag_colors": {"冬季": "neon"}
        }),
    )
    .await;
    assert_eq!(
        invalid_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid}"
    );
    assert_eq!(invalid["code"], "validation_failed");
    assert!(
        invalid["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "tag_colors.冬季")
    );

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "无 Redis 标签装备",
            "tags": ["冬季"],
            "tag_colors": {"冬季": "blue"}
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["tags"], json!(["冬季"]));
    assert_eq!(created["tag_colors"], json!({}));

    let (suggestion_status, suggestions) = send_empty(
        &app.router,
        "GET",
        "/api/me/gears/tag-suggestions",
        Some(&token),
    )
    .await;
    assert_eq!(suggestion_status, StatusCode::OK, "{suggestions}");
    assert_eq!(suggestions, json!({"items": []}));
}

#[tokio::test]
async fn backpack_specs_split_back_length_and_size() {
    let app = test_app().await;
    let token = login(&app.router, "backpack-spec-user").await;

    let (backpack_status, backpack) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "backpack_system",
            "name": "分体背包",
            "specs": {
                "back_length": "48 cm",
                "backpack_size": "M"
            }
        }),
    )
    .await;
    assert_eq!(backpack_status, StatusCode::CREATED, "{backpack}");
    assert_eq!(backpack["specs"]["back_length"], "48 cm");
    assert_eq!(backpack["specs"]["backpack_size"], "M");

    let (old_backpack_status, old_backpack) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "backpack_system",
            "name": "旧字段背包",
            "specs": {
                "back_length_or_size": "M / 48"
            }
        }),
    )
    .await;
    assert_eq!(
        old_backpack_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{old_backpack}"
    );
    assert!(
        old_backpack["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "specs.back_length_or_size")
    );
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
        "description": "冬季徒步备用电源",
        "weight_g": 315,
        "official_price_cents": 69900,
        "official_price_currency": "cny",
        "purchase_date": "2026-01-22",
        "purchase_price_cents": 63900,
        "purchase_price_currency": "CNY",
        "purchase_location": "京东",
        "status": "available",
        "storage_location": "装备柜 A1",
        "specs": {
            "battery_capacity": "20000 mAh",
            "waterproof_rating": "IPX4"
        },
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
    assert_eq!(created["official_price_currency"], "CNY");
    assert_eq!(created["purchase_price_currency"], "CNY");
    assert_eq!(created["specs"]["battery_capacity"], "20000 mAh");

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
    assert_eq!(list["items"][0]["official_price_cents"], 69900);
    assert_eq!(list["items"][0]["purchase_price_currency"], "CNY");

    let (invalid_status, invalid) = send_json(
        &app.router,
        "POST",
        "/api/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "错误参数",
            "purchase_price_cents": -1,
            "purchase_price_currency": "GBP",
            "specs": {"opening_style": "拉链"}
        }),
    )
    .await;
    assert_eq!(
        invalid_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid}"
    );
    assert_eq!(invalid["code"], "validation_failed");
    assert!(
        invalid["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "purchase_price_cents")
    );
    assert!(
        invalid["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "purchase_price_currency")
    );
    assert!(
        invalid["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "specs.opening_style")
    );

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
    assert_eq!(updated["specs"]["battery_capacity"], "20000 mAh");

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
