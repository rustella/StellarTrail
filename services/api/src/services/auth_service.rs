//! Authentication domain service for login, registration, captcha, and token renewal.
//!
//! This module owns the security-sensitive orchestration around credentials and
//! sessions. It validates client input, calls WeChat code2session when required,
//! hashes every opaque token before persistence, and rotates refresh tokens so a
//! refresh token can be used only once. Routes should stay thin and delegate all
//! authentication state changes to this layer.

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
// Access tokens are intentionally short-lived because clients can renew them
// with refresh tokens instead of keeping a month-long bearer token active.
const ACCESS_TOKEN_EXPIRES_HOURS: i64 = 2;
// Refresh tokens keep the existing 30-day session experience but rotate on use
// so a captured refresh token cannot be replayed after a successful refresh.
const REFRESH_TOKEN_EXPIRES_DAYS: i64 = 30;

/// Exchanges a WeChat Mini Program login code for a StellarTrail session.
///
/// Local development may use the mock path, but non-local environments must have
/// WeChat credentials configured and must call `code2session` before any user or
/// session is created. Successful login returns both a short-lived access token
/// and a longer-lived refresh token.
pub async fn wechat_login(
    state: &AppState,
    code: String,
    profile: Option<LoginProfileRequest>,
) -> Result<LoginResponse, ApiError> {
    let code = validate_code(code)?;
    // Only the local environment may use mocked WeChat login so production cannot bypass code2session.
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
    // Production paths must call WeChat code2session to exchange the temporary code for a trusted openid.
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

/// Generates and stores a one-time registration email verification code.
///
/// The stored value is a hash, not the plaintext code. Local environments return
/// the plaintext code for smoke tests, while production can later plug in email
/// delivery without changing registration validation.
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

    // The current server completes the verification-code generation and validation loop first; local returns debug_code for integration testing,
    // and future email delivery only needs to be added here while production never returns the plaintext code.
    let debug_code = (state.config().app_env == "local").then_some(code);
    Ok(EmailVerificationCodeResponse {
        email,
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_code,
    })
}

/// Completes email/username registration by validating the code and password, creating the user, and issuing a session.
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

    // The current requirement explicitly stores password digests with SHA-256, so reuse the token hash hexadecimal implementation.
    let password_hash = hash_token(&password);
    let user = repo
        .create_password_user(&username, &email, &password_hash)
        .await?;
    issue_login_for_user(&repo, user).await
}

/// Authenticates a username/email password login and issues a fresh token pair.
///
/// Accounts with repeated failures must solve the latest captcha challenge before
/// password verification proceeds. Successful login resets the failure counter
/// and creates a new opaque access/refresh session.
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

    // After the failure threshold is reached, require a one-time image captcha first to reduce brute-force guessing risk.
    if user.failed_login_attempts >= LOGIN_CAPTCHA_THRESHOLD {
        // Successful captcha validation consumes the ticket so the same challenge cannot be reused.
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

/// Creates a one-time captcha challenge for accounts that reached the failure threshold.
///
/// The challenge answer is stored as a hash and can be consumed only once. Local
/// environments include the answer in the response so automated tests can cover
/// the protected password-login path.
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

/// Rotates a valid refresh token into a new access/refresh token pair.
///
/// The refresh token supplied by the client is hashed before lookup. A missing,
/// expired, revoked, deleted-user-bound, or already-rotated token is reported as
/// `Unauthorized` without revealing which condition failed.
pub async fn refresh_token(
    state: &AppState,
    refresh_token: String,
) -> Result<LoginResponse, ApiError> {
    let refresh_token = validate_refresh_token(refresh_token)?;
    let repo = AuthRepository::new(state.db().clone());
    // Hash the client-provided token before lookup so plaintext refresh tokens
    // never cross the repository boundary or appear in query logs.
    let refresh_token_hash = hash_token(&refresh_token);
    let Some(session) = repo
        .find_session_by_refresh_token_hash(&refresh_token_hash)
        .await?
    else {
        // Use the same unauthorized response for invalid, expired, revoked, and
        // replayed tokens so callers cannot enumerate session state.
        return Err(ApiError::Unauthorized);
    };
    issue_rotated_login_for_session(&repo, session.session_id, refresh_token_hash, session.user)
        .await
}

/// Performs development-only mock login using the same session issuance path as WeChat login.
///
/// This endpoint is deliberately disabled outside the local environment so a
/// configured production server cannot bypass WeChat code2session.
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

/// Creates or updates the WeChat user identified by `openid` and issues a session.
///
/// Profile updates happen before token issuance so the user snapshot embedded in
/// the login response reflects the latest nickname and avatar supplied by the
/// client.
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

/// Issues a new persisted session for a user and returns the client-visible tokens.
///
/// Both tokens are high-entropy opaque strings. Only SHA-256 hashes are persisted
/// in the `sessions` table, which prevents a database dump from being used as a
/// bearer credential.
async fn issue_login_for_user(
    repo: &AuthRepository,
    user: UserRecord,
) -> Result<LoginResponse, ApiError> {
    let token_pair = generate_token_pair();
    // Store only token hashes. The plaintext values remain in `token_pair` long
    // enough to be returned to the authenticated client in the response body.
    repo.create_session(
        &user.id,
        &hash_token(&token_pair.access_token),
        token_pair.expires_at,
        &hash_token(&token_pair.refresh_token),
        token_pair.refresh_expires_at,
    )
    .await?;
    login_response(user, token_pair)
}

/// Replaces token hashes in an existing session after a refresh-token lookup succeeds.
///
/// The repository update is conditional on the old refresh hash still matching
/// the session row. If another request rotated the same refresh token first, the
/// update affects zero rows and this function returns `Unauthorized`.
async fn issue_rotated_login_for_session(
    repo: &AuthRepository,
    session_id: String,
    old_refresh_token_hash: String,
    user: UserRecord,
) -> Result<LoginResponse, ApiError> {
    let token_pair = generate_token_pair();
    // Generate the replacement pair before the conditional update so the old
    // refresh token and new refresh token are never valid at the same time.
    let updated = repo
        .rotate_session_tokens(
            &session_id,
            &old_refresh_token_hash,
            &hash_token(&token_pair.access_token),
            token_pair.expires_at,
            &hash_token(&token_pair.refresh_token),
            token_pair.refresh_expires_at,
        )
        .await?;
    if !updated {
        // A failed conditional update means another request already rotated the
        // token, or the session became revoked/expired/deleted between lookup
        // and update. Treat all of those as unauthorized replay attempts.
        return Err(ApiError::Unauthorized);
    }
    login_response(user, token_pair)
}

struct TokenPair {
    access_token: String,
    expires_at: OffsetDateTime,
    refresh_token: String,
    refresh_expires_at: OffsetDateTime,
}

/// Builds an access/refresh token pair with independent expiry timestamps.
fn generate_token_pair() -> TokenPair {
    // Each token is generated independently so the refresh token cannot be
    // derived from, compared to, or confused with the access token.
    TokenPair {
        access_token: generate_token(),
        expires_at: OffsetDateTime::now_utc() + Duration::hours(ACCESS_TOKEN_EXPIRES_HOURS),
        refresh_token: generate_token(),
        refresh_expires_at: OffsetDateTime::now_utc() + Duration::days(REFRESH_TOKEN_EXPIRES_DAYS),
    }
}

/// Converts a persisted user and freshly generated token pair into the public login response.
fn login_response(user: UserRecord, token_pair: TokenPair) -> Result<LoginResponse, ApiError> {
    // Format expiry values once at the API boundary so every client receives the
    // same RFC3339 timestamp representation regardless of login method.
    Ok(LoginResponse {
        access_token: token_pair.access_token,
        expires_at: token_pair
            .expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        refresh_token: token_pair.refresh_token,
        refresh_expires_at: token_pair
            .refresh_expires_at
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

/// Parses the Bearer token from the Authorization header and looks up the corresponding active user.
pub async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<UserRecord, ApiError> {
    let token = bearer_token(headers).ok_or(ApiError::Unauthorized)?;
    let token_hash = hash_token(token);
    AuthRepository::new(state.db().clone())
        .find_user_by_token_hash(&token_hash)
        .await?
        .ok_or(ApiError::Unauthorized)
}

/// Trims and validates the temporary WeChat login code before code2session exchange.
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

/// Normalizes and bounds a registration username before it is written to the database.
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

/// Normalizes and validates an email address used for registration or login.
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

/// Validates a registration password and confirms the repeated password matches.
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

/// Validates that a password-login request includes a non-empty password.
fn validate_login_password(password: String) -> Result<String, ApiError> {
    if password.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "password",
            "is required",
        )]));
    }
    Ok(password)
}

/// Trims the username or email identifier used by password login.
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

/// Trims the registration email verification code and rejects empty values.
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

/// Trims the refresh token submitted by clients and rejects empty credentials.
fn validate_refresh_token(token: String) -> Result<String, ApiError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "refresh_token",
            "is required",
        )]));
    }
    Ok(token.to_owned())
}

/// Validates and consumes an image captcha challenge to prevent replaying the same ticket.
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

/// Normalizes captcha answers so user input and generated answers hash consistently.
fn normalize_captcha_answer(answer: &str) -> String {
    answer.trim().to_ascii_uppercase()
}

/// Generates a short numeric captcha answer using the process random generator.
fn generate_captcha_answer() -> String {
    const CHARS: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

/// Renders the captcha answer into a simple SVG image returned to clients.
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

/// Reads a required WeChat configuration value without exposing secret contents in errors.
fn required_wechat_config<'a>(value: Option<&'a str>, name: &str) -> Result<&'a str, ApiError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("{name} is required for WeChat login")))
}

/// Converts code2session failures into API errors while preserving safe client messages.
fn map_wechat_login_error(error: anyhow::Error) -> ApiError {
    match error.downcast::<WechatCodeSessionError>() {
        Ok(WechatCodeSessionError::Rejected { code, message }) => {
            ApiError::BadRequest(format!("wechat login failed: {message} ({code})"))
        }
        Ok(other) => ApiError::internal(other),
        Err(error) => ApiError::internal(error),
    }
}

/// Extracts a bearer token from the Authorization header for authenticated routes.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

/// Generates a URL-safe opaque bearer token with enough entropy for access or refresh use.
fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generates a six-digit registration email code for the user-facing verification step.
fn generate_email_code() -> String {
    format!("{:06}", rand::thread_rng().gen_range(0..=999_999))
}
