//! 认证路由模块，绑定微信登录、邮箱验证码、注册、密码登录和图片验证码 challenge 接口。

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

/// 执行 `routes` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 处理微信小程序 code 登录，成功后签发 StellarTrail 会话 token。
async fn wechat_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<WechatLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::wechat_login(&state, payload.code, payload.profile).await?;
    Ok(Json(response))
}

/// 生成注册邮箱验证码并保存摘要，供后续注册接口消费。
async fn send_email_verification_code(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<EmailVerificationCodeRequest>,
) -> Result<Json<EmailVerificationCodeResponse>, ApiError> {
    let response = auth_service::send_email_verification_code(&state, payload.email).await?;
    Ok(Json(response))
}

/// 执行 `register` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn register(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::register_with_password(&state, payload).await?;
    Ok(Json(response))
}

/// 处理用户名或邮箱密码登录，必要时校验一次性图片验证码。
async fn password_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<PasswordLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = auth_service::password_login(&state, payload).await?;
    Ok(Json(response))
}

/// 执行 `create captcha` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn create_captcha(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<CaptchaChallengeRequest>,
) -> Result<Json<CaptchaChallengeResponse>, ApiError> {
    let response = auth_service::create_captcha_challenge(&state, payload.account).await?;
    Ok(Json(response))
}
