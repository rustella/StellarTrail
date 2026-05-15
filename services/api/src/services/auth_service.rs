use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use stellartrail_db::repositories::{AuthRepository, UserRecord, hash_token};
use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    dto::auth::{LoginProfileRequest, LoginResponse, LoginUserResponse},
    error::ApiError,
    state::AppState,
};

pub async fn mock_login(
    state: &AppState,
    code: String,
    profile: Option<LoginProfileRequest>,
) -> Result<LoginResponse, ApiError> {
    if !state.config().wechat_mock_login || state.config().app_env != "local" {
        return Err(ApiError::BadRequest(
            "wechat mock login is only enabled in local environment".to_owned(),
        ));
    }
    let code = code.trim();
    if code.is_empty() {
        return Err(ApiError::Validation(vec![
            stellartrail_domain::validation::FieldViolation::new("code", "is required"),
        ]));
    }
    let profile = profile.unwrap_or(LoginProfileRequest {
        nickname: Some("本地测试用户".to_owned()),
        avatar_url: None,
    });
    let openid = format!("mock:{code}");
    let repo = AuthRepository::new(state.db().clone());
    let user = repo
        .upsert_mock_user(&openid, profile.nickname, profile.avatar_url)
        .await?;
    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);
    repo.create_session(&user.id, &token_hash, expires_at)
        .await?;
    Ok(LoginResponse {
        access_token: token,
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        user: LoginUserResponse {
            id: user.id,
            nickname: user.nickname,
            avatar_url: user.avatar_url,
        },
    })
}

pub async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<UserRecord, ApiError> {
    let token = bearer_token(headers).ok_or(ApiError::Unauthorized)?;
    let token_hash = hash_token(token);
    AuthRepository::new(state.db().clone())
        .find_user_by_token_hash(&token_hash)
        .await?
        .ok_or(ApiError::Unauthorized)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
