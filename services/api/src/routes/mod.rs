//! Route aggregation module that combines all API subroutes and provides a consistent 404 fallback.

mod admin_knots;
mod auth;
mod content;
mod feedback;
mod gears;
mod health;
mod meta;
mod skills;
mod uploads;

use axum::{Router, extract::DefaultBodyLimit, routing::get};
use tower_http::services::ServeDir;

use crate::error::ApiError;
use crate::state::AppState;

/// Combines all business routes, health checks, static assets, and the 404 fallback.
pub fn build_router(state: AppState) -> Router {
    let assets_dir = state.config().content_assets_dir.clone();
    let body_limit = state
        .config()
        .upload
        .max_image_bytes
        .max(state.config().knots_media_storage.max_video_bytes)
        .saturating_add(1_000_000) as usize;
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(admin_knots::routes())
        .merge(content::routes())
        .merge(skills::routes())
        .merge(gears::routes())
        .merge(uploads::routes())
        .merge(feedback::routes())
        .nest_service("/assets", ServeDir::new(assets_dir))
        .layer(DefaultBodyLimit::max(body_limit))
        .fallback(not_found)
        .with_state(state)
}

/// Runs the `not found` server-side flow while preserving input validation, error propagation, and state invariants.
async fn not_found() -> ApiError {
    ApiError::NotFound
}
