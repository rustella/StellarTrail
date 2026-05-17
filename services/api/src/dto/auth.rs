//! HTTP request and response DTOs for StellarTrail authentication.
//!
//! These structs define the wire format shared by the Rust API, the TypeScript
//! clients, the WeChat Mini Program, and the Android client. Authentication uses
//! opaque access and refresh tokens, so token fields in responses are plaintext
//! values intended for clients while persistence layers store only token hashes.

use serde::{Deserialize, Serialize};

/// Request body for exchanging a WeChat Mini Program login code for a StellarTrail session.
#[derive(Debug, Deserialize)]
pub struct WechatLoginRequest {
    pub code: String,
    #[serde(default)]
    pub profile: Option<LoginProfileRequest>,
}

/// Optional profile fields supplied by the client during login or registration bootstrap.
#[derive(Debug, Deserialize)]
pub struct LoginProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for issuing a one-time email verification code for account registration.
#[derive(Debug, Deserialize)]
pub struct EmailVerificationCodeRequest {
    pub email: String,
}

/// Response describing when the email verification code expires.
///
/// Local environments may include `debug_code` so integration tests can complete
/// the registration flow without a real email delivery provider.
#[derive(Debug, Serialize)]
pub struct EmailVerificationCodeResponse {
    pub email: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_code: Option<String>,
}

/// Request body for username/email/password registration after email verification.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub email_verification_code: String,
}

/// Request body for creating an image captcha challenge for a login account.
#[derive(Debug, Deserialize)]
pub struct CaptchaChallengeRequest {
    pub account: String,
}

/// Response containing the captcha ticket, SVG challenge, and expiry metadata.
#[derive(Debug, Serialize)]
pub struct CaptchaChallengeResponse {
    pub captcha_ticket: String,
    pub captcha_type: &'static str,
    pub image_svg: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_answer: Option<String>,
}

/// Request body for password login using either a username or an email address.
#[derive(Debug, Deserialize)]
pub struct PasswordLoginRequest {
    pub account: String,
    pub password: String,
    #[serde(default)]
    pub captcha_ticket: Option<String>,
    #[serde(default)]
    pub captcha_answer: Option<String>,
}

/// Request body for rotating a refresh token into a new access/refresh token pair.
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Successful authentication response returned by login, registration, and refresh endpoints.
///
/// The access token is short-lived and used in the `Authorization` header. The
/// refresh token is longer-lived, rotated on every refresh call, and should be
/// stored by clients with the same care as a password.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: String,
    pub refresh_token: String,
    pub refresh_expires_at: String,
    pub user: LoginUserResponse,
}

/// User snapshot embedded in authentication responses so clients can restore session UI state.
#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
