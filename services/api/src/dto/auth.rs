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

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: String,
    pub user: LoginUserResponse,
}

#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
