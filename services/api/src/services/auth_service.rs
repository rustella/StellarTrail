//! 认证业务服务模块，封装微信 code2session、邮箱注册、密码登录、验证码和会话签发流程。

use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, RngCore};
use stellartrail_db::repositories::{AuthRepository, UserRecord, hash_token};
use stellartrail_domain::validation::FieldViolation;
use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    dto::auth::{
        CaptchaChallengeResponse, EmailVerificationCodeResponse, LoginProfileRequest,
        LoginResponse, LoginUserResponse, PasswordLoginRequest, RegisterRequest,
    },
    error::ApiError,
    services::wechat::WechatCodeSessionError,
    state::AppState,
};

const EMAIL_CODE_PURPOSE_REGISTER: &str = "register";
const EMAIL_CODE_EXPIRES_MINUTES: i64 = 10;
const CAPTCHA_EXPIRES_MINUTES: i64 = 5;
const LOGIN_CAPTCHA_THRESHOLD: i32 = 3;

/// 处理微信小程序 code 登录，成功后签发 StellarTrail 会话 token。
pub async fn wechat_login(
    state: &AppState,
    code: String,
    profile: Option<LoginProfileRequest>,
) -> Result<LoginResponse, ApiError> {
    let code = validate_code(code)?;
    // 仅 local 环境允许 mock 微信登录，避免生产绕过微信 code2session。
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
    // 生产路径必须调用微信 code2session，用临时 code 换取可信 openid。
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

/// 生成注册邮箱验证码并保存摘要，供后续注册接口消费。
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

/// 完成邮箱用户名注册，校验验证码和密码后创建用户并签发会话。
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

    // 当前需求明确要求 SHA-256 保存密码摘要，因此复用 token hash 的十六进制实现。
    let password_hash = hash_token(&password);
    let user = repo
        .create_password_user(&username, &email, &password_hash)
        .await?;
    issue_login_for_user(&repo, user).await
}

/// 处理用户名或邮箱密码登录，必要时校验一次性图片验证码。
pub async fn password_login(
    state: &AppState,
    payload: PasswordLoginRequest,
) -> Result<LoginResponse, ApiError> {
    let account = validate_login_account(payload.account)?;
    let password = validate_login_password(payload.password)?;
    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_login_account(&account).await? else {
        return Err(ApiError::InvalidCredentials);
    };

    // 达到失败门槛后必须先通过一次性图片验证码，降低暴力猜测风险。
    if user.failed_login_attempts >= LOGIN_CAPTCHA_THRESHOLD {
        // 验证码校验成功会消费 ticket，防止同一个 challenge 被重复使用。
        let captcha_ok =
            verify_captcha(&repo, payload.captcha_ticket, payload.captcha_answer).await?;
        if !captcha_ok {
            return Err(ApiError::CaptchaRequired);
        }
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

/// 创建一次性图片验证码 challenge，并在本地环境返回 debug 答案。
pub async fn create_captcha_challenge(
    state: &AppState,
    account: String,
) -> Result<CaptchaChallengeResponse, ApiError> {
    let account = validate_login_account(account)?;
    let answer = generate_captcha_answer();
    let ticket = generate_token();
    let answer_hash = hash_token(&normalize_captcha_answer(&answer));
    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(CAPTCHA_EXPIRES_MINUTES);
    AuthRepository::new(state.db().clone())
        .create_captcha_challenge(&account, &ticket, &answer_hash, expires_at)
        .await?;

    Ok(CaptchaChallengeResponse {
        captcha_ticket: ticket,
        captcha_type: "image",
        image_svg: render_captcha_svg(&answer),
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_answer: (state.config().app_env == "local").then_some(answer),
    })
}

/// 执行 `mock login` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `issue login for openid` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 为指定用户生成随机 token、保存 token hash，并返回登录响应。
async fn issue_login_for_user(
    repo: &AuthRepository,
    user: UserRecord,
) -> Result<LoginResponse, ApiError> {
    // 返回给客户端的是随机 token；数据库只保存 hash，泄露数据库也不能直接伪造请求。
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

/// 从 Authorization 头解析 Bearer token，并查找对应有效用户。
pub async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<UserRecord, ApiError> {
    let token = bearer_token(headers).ok_or(ApiError::Unauthorized)?;
    let token_hash = hash_token(token);
    AuthRepository::new(state.db().clone())
        .find_user_by_token_hash(&token_hash)
        .await?
        .ok_or(ApiError::Unauthorized)
}

/// 执行 `validate code` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `validate username` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `validate email` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `validate password` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `validate login password` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn validate_login_password(password: String) -> Result<String, ApiError> {
    if password.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "password",
            "is required",
        )]));
    }
    Ok(password)
}

/// 执行 `validate login account` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `validate verification code` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 校验并消费图片验证码 challenge，防止同一 ticket 重放。
async fn verify_captcha(
    repo: &AuthRepository,
    ticket: Option<String>,
    answer: Option<String>,
) -> Result<bool, ApiError> {
    let (Some(ticket), Some(answer)) = (ticket, answer) else {
        return Ok(false);
    };
    let ticket = ticket.trim();
    if ticket.is_empty() {
        return Ok(false);
    }
    let answer = normalize_captcha_answer(&answer);
    if answer.is_empty() {
        return Ok(false);
    }
    repo.consume_captcha_challenge(ticket, &hash_token(&answer))
        .await
        .map_err(ApiError::from)
}

/// 执行 `normalize captcha answer` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn normalize_captcha_answer(answer: &str) -> String {
    answer.trim().to_ascii_uppercase()
}

/// 执行 `generate captcha answer` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn generate_captcha_answer() -> String {
    const CHARS: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

/// 执行 `render captcha svg` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn render_captcha_svg(answer: &str) -> String {
    let chars = answer
        .chars()
        .enumerate()
        .map(|(idx, ch)| {
            let x = 18 + idx * 28;
            let y = if idx % 2 == 0 { 42 } else { 35 };
            format!(
                r#"<text x="{x}" y="{y}" transform="rotate({rotate} {x},{y})">{ch}</text>"#,
                rotate = if idx % 2 == 0 { -8 } else { 7 },
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="54" viewBox="0 0 140 54" role="img" aria-label="captcha"><rect width="140" height="54" rx="8" fill="#f8fafc"/><path d="M8 18 C40 2, 80 52, 132 20" stroke="#94a3b8" stroke-width="2" fill="none"/><path d="M10 40 C44 54, 86 8, 130 36" stroke="#cbd5e1" stroke-width="2" fill="none"/><g font-family="monospace" font-size="26" font-weight="700" fill="#0f172a">{chars}</g></svg>"##
    )
}

/// 执行 `required wechat config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn required_wechat_config<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, ApiError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("{name} is required for WeChat login")))
}

/// 执行 `map wechat login error` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn map_wechat_login_error(error: anyhow::Error) -> ApiError {
    match error.downcast::<WechatCodeSessionError>() {
        Ok(WechatCodeSessionError::Rejected { code, message }) => {
            ApiError::BadRequest(format!("wechat login failed: {message} ({code})"))
        }
        Ok(other) => ApiError::internal(other),
        Err(error) => ApiError::internal(error),
    }
}

/// 执行 `bearer token` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

/// 执行 `generate token` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 执行 `generate email code` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn generate_email_code() -> String {
    format!("{:06}", rand::thread_rng().gen_range(0..=999_999))
}
