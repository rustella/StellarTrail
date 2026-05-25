use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode, header},
};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use stellartrail_api::{
    cache::{Cache, InMemoryCacheStore},
    config::{ApiConfig, CorsConfig, RedisCacheConfig},
    migrate_database,
    routes::build_router,
    state::AppState,
};
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AdminRoleRepository, GearAtlasRepository},
};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs},
    gear_atlas::GearAtlasExternalImportDraft,
};
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    db: sea_orm::DatabaseConnection,
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
        public_api: Default::default(),
        rate_limit: Default::default(),
        cors: CorsConfig::default(),
        mail: Default::default(),
    };
    TestApp {
        router: build_router(AppState::new_with_cache(config, db.clone(), cache)),
        db,
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
    let (status, _, value) = send_empty_with_headers(app, method, path, token, &[]).await;
    (status, value)
}

async fn send_empty_with_headers(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    headers: &[(&str, &str)],
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
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

async fn login(app: &Router, code: &str) -> String {
    let value = login_response(app, code).await;
    value["access_token"].as_str().unwrap().to_owned()
}

async fn login_response(app: &Router, code: &str) -> Value {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": code, "profile": {"nickname": "测试用户", "avatar_url": null}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}

async fn register_password_user_response(app: &Router, username: &str, email: &str) -> Value {
    let (code_status, code_body) = send_json(
        app,
        "POST",
        "/api/v1/auth/email-verification-code",
        None,
        json!({ "email": email }),
    )
    .await;
    assert_eq!(code_status, StatusCode::OK, "{code_body}");
    let code = code_body["debug_code"].as_str().unwrap();

    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        json!({
            "username": username,
            "email": email,
            "password": "Password1",
            "confirm_password": "Password1",
            "email_verification_code": code
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}

async fn grant_admin_role(app: &TestApp, target_user_id: &str, granted_by_user_id: &str) {
    let result = AdminRoleRepository::new(app.db.clone())
        .grant_admin(target_user_id, granted_by_user_id)
        .await
        .unwrap();
    assert!(result.record.role.can_administer());
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
        "/api/v1/me/gears",
        Some(&token),
        json!({"category": "lighting_system", "name": "缓存头灯"}),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");

    let (first_status, first_stats) =
        send_empty(&app.router, "GET", "/api/v1/me/gears/stats", Some(&token)).await;
    assert_eq!(first_status, StatusCode::OK, "{first_stats}");
    assert_eq!(first_stats["current_count"], 1);
    let after_first_read = store.stats();
    assert!(
        after_first_read.set_count >= 1,
        "first read should populate Redis-compatible cache: {after_first_read:?}",
    );

    let (second_status, second_stats) =
        send_empty(&app.router, "GET", "/api/v1/me/gears/stats", Some(&token)).await;
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
        "/api/v1/me/gears",
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
        send_empty(&app.router, "GET", "/api/v1/me/gears/stats", Some(&token)).await;
    assert_eq!(fresh_status, StatusCode::OK, "{fresh_stats}");
    assert_eq!(fresh_stats["current_count"], 2);
}

#[tokio::test]
async fn gear_overview_aggregates_first_screen_reads_and_uses_cache_version() {
    let store = InMemoryCacheStore::default();
    let app = test_app_with_cache(Cache::with_store_for_tests(
        store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;
    let token = login(&app.router, "gear-overview-user").await;

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({
            "category": "lighting_system",
            "name": "首屏头灯",
            "weight_g": 92,
            "purchase_price_cents": 19900
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");

    let (first_status, first) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/overview?tab=available&limit=2&sort=created_at_desc",
        Some(&token),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK, "{first}");
    assert_eq!(first["stats"]["current_count"], 1);
    assert_eq!(first["stats"]["total_value_cents"], 19900);
    assert_eq!(first["categories"]["items"][0]["id"], "all");
    assert_eq!(first["list"]["items"][0]["name"], "首屏头灯");
    let after_first = store.stats();
    assert!(after_first.set_count >= 1);

    let (second_status, second) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/overview?tab=available&limit=2&sort=created_at_desc",
        Some(&token),
    )
    .await;
    assert_eq!(second_status, StatusCode::OK, "{second}");
    assert_eq!(second["stats"]["current_count"], 1);
    let after_second = store.stats();
    assert!(
        after_second.hit_count > after_first.hit_count,
        "second overview read should hit cache: before={after_first:?} after={after_second:?}",
    );

    let (unsupported_status, unsupported) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/overview?q=headlamp",
        Some(&token),
    )
    .await;
    assert_eq!(unsupported_status, StatusCode::BAD_REQUEST, "{unsupported}");
    assert_eq!(unsupported["code"], "unsupported_query_parameter");
    assert_eq!(unsupported["parameter"], "q");

    let (second_create_status, second_created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({"category": "lighting_system", "name": "首屏营灯"}),
    )
    .await;
    assert_eq!(
        second_create_status,
        StatusCode::CREATED,
        "{second_created}",
    );

    let (fresh_status, fresh) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/overview?tab=available&limit=2&sort=created_at_desc",
        Some(&token),
    )
    .await;
    assert_eq!(fresh_status, StatusCode::OK, "{fresh}");
    assert_eq!(fresh["stats"]["current_count"], 2);
}

#[tokio::test]
async fn gear_create_merges_same_item_and_stats_count_physical_quantity() {
    let app = test_app().await;
    let token = login(&app.router, "gear-quantity-user").await;

    let create_body = json!({
        "category": "electronics_system",
        "name": "华为 12000mAh 充电宝",
        "brand": "华为",
        "model": "12000mAh",
        "weight_g": 220,
        "purchase_price_cents": 19900,
        "specs": {"battery_capacity": "12000 mAh"},
        "tags": ["电子"]
    });
    let (first_status, first) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        create_body.clone(),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED, "{first}");
    let gear_id = first["id"].as_str().unwrap();
    assert_eq!(first["quantity"], 1);

    let mut second_body = create_body;
    second_body["quantity"] = json!(1);
    second_body["tags"] = json!(["备用"]);
    second_body["purchase_location"] = json!("天猫");
    let (second_status, second) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        second_body,
    )
    .await;
    assert_eq!(second_status, StatusCode::CREATED, "{second}");
    assert_eq!(second["id"], gear_id);
    assert_eq!(second["quantity"], 2);
    assert_eq!(second["tags"], json!(["电子", "备用"]));
    assert!(
        second["notes"]
            .as_str()
            .is_some_and(|notes| notes.contains("天猫")),
        "merge note should preserve incoming differing purchase data: {second}"
    );

    let (list_status, list) =
        send_empty(&app.router, "GET", "/api/v1/me/gears", Some(&token)).await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["quantity"], 2);

    let (stats_status, stats) =
        send_empty(&app.router, "GET", "/api/v1/me/gears/stats", Some(&token)).await;
    assert_eq!(stats_status, StatusCode::OK, "{stats}");
    assert_eq!(stats["current_count"], 2);
    assert_eq!(stats["total_weight_g"], 440);
    assert_eq!(stats["total_value_cents"], 39800);
    assert_eq!(stats["by_category"][7]["count"], 2);
}

#[tokio::test]
async fn gear_packing_lists_create_add_check_and_keep_unavailable_items_visible() {
    let app = test_app().await;
    let token = login(&app.router, "gear-packing-user").await;

    let (backpack_status, backpack) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({"category": "backpack_system", "name": "轻量小包", "weight_g": 800, "quantity": 2}),
    )
    .await;
    assert_eq!(backpack_status, StatusCode::CREATED, "{backpack}");
    let backpack_id = backpack["id"].as_str().unwrap();
    let (headlamp_status, headlamp) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({"category": "lighting_system", "name": "头灯", "weight_g": 90}),
    )
    .await;
    assert_eq!(headlamp_status, StatusCode::CREATED, "{headlamp}");
    let headlamp_id = headlamp["id"].as_str().unwrap();

    let (unauth_status, unauth_body) =
        send_empty(&app.router, "GET", "/api/v1/me/packing-lists", None).await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED, "{unauth_body}");

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/packing-lists",
        Some(&token),
        json!({
            "name": " 武功山一日 ",
            "route_name": "武功山",
            "duration_label": "一日"
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["name"], "武功山一日");
    assert_eq!(created["stats"]["item_count"], 0);
    let list_id = created["id"].as_str().unwrap();

    let (add_status, added) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/packing-lists/{list_id}/items"),
        Some(&token),
        json!({"gear_ids": [backpack_id, headlamp_id, backpack_id]}),
    )
    .await;
    assert_eq!(add_status, StatusCode::OK, "{added}");
    assert_eq!(added["stats"]["item_count"], 2);
    assert_eq!(added["stats"]["total_weight_g"], 890);
    assert_eq!(added["items"][0]["planned_quantity"], 1);
    assert_eq!(added["items"][0]["gear"]["quantity"], 2);
    let first_item_id = added["items"][0]["id"].as_str().unwrap();

    let (planned_status, planned) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/packing-lists/{list_id}/items/{first_item_id}"),
        Some(&token),
        json!({"planned_quantity": 2}),
    )
    .await;
    assert_eq!(planned_status, StatusCode::OK, "{planned}");
    assert_eq!(planned["stats"]["item_count"], 3);
    assert_eq!(planned["stats"]["total_weight_g"], 1690);

    let (packed_status, packed) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/packing-lists/{list_id}/items/{first_item_id}"),
        Some(&token),
        json!({"packed_quantity": 1}),
    )
    .await;
    assert_eq!(packed_status, StatusCode::OK, "{packed}");
    assert_eq!(packed["stats"]["packed_count"], 1);
    assert_eq!(packed["items"][0]["packed"], false);

    let (list_status, list) =
        send_empty(&app.router, "GET", "/api/v1/me/packing-lists", Some(&token)).await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"][0]["id"], list_id);
    assert_eq!(list["items"][0]["packed_count"], 1);

    let (archive_status, archive_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/gears/{backpack_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(archive_status, StatusCode::NO_CONTENT, "{archive_body}");
    let (delete_status, delete_body) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{headlamp_id}/delete"),
        Some(&token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (detail_status, detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/packing-lists/{list_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK, "{detail}");
    let item_reasons = detail["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["unavailable_reason"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(item_reasons.contains(&"archived"));
    assert!(item_reasons.contains(&"deleted"));

    let (invalid_add_status, invalid_add) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/packing-lists/{list_id}/items"),
        Some(&token),
        json!({"gear_ids": [backpack_id]}),
    )
    .await;
    assert_eq!(
        invalid_add_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid_add}"
    );
    assert_eq!(invalid_add["code"], "validation_failed");

    let item_to_remove = detail["items"][0]["id"].as_str().unwrap();
    let (remove_status, removed) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/packing-lists/{list_id}/items/{item_to_remove}"),
        Some(&token),
    )
    .await;
    assert_eq!(remove_status, StatusCode::OK, "{removed}");
    assert_eq!(removed["stats"]["item_count"], 1);

    let (delete_list_status, delete_list_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/packing-lists/{list_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(
        delete_list_status,
        StatusCode::NO_CONTENT,
        "{delete_list_body}"
    );
    let (missing_status, missing) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/packing-lists/{list_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND, "{missing}");
}

#[tokio::test]
async fn gear_packing_lists_reject_cross_user_gear() {
    let app = test_app().await;
    let owner_token = login(&app.router, "gear-packing-owner").await;
    let other_token = login(&app.router, "gear-packing-other").await;

    let (gear_status, gear) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&owner_token),
        json!({"category": "backpack_system", "name": "所有者装备"}),
    )
    .await;
    assert_eq!(gear_status, StatusCode::CREATED, "{gear}");
    let owner_gear_id = gear["id"].as_str().unwrap();

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/packing-lists",
        Some(&other_token),
        json!({"name": "其它用户清单"}),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    let other_list_id = created["id"].as_str().unwrap();

    let (add_status, add_body) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/packing-lists/{other_list_id}/items"),
        Some(&other_token),
        json!({"gear_ids": [owner_gear_id]}),
    )
    .await;
    assert_eq!(add_status, StatusCode::UNPROCESSABLE_ENTITY, "{add_body}");
    assert_eq!(add_body["code"], "validation_failed");
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
        "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
        Some(&token),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK, "{initial}");
    assert_eq!(initial, json!({"keys": []}));

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
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
        "/api/v1/me/gears",
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
        "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
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
        &format!("/api/v1/me/gears/{gear_id}"),
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
        "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
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
        "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
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
        "/api/v1/me/gears",
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
        "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
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
        "/api/v1/me/gears/tag-suggestions",
        Some(&token),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK, "{initial}");
    assert_eq!(initial, json!({"items": []}));

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
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
        "/api/v1/me/gears/tag-suggestions?limit=20",
        Some(&token),
    )
    .await;
    assert_eq!(suggestion_status, StatusCode::OK, "{suggestions}");
    let suggestion_items = suggestions["items"].as_array().unwrap();
    assert!(suggestion_items.contains(&json!({"tag": "冬季", "color": "blue"})));
    assert!(suggestion_items.contains(&json!({"tag": "电子", "color": "teal"})));

    let (list_status, list) =
        send_empty(&app.router, "GET", "/api/v1/me/gears", Some(&token)).await;
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
        "/api/v1/me/gears/tag-suggestions",
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
        "/api/v1/me/gears",
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
        "/api/v1/me/gears",
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
        "/api/v1/me/gears/tag-suggestions",
        Some(&token),
    )
    .await;
    assert_eq!(suggestion_status, StatusCode::OK, "{suggestions}");
    assert_eq!(suggestions, json!({"items": []}));
}

#[tokio::test]
async fn backpack_specs_keep_back_length_and_reject_size_specs() {
    let app = test_app().await;
    let token = login(&app.router, "backpack-spec-user").await;

    let (backpack_status, backpack) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({
            "category": "backpack_system",
            "name": "分体背包",
            "selected_variant_label": "M",
            "specs": {
                "back_length": "48 cm"
            }
        }),
    )
    .await;
    assert_eq!(backpack_status, StatusCode::CREATED, "{backpack}");
    assert_eq!(backpack["specs"]["back_length"], "48 cm");
    assert_eq!(backpack["selected_variant_label"], "M");

    let (old_backpack_status, old_backpack) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({
            "category": "backpack_system",
            "name": "旧字段背包",
            "specs": {
                "backpack_size": "M"
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
            .any(|field| field["field"] == "specs.backpack_size")
    );
}

#[tokio::test]
async fn gear_atlas_submission_copies_only_public_fields_and_waits_for_admin_review() {
    let admin_email = "atlas-admin@example.test";
    let app = test_app().await;
    let token = login(&app.router, "atlas-submit-user").await;

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({
            "category": "electronics_system",
            "name": "公共充电宝",
            "brand": "NITECORE",
            "model": "SUMMIT 20000",
            "description": "冬季徒步备用电源",
            "weight_g": 315,
            "official_price_cents": 69900,
            "official_price_currency": "CNY",
            "purchase_date": "2026-01-22",
            "purchase_price_cents": 63900,
            "purchase_price_currency": "CNY",
            "purchase_location": "京东",
            "status": "maintenance",
            "storage_location": "装备柜 A1",
            "selected_variant_label": "20000mAh 标准版",
            "specs": {
                "battery_capacity": "20000 mAh",
                "rated_energy": "74 Wh"
            },
            "tags": ["冬季", "电子"],
            "tag_colors": {"冬季": "blue", "电子": "teal"},
            "notes": "不要公开的个人备注"
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    let gear_id = created["id"].as_str().unwrap();

    let (submission_status, submission) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{gear_id}/atlas-submission"),
        Some(&token),
    )
    .await;
    assert_eq!(submission_status, StatusCode::CREATED, "{submission}");
    assert_eq!(submission["status"], "pending");
    assert_eq!(submission["source_type"], "user_gear");
    assert_eq!(submission["source_user_gear_id"], gear_id);
    assert_eq!(submission["official_price_cents"], 69900);
    assert_eq!(submission["variants"][0]["label"], "20000mAh 标准版");
    assert_eq!(submission["specs"]["battery_capacity"], "20000 mAh");
    assert!(submission.get("purchase_price_cents").is_none());
    assert!(submission.get("purchase_location").is_none());
    assert!(submission.get("storage_location").is_none());
    assert!(submission.get("notes").is_none());
    assert!(submission.get("tags").is_none());

    let (public_pending_status, public_pending) =
        send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(public_pending_status, StatusCode::OK, "{public_pending}");
    assert_eq!(public_pending["items"].as_array().unwrap().len(), 0);

    let (duplicate_status, duplicate) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{gear_id}/atlas-submission"),
        Some(&token),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::OK, "{duplicate}");
    assert_eq!(duplicate["id"], submission["id"]);

    let (non_admin_status, non_admin) = send_empty(
        &app.router,
        "GET",
        "/api/v1/admin/gear-atlas-submissions",
        Some(&token),
    )
    .await;
    assert_eq!(non_admin_status, StatusCode::FORBIDDEN, "{non_admin}");

    let admin_login =
        register_password_user_response(&app.router, "atlas_admin", admin_email).await;
    let admin_token = admin_login["access_token"].as_str().unwrap().to_owned();
    let admin_user_id = admin_login["user"]["id"].as_str().unwrap();
    grant_admin_role(&app, admin_user_id, admin_user_id).await;
    let (admin_list_status, admin_list) = send_empty(
        &app.router,
        "GET",
        "/api/v1/admin/gear-atlas-submissions?status=pending",
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(admin_list_status, StatusCode::OK, "{admin_list}");
    assert_eq!(admin_list["items"].as_array().unwrap().len(), 1);
    let submission_id = submission["id"].as_str().unwrap();

    let (approve_status, approved) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}/approve"),
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK, "{approved}");
    assert_eq!(approved["status"], "approved");
    assert!(!approved["approved_at"].is_null());
    assert_eq!(approved["is_deleted"], false);

    let (fallback_status, fallback_headers, fallback_public) = send_empty_with_headers(
        &app.router,
        "GET",
        "/api/v1/gear-atlas",
        None,
        &[("X-StellarTrail-Locale", "en")],
    )
    .await;
    assert_eq!(fallback_status, StatusCode::OK, "{fallback_public}");
    assert_eq!(
        fallback_headers.get(header::CONTENT_LANGUAGE).unwrap(),
        "en"
    );
    assert_eq!(fallback_public["items"][0]["name"], "公共充电宝");
    assert_eq!(
        fallback_public["items"][0]["category_label"],
        "Electronics System"
    );

    app.db
        .execute(Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "INSERT INTO gear_atlas_item_localizations(atlas_item_id, locale, name, description) \
             VALUES (?, 'en', ?, ?) ON CONFLICT(atlas_item_id, locale) DO UPDATE SET name = excluded.name, description = excluded.description",
            vec![
                submission_id.to_owned().into(),
                "Public power bank".to_owned().into(),
                "Backup power for winter hiking".to_owned().into(),
            ],
        ))
        .await
        .expect("insert english atlas localization");

    let (public_status, public) = send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(public_status, StatusCode::OK, "{public}");
    assert_eq!(public["items"].as_array().unwrap().len(), 1);
    assert_eq!(public["items"][0]["id"], submission_id);
    assert_eq!(public["items"][0]["name"], "公共充电宝");
    assert_eq!(public["items"][0]["is_deleted"], false);

    let (english_public_status, english_headers, english_public) = send_empty_with_headers(
        &app.router,
        "GET",
        "/api/v1/gear-atlas?limit=21",
        None,
        &[("X-StellarTrail-Locale", "en")],
    )
    .await;
    assert_eq!(english_public_status, StatusCode::OK, "{english_public}");
    assert_eq!(english_headers.get(header::CONTENT_LANGUAGE).unwrap(), "en");
    assert_eq!(english_public["items"][0]["name"], "Public power bank");
    assert_eq!(
        english_public["items"][0]["category_label"],
        "Electronics System"
    );

    let (detail_status, detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/gear-atlas/{submission_id}"),
        None,
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK, "{detail}");
    assert_eq!(detail["name"], "公共充电宝");
    assert!(detail.get("purchase_price_cents").is_none());
    assert!(detail.get("purchase_location").is_none());
    assert!(detail.get("storage_location").is_none());
    assert!(detail.get("notes").is_none());
    assert!(detail.get("tags").is_none());
    assert_eq!(detail["is_deleted"], false);

    let (delete_status, delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}"),
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (hidden_public_status, hidden_public) =
        send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(hidden_public_status, StatusCode::OK, "{hidden_public}");
    assert_eq!(hidden_public["items"].as_array().unwrap().len(), 0);

    let (deleted_admin_status, deleted_admin) = send_empty(
        &app.router,
        "GET",
        "/api/v1/admin/gear-atlas-submissions?deleted=deleted",
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(deleted_admin_status, StatusCode::OK, "{deleted_admin}");
    assert_eq!(deleted_admin["items"].as_array().unwrap().len(), 1);
    assert_eq!(deleted_admin["items"][0]["is_deleted"], true);

    let (restore_status, restored_atlas) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}/restore"),
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(restore_status, StatusCode::OK, "{restored_atlas}");
    assert_eq!(restored_atlas["is_deleted"], false);

    let (english_detail_status, english_detail_headers, english_detail) = send_empty_with_headers(
        &app.router,
        "GET",
        &format!("/api/v1/gear-atlas/{submission_id}"),
        None,
        &[("Accept-Language", "en-US,en;q=0.8")],
    )
    .await;
    assert_eq!(english_detail_status, StatusCode::OK, "{english_detail}");
    assert_eq!(
        english_detail_headers
            .get(header::CONTENT_LANGUAGE)
            .unwrap(),
        "en"
    );
    assert_eq!(english_detail["name"], "Public power bank");
    assert_eq!(
        english_detail["description"],
        "Backup power for winter hiking"
    );

    let (locale_status, _, locale_body) = send_empty_with_headers(
        &app.router,
        "GET",
        "/api/v1/gear-atlas?locale=en",
        None,
        &[],
    )
    .await;
    assert_eq!(locale_status, StatusCode::BAD_REQUEST, "{locale_body}");
    assert_eq!(locale_body["code"], "unsupported_query_parameter");
    assert_eq!(locale_body["parameter"], "locale");

    let (detail_locale_status, _, detail_locale_body) = send_empty_with_headers(
        &app.router,
        "GET",
        &format!("/api/v1/gear-atlas/{submission_id}?locale=en"),
        None,
        &[],
    )
    .await;
    assert_eq!(
        detail_locale_status,
        StatusCode::BAD_REQUEST,
        "{detail_locale_body}"
    );
    assert_eq!(detail_locale_body["code"], "unsupported_query_parameter");
    assert_eq!(detail_locale_body["parameter"], "locale");
}

#[tokio::test]
async fn gear_atlas_public_routes_hide_external_source_audit_but_admin_routes_expose_it() {
    let app = test_app().await;
    let login_body = login_response(&app.router, "atlas-external-source").await;
    let user_id = login_body["user"]["id"].as_str().unwrap().to_owned();
    let token = login_body["access_token"].as_str().unwrap().to_owned();
    let repo = GearAtlasRepository::new(app.db.clone());
    let mut draft = GearAtlasExternalImportDraft {
        category: GearCategory::BackpackSystem,
        name: "探路者38L户外背包".to_owned(),
        brand: None,
        model: None,
        description: Some(
            "来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。".to_owned(),
        ),
        weight_g: None,
        official_price_cents: Some(34_900),
        official_price_currency: Some("CNY".to_owned()),
        variants: Vec::new(),
        specs: GearSpecs::new(),
        submitted_by_user_id: user_id.clone(),
        source_key: "8264:2074165".to_owned(),
        source_name: "8264 户外用品点评".to_owned(),
        source_url: Some("https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html".to_owned()),
        source_license_note: Some("facts and source link only".to_owned()),
        import_batch_id: Some("batch-20260521".to_owned()),
        source_rating_score: Some(8.6),
        source_rating_count: Some(7),
    };
    draft
        .validate_and_normalize()
        .expect("valid external import");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("import atlas source")
        .item;
    repo.approve(&imported.id, &user_id)
        .await
        .expect("approve import")
        .expect("approved import");

    let (list_status, list) = send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    let item = &list["items"][0];
    assert!(item.get("source_name").is_none());
    assert!(item.get("source_url").is_none());
    assert!(item.get("source_rating_score").is_none());
    assert!(item.get("source_rating_count").is_none());
    assert!(item.get("source_key").is_none());
    assert!(item.get("source_license_note").is_none());
    assert!(item.get("import_batch_id").is_none());

    let (detail_status, detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/gear-atlas/{}", imported.id),
        None,
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK, "{detail}");
    assert!(detail.get("source_name").is_none());
    assert!(detail.get("source_url").is_none());
    assert!(detail.get("source_rating_score").is_none());
    assert!(detail.get("source_rating_count").is_none());
    assert!(detail.get("source_key").is_none());

    grant_admin_role(&app, &user_id, &user_id).await;
    let (admin_status, admin_detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/admin/gear-atlas-submissions/{}", imported.id),
        Some(token.as_str()),
    )
    .await;
    assert_eq!(admin_status, StatusCode::OK, "{admin_detail}");
    assert_eq!(admin_detail["source_name"], "8264 户外用品点评");
    assert_eq!(
        admin_detail["source_url"],
        "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html"
    );
    assert_eq!(admin_detail["source_rating_score"].as_f64(), Some(8.6));
    assert_eq!(admin_detail["source_rating_count"].as_i64(), Some(7));
    assert!(admin_detail.get("source_key").is_none());
    assert!(admin_detail.get("source_license_note").is_none());
    assert!(admin_detail.get("import_batch_id").is_none());
}

#[tokio::test]
async fn gear_atlas_public_routes_use_response_cache_and_etag() {
    let store = InMemoryCacheStore::default();
    let app = test_app_with_cache(Cache::with_store_for_tests(
        store.clone(),
        "test-stellartrail",
        Duration::from_secs(300),
    ))
    .await;
    let login_body = login_response(&app.router, "atlas-cache-source").await;
    let user_id = login_body["user"]["id"].as_str().unwrap().to_owned();
    let repo = GearAtlasRepository::new(app.db.clone());
    let mut draft = GearAtlasExternalImportDraft {
        category: GearCategory::ElectronicsSystem,
        name: "缓存图鉴头灯".to_owned(),
        brand: Some("NITECORE".to_owned()),
        model: Some("NU25".to_owned()),
        description: Some("轻量头灯".to_owned()),
        weight_g: Some(56),
        official_price_cents: Some(29_900),
        official_price_currency: Some("CNY".to_owned()),
        variants: Vec::new(),
        specs: GearSpecs::new(),
        submitted_by_user_id: user_id.clone(),
        source_key: "test:atlas-cache-headlamp".to_owned(),
        source_name: "测试来源".to_owned(),
        source_url: Some("https://example.test/headlamp".to_owned()),
        source_license_note: Some("test fixture".to_owned()),
        import_batch_id: Some("batch-cache".to_owned()),
        source_rating_score: None,
        source_rating_count: None,
    };
    draft
        .validate_and_normalize()
        .expect("valid external import");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("import atlas source")
        .item;
    repo.approve(&imported.id, &user_id)
        .await
        .expect("approve import")
        .expect("approved import");

    let (first_list_status, first_list_headers, first_list) =
        send_empty_with_headers(&app.router, "GET", "/api/v1/gear-atlas?limit=10", None, &[]).await;
    assert_eq!(first_list_status, StatusCode::OK, "{first_list}");
    assert_eq!(first_list["items"][0]["name"], "缓存图鉴头灯");
    let list_etag = first_list_headers
        .get(header::ETAG)
        .expect("list etag")
        .to_str()
        .expect("list etag string")
        .to_owned();
    let after_first_list = store.stats();
    assert!(after_first_list.set_count >= 1);

    let (second_list_status, _second_list_headers, second_list) = send_empty_with_headers(
        &app.router,
        "GET",
        "/api/v1/gear-atlas?limit=10",
        None,
        &[(header::IF_NONE_MATCH.as_str(), list_etag.as_str())],
    )
    .await;
    assert_eq!(
        second_list_status,
        StatusCode::NOT_MODIFIED,
        "{second_list}"
    );
    let after_second_list = store.stats();
    assert!(after_second_list.hit_count > after_first_list.hit_count);

    let detail_path = format!("/api/v1/gear-atlas/{}", imported.id);
    let (first_detail_status, first_detail_headers, first_detail) =
        send_empty_with_headers(&app.router, "GET", &detail_path, None, &[]).await;
    assert_eq!(first_detail_status, StatusCode::OK, "{first_detail}");
    assert_eq!(first_detail["name"], "缓存图鉴头灯");
    let detail_etag = first_detail_headers
        .get(header::ETAG)
        .expect("detail etag")
        .to_str()
        .expect("detail etag string")
        .to_owned();
    let after_first_detail = store.stats();

    let (second_detail_status, _second_detail_headers, second_detail) = send_empty_with_headers(
        &app.router,
        "GET",
        &detail_path,
        None,
        &[(header::IF_NONE_MATCH.as_str(), detail_etag.as_str())],
    )
    .await;
    assert_eq!(
        second_detail_status,
        StatusCode::NOT_MODIFIED,
        "{second_detail}"
    );
    let after_second_detail = store.stats();
    assert!(after_second_detail.hit_count > after_first_detail.hit_count);
}

#[tokio::test]
async fn gear_atlas_manual_submissions_validate_specs_and_rejections_stay_private() {
    let admin_email = "atlas-reviewer@example.test";
    let app = test_app().await;
    let token = login(&app.router, "atlas-manual-user").await;

    let (unauth_status, unauth) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gear-atlas-submissions",
        None,
        json!({"category": "lighting_system", "name": "未登录头灯"}),
    )
    .await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED, "{unauth}");

    let (invalid_status, invalid) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gear-atlas-submissions",
        Some(&token),
        json!({
            "category": "lighting_system",
            "name": "错误头灯",
            "official_price_cents": -1,
            "official_price_currency": "GBP",
            "specs": {"battery_capacity": "2000 mAh"}
        }),
    )
    .await;
    assert_eq!(
        invalid_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid}"
    );
    assert!(
        invalid["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "specs.battery_capacity")
    );
    let (invalid_size_status, invalid_size) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gear-atlas-submissions",
        Some(&token),
        json!({
            "category": "clothing_system",
            "name": "错误尺码衣物",
            "specs": {"size": "M"}
        }),
    )
    .await;
    assert_eq!(
        invalid_size_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid_size}"
    );
    assert!(
        invalid_size["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "specs.size")
    );

    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gear-atlas-submissions",
        Some(&token),
        json!({
            "category": "lighting_system",
            "name": "手动头灯",
            "brand": "NITECORE",
            "official_price_cents": 19900,
            "variants": [{"key": "standard", "label": "标准版", "official_price_cents": 19900, "official_price_currency": "CNY"}],
            "specs": {"max_brightness": "400 lm"}
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");
    assert_eq!(created["status"], "pending");
    assert_eq!(created["variants"][0]["label"], "标准版");

    let admin_login =
        register_password_user_response(&app.router, "atlas_reviewer", admin_email).await;
    let admin_token = admin_login["access_token"].as_str().unwrap().to_owned();
    let admin_user_id = admin_login["user"]["id"].as_str().unwrap();
    grant_admin_role(&app, admin_user_id, admin_user_id).await;
    let submission_id = created["id"].as_str().unwrap();
    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}"),
        Some(admin_token.as_str()),
        json!({
            "category": "lighting_system",
            "name": "手动头灯",
            "brand": "NITECORE",
            "model": null,
            "description": null,
            "weight_g": null,
            "official_price_cents": 19900,
            "official_price_currency": "CNY",
            "variants": [{"key": "wide", "label": "宽版", "weight_g": 88}],
            "specs": {"max_brightness": "400 lm"}
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert_eq!(updated["variants"][0]["label"], "宽版");
    assert_eq!(updated["variants"][0]["weight_g"], 88);

    let (blank_reject_status, blank_reject) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}/reject"),
        Some(admin_token.as_str()),
        json!({"reason": "  "}),
    )
    .await;
    assert_eq!(
        blank_reject_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{blank_reject}"
    );

    let (reject_status, rejected) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}/reject"),
        Some(admin_token.as_str()),
        json!({"reason": "信息不足"}),
    )
    .await;
    assert_eq!(reject_status, StatusCode::OK, "{rejected}");
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejection_reason"], "信息不足");

    let (public_status, public) = send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(public_status, StatusCode::OK, "{public}");
    assert_eq!(public["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn gear_atlas_admin_edits_are_reported_to_submitter_after_approval() {
    let app = test_app().await;
    let token = login(&app.router, "atlas-review-change-user").await;
    let (create_status, created) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gear-atlas-submissions",
        Some(&token),
        json!({
            "category": "sleep_system",
            "name": "原始睡袋",
            "brand": "BLACKICE",
            "model": "G700",
            "description": "原始描述",
            "weight_g": 1000,
            "official_price_cents": 90000,
            "official_price_currency": "CNY",
            "variants": [{"key": "m-75-195", "label": "M 75*195"}],
            "specs": {"fill_weight": "700 g", "filling": "白鹅绒"}
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED, "{created}");

    let admin_login = register_password_user_response(
        &app.router,
        "atlas_review_change_admin",
        "atlas-review-change-admin@example.test",
    )
    .await;
    let admin_token = admin_login["access_token"].as_str().unwrap().to_owned();
    let admin_user_id = admin_login["user"]["id"].as_str().unwrap();
    grant_admin_role(&app, admin_user_id, admin_user_id).await;
    let submission_id = created["id"].as_str().unwrap();

    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}"),
        Some(admin_token.as_str()),
        json!({
            "category": "sleep_system",
            "name": "审核后睡袋",
            "brand": "BLACKICE",
            "model": "G700 Pro",
            "description": "公开展示描述",
            "weight_g": 980,
            "official_price_cents": 95000,
            "official_price_currency": "CNY",
            "variants": [
                {"key": "m-75-195", "label": "M 75*195", "official_price_cents": 95000, "official_price_currency": "CNY"},
                {"key": "l-80-205", "label": "L 80*205", "weight_g": 1020}
            ],
            "specs": {"fill_weight": "720 g", "filling": "白鹅绒"}
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");

    let (approve_status, approved) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/admin/gear-atlas-submissions/{submission_id}/approve"),
        Some(admin_token.as_str()),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK, "{approved}");
    assert_eq!(approved["status"], "approved");
    let changes = approved["review_changes"].as_array().unwrap();
    assert!(changes.iter().any(|change| {
        change["field"] == "name"
            && change["label"] == "名称"
            && change["before"] == "原始睡袋"
            && change["after"] == "审核后睡袋"
    }));
    assert!(changes.iter().any(|change| change["field"] == "variants"));
    assert!(changes.iter().any(|change| {
        change["field"] == "specs.fill_weight" && change["label"] == "分类参数 · 填充重量"
    }));

    let (mine_status, mine) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gear-atlas-submissions",
        Some(&token),
    )
    .await;
    assert_eq!(mine_status, StatusCode::OK, "{mine}");
    assert!(
        mine["items"][0]["review_changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "name")
    );

    let (public_status, public) = send_empty(&app.router, "GET", "/api/v1/gear-atlas", None).await;
    assert_eq!(public_status, StatusCode::OK, "{public}");
    assert_eq!(public["items"][0]["name"], "审核后睡袋");
    assert!(public["items"][0].get("review_changes").is_none());
    assert!(public["items"][0].get("rejection_reason").is_none());
    assert!(public["items"][0].get("submitted_snapshot").is_none());
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
        "selected_variant_label": "标准版",
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
        "/api/v1/me/gears",
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
    assert_eq!(created["selected_variant_label"], "标准版");
    assert_eq!(created["quantity"], 1);
    assert_eq!(created["specs"]["battery_capacity"], "20000 mAh");
    assert_eq!(created["is_deleted"], false);

    let (stats_status, stats) =
        send_empty(&app.router, "GET", "/api/v1/me/gears/stats", Some(&token)).await;
    assert_eq!(stats_status, StatusCode::OK, "{stats}");
    assert_eq!(stats["current_count"], 1);
    assert_eq!(stats["total_value_cents"], 63900);
    assert_eq!(stats["total_weight_g"], 315);

    let (category_status, categories) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears/categories",
        Some(&token),
    )
    .await;
    assert_eq!(category_status, StatusCode::OK, "{categories}");
    assert_eq!(categories["items"][0]["id"], "all");
    assert_eq!(categories["items"][0]["count"], 1);

    let (list_status, list) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears?category=electronics_system&q=nitecore&sort=created_at_desc",
        Some(&token),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK, "{list}");
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["category_label"], "电子系统");
    assert_eq!(list["items"][0]["status_label"], "可用");
    assert_eq!(list["items"][0]["official_price_cents"], 69900);
    assert_eq!(list["items"][0]["purchase_price_currency"], "CNY");
    assert_eq!(list["items"][0]["selected_variant_label"], "标准版");
    assert_eq!(list["items"][0]["quantity"], 1);
    assert_eq!(list["items"][0]["is_deleted"], false);

    let (invalid_status, invalid) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
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
        &format!("/api/v1/me/gears/{gear_id}"),
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
        &format!("/api/v1/me/gears/{gear_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT, "{delete_body}");

    let (available_status, available) =
        send_empty(&app.router, "GET", "/api/v1/me/gears", Some(&token)).await;
    assert_eq!(available_status, StatusCode::OK, "{available}");
    assert_eq!(available["items"].as_array().unwrap().len(), 0);

    let (history_status, history) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears?tab=history",
        Some(&token),
    )
    .await;
    assert_eq!(history_status, StatusCode::OK, "{history}");
    assert_eq!(history["items"].as_array().unwrap().len(), 1);

    let (soft_delete_status, soft_delete_body) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{gear_id}/delete"),
        Some(&token),
    )
    .await;
    assert_eq!(
        soft_delete_status,
        StatusCode::NO_CONTENT,
        "{soft_delete_body}"
    );

    let (deleted_detail_status, deleted_detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/gears/{gear_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(
        deleted_detail_status,
        StatusCode::NOT_FOUND,
        "{deleted_detail}"
    );

    let (hidden_history_status, hidden_history) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears?tab=history",
        Some(&token),
    )
    .await;
    assert_eq!(hidden_history_status, StatusCode::OK, "{hidden_history}");
    assert_eq!(hidden_history["items"].as_array().unwrap().len(), 0);

    let (deleted_history_status, deleted_history) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/gears?tab=history&deleted=deleted",
        Some(&token),
    )
    .await;
    assert_eq!(deleted_history_status, StatusCode::OK, "{deleted_history}");
    assert_eq!(deleted_history["items"].as_array().unwrap().len(), 1);
    assert_eq!(deleted_history["items"][0]["is_deleted"], true);

    let (undelete_status, undeleted) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{gear_id}/undelete"),
        Some(&token),
    )
    .await;
    assert_eq!(undelete_status, StatusCode::OK, "{undeleted}");
    assert_eq!(undeleted["is_deleted"], false);
    assert!(!undeleted["archived_at"].is_null());

    let (restore_status, restored) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/gears/{gear_id}/restore"),
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
        "/api/v1/me/gears/import",
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
        "/api/v1/me/gears/import",
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
                .uri("/api/v1/me/gears/export?format=csv")
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
    assert!(csv.contains("category,name,brand,model,description,quantity"));
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
        "/api/v1/me/gears",
        Some(&token_a),
        json!({"category": "lighting_system", "name": "A 的头灯"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let gear_id = created["id"].as_str().unwrap();

    let (read_status, value) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/gears/{gear_id}"),
        Some(&token_b),
    )
    .await;

    assert_eq!(read_status, StatusCode::NOT_FOUND, "{value}");
}
