mod health;
mod meta;
mod skills;

use axum::{routing::get, Router};
use tower_http::services::ServeDir;

use crate::error::ApiError;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let assets_dir = state.config().content_assets_dir.clone();

    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .route("/api/skills", get(skills::skill_categories))
        .route("/api/skills/knots/list", get(skills::knot_list))
        .route("/api/skills/knots/detail/:id", get(skills::knot_detail))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .fallback(not_found)
        .with_state(state)
}

async fn not_found() -> ApiError {
    ApiError::NotFound
}
