//! API 错误模型与 Axum 响应转换模块，统一业务错误、校验错误和认证错误的 HTTP 表达。

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use stellartrail_domain::validation::FieldViolation;

/// API 统一错误枚举，覆盖校验失败、认证失败、资源不存在、验证码和内部错误。
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

/// ErrorBody 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldViolation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    captcha: Option<CaptchaChallenge>,
}

/// CaptchaChallenge 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Serialize)]
struct CaptchaChallenge {
    #[serde(rename = "type")]
    captcha_type: &'static str,
    endpoint: &'static str,
}

impl ApiError {
    /// 执行 `internal` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        Self::Internal(error.into())
    }
}

impl IntoResponse for ApiError {
    /// 执行 `into response` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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
                    endpoint: "/api/auth/captcha",
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
    /// 执行 `from` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn from(value: sea_orm::DbErr) -> Self {
        Self::internal(value)
    }
}

impl From<anyhow::Error> for ApiError {
    /// 执行 `from` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<stellartrail_domain::validation::ValidationError> for ApiError {
    /// 执行 `from` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn from(value: stellartrail_domain::validation::ValidationError) -> Self {
        Self::Validation(value.fields)
    }
}
