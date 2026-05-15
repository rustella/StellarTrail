use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct MetaResponse {
    name: &'static str,
    env: String,
    database_kind: String,
}

pub async fn meta(State(state): State<AppState>) -> Json<MetaResponse> {
    Json(MetaResponse {
        name: "StellarTrail",
        env: state.config().app_env.clone(),
        database_kind: state.config().database_kind().as_str().to_owned(),
    })
}
