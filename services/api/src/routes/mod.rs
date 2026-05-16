//! Route aggregation module that combines all API subroutes and provides a consistent 404 fallback.

mod auth;
mod content;
mod gears;
mod health;
mod meta;

use axum::{Router, routing::get};

use crate::error::ApiError;
use crate::state::AppState;

/// Combines all business routes, health checks, and the 404 fallback.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(content::routes())
        .merge(gears::routes())
        .fallback(not_found)
        .with_state(state)
}

/// Runs the `not found` server-side flow while preserving input validation, error propagation, and state invariants.
async fn not_found() -> ApiError {
    ApiError::NotFound
}
