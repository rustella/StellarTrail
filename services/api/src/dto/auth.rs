//! Authentication HTTP DTOs for WeChat login, email registration, password login, and captcha request/response payloads.

use serde::{Deserialize, Serialize};

/// Stable data boundary for `WechatLoginRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct WechatLoginRequest {
    pub code: String,
    #[serde(default)]
    pub profile: Option<LoginProfileRequest>,
}

/// Stable data boundary for `LoginProfileRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct LoginProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// Stable data boundary for `EmailVerificationCodeRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct EmailVerificationCodeRequest {
    pub email: String,
}

/// Stable data boundary for `EmailVerificationCodeResponse`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct EmailVerificationCodeResponse {
    pub email: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_code: Option<String>,
}

/// Stable data boundary for `RegisterRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub email_verification_code: String,
}

/// Stable data boundary for `CaptchaChallengeRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct CaptchaChallengeRequest {
    pub account: String,
}

/// Stable data boundary for `CaptchaChallengeResponse`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct CaptchaChallengeResponse {
    pub captcha_ticket: String,
    pub captcha_type: &'static str,
    pub image_svg: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_answer: Option<String>,
}

/// Stable data boundary for `PasswordLoginRequest`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
pub struct PasswordLoginRequest {
    pub account: String,
    pub password: String,
    #[serde(default)]
    pub captcha_ticket: Option<String>,
    #[serde(default)]
    pub captcha_answer: Option<String>,
}

/// Stable data boundary for `LoginResponse`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: String,
    pub user: LoginUserResponse,
}

/// Stable data boundary for `LoginUserResponse`, exposed by or reused within this module.
#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
