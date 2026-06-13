//! Public map configuration and hosted map style JSON routes.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};

use crate::{dto::trail::MapConfigResponse, error::ApiError, state::AppState};

const MAP_STYLE_ROUTE_PREFIX: &str = "/api/v1/map/styles/";
const MAP_STYLE_ROUTE_SUFFIX: &str = "/style.json";

/// Builds public map configuration and hosted style JSON routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/map/config", get(map_config))
        .route("/map/styles/:style_id/style.json", get(map_style_json))
}

/// Returns true for hosted map style JSON requests that the SDK loads without API headers.
pub(crate) fn is_map_style_json_path(path: &str) -> bool {
    path.starts_with(MAP_STYLE_ROUTE_PREFIX) && path.ends_with(MAP_STYLE_ROUTE_SUFFIX)
}

/// Builds the client-visible map configuration using the current public API origin.
pub(crate) fn map_config_response(state: &AppState, headers: &HeaderMap) -> MapConfigResponse {
    let origin = public_map_origin(state, headers);
    MapConfigResponse::from_config(&state.config().map, &origin)
}

async fn map_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MapConfigResponse>, ApiError> {
    Ok(Json(map_config_response(&state, &headers)))
}

async fn map_style_json(
    State(state): State<AppState>,
    Path(style_id): Path<String>,
) -> Result<Response, ApiError> {
    if !state
        .config()
        .map
        .styles
        .iter()
        .any(|style| style.id == style_id)
    {
        return Err(ApiError::NotFound);
    }

    let Some(style_json) = state.map_style_cache().style_json(&style_id) else {
        return Err(ApiError::ServiceUnavailable {
            code: "map_style_unavailable",
            message: "map style cache is not ready".to_owned(),
        });
    };

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        style_json,
    )
        .into_response())
}

fn public_map_origin(state: &AppState, headers: &HeaderMap) -> String {
    if let Some(origin) = state.config().map.public_api_origin.as_deref() {
        return origin.trim_end_matches('/').to_owned();
    }

    let trust_proxy = state.config().public_api.trust_proxy_headers;
    let host = if trust_proxy {
        first_header_value(headers, "x-forwarded-host")
            .or_else(|| first_header_value(headers, "host"))
    } else {
        first_header_value(headers, "host")
    };
    let scheme = if trust_proxy {
        first_header_value(headers, "x-forwarded-proto").unwrap_or_else(|| "http".to_owned())
    } else {
        "http".to_owned()
    };

    host.map(|host| format!("{}://{}", scheme.trim(), host.trim()))
        .unwrap_or_else(|| format!("http://{}", state.config().bind_addr()))
}

fn first_header_value(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
