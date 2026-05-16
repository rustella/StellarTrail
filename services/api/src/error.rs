use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use stellartrail_domain::validation::FieldViolation;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    InvalidCredentials,
    CaptchaRequired,
    NotFound,
    Validation(Vec<FieldViolation>),
    Internal(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldViolation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    captcha: Option<CaptchaChallenge>,
}

#[derive(Serialize)]
struct CaptchaChallenge {
    #[serde(rename = "type")]
    captcha_type: &'static str,
}

impl ApiError {
    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        Self::Internal(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message, fields, captcha) = match self {
            Self::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, "bad_request", message, None, None)
            }
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid bearer token".to_owned(),
                None,
                None,
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "用户名/邮箱或密码不正确".to_owned(),
                None,
                None,
            ),
            Self::CaptchaRequired => (
                StatusCode::PRECONDITION_REQUIRED,
                "captcha_required",
                "多次登录失败，请先完成验证码验证".to_owned(),
                None,
                Some(CaptchaChallenge {
                    captcha_type: "image",
                }),
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_owned(),
                None,
                None,
            ),
            Self::Validation(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                "request validation failed".to_owned(),
                Some(fields),
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
                )
            }
        };

        (
            status,
            Json(ErrorBody {
                code,
                message,
                fields,
                captcha,
            }),
        )
            .into_response()
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::internal(value)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<stellartrail_domain::validation::ValidationError> for ApiError {
    fn from(value: stellartrail_domain::validation::ValidationError) -> Self {
        Self::Validation(value.fields)
    }
}
