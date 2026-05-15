use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::Serialize;
use stellartrail_importer::{GearTemplate, MountainContent, RouteContent, SkillContent};

use crate::{error::ApiError, state::AppState};

#[derive(Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
}

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

async fn list_mountains(State(state): State<AppState>) -> Json<ListResponse<MountainContent>> {
    Json(ListResponse {
        items: state.content().mountains.clone(),
    })
}

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

async fn list_routes(State(state): State<AppState>) -> Json<ListResponse<RouteContent>> {
    Json(ListResponse {
        items: state.content().routes.clone(),
    })
}

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

async fn list_skills(State(state): State<AppState>) -> Json<ListResponse<SkillContent>> {
    Json(ListResponse {
        items: state.content().skills.clone(),
    })
}

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

async fn list_gear_templates(State(state): State<AppState>) -> Json<ListResponse<GearTemplate>> {
    Json(ListResponse {
        items: state.content().gear_templates.clone(),
    })
}

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
