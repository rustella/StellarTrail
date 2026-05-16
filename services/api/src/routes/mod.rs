//! Route aggregation module that combines all API subroutes and provides a consistent 404 fallback.

mod auth;
mod content;
mod feedback;
mod gears;
mod health;
mod meta;
mod uploads;

use axum::{Router, extract::DefaultBodyLimit, routing::get};

use crate::error::ApiError;
use crate::state::AppState;

/// Combines all business routes, health checks, and the 404 fallback.
pub fn build_router(state: AppState) -> Router {
    let body_limit = state
        .config()
        .upload
        .max_image_bytes
        .saturating_add(1_000_000) as usize;
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(content::routes())
        .merge(gears::routes())
        .merge(uploads::routes())
        .merge(feedback::routes())
        .layer(DefaultBodyLimit::max(body_limit))
        .fallback(not_found)
        .with_state(state)
}

/// Runs the `not found` server-side flow while preserving input validation, error propagation, and state invariants.
async fn not_found() -> ApiError {
    ApiError::NotFound
}
