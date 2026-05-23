//! Route aggregation module that combines all API subroutes and provides a consistent 404 fallback.

mod admin_api_usage;
mod admin_knots;
mod admin_roles;
mod auth;
mod client_versions;
mod content;
mod feedback;
mod gear_atlas;
mod gear_packing;
mod gears;
mod health;
mod localization;
mod meta;
mod profile;
mod skills;
mod uploads;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{
        HeaderName, HeaderValue, Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware,
    routing::get,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{
    api_usage, config::CorsConfig, error::ApiError, services::rate_limit_service, state::AppState,
};

/// Versioned prefix for every business API route.
pub const API_PREFIX: &str = "/api/v1";

/// Versioned prefix with a trailing separator for route-template matching.
pub const API_PREFIX_WITH_SLASH: &str = "/api/v1/";

/// Versioned captcha endpoint returned in authentication error envelopes.
pub const CAPTCHA_ENDPOINT: &str = "/api/v1/auth/captcha";

/// Builds a versioned API path for response payloads that expose relative links.
pub fn api_path(path: impl AsRef<str>) -> String {
    let path = path.as_ref();
    if path.starts_with('/') {
        format!("{API_PREFIX}{path}")
    } else {
        format!("{API_PREFIX}/{path}")
    }
}

/// Combines all business routes, health checks, and the 404 fallback.
pub fn build_router(state: AppState) -> Router {
    let body_limit = state
        .config()
        .upload
        .max_image_bytes
        .max(state.config().avatar_storage.max_image_bytes)
        .max(state.config().knots_media_storage.max_video_bytes)
        .saturating_add(1_000_000) as usize;
    let cors_layer = build_cors_layer(&state.config().cors);
    let usage_state = state.clone();
    let rate_limit_layer = axum::middleware::from_fn_with_state(
        state.clone(),
        rate_limit_service::enforce_global_rate_limit,
    );
    let api_router = Router::new()
        .route("/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(admin_api_usage::routes())
        .merge(admin_knots::routes())
        .merge(admin_roles::routes())
        .merge(client_versions::routes())
        .merge(content::routes())
        .merge(skills::routes())
        .merge(gear_atlas::routes())
        .merge(gear_packing::routes())
        .merge(gears::routes())
        .merge(profile::routes())
        .merge(uploads::routes())
        .merge(feedback::routes());
    Router::new()
        .route("/healthz", get(health::healthz))
        .nest(API_PREFIX, api_router)
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(rate_limit_layer)
        .route_layer(middleware::from_fn_with_state(
            usage_state,
            api_usage::track_api_usage,
        ))
        .layer(cors_layer)
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
