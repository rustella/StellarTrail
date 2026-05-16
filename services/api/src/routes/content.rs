//! 公共内容路由模块，读取种子内容目录并暴露山峰、路线、技能和装备模板接口。

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use stellartrail_importer::{GearTemplate, MountainContent, RouteContent, SkillContent};

use crate::{error::ApiError, state::AppState};

/// ListResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
}

/// 执行 `routes` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/mountains", get(list_mountains))
        .route("/api/mountains/:id", get(get_mountain))
        .route("/api/routes", get(list_routes))
        .route("/api/routes/:id", get(get_route))
        .route("/api/skills", get(list_skills))
        .route("/api/skills/:id", get(get_skill))
        .route("/api/gear-templates", get(list_gear_templates))
        .route("/api/gear-templates/:id", get(get_gear_template))
}

/// 执行 `list mountains` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn list_mountains(State(state): State<AppState>) -> Json<ListResponse<MountainContent>> {
    Json(ListResponse {
        items: state.content().mountains.clone(),
    })
}

/// 执行 `get mountain` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn get_mountain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MountainContent>, ApiError> {
    state
        .content()
        .mountains
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// 执行 `list routes` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn list_routes(State(state): State<AppState>) -> Json<ListResponse<RouteContent>> {
    Json(ListResponse {
        items: state.content().routes.clone(),
    })
}

/// 执行 `get route` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RouteContent>, ApiError> {
    state
        .content()
        .routes
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// 执行 `list skills` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn list_skills(State(state): State<AppState>) -> Json<ListResponse<SkillContent>> {
    Json(ListResponse {
        items: state.content().skills.clone(),
    })
}

/// 执行 `get skill` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn get_skill(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SkillContent>, ApiError> {
    state
        .content()
        .skills
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// 执行 `list gear templates` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn list_gear_templates(State(state): State<AppState>) -> Json<ListResponse<GearTemplate>> {
    Json(ListResponse {
        items: state.content().gear_templates.clone(),
    })
}

/// 执行 `get gear template` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn get_gear_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GearTemplate>, ApiError> {
    state
        .content()
        .gear_templates
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}
