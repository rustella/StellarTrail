//! API error model and Axum response conversion module for consistent HTTP representations of business, validation, and authentication errors.

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;
use stellartrail_domain::{trip::FieldConflict, validation::FieldViolation};

use crate::routes::CAPTCHA_ENDPOINT;

/// Unified API error enum covering validation failures, authentication failures, missing resources, captcha requirements, and internal errors.
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    BadRequestWithCode {
        code: &'static str,
        message: String,
        parameter: Option<String>,
    },
    Unauthorized,
    Forbidden,
    InvalidCredentials,
    CaptchaRequired,
    NotFound,
    Validation(Vec<FieldViolation>),
    EditConflict(Vec<FieldConflict>),
    PayloadTooLarge {
        max_bytes: u64,
    },
    UnsupportedMediaType(String),
    RateLimited {
        retry_after_seconds: u64,
    },
    EmailDeliveryFailed,
    SmsDeliveryFailed,
    Internal(anyhow::Error),
}

/// Stable data boundary for `ErrorBody`, exposed by or reused within this module.
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldViolation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflicts: Option<Vec<FieldConflict>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    captcha: Option<CaptchaChallenge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter: Option<String>,
}

/// Stable data boundary for `CaptchaChallenge`, exposed by or reused within this module.
#[derive(Serialize)]
struct CaptchaChallenge {
    #[serde(rename = "type")]
    captcha_type: &'static str,
    endpoint: &'static str,
}

impl ApiError {
    /// Runs the `internal` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        Self::Internal(error.into())
    }

    /// Builds an unsupported query parameter error with a stable machine-readable code.
    pub fn unsupported_query_parameter(parameter: impl Into<String>) -> Self {
        let parameter = parameter.into();
        Self::BadRequestWithCode {
            code: "unsupported_query_parameter",
            message: format!("query parameter `{parameter}` is not supported"),
            parameter: Some(parameter),
        }
    }

    /// Builds an invalid header error with a stable machine-readable code.
    pub fn invalid_header(parameter: impl Into<String>) -> Self {
        let parameter = parameter.into();
        Self::BadRequestWithCode {
            code: "invalid_header",
            message: format!("header `{parameter}` is invalid"),
            parameter: Some(parameter),
        }
    }

    /// Builds an invalid query parameter error with a stable machine-readable code.
    pub fn invalid_query_parameter(parameter: impl Into<String>, message: String) -> Self {
        Self::BadRequestWithCode {
            code: "invalid_query_parameter",
            message,
            parameter: Some(parameter.into()),
        }
    }
}

impl IntoResponse for ApiError {
    /// Runs the `into response` server-side flow while preserving input validation, error propagation, and state invariants.
    fn into_response(self) -> axum::response::Response {
        let (status, code, message, fields, conflicts, captcha, parameter, retry_after) = match self
        {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                message,
                None,
                None,
                None,
                None,
                None,
            ),
            Self::BadRequestWithCode {
                code,
                message,
                parameter,
            } => (
                StatusCode::BAD_REQUEST,
                code,
                message,
                None,
                None,
                None,
                parameter,
                None,
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid bearer token".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "administrator permission is required".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "用户名/邮箱/手机号或密码不正确".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::CaptchaRequired => (
                StatusCode::PRECONDITION_REQUIRED,
                "captcha_required",
                "多次登录失败，请先完成验证码验证".to_owned(),
                None,
                None,
                Some(CaptchaChallenge {
                    captcha_type: "image",
                    endpoint: CAPTCHA_ENDPOINT,
                }),
                None,
                None,
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::Validation(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                "request validation failed".to_owned(),
                Some(fields),
                None,
                None,
                None,
                None,
            ),
            Self::EditConflict(conflicts) => (
                StatusCode::CONFLICT,
                "edit_conflict",
                "record was changed by another member".to_owned(),
                None,
                Some(conflicts),
                None,
                None,
                None,
            ),
            Self::PayloadTooLarge { max_bytes } => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                format!("file must be at most {max_bytes} bytes"),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::UnsupportedMediaType(message) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                message,
                None,
                None,
                None,
                None,
                None,
            ),
            Self::RateLimited {
                retry_after_seconds,
            } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                format!("Too many requests. Please retry after {retry_after_seconds} seconds."),
                None,
                None,
                None,
                None,
                Some(retry_after_seconds),
            ),
            Self::EmailDeliveryFailed => (
                StatusCode::BAD_GATEWAY,
                "email_delivery_failed",
                "邮箱验证码发送失败，请稍后重试".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::SmsDeliveryFailed => (
                StatusCode::BAD_GATEWAY,
                "sms_delivery_failed",
                "短信验证码发送失败，请稍后重试".to_owned(),
                None,
                None,
                None,
                None,
                None,
            ),
            Self::Internal(error) => {
                tracing::error!(error = %error, "api internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_owned(),
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            }
        };

        let mut response = (
            status,
            Json(ErrorBody {
                code,
                message,
                fields,
                conflicts,
                captcha,
                parameter,
            }),
        )
            .into_response();
        if let Some(value) =
            retry_after.and_then(|retry_after| HeaderValue::from_str(&retry_after.to_string()).ok())
        {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        response
    }
}

impl From<sea_orm::DbErr> for ApiError {
    /// Runs the `from` server-side flow while preserving input validation, error propagation, and state invariants.
    fn from(value: sea_orm::DbErr) -> Self {
        Self::internal(value)
    }
}

impl From<anyhow::Error> for ApiError {
    /// Runs the `from` server-side flow while preserving input validation, error propagation, and state invariants.
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<stellartrail_domain::validation::ValidationError> for ApiError {
    /// Runs the `from` server-side flow while preserving input validation, error propagation, and state invariants.
    fn from(value: stellartrail_domain::validation::ValidationError) -> Self {
        Self::Validation(value.fields)
    }
}

impl From<stellartrail_db::repositories::TripRepositoryError> for ApiError {
    /// Converts trip persistence failures into the shared API error envelope.
    fn from(value: stellartrail_db::repositories::TripRepositoryError) -> Self {
        match value {
            stellartrail_db::repositories::TripRepositoryError::Db(error) => error.into(),
            stellartrail_db::repositories::TripRepositoryError::Validation(error) => error.into(),
            stellartrail_db::repositories::TripRepositoryError::Conflict(conflicts) => {
                Self::EditConflict(conflicts)
            }
        }
    }
}
