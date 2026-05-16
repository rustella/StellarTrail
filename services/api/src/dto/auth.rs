use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WechatLoginRequest {
    pub code: String,
    #[serde(default)]
    pub profile: Option<LoginProfileRequest>,
}

#[derive(Debug, Deserialize)]
pub struct LoginProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailVerificationCodeRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct EmailVerificationCodeResponse {
    pub email: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub email_verification_code: String,
}

#[derive(Debug, Deserialize)]
pub struct CaptchaChallengeRequest {
    pub account: String,
}

#[derive(Debug, Serialize)]
pub struct CaptchaChallengeResponse {
    pub captcha_ticket: String,
    pub captcha_type: &'static str,
    pub image_svg: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_answer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordLoginRequest {
    pub account: String,
    pub password: String,
    #[serde(default)]
    pub captcha_ticket: Option<String>,
    #[serde(default)]
    pub captcha_answer: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: String,
    pub user: LoginUserResponse,
}

#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
