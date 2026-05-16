use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, RngCore};
use stellartrail_db::repositories::{AuthRepository, UserRecord, hash_token};
use stellartrail_domain::validation::FieldViolation;
use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    dto::auth::{
        EmailVerificationCodeResponse, LoginProfileRequest, LoginResponse, LoginUserResponse,
        PasswordLoginRequest, RegisterRequest,
    },
    error::ApiError,
    services::wechat::WechatCodeSessionError,
    state::AppState,
};

const EMAIL_CODE_PURPOSE_REGISTER: &str = "register";
const EMAIL_CODE_EXPIRES_MINUTES: i64 = 10;
const LOGIN_CAPTCHA_THRESHOLD: i32 = 3;

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

pub async fn send_email_verification_code(
    state: &AppState,
    email: String,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    let email = validate_email(email)?;
    let code = generate_email_code();
    let code_hash = hash_token(&code);
    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(EMAIL_CODE_EXPIRES_MINUTES);
    AuthRepository::new(state.db().clone())
        .create_email_verification_code(&email, EMAIL_CODE_PURPOSE_REGISTER, &code_hash, expires_at)
        .await?;

    // 当前服务端先完成验证码生成与校验闭环；本地环境返回 debug_code 便于联调，
    // 后续接入邮件服务时只需要在这里投递邮件，生产环境不会返回明文验证码。
    let debug_code = (state.config().app_env == "local").then_some(code);
    Ok(EmailVerificationCodeResponse {
        email,
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_code,
    })
}

pub async fn register_with_password(
    state: &AppState,
    payload: RegisterRequest,
) -> Result<LoginResponse, ApiError> {
    let username = validate_username(payload.username)?;
    let email = validate_email(payload.email)?;
    let password = validate_password(payload.password)?;
    let confirm_password = payload.confirm_password;
    let verification_code = validate_verification_code(payload.email_verification_code)?;

    let mut errors = Vec::new();
    if password != confirm_password {
        errors.push(FieldViolation::new(
            "confirm_password",
            "does not match password",
        ));
    }

    let repo = AuthRepository::new(state.db().clone());
    if repo.find_user_by_username(&username).await?.is_some() {
        errors.push(FieldViolation::new(
            "username",
            "has already been registered",
        ));
    }
    if repo.find_user_by_email(&email).await?.is_some() {
        errors.push(FieldViolation::new("email", "has already been registered"));
    }
    if !errors.is_empty() {
        return Err(ApiError::Validation(errors));
    }

    let code_hash = hash_token(&verification_code);
    let code_ok = repo
        .consume_email_verification_code(&email, EMAIL_CODE_PURPOSE_REGISTER, &code_hash)
        .await?;
    if !code_ok {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email_verification_code",
            "is invalid or expired",
        )]));
    }

    let password_hash = hash_token(&password);
    let user = repo
        .create_password_user(&username, &email, &password_hash)
        .await?;
    issue_login_for_user(&repo, user).await
}

pub async fn password_login(
    state: &AppState,
    payload: PasswordLoginRequest,
) -> Result<LoginResponse, ApiError> {
    let captcha_ok = valid_local_captcha(&payload);
    let account = validate_login_account(payload.account)?;
    let password = validate_login_password(payload.password)?;
    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_login_account(&account).await? else {
        return Err(ApiError::InvalidCredentials);
    };

    if user.failed_login_attempts >= LOGIN_CAPTCHA_THRESHOLD && !captcha_ok {
        return Err(ApiError::CaptchaRequired);
    }

    let Some(password_hash) = user.password_hash.as_deref() else {
        return Err(ApiError::InvalidCredentials);
    };
    if password_hash != hash_token(&password) {
        repo.record_failed_password_login(&user.id).await?;
        return Err(ApiError::InvalidCredentials);
    }

    repo.reset_failed_password_login(&user.id).await?;
    issue_login_for_user(&repo, user).await
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
    issue_login_for_user(&repo, user).await
}

async fn issue_login_for_user(
    repo: &AuthRepository,
    user: UserRecord,
) -> Result<LoginResponse, ApiError> {
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
            username: user.username,
            email: user.email,
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
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "code",
            "is required",
        )]));
    }
    Ok(code.to_owned())
}

fn validate_username(username: String) -> Result<String, ApiError> {
    let username = username.trim().to_ascii_lowercase();
    let mut errors = Vec::new();
    let len = username.chars().count();
    if username.is_empty() {
        errors.push(FieldViolation::new("username", "is required"));
    } else {
        if !(3..=32).contains(&len) {
            errors.push(FieldViolation::new(
                "username",
                "must be between 3 and 32 characters",
            ));
        }
        if !username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            errors.push(FieldViolation::new(
                "username",
                "only letters, numbers, underscores and hyphens are allowed",
            ));
        }
    }
    if errors.is_empty() {
        Ok(username)
    } else {
        Err(ApiError::Validation(errors))
    }
}

fn validate_email(email: String) -> Result<String, ApiError> {
    let email = email.trim().to_ascii_lowercase();
    let valid_shape = email.len() <= 254
        && email.contains('@')
        && email.split('@').count() == 2
        && email
            .rsplit('@')
            .next()
            .is_some_and(|domain| domain.contains('.'));
    if !valid_shape {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email",
            "must be a valid email address",
        )]));
    }
    Ok(email)
}

fn validate_password(password: String) -> Result<String, ApiError> {
    let len = password.chars().count();
    if !(8..=128).contains(&len) {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "password",
            "must be between 8 and 128 characters",
        )]));
    }
    Ok(password)
}

fn validate_login_password(password: String) -> Result<String, ApiError> {
    if password.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "password",
            "is required",
        )]));
    }
    Ok(password)
}

fn validate_login_account(account: String) -> Result<String, ApiError> {
    let account = account.trim().to_ascii_lowercase();
    if account.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "account",
            "is required",
        )]));
    }
    Ok(account)
}

fn validate_verification_code(code: String) -> Result<String, ApiError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email_verification_code",
            "is required",
        )]));
    }
    Ok(code.to_owned())
}

fn valid_local_captcha(payload: &PasswordLoginRequest) -> bool {
    matches!(
        (
            payload.captcha_ticket.as_deref(),
            payload.captcha_answer.as_deref().map(str::trim),
        ),
        (Some("local-dev-captcha"), Some("pass"))
    )
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

fn generate_email_code() -> String {
    format!("{:06}", rand::thread_rng().gen_range(0..=999_999))
}
