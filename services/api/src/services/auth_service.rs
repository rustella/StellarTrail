use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use stellartrail_db::repositories::{AuthRepository, UserRecord, hash_token};
use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    dto::auth::{LoginProfileRequest, LoginResponse, LoginUserResponse},
    error::ApiError,
    services::wechat::WechatCodeSessionError,
    state::AppState,
};

pub async fn wechat_login(
    state: &AppState,
    code: String,
    profile: Option<LoginProfileRequest>,
) -> Result<LoginResponse, ApiError> {
    let code = validate_code(code)?;
    if state.config().wechat_mock_login && state.config().app_env == "local" {
        let profile = profile.unwrap_or(LoginProfileRequest {
            nickname: Some("本地测试用户".to_owned()),
            avatar_url: None,
        });
        let openid = format!("mock:{code}");
        return issue_login_for_openid(state, &openid, profile).await;
    }

    let app_id = required_wechat_config(state.config().wechat_app_id.as_deref(), "WECHAT_APP_ID")?;
    let app_secret = required_wechat_config(
        state.config().wechat_app_secret.as_deref(),
        "WECHAT_APP_SECRET",
    )?;
    let wechat_client = state.wechat_client();
    let app_id = app_id.to_owned();
    let app_secret = app_secret.to_owned();
    let code_session = tokio::task::spawn_blocking(move || {
        wechat_client.code2session(&app_id, &app_secret, &code)
    })
    .await
    .map_err(ApiError::internal)?
    .map_err(map_wechat_login_error)?;
    let profile = profile.unwrap_or(LoginProfileRequest {
        nickname: None,
        avatar_url: None,
    });
    issue_login_for_openid(state, &code_session.openid, profile).await
}

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
    let code = validate_code(code)?;
    let profile = profile.unwrap_or(LoginProfileRequest {
        nickname: Some("本地测试用户".to_owned()),
        avatar_url: None,
    });
    let openid = format!("mock:{code}");
    issue_login_for_openid(state, &openid, profile).await
}

async fn issue_login_for_openid(
    state: &AppState,
    openid: &str,
    profile: LoginProfileRequest,
) -> Result<LoginResponse, ApiError> {
    let repo = AuthRepository::new(state.db().clone());
    let user = repo
        .upsert_wechat_user(openid, profile.nickname, profile.avatar_url)
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

fn validate_code(code: String) -> Result<String, ApiError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(ApiError::Validation(vec![
            stellartrail_domain::validation::FieldViolation::new("code", "is required"),
        ]));
    }
    Ok(code.to_owned())
}

fn required_wechat_config<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, ApiError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("{name} is required for WeChat login")))
}

fn map_wechat_login_error(error: anyhow::Error) -> ApiError {
    match error.downcast::<WechatCodeSessionError>() {
        Ok(WechatCodeSessionError::Rejected { code, message }) => {
            ApiError::BadRequest(format!("wechat login failed: {message} ({code})"))
        }
        Ok(other) => ApiError::internal(other),
        Err(error) => ApiError::internal(error),
    }
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
