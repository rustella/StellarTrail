use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stellartrail_api::{
    config::{
        ApiConfig, CorsConfig, RedisCacheConfig, RequestSignatureClientConfig,
        RequestSignatureConfig,
    },
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
    test_app_with_request_signature(Default::default()).await
}

async fn test_app_with_request_signature(request_signature: RequestSignatureConfig) -> TestApp {
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
        request_signature,
        cors: CorsConfig::default(),
        mail: Default::default(),
        sms: Default::default(),
    };
    TestApp {
        router: build_router(AppState::new(config, db)),
        _temp_dir: temp_dir,
    }
}

async fn send_json(app: &Router, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
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

#[tokio::test]
async fn request_signature_exempts_healthz_and_options() {
    let app = test_app_with_request_signature(signature_config()).await;

    let (health_status, health) = send_empty(&app.router, "GET", "/healthz", None).await;
    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(health["status"], "ok");

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/meta")
                .header(header::ORIGIN, "https://app.example.invalid")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn request_signature_rejects_unsigned_api_request() {
    let app = test_app_with_request_signature(signature_config()).await;

    let (status, body) = send_empty(&app.router, "GET", "/api/v1/meta", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "invalid_request_signature");
}

#[tokio::test]
async fn request_signature_accepts_signed_query_request_and_rejects_replay() {
    let app = test_app_with_request_signature(signature_config()).await;
    let path = signed_get_path("/api/v1/meta", "nonce-query-1");

    let (first_status, first_body) = send_empty(&app.router, "GET", &path, None).await;
    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(first_body["name"], "StellarTrail");

    let (second_status, second_body) = send_empty(&app.router, "GET", &path, None).await;
    assert_eq!(second_status, StatusCode::UNAUTHORIZED);
    assert_eq!(second_body["code"], "invalid_request_signature");
}

#[tokio::test]
async fn request_signature_accepts_signed_json_without_breaking_dto_parsing() {
    let app = test_app_with_request_signature(signature_config()).await;
    let body = signed_json_body(
        "/api/v1/auth/captcha",
        "nonce-json-1",
        json!({ "account": "alice@example.test" }),
    );

    let (status, response) = send_json(&app.router, "/api/v1/auth/captcha", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["captcha_type"], "image");
    assert!(response["captcha_ticket"].as_str().is_some());
}

fn signature_config() -> RequestSignatureConfig {
    RequestSignatureConfig {
        enabled: true,
        nonce_ttl_seconds: 300,
        clients: vec![RequestSignatureClientConfig {
            app_id: "test-client".to_owned(),
            app_secret: "test-secret".to_owned(),
        }],
    }
}

fn signed_get_path(path: &str, nonce: &str) -> String {
    let canonical_query = format!("app_id=test-client&nonce={nonce}");
    let canonical = canonical_request(
        "GET",
        path,
        &canonical_query,
        &sha256_hex(b""),
        "test-client",
        nonce,
    );
    let signature = hmac_sha256_hex("test-secret", &canonical);
    format!("{path}?{canonical_query}&signature={signature}")
}

fn signed_json_body(path: &str, nonce: &str, mut body: Value) -> Value {
    let body_hash = stable_json_sha256_hex(&body);
    let canonical = canonical_request("POST", path, "", &body_hash, "test-client", nonce);
    let signature = hmac_sha256_hex("test-secret", &canonical);
    let object = body.as_object_mut().expect("test body must be an object");
    object.insert("app_id".to_owned(), Value::String("test-client".to_owned()));
    object.insert("nonce".to_owned(), Value::String(nonce.to_owned()));
    object.insert("signature".to_owned(), Value::String(signature));
    body
}

fn canonical_request(
    method: &str,
    path: &str,
    canonical_query: &str,
    body_hash_hex: &str,
    app_id: &str,
    nonce: &str,
) -> String {
    [
        "STELLARTRAIL-HMAC-SHA256",
        method,
        path,
        canonical_query,
        body_hash_hex,
        app_id,
        nonce,
    ]
    .join("\n")
}

fn hmac_sha256_hex(secret: &str, canonical: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(canonical.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn stable_json_sha256_hex(value: &Value) -> String {
    let mut canonical = String::new();
    write_canonical_json(value, &mut canonical);
    sha256_hex(canonical.as_bytes())
}

fn write_canonical_json(value: &Value, output: &mut String) {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            output.push_str(&serde_json::to_string(value).unwrap());
        }
        Value::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(item, output);
            }
            output.push(']');
        }
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            output.push('{');
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).unwrap());
                output.push(':');
                write_canonical_json(item, output);
            }
            output.push('}');
        }
    }
}

fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes.as_ref()))
}
