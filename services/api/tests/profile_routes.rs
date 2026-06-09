//! Integration tests for authenticated profile and outdoor profile routes.

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
        .header("X-StellarTrail-Client", "web/test")
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
    let mut builder = Request::builder()
        .header("X-StellarTrail-Client", "web/test")
        .method(method)
        .uri(path);
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

async fn login(app: &Router, code: &str, nickname: &str) -> (String, String) {
    let (status, value) = send_json(
        app,
        "POST",
        "/api/v1/auth/wechat-login",
        None,
        json!({"code": code, "profile": {"nickname": nickname, "avatar_url": null}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    (
        value["access_token"].as_str().unwrap().to_owned(),
        value["user"]["id"].as_str().unwrap().to_owned(),
    )
}

#[tokio::test]
async fn outdoor_profile_can_be_created_read_updated_and_cleared() {
    let app = test_app().await;
    let (token, user_id) = login(&app.router, "profile-outdoor", "山友").await;

    let (unauth_status, unauth_body) =
        send_empty(&app.router, "GET", "/api/v1/me/profile/outdoor", None).await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED, "{unauth_body}");

    let (empty_status, empty) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/profile/outdoor",
        Some(&token),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK, "{empty}");
    assert_eq!(empty["profile"]["user_id"], user_id);
    assert!(empty["profile"]["birth_date"].is_null());
    assert!(empty["profile"]["height_cm"].is_null());
    assert!(empty["profile"]["updated_at"].is_null());

    let (update_status, updated) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({
            "outdoor_id": " 星星 ",
            "real_name": "王鑫",
            "gender": "男",
            "birth_date": "1990-05-27",
            "height_cm": 176,
            "phone": "15696331949",
            "emergency_contact": "吕荟琪",
            "emergency_contact_relationship": "家属",
            "emergency_phone": "18976951563",
            "blood_type": "O",
            "medical_history": "无",
            "allergy_history": "无",
            "medical_response_note": "无特殊处置",
            "diet_preference": "不吃牛羊肉",
            "insurance_policy_no": "11209616600972792644",
            "insurance_company_phone": "95500",
            "experience_note": "四姑娘山三峰，贡嘎环线等"
        }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK, "{updated}");
    assert_eq!(updated["profile"]["outdoor_id"], "星星");
    assert_eq!(updated["profile"]["birth_date"], "1990-05-27");
    assert_eq!(updated["profile"]["height_cm"], 176);
    assert_eq!(updated["profile"]["emergency_contact_relationship"], "家属");
    assert_eq!(updated["profile"]["medical_response_note"], "无特殊处置");
    assert_eq!(updated["profile"]["diet_preference"], "不吃牛羊肉");
    assert_eq!(updated["profile"]["insurance_company_phone"], "95500");
    assert!(updated["profile"]["created_at"].is_string());
    assert!(updated["profile"]["updated_at"].is_string());

    let (clear_status, cleared) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({
            "phone": null,
            "birth_date": null,
            "height_cm": null
        }),
    )
    .await;
    assert_eq!(clear_status, StatusCode::OK, "{cleared}");
    assert!(cleared["profile"]["phone"].is_null());
    assert!(cleared["profile"]["birth_date"].is_null());
    assert!(cleared["profile"]["height_cm"].is_null());
    assert_eq!(cleared["profile"]["blood_type"], "O");

    let (read_status, read) = send_empty(
        &app.router,
        "GET",
        "/api/v1/me/profile/outdoor",
        Some(&token),
    )
    .await;
    assert_eq!(read_status, StatusCode::OK, "{read}");
    assert_eq!(read["profile"]["blood_type"], "O");
    assert!(read["profile"]["birth_date"].is_null());
    assert!(read["profile"]["phone"].is_null());
}

#[tokio::test]
async fn outdoor_profile_rejects_invalid_fields() {
    let app = test_app().await;
    let (token, _) = login(&app.router, "profile-invalid", "山友").await;

    let (height_status, height_body) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({"height_cm": 300}),
    )
    .await;
    assert_eq!(
        height_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{height_body}"
    );
    assert_eq!(height_body["fields"][0]["field"], "height_cm");

    let (birth_status, birth_body) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({"birth_date": "2999-01-01"}),
    )
    .await;
    assert_eq!(
        birth_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{birth_body}"
    );
    assert_eq!(birth_body["fields"][0]["field"], "birth_date");

    let (invalid_birth_status, invalid_birth_body) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({"birth_date": "1990-02-31"}),
    )
    .await;
    assert_eq!(
        invalid_birth_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{invalid_birth_body}"
    );
    assert_eq!(invalid_birth_body["fields"][0]["field"], "birth_date");

    let (unknown_status, unknown_body) = send_json(
        &app.router,
        "PATCH",
        "/api/v1/me/profile/outdoor",
        Some(&token),
        json!({"role_label": "领队"}),
    )
    .await;
    assert_eq!(
        unknown_status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{unknown_body}"
    );
    assert_eq!(unknown_body["fields"][0]["field"], "role_label");
}
