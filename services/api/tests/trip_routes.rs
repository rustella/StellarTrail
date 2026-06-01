use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use stellartrail_api::{
    cache::Cache,
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
        sms: Default::default(),
    };
    TestApp {
        router: build_router(AppState::new_with_cache(config, db, Cache::disabled())),
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
    response_value(response).await
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
    response_value(response).await
}

async fn response_value(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, value)
}

async fn login(app: &Router, code: &str, nickname: &str) -> String {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": code, "profile": {"nickname": nickname, "avatar_url": null}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["access_token"].as_str().unwrap().to_owned()
}

async fn create_plan(app: &Router, token: &str) -> Value {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/me/trips",
        Some(token),
        json!({"trip_type": "team", "title": "端午重装计划"}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{value}");
    value
}

async fn create_route_segment(
    app: &Router,
    token: &str,
    plan_id: &str,
    name: &str,
    distance_km: f64,
    ascent_m: i32,
    descent_m: i32,
) -> Value {
    let (status, value) = send_json(
        app,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/route-segments"),
        Some(token),
        json!({
            "name": name,
            "distance_km": distance_km,
            "ascent_m": ascent_m,
            "descent_m": descent_m
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{value}");
    value
}

fn route_segment_by_name<'a>(detail: &'a Value, name: &str) -> &'a Value {
    detail["route_segments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|segment| segment["name"] == name)
        .unwrap_or_else(|| panic!("route segment {name} not found in {detail}"))
}

async fn create_dated_plan(
    app: &Router,
    token: &str,
    name: &str,
    start_date: &str,
    end_date: Option<&str>,
) -> Value {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/me/trips",
        Some(token),
        json!({
            "trip_type": "team",
            "title": name,
            "start_date": start_date,
            "end_date": end_date,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{value}");
    value
}

#[tokio::test]
async fn trip_home_highlight_selects_current_then_next_trip() {
    let app = test_app().await;
    let owner_token = login(&app.router, "highlight-owner", "队长").await;
    let future_token = login(&app.router, "highlight-future", "山友").await;
    let outsider_token = login(&app.router, "highlight-outsider", "别人").await;

    let (empty_status, empty) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/trips/home-highlight?today=2026-05-27",
        Some(&owner_token),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK, "{empty}");
    assert!(empty["item"].is_null());

    create_dated_plan(
        &app.router,
        &outsider_token,
        "别人的进行中",
        "2026-05-26",
        Some("2026-05-28"),
    )
    .await;
    create_dated_plan(
        &app.router,
        &owner_token,
        "银湖山",
        "2026-05-26",
        Some("2026-05-29"),
    )
    .await;
    create_dated_plan(
        &app.router,
        &owner_token,
        "当天短线",
        "2026-05-25",
        Some("2026-05-27"),
    )
    .await;
    create_dated_plan(
        &app.router,
        &owner_token,
        "周末计划",
        "2026-06-01",
        Some("2026-06-02"),
    )
    .await;

    let (ongoing_status, ongoing) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/trips/home-highlight?today=2026-05-27",
        Some(&owner_token),
    )
    .await;
    assert_eq!(ongoing_status, StatusCode::OK, "{ongoing}");
    assert_eq!(ongoing["item"]["status"], "ongoing");
    assert_eq!(ongoing["item"]["trip"]["title"], "当天短线");
    assert_eq!(ongoing["item"]["days_until_start"], -2);
    assert_eq!(ongoing["item"]["days_until_end"], 0);

    create_dated_plan(
        &app.router,
        &future_token,
        "远期计划",
        "2026-06-05",
        Some("2026-06-06"),
    )
    .await;
    create_dated_plan(&app.router, &future_token, "单日计划", "2026-06-01", None).await;
    let (upcoming_status, upcoming) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/trips/home-highlight?today=2026-05-27",
        Some(&future_token),
    )
    .await;
    assert_eq!(upcoming_status, StatusCode::OK, "{upcoming}");
    assert_eq!(upcoming["item"]["status"], "upcoming");
    assert_eq!(upcoming["item"]["trip"]["title"], "单日计划");
    assert_eq!(upcoming["item"]["days_until_start"], 5);
    assert_eq!(upcoming["item"]["days_until_end"], 5);

    let (invalid_status, invalid) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/trips/home-highlight?today=2026-02-30",
        Some(&owner_token),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::BAD_REQUEST, "{invalid}");
    assert_eq!(invalid["code"], "invalid_query_parameter");

    let (unauthorized_status, _) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/trips/home-highlight?today=2026-05-27",
        None,
    )
    .await;
    assert_eq!(unauthorized_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn trip_defaults_invitation_and_member_access_work() {
    let app = test_app().await;
    let owner_token = login(&app.router, "team-owner", "队长").await;
    let member_token = login(&app.router, "team-member", "队员").await;
    let outsider_token = login(&app.router, "team-outsider", "路人").await;

    let created = create_plan(&app.router, &owner_token).await;
    let plan_id = created["trip"]["id"].as_str().unwrap();
    assert_eq!(
        created["trip"]["enabled_sections"],
        json!(["members", "personal_gear"])
    );
    assert_eq!(created["trip"]["day_count"], 0);
    assert!(created["trip"].get("destination").is_none());
    assert_eq!(created["members"].as_array().unwrap().len(), 1);
    assert!(
        created["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["template_key"] == "common_stove_burner"
                && item["demand_name"] == "炉头"),
        "{created}"
    );
    let template_items = created["shared_gear_demands"].as_array().unwrap();
    assert!(
        template_items.iter().any(|item| {
            item["template_key"] == "common_water_bag" && item["category"] == "kitchen_system"
        }),
        "{created}"
    );
    assert!(
        template_items.iter().any(|item| {
            item["template_key"] == "common_picnic_mat" && item["category"] == "kitchen_system"
        }),
        "{created}"
    );
    assert!(
        template_items
            .iter()
            .any(|item| item["template_key"] == "common_main_carabiner"
                && item["demand_name"] == "主锁"),
        "{created}"
    );
    assert!(
        template_items
            .iter()
            .any(|item| item["template_key"] == "common_aux_rope" && item["demand_name"] == "辅绳"),
        "{created}"
    );
    assert!(
        !template_items
            .iter()
            .any(|item| item["template_key"] == "common_carabiners_aux_rope"),
        "{created}"
    );
    let water_filter_id = template_items
        .iter()
        .find(|item| item["template_key"] == "common_water_filter")
        .and_then(|item| item["id"].as_str())
        .unwrap();
    let (delete_demand_status, deleted_demand_detail) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}/shared-gear-demands/{water_filter_id}"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(
        delete_demand_status,
        StatusCode::OK,
        "{deleted_demand_detail}"
    );
    assert!(
        !deleted_demand_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["template_key"] == "common_water_filter"),
        "{deleted_demand_detail}"
    );
    let (reloaded_demand_status, reloaded_demand_detail) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(
        reloaded_demand_status,
        StatusCode::OK,
        "{reloaded_demand_detail}"
    );
    assert!(
        !reloaded_demand_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["template_key"] == "common_water_filter"),
        "{reloaded_demand_detail}"
    );
    let (invalid_delete_status, invalid_delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}/shared-gear-demands/not_a_demand"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(
        invalid_delete_status,
        StatusCode::NOT_FOUND,
        "{invalid_delete_body}"
    );

    let (list_status, listed) =
        send_empty(&app.router, "GET", "/api/v1/me/trips", Some(&owner_token)).await;
    assert_eq!(list_status, StatusCode::OK, "{listed}");
    assert_eq!(listed["items"][0]["day_count"], 0);

    let (legacy_destination_status, legacy_destination_body) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/trips",
        Some(&owner_token),
        json!({"trip_type": "team", "title": "旧字段计划", "destination": "武功山"}),
    )
    .await;
    assert_eq!(
        legacy_destination_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{legacy_destination_body}"
    );

    let (invite_status, invite) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/invitations"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(invite_status, StatusCode::CREATED, "{invite}");
    let token = invite["invitation"]["token"].as_str().unwrap();

    let (accept_status, accepted) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/trip-invitations/{token}/accept"),
        Some(&member_token),
    )
    .await;
    assert_eq!(accept_status, StatusCode::OK, "{accepted}");
    assert_eq!(accepted["members"].as_array().unwrap().len(), 2);
    let owner_member_id = created["my_member_id"].as_str().unwrap();
    let invited_member_id = accepted["my_member_id"].as_str().unwrap();

    let (self_update_status, self_update) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/members/{invited_member_id}"),
        Some(&member_token),
        json!({
            "age": 36,
            "height_cm": 176,
            "blood_type": "O",
            "base_field_versions": {"age": 0, "height_cm": 0, "blood_type": 0}
        }),
    )
    .await;
    assert_eq!(self_update_status, StatusCode::OK, "{self_update}");
    assert_eq!(self_update["members"][1]["profile"]["age"], 36);
    assert_eq!(self_update["members"][1]["profile"]["height_cm"], 176);
    assert_eq!(self_update["members"][1]["profile"]["blood_type"], "O");

    let (invalid_age_status, invalid_age_body) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/members/{invited_member_id}"),
        Some(&member_token),
        json!({
            "age": 121,
            "base_field_versions": {"age": 1}
        }),
    )
    .await;
    assert_eq!(
        invalid_age_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid_age_body}"
    );
    assert_eq!(invalid_age_body["fields"][0]["field"], "age");

    let (owner_update_status, owner_update) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/members/{invited_member_id}"),
        Some(&owner_token),
        json!({
            "real_name": "王鑫",
            "base_field_versions": {"real_name": 0}
        }),
    )
    .await;
    assert_eq!(owner_update_status, StatusCode::OK, "{owner_update}");
    assert_eq!(owner_update["members"][1]["profile"]["real_name"], "王鑫");

    let (forbidden_status, forbidden_body) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/members/{owner_member_id}"),
        Some(&member_token),
        json!({
            "height_cm": 170,
            "base_field_versions": {"height_cm": 0}
        }),
    )
    .await;
    assert_eq!(
        forbidden_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{forbidden_body}"
    );

    let (member_delete_status, member_delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&member_token),
    )
    .await;
    assert_eq!(
        member_delete_status,
        StatusCode::NOT_FOUND,
        "{member_delete_body}"
    );

    let (outsider_status, outsider_body) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&outsider_token),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND, "{outsider_body}");

    let (owner_delete_status, owner_delete_body) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(
        owner_delete_status,
        StatusCode::NO_CONTENT,
        "{owner_delete_body}"
    );

    let (deleted_status, deleted_body) = send_empty(
        &app.router,
        "GET",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(deleted_status, StatusCode::NOT_FOUND, "{deleted_body}");
}

#[tokio::test]
async fn trip_sections_weights_itinerary_food_and_conflicts_work() {
    let app = test_app().await;
    let token = login(&app.router, "team-rich", "队长").await;
    let created = create_plan(&app.router, &token).await;
    let plan_id = created["trip"]["id"].as_str().unwrap();
    let owner_user_id = created["trip"]["owner_user_id"].as_str().unwrap();
    let my_member_id = created["my_member_id"].as_str().unwrap();

    let (section_status, section_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/sections"),
        Some(&token),
        json!({
            "enabled_sections": [
                "medical_kit",
                "personal_gear",
                "shared_gear",
                "members",
                "itinerary",
                "food_plan",
                "safety_plan",
                "rescue_info",
                "budget",
                "goals"
            ],
            "base_field_versions": {"enabled_sections": 1}
        }),
    )
    .await;
    assert_eq!(section_status, StatusCode::OK, "{section_detail}");
    assert_eq!(
        section_detail["trip"]["enabled_sections"],
        json!([
            "members",
            "personal_gear",
            "medical_kit",
            "shared_gear",
            "itinerary",
            "food_plan",
            "safety_plan",
            "rescue_info",
            "budget",
            "goals"
        ])
    );

    let (route_status, route_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/route-segments"),
        Some(&token),
        json!({
            "name": "金顶到发云界",
            "checkpoint": "发云界营地",
            "leader_member_id": my_member_id,
            "bailout_route": "发云界下撤到龙山村",
            "trail_condition": "草甸土路，雨后湿滑",
            "distance_km": 5.0,
            "ascent_m": 600,
            "descent_m": 0
        }),
    )
    .await;
    assert_eq!(route_status, StatusCode::CREATED, "{route_detail}");
    assert_eq!(
        route_detail["route_segments"][0]["formula_estimate_minutes"],
        120
    );
    assert_eq!(
        route_detail["route_segments"][0]["final_estimate_minutes"],
        120
    );
    assert_eq!(
        route_detail["route_segments"][0]["checkpoint"],
        "发云界营地"
    );
    assert_eq!(
        route_detail["route_segments"][0]["bailout_route"],
        "发云界下撤到龙山村"
    );

    let (day_status, day_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/itinerary-days"),
        Some(&token),
        json!({
            "day_index": 1,
            "title": "上山",
            "weather": "多云",
            "high_temperature_c": 18,
            "low_temperature_c": 6,
            "weather_summary": "可出行，注意防风",
            "camp_name": "发云界营地",
            "camp_altitude_m": 1600,
            "camp_water_source": "溪流"
        }),
    )
    .await;
    assert_eq!(day_status, StatusCode::CREATED, "{day_detail}");
    assert_eq!(day_detail["food_meals"].as_array().unwrap().len(), 3);
    assert_eq!(day_detail["food_meals"][0]["meal_type"], Value::Null);
    assert_eq!(day_detail["itinerary_days"][0]["weather"], "多云");
    assert_eq!(day_detail["itinerary_days"][0]["camp_name"], "发云界营地");

    let (assignment_status, assignment_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/segment-assignments"),
        Some(&token),
        json!({
            "checkpoint": "发云界营地",
            "leader_record_member_id": my_member_id,
            "navigator_safety_member_id": my_member_id,
            "sweeper_member_id": my_member_id
        }),
    )
    .await;
    assert_eq!(
        assignment_status,
        StatusCode::CREATED,
        "{assignment_detail}"
    );
    assert_eq!(
        assignment_detail["segment_assignments"][0]["checkpoint"],
        "发云界营地"
    );

    let (food_supply_status, food_supply_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/food-supplies"),
        Some(&token),
        json!({"name": "盐", "supply_type": "调味品", "amount_g": 100}),
    )
    .await;
    assert_eq!(
        food_supply_status,
        StatusCode::CREATED,
        "{food_supply_detail}"
    );
    assert_eq!(
        food_supply_detail["food_supplies"][0]["supply_type"],
        "调味品"
    );
    assert_eq!(
        food_supply_detail["food_supplies"][0]["total_price_cents"],
        Value::Null
    );
    assert!(
        !food_supply_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "盐"),
        "{food_supply_detail}"
    );

    let meal_id = day_detail["food_meals"][0]["id"].as_str().unwrap();
    let (food_item_status, food_item_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/food-meals/{meal_id}/items"),
        Some(&token),
        json!({
            "name": "米",
            "amount_g": 250,
            "total_price_cents": 1200,
            "responsible_member_id": my_member_id
        }),
    )
    .await;
    assert_eq!(food_item_status, StatusCode::CREATED, "{food_item_detail}");
    assert_eq!(
        food_item_detail["food_meals"][0]["items"][0]["total_price_cents"],
        1200
    );
    let food_item_id = food_item_detail["food_meals"][0]["items"][0]["id"]
        .as_str()
        .unwrap();
    assert!(
        !food_item_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "米"),
        "{food_item_detail}"
    );
    let skipped_version = food_item_detail["food_meals"][0]["field_versions"]["skipped"]
        .as_i64()
        .unwrap_or(0);
    let (skip_status, skip_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/food-meals/{meal_id}"),
        Some(&token),
        json!({
            "skipped": true,
            "base_field_versions": {"skipped": skipped_version}
        }),
    )
    .await;
    assert_eq!(skip_status, StatusCode::OK, "{skip_detail}");
    assert!(
        !skip_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "米"),
        "{skip_detail}"
    );
    let restored_version = skip_detail["food_meals"][0]["field_versions"]["skipped"]
        .as_i64()
        .unwrap_or(0);
    let (restore_status, restore_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/food-meals/{meal_id}"),
        Some(&token),
        json!({
            "skipped": false,
            "base_field_versions": {"skipped": restored_version}
        }),
    )
    .await;
    assert_eq!(restore_status, StatusCode::OK, "{restore_detail}");
    assert!(
        !restore_detail["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "米"),
        "{restore_detail}"
    );
    let (food_item_delete_status, food_item_deleted) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}/food-meals/{meal_id}/items/{food_item_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(
        food_item_delete_status,
        StatusCode::OK,
        "{food_item_deleted}"
    );
    assert!(
        !food_item_deleted["shared_gear_demands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "米"),
        "{food_item_deleted}"
    );

    let (medical_status, medical_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/medical-items"),
        Some(&token),
        json!({
            "name": "弹力绷带",
            "item_type": "外伤",
            "scope": "public_first_aid",
            "suggested_quantity": 2,
            "required_quantity": 2,
            "packed_quantity": 1
        }),
    )
    .await;
    assert_eq!(medical_status, StatusCode::CREATED, "{medical_detail}");
    assert_eq!(medical_detail["medical_items"][0]["item_type"], "外伤");

    let (safety_status, safety_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/safety-risks"),
        Some(&token),
        json!({
            "risk_type": "失温",
            "prevention": "备保暖层",
            "response": "下撤并补充热量",
            "responsible_member_id": my_member_id
        }),
    )
    .await;
    assert_eq!(safety_status, StatusCode::CREATED, "{safety_detail}");
    assert_eq!(safety_detail["safety_risks"][0]["risk_type"], "失温");

    let (rescue_status, rescue_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/rescue-contacts"),
        Some(&token),
        json!({"organization": "龙山派出所", "phone": "110", "address": "龙山镇"}),
    )
    .await;
    assert_eq!(rescue_status, StatusCode::CREATED, "{rescue_detail}");
    assert_eq!(
        rescue_detail["rescue_contacts"][0]["organization"],
        "龙山派出所"
    );

    let (list_status, listed) =
        send_empty(&app.router, "GET", "/api/v1/me/trips", Some(&token)).await;
    assert_eq!(list_status, StatusCode::OK, "{listed}");
    assert_eq!(listed["items"][0]["day_count"], 1);

    let (shared_status, shared_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/shared-gear-demands"),
        Some(&token),
        json!({
            "name": "炉头",
            "template_key": "common_stove_burner",
            "demand_name": "炉头",
            "concrete_name": "火枫炉头",
            "source_gear_id": "gear-source-1",
            "category": "kitchen_system",
            "responsible_member_id": my_member_id,
            "planned_quantity": 2,
            "packed_quantity": 1,
            "unit_weight_g": 300
        }),
    )
    .await;
    assert_eq!(shared_status, StatusCode::CREATED, "{shared_detail}");
    let shared_gear = shared_detail["shared_gear_demands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["template_key"] == "common_stove_burner")
        .unwrap();
    assert_eq!(
        shared_gear["created_by_user_id"].as_str().unwrap(),
        owner_user_id
    );
    assert_eq!(shared_gear["template_key"], "common_stove_burner");
    assert_eq!(shared_gear["concrete_name"], "火枫炉头");
    assert_eq!(shared_gear["source_gear_id"], "gear-source-1");
    let shared_gear_id = shared_gear["id"].as_str().unwrap();
    let (budget_status, budget_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/budget-items"),
        Some(&token),
        json!({
            "category": "装备",
            "name": "炉头分摊",
            "quantity": 2,
            "unit_price_cents": 5000,
            "total_price_cents": 10000,
            "split_member_count": 2,
            "linked_shared_gear_id": shared_gear_id
        }),
    )
    .await;
    assert_eq!(budget_status, StatusCode::CREATED, "{budget_detail}");
    assert_eq!(
        budget_detail["budget_items"][0]["linked_shared_gear_name"],
        "火枫炉头"
    );
    assert_eq!(
        budget_detail["budget_items"][0]["linked_shared_gear_deleted"],
        false
    );

    let (goal_status, goal_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/goals"),
        Some(&token),
        json!({"scope": "team", "content": "安全完成穿越"}),
    )
    .await;
    assert_eq!(goal_status, StatusCode::CREATED, "{goal_detail}");
    assert_eq!(goal_detail["goals"][0]["content"], "安全完成穿越");

    let my_view = shared_detail["member_gear_views"]
        .as_array()
        .unwrap()
        .iter()
        .find(|view| view["member_id"] == my_member_id)
        .unwrap();
    assert_eq!(my_view["all_weight_g"], 600);
    assert_eq!(my_view["actual_weight_g"], 300);
    let stove_view = my_view["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "火枫炉头")
        .unwrap();
    assert_eq!(stove_view["labels"], json!(["公共装备", "我负责"]));

    let (empty_slot_status, empty_slot_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/shared-gear-demands"),
        Some(&token),
        json!({
            "name": "滤水器",
            "template_key": "common_water_filter",
            "demand_name": "滤水器",
            "category": "other_gear",
            "responsible_member_id": my_member_id,
            "planned_quantity": 1
        }),
    )
    .await;
    assert_eq!(
        empty_slot_status,
        StatusCode::CREATED,
        "{empty_slot_detail}"
    );
    let my_view_after_empty = empty_slot_detail["member_gear_views"]
        .as_array()
        .unwrap()
        .iter()
        .find(|view| view["member_id"] == my_member_id)
        .unwrap();
    assert_eq!(my_view_after_empty["all_weight_g"], 600);
    assert!(
        !my_view_after_empty["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["name"] == "滤水器")
    );

    let budget_item_id = budget_detail["budget_items"][0]["id"].as_str().unwrap();
    let (shared_delete_status, shared_deleted_detail) = send_empty(
        &app.router,
        "DELETE",
        &format!("/api/v1/me/trips/{plan_id}/shared-gear-demands/{shared_gear_id}"),
        Some(&token),
    )
    .await;
    assert_eq!(
        shared_delete_status,
        StatusCode::OK,
        "{shared_deleted_detail}"
    );
    assert!(
        shared_deleted_detail["budget_items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == budget_item_id && item["linked_shared_gear_deleted"] == true),
        "{shared_deleted_detail}"
    );

    let (tent_status, tent) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/gears",
        Some(&token),
        json!({"category": "sleep_system", "name": "帐篷", "weight_g": 1200}),
    )
    .await;
    assert_eq!(tent_status, StatusCode::CREATED, "{tent}");
    let tent_id = tent["id"].as_str().unwrap();
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
    let (packing_status, packing) = send_json(
        &app.router,
        "POST",
        "/api/v1/me/packing-lists",
        Some(&token),
        json!({"name": "重装清单"}),
    )
    .await;
    assert_eq!(packing_status, StatusCode::CREATED, "{packing}");
    let packing_id = packing["id"].as_str().unwrap();
    let (packing_add_status, packing_add) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/packing-lists/{packing_id}/items"),
        Some(&token),
        json!({"gear_ids": [tent_id]}),
    )
    .await;
    assert_eq!(packing_add_status, StatusCode::OK, "{packing_add}");
    let (first_import_status, first_import) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/personal-gear/import-packing-list"),
        Some(&token),
        json!({"packing_list_id": packing_id}),
    )
    .await;
    assert_eq!(first_import_status, StatusCode::OK, "{first_import}");
    let (single_status, single_detail) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/personal-gear"),
        Some(&token),
        json!({
            "source_gear_id": headlamp_id,
            "name": "头灯",
            "category": "lighting_system",
            "planned_quantity": 1,
            "packed_quantity": 0,
            "unit_weight_g": 90
        }),
    )
    .await;
    assert_eq!(single_status, StatusCode::CREATED, "{single_detail}");
    let (second_import_status, second_import) = send_json(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/personal-gear/import-packing-list"),
        Some(&token),
        json!({"packing_list_id": packing_id}),
    )
    .await;
    assert_eq!(second_import_status, StatusCode::OK, "{second_import}");
    let personal_gear = second_import["personal_gear"].as_array().unwrap();
    assert!(
        personal_gear
            .iter()
            .any(|item| item["source_gear_id"].as_str() == Some(headlamp_id)),
        "{second_import}"
    );

    let (first_update_status, first_update) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&token),
        json!({"title": "端午重装计划 v2", "base_field_versions": {"title": 1}}),
    )
    .await;
    assert_eq!(first_update_status, StatusCode::OK, "{first_update}");
    assert!(first_update["trip"].get("destination").is_none());

    let (legacy_update_status, legacy_update_body) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&token),
        json!({"destination": "武功山", "base_field_versions": {"destination": 0}}),
    )
    .await;
    assert_eq!(
        legacy_update_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{legacy_update_body}"
    );

    let (conflict_status, conflict) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&token),
        json!({"title": "本地旧版本", "base_field_versions": {"title": 1}}),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT, "{conflict}");
    assert_eq!(conflict["code"], "edit_conflict");
    assert_eq!(conflict["conflicts"][0]["field"], "title");
}

#[tokio::test]
async fn trip_route_estimation_settings_recalculate_segments() {
    let app = test_app().await;
    let owner_token = login(&app.router, "team-route-owner", "队长").await;
    let member_token = login(&app.router, "team-route-member", "队员").await;
    let created = create_plan(&app.router, &owner_token).await;
    let plan_id = created["trip"]["id"].as_str().unwrap();
    assert_eq!(created["trip"]["route_use_slope_adjustment"], false);
    assert_eq!(created["trip"]["route_use_high_altitude_adjustment"], false);
    assert!(created["trip"]["route_start_altitude_m"].is_null());

    let (section_status, section_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}/sections"),
        Some(&owner_token),
        json!({
            "enabled_sections": ["members", "personal_gear", "itinerary"],
            "base_field_versions": {"enabled_sections": 1}
        }),
    )
    .await;
    assert_eq!(section_status, StatusCode::OK, "{section_detail}");

    let route_detail = create_route_segment(
        &app.router,
        &owner_token,
        plan_id,
        "垭口上下",
        5.0,
        300,
        900,
    )
    .await;
    let valley_segment = route_segment_by_name(&route_detail, "垭口上下");
    assert_eq!(valley_segment["formula_estimate_minutes"], 90);
    assert_eq!(valley_segment["high_altitude_factor"], 1.0);
    assert!(
        valley_segment["estimated_start_altitude_m"].is_null(),
        "{route_detail}"
    );

    let route_detail =
        create_route_segment(&app.router, &owner_token, plan_id, "缓坡上升", 5.0, 300, 0).await;
    assert_eq!(
        route_segment_by_name(&route_detail, "缓坡上升")["formula_estimate_minutes"],
        90
    );

    let route_detail =
        create_route_segment(&app.router, &owner_token, plan_id, "陡上坡", 2.0, 600, 0).await;
    assert_eq!(
        route_segment_by_name(&route_detail, "陡上坡")["formula_estimate_minutes"],
        85
    );

    let route_detail =
        create_route_segment(&app.router, &owner_token, plan_id, "陡下坡", 2.0, 0, 600).await;
    assert_eq!(
        route_segment_by_name(&route_detail, "陡下坡")["formula_estimate_minutes"],
        25
    );

    let route_detail = create_route_segment(
        &app.router,
        &owner_token,
        plan_id,
        "零距离爬升",
        0.0,
        100,
        0,
    )
    .await;
    assert_eq!(
        route_segment_by_name(&route_detail, "零距离爬升")["formula_estimate_minutes"],
        10
    );

    let (missing_altitude_status, missing_altitude_body) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
        json!({
            "route_use_high_altitude_adjustment": true,
            "base_field_versions": {"route_use_high_altitude_adjustment": 1}
        }),
    )
    .await;
    assert_eq!(
        missing_altitude_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{missing_altitude_body}"
    );
    assert_eq!(
        missing_altitude_body["fields"][0]["field"],
        "route_start_altitude_m"
    );

    let (slope_status, slope_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
        json!({
            "route_use_slope_adjustment": true,
            "base_field_versions": {"route_use_slope_adjustment": 1}
        }),
    )
    .await;
    assert_eq!(slope_status, StatusCode::OK, "{slope_detail}");
    assert_eq!(slope_detail["trip"]["route_use_slope_adjustment"], true);
    let valley_segment = route_segment_by_name(&slope_detail, "垭口上下");
    assert_eq!(valley_segment["formula_estimate_minutes"], 150);
    assert_eq!(valley_segment["high_altitude_factor"], 1.0);
    assert_eq!(
        route_segment_by_name(&slope_detail, "缓坡上升")["formula_estimate_minutes"],
        90
    );
    assert_eq!(
        route_segment_by_name(&slope_detail, "陡上坡")["formula_estimate_minutes"],
        145
    );
    assert_eq!(
        route_segment_by_name(&slope_detail, "陡下坡")["formula_estimate_minutes"],
        85
    );
    assert_eq!(
        route_segment_by_name(&slope_detail, "零距离爬升")["formula_estimate_minutes"],
        20
    );

    let (altitude_status, altitude_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
        json!({
            "route_use_high_altitude_adjustment": true,
            "route_start_altitude_m": 2300,
            "base_field_versions": {
                "route_use_high_altitude_adjustment": 1,
                "route_start_altitude_m": 1
            }
        }),
    )
    .await;
    assert_eq!(altitude_status, StatusCode::OK, "{altitude_detail}");
    assert_eq!(
        altitude_detail["trip"]["route_use_high_altitude_adjustment"],
        true
    );
    assert_eq!(altitude_detail["trip"]["route_start_altitude_m"], 2300);
    let valley_segment = route_segment_by_name(&altitude_detail, "垭口上下");
    assert_eq!(valley_segment["formula_estimate_minutes"], 165);
    assert_eq!(valley_segment["estimated_start_altitude_m"], 2300);
    assert_eq!(valley_segment["estimated_end_altitude_m"], 1700);
    assert_eq!(valley_segment["estimated_highest_altitude_m"], 2600);
    assert_eq!(valley_segment["high_altitude_factor"], 1.1);
    assert_eq!(
        route_segment_by_name(&altitude_detail, "陡上坡")["formula_estimate_minutes"],
        160
    );
    assert_eq!(
        route_segment_by_name(&altitude_detail, "陡下坡")["formula_estimate_minutes"],
        85
    );

    let (naismith_altitude_status, naismith_altitude_detail) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&owner_token),
        json!({
            "route_use_slope_adjustment": false,
            "base_field_versions": {"route_use_slope_adjustment": 2}
        }),
    )
    .await;
    assert_eq!(
        naismith_altitude_status,
        StatusCode::OK,
        "{naismith_altitude_detail}"
    );
    assert_eq!(
        route_segment_by_name(&naismith_altitude_detail, "垭口上下")["formula_estimate_minutes"],
        100
    );
    assert_eq!(
        route_segment_by_name(&naismith_altitude_detail, "垭口上下")["high_altitude_factor"],
        1.1
    );

    let (invite_status, invite) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/trips/{plan_id}/invitations"),
        Some(&owner_token),
    )
    .await;
    assert_eq!(invite_status, StatusCode::CREATED, "{invite}");
    let token = invite["invitation"]["token"].as_str().unwrap();
    let (accept_status, accepted) = send_empty(
        &app.router,
        "POST",
        &format!("/api/v1/me/trip-invitations/{token}/accept"),
        Some(&member_token),
    )
    .await;
    assert_eq!(accept_status, StatusCode::OK, "{accepted}");

    let (member_patch_status, member_patch_body) = send_json(
        &app.router,
        "PATCH",
        &format!("/api/v1/me/trips/{plan_id}"),
        Some(&member_token),
        json!({
            "route_use_slope_adjustment": true,
            "base_field_versions": {"route_use_slope_adjustment": 3}
        }),
    )
    .await;
    assert_eq!(
        member_patch_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{member_patch_body}"
    );
}
