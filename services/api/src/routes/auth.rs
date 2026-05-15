use axum::{Json, Router, routing::post};

use crate::{
    dto::auth::{LoginResponse, WechatLoginRequest},
    error::ApiError,
    services::auth_service,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/auth/wechat-login", post(wechat_login))
}

async fn wechat_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<WechatLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::mock_login(&state, payload.code, payload.profile).await?;
    Ok(Json(response))
}
