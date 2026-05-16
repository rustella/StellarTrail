use std::{
    fs,
    path::{Path, PathBuf},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use stellartrail_api::{build_state, config::ApiConfig, routes::build_router};
use stellartrail_db::DatabaseConfig;
use tempfile::TempDir;
use tower::ServiceExt;

struct TestApp {
    router: Router,
    _temp_dir: TempDir,
}

async fn test_app() -> TestApp {
    let temp_dir = tempfile::tempdir().unwrap();
    let content_dir = temp_dir.path().join("content");
    write_sample_content(&content_dir);
    test_app_with_content_dir(temp_dir, content_dir).await
}

async fn test_app_with_content_dir(temp_dir: TempDir, content_dir: PathBuf) -> TestApp {
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
        content_dir,
    };
    let state = build_state(config).await.unwrap();
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
async fn public_content_routes_expose_seed_catalog() {
    let app = test_app().await;

    let (status, mountains) = get_json(&app.router, "/api/mountains").await;
    assert_eq!(status, StatusCode::OK, "{mountains}");
    assert_eq!(mountains["items"].as_array().unwrap().len(), 1);
    assert_eq!(mountains["items"][0]["id"], "wugongshan");
    assert_eq!(mountains["items"][0]["difficulty_level"], "beginner");

    let (status, route) = get_json(&app.router, "/api/routes/wugongshan-classic-2d1n").await;
    assert_eq!(status, StatusCode::OK, "{route}");
    assert_eq!(route["mountain_id"], "wugongshan");
    assert_eq!(route["points"][0]["type"], "start");
    assert_eq!(route["gear_suggestions"][0]["gear_name"], "雨衣或硬壳");
    assert_eq!(route["skill_links"][0]["skill_id"], "taut-line-hitch");

    let (status, skill) = get_json(&app.router, "/api/skills/taut-line-hitch").await;
    assert_eq!(status, StatusCode::OK, "{skill}");
    assert_eq!(skill["category"], "knot");
    assert!(skill["body_markdown"].as_str().unwrap().contains("## 步骤"));

    let (status, template) = get_json(&app.router, "/api/gear-templates/backpacking-basic").await;
    assert_eq!(status, StatusCode::OK, "{template}");
    assert_eq!(template["categories"][0]["items"][0], "雨衣或硬壳");
}

#[tokio::test]
async fn public_content_routes_parse_repository_seed_content() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let app = test_app_with_content_dir(temp_dir, content_dir).await;

    let (status, routes) = get_json(&app.router, "/api/routes").await;

    assert_eq!(status, StatusCode::OK, "{routes}");
    assert!(
        routes["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["id"] == "wugongshan-classic-2d1n")
    );
}

#[tokio::test]
async fn public_content_routes_return_not_found_for_unknown_ids() {
    let app = test_app().await;

    let (status, body) = get_json(&app.router, "/api/routes/missing-route").await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["code"], "not_found");
}

fn write_sample_content(root: &Path) {
    fs::create_dir_all(root.join("mountains")).unwrap();
    fs::create_dir_all(root.join("routes")).unwrap();
    fs::create_dir_all(root.join("skills/knots")).unwrap();
    fs::create_dir_all(root.join("gear-templates")).unwrap();

    fs::write(
        root.join("mountains/wugongshan.yaml"),
        r#"id: wugongshan
name: 武功山
aliases: [萍乡武功山]
province: 江西
city: 萍乡
area: 芦溪县
elevation_m: 1918
lat: 27.471
lng: 114.175
summary: 华东经典高山草甸徒步目的地。
best_seasons: [春, 夏, 秋]
difficulty_level: beginner
status: draft
"#,
    )
    .unwrap();

    fs::write(
        root.join("routes/wugongshan-classic-2d1n.yaml"),
        r#"id: wugongshan-classic-2d1n
mountain_id: wugongshan
title: 武功山经典 2 天 1 夜穿越
province: 江西
city: 萍乡
route_type: backpacking
difficulty_level: beginner
distance_m: 18000
ascent_m: 1200
descent_m: 1200
duration_min: 900
best_seasons: [春, 夏, 秋]
summary: 适合有基础体能的新手入门。
transport_info: 可从萍乡方向抵达。
permit_info: 关注景区公告。
risk_summary: 山脊风大。
status: draft
points:
  - type: start
    name: 徒步起点
    description: 出发前确认交通和天气。
    sort_order: 1
gear_suggestions:
  - gear_category: rain_protection
    gear_name: 雨衣或硬壳
    required_level: required
    reason: 山区天气变化快。
skill_links:
  - skill_id: taut-line-hitch
    reason: 风绳调节常用。
"#,
    )
    .unwrap();

    fs::write(
        root.join("skills/knots/taut-line-hitch.md"),
        r#"---
id: taut-line-hitch
title: 可调节帐绳结
category: knot
difficulty_level: beginner
summary: 常用于帐篷和天幕风绳张力调节。
related_gear_categories: [tent, tarp, guyline]
---

# 可调节帐绳结

## 步骤

1. 绳子绕过固定点。
"#,
    )
    .unwrap();

    fs::write(
        root.join("gear-templates/backpacking-basic.yaml"),
        r#"id: backpacking-basic
title: 入门徒步基础装备模板
categories:
  - id: rain_protection
    name: 防雨防风
    items:
      - 雨衣或硬壳
      - 背包防雨罩
"#,
    )
    .unwrap();
}
