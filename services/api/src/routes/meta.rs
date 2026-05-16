//! Metadata route module that returns the API version and current environment for frontend and diagnostic tools.

use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

/// Stable data boundary for `MetaResponse`, exposed by or reused within this module.
#[derive(Serialize)]
pub struct MetaResponse {
    name: &'static str,
    env: String,
    database_kind: String,
}

/// Runs the `meta` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn meta(State(state): State<AppState>) -> Json<MetaResponse> {
    Json(MetaResponse {
        name: "StellarTrail",
        env: state.config().app_env.clone(),
        database_kind: state.config().database_kind().as_str().to_owned(),
    })
}
