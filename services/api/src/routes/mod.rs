mod auth;
mod gears;
mod health;
mod meta;

use axum::{Router, routing::get};

use crate::error::ApiError;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(gears::routes())
        .fallback(not_found)
        .with_state(state)
}

async fn not_found() -> ApiError {
    ApiError::NotFound
}
