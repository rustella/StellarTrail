//! Authentication routes for WeChat login, email verification codes, registration, password login, and image captcha challenges.

use axum::{Json, Router, routing::post};

use crate::{
    dto::auth::{
        CaptchaChallengeRequest, CaptchaChallengeResponse, EmailVerificationCodeRequest,
        EmailVerificationCodeResponse, LoginResponse, PasswordLoginRequest, RegisterRequest,
        WechatLoginRequest,
    },
    error::ApiError,
    services::auth_service,
    state::AppState,
};

/// Runs the `routes` server-side flow while preserving input validation, error propagation, and state invariants.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/wechat-login", post(wechat_login))
        .route(
            "/api/auth/email-verification-code",
            post(send_email_verification_code),
        )
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(password_login))
        .route("/api/auth/captcha", post(create_captcha))
}

/// Handles WeChat Mini Program code login and issues a StellarTrail session token on success.
async fn wechat_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<WechatLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::wechat_login(&state, payload.code, payload.profile).await?;
    Ok(Json(response))
}

/// Generates a registration email verification code and stores its digest for the registration endpoint to consume later.
async fn send_email_verification_code(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<EmailVerificationCodeRequest>,
) -> Result<Json<EmailVerificationCodeResponse>, ApiError> {
    let response = auth_service::send_email_verification_code(&state, payload.email).await?;
    Ok(Json(response))
}

/// Runs the `register` server-side flow while preserving input validation, error propagation, and state invariants.
async fn register(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::register_with_password(&state, payload).await?;
    Ok(Json(response))
}

/// Handles password login by username or email and verifies a one-time image captcha when required.
async fn password_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<PasswordLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::password_login(&state, payload).await?;
    Ok(Json(response))
}

/// Runs the `create captcha` server-side flow while preserving input validation, error propagation, and state invariants.
async fn create_captcha(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CaptchaChallengeRequest>,
) -> Result<Json<CaptchaChallengeResponse>, ApiError> {
    let response = auth_service::create_captcha_challenge(&state, payload.account).await?;
    Ok(Json(response))
}
