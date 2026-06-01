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
use serde_json::json;
use stellartrail_db::repositories::{AuthRepository, UserRecord, hash_token};
use stellartrail_domain::validation::FieldViolation;
use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

use crate::{
    dto::auth::{
        BindEmailRequest, BindEmailResponse, BindPhoneRequest, BindPhoneResponse,
        CaptchaChallengeResponse, EmailLoginRequest, EmailVerificationCodeResponse,
        LoginProfileRequest, LoginResponse, LoginUserResponse, PasswordLoginRequest,
        PasswordResetRequest, RegisterRequest, SmsCodeResponse, SmsLoginRequest,
        SmsPasswordResetRequest, SmsRegisterRequest,
    },
    email::VerificationEmail,
    error::ApiError,
    services::{
        sms::{SmsCheckCodeRequest, SmsSendCodeRequest, SmsVerificationError},
        wechat::WechatCodeSessionError,
    },
    state::AppState,
};

const EMAIL_CODE_PURPOSE_REGISTER: &str = "register";
const EMAIL_CODE_PURPOSE_EMAIL_LOGIN: &str = "email_login";
const EMAIL_CODE_PURPOSE_PASSWORD_RESET: &str = "password_reset";
const EMAIL_CODE_PURPOSE_BIND_EMAIL: &str = "bind_email";
const EMAIL_CODE_EXPIRES_MINUTES: i64 = 10;
const SMS_CODE_PURPOSE_REGISTER: &str = "sms_register";
const SMS_CODE_PURPOSE_LOGIN: &str = "sms_login";
const SMS_CODE_PURPOSE_PASSWORD_RESET: &str = "sms_password_reset";
const SMS_CODE_PURPOSE_BIND_PHONE_NEW: &str = "bind_phone_new";
const SMS_CODE_PURPOSE_REBIND_PHONE_CURRENT: &str = "rebind_phone_current";
const EMAIL_LOGIN_SUBJECT: &str = "寻径星野登录验证码";
const PASSWORD_RESET_SUBJECT: &str = "寻径星野找回密码验证码";
const BIND_EMAIL_SUBJECT: &str = "寻径星野绑定邮箱验证码";
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
            nickname: None,
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
/// the plaintext code for smoke tests, while production sends it by SMTP.
pub async fn send_email_verification_code(
    state: &AppState,
    email: String,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    send_email_code_for_purpose(
        state,
        email,
        EMAIL_CODE_PURPOSE_REGISTER,
        state.config().mail.verification_subject.as_str(),
        false,
    )
    .await
}

/// Generates an email-login code for an existing account without revealing missing accounts.
pub async fn send_email_login_code(
    state: &AppState,
    email: String,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    send_email_code_for_purpose(
        state,
        email,
        EMAIL_CODE_PURPOSE_EMAIL_LOGIN,
        EMAIL_LOGIN_SUBJECT,
        true,
    )
    .await
}

/// Generates a password-reset code for an existing account without revealing missing accounts.
pub async fn send_password_reset_code(
    state: &AppState,
    email: String,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    send_email_code_for_purpose(
        state,
        email,
        EMAIL_CODE_PURPOSE_PASSWORD_RESET,
        PASSWORD_RESET_SUBJECT,
        true,
    )
    .await
}

/// Generates a one-time email verification code for binding an email to the current account.
pub async fn send_bind_email_code(
    state: &AppState,
    user: &UserRecord,
    email: String,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    let email = validate_email(email)?;
    ensure_user_can_bind_email(user, &email)?;
    let repo = AuthRepository::new(state.db().clone());
    ensure_email_available_for_binding(&repo, user, &email).await?;
    send_email_code_for_purpose(
        state,
        email,
        EMAIL_CODE_PURPOSE_BIND_EMAIL,
        BIND_EMAIL_SUBJECT,
        false,
    )
    .await
}

/// Generates an SMS code for phone/username/password registration.
pub async fn send_sms_registration_code(
    state: &AppState,
    phone: String,
) -> Result<SmsCodeResponse, ApiError> {
    let phone = validate_phone(phone)?;
    let repo = AuthRepository::new(state.db().clone());
    if repo.find_user_by_phone(&phone).await?.is_some() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "has already been registered",
        )]));
    }
    send_sms_code_for_purpose(
        state,
        &repo,
        phone,
        SMS_CODE_PURPOSE_REGISTER,
        state.config().sms.login_register_template_code.clone(),
    )
    .await
}

/// Generates an SMS login code for an existing phone account without revealing missing accounts.
pub async fn send_sms_login_code(
    state: &AppState,
    phone: String,
) -> Result<SmsCodeResponse, ApiError> {
    let phone = validate_phone(phone)?;
    ensure_sms_delivery_available(state)?;
    let repo = AuthRepository::new(state.db().clone());
    if repo.find_user_by_phone(&phone).await?.is_none() {
        return phantom_sms_code_response(state, phone);
    }
    send_sms_code_for_purpose(
        state,
        &repo,
        phone,
        SMS_CODE_PURPOSE_LOGIN,
        state.config().sms.login_register_template_code.clone(),
    )
    .await
}

/// Generates an SMS password-reset code for an existing phone account without revealing missing accounts.
pub async fn send_sms_password_reset_code(
    state: &AppState,
    phone: String,
) -> Result<SmsCodeResponse, ApiError> {
    let phone = validate_phone(phone)?;
    ensure_sms_delivery_available(state)?;
    let repo = AuthRepository::new(state.db().clone());
    if repo.find_user_by_phone(&phone).await?.is_none() {
        return phantom_sms_code_response(state, phone);
    }
    send_sms_code_for_purpose(
        state,
        &repo,
        phone,
        SMS_CODE_PURPOSE_PASSWORD_RESET,
        state.config().sms.password_reset_template_code.clone(),
    )
    .await
}

/// Generates an SMS code for binding the requested new phone to the current account.
pub async fn send_bind_phone_code(
    state: &AppState,
    user: &UserRecord,
    phone: String,
) -> Result<SmsCodeResponse, ApiError> {
    let phone = validate_phone(phone)?;
    ensure_user_can_bind_phone(user, &phone)?;
    let repo = AuthRepository::new(state.db().clone());
    ensure_phone_available_for_binding(&repo, user, &phone).await?;
    send_sms_code_for_purpose(
        state,
        &repo,
        phone,
        SMS_CODE_PURPOSE_BIND_PHONE_NEW,
        state.config().sms.bind_new_phone_template_code.clone(),
    )
    .await
}

/// Generates an SMS code to the current bound phone before replacing it.
pub async fn send_rebind_current_phone_code(
    state: &AppState,
    user: &UserRecord,
) -> Result<SmsCodeResponse, ApiError> {
    let Some(phone) = user.phone.clone() else {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "account does not have a bound phone",
        )]));
    };
    let repo = AuthRepository::new(state.db().clone());
    send_sms_code_for_purpose(
        state,
        &repo,
        phone,
        SMS_CODE_PURPOSE_REBIND_PHONE_CURRENT,
        state.config().sms.change_bound_phone_template_code.clone(),
    )
    .await
}

async fn send_email_code_for_purpose(
    state: &AppState,
    email: String,
    purpose: &'static str,
    subject: &str,
    require_existing_user: bool,
) -> Result<EmailVerificationCodeResponse, ApiError> {
    let email = validate_email(email)?;
    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(EMAIL_CODE_EXPIRES_MINUTES);
    let repo = AuthRepository::new(state.db().clone());
    let target_user_exists = if require_existing_user {
        repo.find_user_by_email(&email).await?.is_some()
    } else {
        true
    };

    if !target_user_exists {
        return Ok(EmailVerificationCodeResponse {
            email,
            expires_at: expires_at
                .format(&Iso8601::DEFAULT)
                .map_err(ApiError::internal)?,
            debug_code: None,
        });
    }

    let code = generate_email_code();
    let code_hash = hash_token(&code);
    repo.create_email_verification_code(&email, purpose, &code_hash, expires_at)
        .await?;

    if state.config().mail.enabled {
        state
            .email_sender()
            .send_verification_code(VerificationEmail {
                to: email.clone(),
                code: code.clone(),
                expires_minutes: EMAIL_CODE_EXPIRES_MINUTES,
                from: state.config().mail.from.clone(),
                subject: subject.to_owned(),
            })
            .await
            .map_err(|_error| ApiError::EmailDeliveryFailed)?;
    } else if state.config().app_env != "local" {
        return Err(ApiError::EmailDeliveryFailed);
    }

    // Local smoke tests get the plaintext code directly; production relies on SMTP delivery and never exposes it in the response.
    let debug_code = (state.config().app_env == "local").then_some(code);
    Ok(EmailVerificationCodeResponse {
        email,
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_code,
    })
}

async fn send_sms_code_for_purpose(
    state: &AppState,
    repo: &AuthRepository,
    phone: String,
    purpose: &'static str,
    template_code: String,
) -> Result<SmsCodeResponse, ApiError> {
    ensure_sms_delivery_available(state)?;
    let expires_at =
        OffsetDateTime::now_utc() + Duration::seconds(state.config().sms.valid_time_seconds as i64);
    let out_id = Uuid::new_v4().to_string();
    repo.create_sms_verification_challenge(&phone, purpose, &out_id, expires_at)
        .await?;

    let request = SmsSendCodeRequest {
        phone: phone.clone(),
        out_id: out_id.clone(),
        template_code,
        template_param: sms_template_param(state.config().sms.valid_time_seconds),
    };
    let sms_client = state.sms_client();
    let outcome = tokio::task::spawn_blocking(move || sms_client.send_verify_code(request))
        .await
        .map_err(ApiError::internal)?
        .map_err(map_sms_provider_error)?;

    Ok(SmsCodeResponse {
        phone,
        sms_ticket: out_id,
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_code: (state.config().app_env == "local")
            .then_some(outcome.debug_code)
            .flatten(),
    })
}

fn phantom_sms_code_response(state: &AppState, phone: String) -> Result<SmsCodeResponse, ApiError> {
    let expires_at =
        OffsetDateTime::now_utc() + Duration::seconds(state.config().sms.valid_time_seconds as i64);
    Ok(SmsCodeResponse {
        phone,
        sms_ticket: Uuid::new_v4().to_string(),
        expires_at: expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(ApiError::internal)?,
        debug_code: None,
    })
}

fn sms_template_param(valid_time_seconds: u64) -> String {
    let min = std::cmp::max(1, valid_time_seconds / 60);
    json!({
        "code": "##code##",
        "min": min.to_string(),
    })
    .to_string()
}

/// Logs in an existing account by consuming a one-time email login code.
pub async fn email_login(
    state: &AppState,
    payload: EmailLoginRequest,
) -> Result<LoginResponse, ApiError> {
    let email = validate_email(payload.email)?;
    let verification_code = validate_verification_code(payload.email_verification_code)?;
    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_email(&email).await? else {
        return Err(ApiError::InvalidCredentials);
    };
    let code_ok = repo
        .consume_email_verification_code(
            &email,
            EMAIL_CODE_PURPOSE_EMAIL_LOGIN,
            &hash_token(&verification_code),
        )
        .await?;
    if !code_ok {
        return Err(ApiError::InvalidCredentials);
    }
    repo.reset_failed_password_login(&user.id).await?;
    issue_login_for_user(&repo, user).await
}

/// Resets an existing account password after consuming a password-reset email code.
pub async fn password_reset(
    state: &AppState,
    payload: PasswordResetRequest,
) -> Result<LoginResponse, ApiError> {
    let email = validate_email(payload.email)?;
    let password = validate_password(payload.password)?;
    let confirm_password = payload.confirm_password;
    let verification_code = validate_verification_code(payload.email_verification_code)?;
    if password != confirm_password {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "confirm_password",
            "does not match password",
        )]));
    }

    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_email(&email).await? else {
        return Err(ApiError::InvalidCredentials);
    };
    let code_ok = repo
        .consume_email_verification_code(
            &email,
            EMAIL_CODE_PURPOSE_PASSWORD_RESET,
            &hash_token(&verification_code),
        )
        .await?;
    if !code_ok {
        return Err(ApiError::InvalidCredentials);
    }

    let updated = repo
        .update_user_password_hash(&user.id, &hash_token(&password))
        .await?;
    if !updated {
        return Err(ApiError::Unauthorized);
    }
    repo.revoke_user_sessions(&user.id).await?;
    issue_login_for_user(&repo, user).await
}

/// Logs in an existing account by checking a one-time SMS code.
pub async fn sms_login(
    state: &AppState,
    payload: SmsLoginRequest,
) -> Result<LoginResponse, ApiError> {
    let phone = validate_phone(payload.phone)?;
    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_phone(&phone).await? else {
        return Err(ApiError::InvalidCredentials);
    };
    verify_sms_code(
        state,
        &repo,
        &phone,
        SMS_CODE_PURPOSE_LOGIN,
        payload.sms_ticket,
        payload.sms_verification_code,
    )
    .await?;
    repo.reset_failed_password_login(&user.id).await?;
    issue_login_for_user(&repo, user).await
}

/// Resets an existing account password after checking a one-time SMS code.
pub async fn sms_password_reset(
    state: &AppState,
    payload: SmsPasswordResetRequest,
) -> Result<LoginResponse, ApiError> {
    let phone = validate_phone(payload.phone)?;
    let password = validate_password(payload.password)?;
    let confirm_password = payload.confirm_password;
    if password != confirm_password {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "confirm_password",
            "does not match password",
        )]));
    }

    let repo = AuthRepository::new(state.db().clone());
    let Some(user) = repo.find_user_by_phone(&phone).await? else {
        return Err(ApiError::InvalidCredentials);
    };
    verify_sms_code(
        state,
        &repo,
        &phone,
        SMS_CODE_PURPOSE_PASSWORD_RESET,
        payload.sms_ticket,
        payload.sms_verification_code,
    )
    .await?;

    let updated = repo
        .update_user_password_hash(&user.id, &hash_token(&password))
        .await?;
    if !updated {
        return Err(ApiError::Unauthorized);
    }
    repo.revoke_user_sessions(&user.id).await?;
    issue_login_for_user(&repo, user).await
}

/// Binds a verified email address to the current account.
///
/// This is primarily used by accounts created through WeChat one-tap login, whose
/// initial user row has no email or password credentials. Once an email is bound,
/// the same password-reset flow can set a password for that account.
pub async fn bind_email(
    state: &AppState,
    user: UserRecord,
    payload: BindEmailRequest,
) -> Result<BindEmailResponse, ApiError> {
    let email = validate_email(payload.email)?;
    let verification_code = validate_verification_code(payload.email_verification_code)?;
    ensure_user_can_bind_email(&user, &email)?;

    let repo = AuthRepository::new(state.db().clone());
    ensure_email_available_for_binding(&repo, &user, &email).await?;
    let code_ok = repo
        .consume_email_verification_code(
            &email,
            EMAIL_CODE_PURPOSE_BIND_EMAIL,
            &hash_token(&verification_code),
        )
        .await?;
    if !code_ok {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email_verification_code",
            "is invalid or expired",
        )]));
    }

    let Some(updated_user) = repo.bind_user_email(&user.id, &email).await? else {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email",
            "has already been registered",
        )]));
    };
    Ok(BindEmailResponse {
        user: login_user_response(updated_user),
    })
}

/// Binds or replaces the current account's phone number after SMS verification.
pub async fn bind_phone(
    state: &AppState,
    user: UserRecord,
    payload: BindPhoneRequest,
) -> Result<BindPhoneResponse, ApiError> {
    let phone = validate_phone(payload.phone)?;
    ensure_user_can_bind_phone(&user, &phone)?;
    let repo = AuthRepository::new(state.db().clone());
    ensure_phone_available_for_binding(&repo, &user, &phone).await?;

    if let Some(current_phone) = user.phone.as_deref() {
        let current_ticket = payload.current_sms_ticket.ok_or_else(|| {
            ApiError::Validation(vec![FieldViolation::new(
                "current_sms_ticket",
                "is required when replacing a bound phone",
            )])
        })?;
        let current_code = payload.current_sms_verification_code.ok_or_else(|| {
            ApiError::Validation(vec![FieldViolation::new(
                "current_sms_verification_code",
                "is required when replacing a bound phone",
            )])
        })?;
        verify_sms_code(
            state,
            &repo,
            current_phone,
            SMS_CODE_PURPOSE_REBIND_PHONE_CURRENT,
            current_ticket,
            current_code,
        )
        .await?;
    }

    verify_sms_code(
        state,
        &repo,
        &phone,
        SMS_CODE_PURPOSE_BIND_PHONE_NEW,
        payload.sms_ticket,
        payload.sms_verification_code,
    )
    .await?;

    let Some(updated_user) = repo.bind_user_phone(&user.id, &phone).await? else {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "has already been registered",
        )]));
    };
    Ok(BindPhoneResponse {
        user: login_user_response(updated_user),
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

/// Completes phone/username/password registration by validating the SMS code and password.
pub async fn register_with_sms(
    state: &AppState,
    payload: SmsRegisterRequest,
) -> Result<LoginResponse, ApiError> {
    let username = validate_username(payload.username)?;
    let nickname = validate_nickname(payload.nickname)?;
    let phone = validate_phone(payload.phone)?;
    let password = validate_password(payload.password)?;
    let confirm_password = payload.confirm_password;

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
    if repo.find_user_by_phone(&phone).await?.is_some() {
        errors.push(FieldViolation::new("phone", "has already been registered"));
    }
    if !errors.is_empty() {
        return Err(ApiError::Validation(errors));
    }

    verify_sms_code(
        state,
        &repo,
        &phone,
        SMS_CODE_PURPOSE_REGISTER,
        payload.sms_ticket,
        payload.sms_verification_code,
    )
    .await?;

    let user = repo
        .create_phone_password_user(&username, &nickname, &phone, &hash_token(&password))
        .await?;
    issue_login_for_user(&repo, user).await
}

/// Authenticates a username, email, or phone password login and issues a fresh token pair.
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
        nickname: None,
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
        .upsert_wechat_user(
            openid,
            normalize_optional_profile_field(profile.nickname),
            normalize_optional_profile_field(profile.avatar_url),
        )
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
        user: login_user_response(user),
    })
}

pub(crate) fn login_user_response(user: UserRecord) -> LoginUserResponse {
    LoginUserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        phone: user.phone,
        nickname: user.nickname,
        avatar_url: user.avatar_url,
    }
}

fn normalize_optional_profile_field(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn ensure_user_can_bind_email(user: &UserRecord, email: &str) -> Result<(), ApiError> {
    if let Some(existing_email) = user.email.as_deref() {
        let message = if existing_email == email {
            "is already bound to this account"
        } else {
            "account already has a bound email"
        };
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email", message,
        )]));
    }
    Ok(())
}

async fn ensure_email_available_for_binding(
    repo: &AuthRepository,
    user: &UserRecord,
    email: &str,
) -> Result<(), ApiError> {
    if let Some(existing_user) = repo.find_user_by_email(email).await?
        && existing_user.id != user.id
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "email",
            "has already been registered",
        )]));
    }
    Ok(())
}

fn ensure_user_can_bind_phone(user: &UserRecord, phone: &str) -> Result<(), ApiError> {
    if let Some(existing_phone) = user.phone.as_deref()
        && existing_phone == phone
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "is already bound to this account",
        )]));
    }
    Ok(())
}

async fn ensure_phone_available_for_binding(
    repo: &AuthRepository,
    user: &UserRecord,
    phone: &str,
) -> Result<(), ApiError> {
    if let Some(existing_user) = repo.find_user_by_phone(phone).await?
        && existing_user.id != user.id
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "has already been registered",
        )]));
    }
    Ok(())
}

async fn verify_sms_code(
    state: &AppState,
    repo: &AuthRepository,
    phone: &str,
    purpose: &'static str,
    sms_ticket: String,
    sms_verification_code: String,
) -> Result<(), ApiError> {
    let sms_ticket = validate_sms_ticket(sms_ticket)?;
    let sms_verification_code = validate_sms_verification_code(sms_verification_code)?;
    let Some(challenge) = repo
        .find_active_sms_verification_challenge(phone, purpose, &sms_ticket)
        .await?
    else {
        return Err(invalid_sms_code_error());
    };
    let request = SmsCheckCodeRequest {
        phone: phone.to_owned(),
        out_id: challenge.out_id.clone(),
        verify_code: sms_verification_code,
    };
    let sms_client = state.sms_client();
    let outcome = tokio::task::spawn_blocking(move || sms_client.check_verify_code(request))
        .await
        .map_err(ApiError::internal)?
        .map_err(map_sms_provider_error)?;
    if !outcome.passed {
        return Err(invalid_sms_code_error());
    }
    if !repo
        .consume_sms_verification_challenge(&challenge.id)
        .await?
    {
        return Err(invalid_sms_code_error());
    }
    Ok(())
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

/// Normalizes a Chinese mainland phone number into 11 digits.
fn validate_phone(phone: String) -> Result<String, ApiError> {
    let mut phone = phone
        .trim()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace() && *ch != '-')
        .collect::<String>();
    if let Some(rest) = phone.strip_prefix("+86") {
        phone = rest.to_owned();
    } else if phone.len() == 13 && phone.starts_with("86") {
        phone = phone[2..].to_owned();
    }
    let valid =
        phone.len() == 11 && phone.starts_with('1') && phone.chars().all(|ch| ch.is_ascii_digit());
    if !valid {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "phone",
            "must be a valid Chinese mainland phone number",
        )]));
    }
    Ok(phone)
}

fn validate_nickname(nickname: String) -> Result<String, ApiError> {
    let nickname = nickname.trim();
    let len = nickname.chars().count();
    if !(1..=64).contains(&len) {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "nickname",
            "must be between 1 and 64 characters",
        )]));
    }
    Ok(nickname.to_owned())
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

/// Trims the username, email, or phone identifier used by password login.
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

fn validate_sms_ticket(ticket: String) -> Result<String, ApiError> {
    let ticket = ticket.trim();
    if ticket.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "sms_ticket",
            "is required",
        )]));
    }
    Ok(ticket.to_owned())
}

fn validate_sms_verification_code(code: String) -> Result<String, ApiError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "sms_verification_code",
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

fn ensure_sms_delivery_available(state: &AppState) -> Result<(), ApiError> {
    if state.config().sms.enabled || state.config().app_env == "local" {
        Ok(())
    } else {
        Err(ApiError::SmsDeliveryFailed)
    }
}

fn invalid_sms_code_error() -> ApiError {
    ApiError::Validation(vec![FieldViolation::new(
        "sms_verification_code",
        "is invalid or expired",
    )])
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

fn map_sms_provider_error(error: SmsVerificationError) -> ApiError {
    match error {
        SmsVerificationError::RateLimited { .. } => ApiError::RateLimited {
            retry_after_seconds: 60,
        },
        SmsVerificationError::Rejected { .. } | SmsVerificationError::HttpStatus { .. } => {
            ApiError::SmsDeliveryFailed
        }
        other => ApiError::internal(other),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sms_template_param_uses_aliyun_generated_code_placeholder() {
        let value: serde_json::Value = serde_json::from_str(&sms_template_param(300)).unwrap();

        assert_eq!(value["code"], "##code##");
        assert_eq!(value["min"], "5");
    }

    #[test]
    fn sms_http_status_errors_are_delivery_failures() {
        let error = map_sms_provider_error(SmsVerificationError::HttpStatus {
            status: 403,
            body: "forbidden".to_owned(),
        });

        assert!(matches!(error, ApiError::SmsDeliveryFailed));
    }
}
