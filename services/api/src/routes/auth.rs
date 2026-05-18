//! Axum route definitions for every authentication endpoint.
//!
//! This module is intentionally thin: handlers deserialize HTTP payloads, call
//! `auth_service` for validation and state changes, and serialize the response.
//! Keeping token issuance, refresh rotation, and captcha validation in the
//! service layer makes the route table easy to audit for public authentication
//! surface area.

use axum::{Json, Router, routing::post};

use crate::{
    dto::auth::{
        BindEmailCodeRequest, BindEmailRequest, BindEmailResponse, CaptchaChallengeRequest,
        CaptchaChallengeResponse, EmailLoginCodeRequest, EmailLoginRequest,
        EmailVerificationCodeRequest, EmailVerificationCodeResponse, LoginResponse,
        PasswordLoginRequest, PasswordResetCodeRequest, PasswordResetRequest, RefreshTokenRequest,
        RegisterRequest, WechatLoginRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::auth_service,
    state::AppState,
};

/// Builds the authentication router mounted by the API application.
///
/// Login, registration, captcha, and refresh routes are all unauthenticated by
/// design because they establish or renew a session. The authenticated email-binding
/// handlers still require Bearer Token before changing `/api/me/*` account data.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/wechat-login", post(wechat_login))
        .route(
            "/api/auth/email-verification-code",
            post(send_email_verification_code),
        )
        .route("/api/auth/email-login-code", post(send_email_login_code))
        .route("/api/auth/email-login", post(email_login))
        .route(
            "/api/auth/password-reset-code",
            post(send_password_reset_code),
        )
        .route("/api/auth/password-reset", post(password_reset))
        .route("/api/me/email-binding-code", post(send_bind_email_code))
        .route("/api/me/email-binding", post(bind_email))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(password_login))
        // Refresh is intentionally public: the refresh token itself is the credential.
        .route("/api/auth/refresh", post(refresh_token))
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

/// Generates a login email verification code for an existing account without revealing missing accounts.
async fn send_email_login_code(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<EmailLoginCodeRequest>,
) -> Result<Json<EmailVerificationCodeResponse>, ApiError> {
    let response = auth_service::send_email_login_code(&state, payload.email).await?;
    Ok(Json(response))
}

/// Logs in an existing account by consuming a one-time email verification code.
async fn email_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<EmailLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::email_login(&state, payload).await?;
    Ok(Json(response))
}

/// Generates a password-reset email verification code for an existing account without revealing missing accounts.
async fn send_password_reset_code(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<PasswordResetCodeRequest>,
) -> Result<Json<EmailVerificationCodeResponse>, ApiError> {
    let response = auth_service::send_password_reset_code(&state, payload.email).await?;
    Ok(Json(response))
}

/// Resets an account password by consuming a one-time email verification code and returns a fresh session.
async fn password_reset(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::password_reset(&state, payload).await?;
    Ok(Json(response))
}

/// Generates an email verification code for binding an email to the current account.
async fn send_bind_email_code(
    axum::extract::State(state): axum::extract::State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<BindEmailCodeRequest>,
) -> Result<Json<EmailVerificationCodeResponse>, ApiError> {
    let response = auth_service::send_bind_email_code(&state, &user, payload.email).await?;
    Ok(Json(response))
}

/// Binds a verified email address to the current account.
async fn bind_email(
    axum::extract::State(state): axum::extract::State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<BindEmailRequest>,
) -> Result<Json<BindEmailResponse>, ApiError> {
    let response = auth_service::bind_email(&state, user, payload).await?;
    Ok(Json(response))
}

/// Registers a password account and returns the first access/refresh token pair for the new user.
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

/// Rotates an active refresh token and returns a fresh access/refresh token pair.
async fn refresh_token(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::refresh_token(&state, payload.refresh_token).await?;
    Ok(Json(response))
}

/// Creates a captcha challenge used to slow repeated password-login failures for an account.
async fn create_captcha(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CaptchaChallengeRequest>,
) -> Result<Json<CaptchaChallengeResponse>, ApiError> {
    let response = auth_service::create_captcha_challenge(&state, payload.account).await?;
    Ok(Json(response))
}
