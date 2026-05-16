//! 认证相关 HTTP DTO，定义微信登录、邮箱注册、密码登录与验证码接口的请求和响应结构。

use serde::{Deserialize, Serialize};

/// WechatLoginRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct WechatLoginRequest {
    pub code: String,
    #[serde(default)]
    pub profile: Option<LoginProfileRequest>,
}

/// LoginProfileRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct LoginProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// EmailVerificationCodeRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct EmailVerificationCodeRequest {
    pub email: String,
}

/// EmailVerificationCodeResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Serialize)]
pub struct EmailVerificationCodeResponse {
    pub email: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_code: Option<String>,
}

/// RegisterRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub email_verification_code: String,
}

/// CaptchaChallengeRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct CaptchaChallengeRequest {
    pub account: String,
}

/// CaptchaChallengeResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Serialize)]
pub struct CaptchaChallengeResponse {
    pub captcha_ticket: String,
    pub captcha_type: &'static str,
    pub image_svg: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_answer: Option<String>,
}

/// PasswordLoginRequest 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
pub struct PasswordLoginRequest {
    pub account: String,
    pub password: String,
    #[serde(default)]
    pub captcha_ticket: Option<String>,
    #[serde(default)]
    pub captcha_answer: Option<String>,
}

/// LoginResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: String,
    pub user: LoginUserResponse,
}

/// LoginUserResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
