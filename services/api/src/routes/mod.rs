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

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        HeaderName, HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};

use crate::config::CorsConfig;
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
    let cors_layer = build_cors_layer(&state.config().cors);
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
        .layer(cors_layer)
        .fallback(not_found)
        .with_state(state)
}

fn build_cors_layer(config: &CorsConfig) -> CorsLayer {
    let allowed_origins = config
        .allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).expect("validated CORS origin"))
        .collect::<Vec<_>>();
    let mut layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-stellartrail-locale"),
        ]);
    if !allowed_origins.is_empty() {
        layer = layer.allow_origin(AllowOrigin::list(allowed_origins));
    }
    if config.allow_credentials {
        layer = layer.allow_credentials(true);
    }
    layer
}

/// Runs the `not found` server-side flow while preserving input validation, error propagation, and state invariants.
async fn not_found() -> ApiError {
    ApiError::NotFound
}
