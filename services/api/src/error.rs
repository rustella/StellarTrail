use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    BadRequest {
        code: &'static str,
        message: String,
        parameter: Option<String>,
    },
    NotFound,
    Internal(String),
}

impl ApiError {
    pub fn unsupported_query_parameter(parameter: impl Into<String>) -> Self {
        let parameter = parameter.into();
        Self::BadRequest {
            code: "unsupported_query_parameter",
            message: format!("query parameter `{parameter}` is not supported"),
            parameter: Some(parameter),
        }
    }

    pub fn invalid_header(header: impl Into<String>) -> Self {
        let header = header.into();
        Self::BadRequest {
            code: "invalid_header",
            message: format!("header `{header}` is invalid"),
            parameter: Some(header),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message, parameter) = match self {
            Self::BadRequest {
                code,
                message,
                parameter,
            } => (StatusCode::BAD_REQUEST, code, message, parameter),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_owned(),
                None,
            ),
            Self::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                message,
                None,
            ),
        };

        (
            status,
            Json(ErrorBody {
                code,
                message,
                parameter,
            }),
        )
            .into_response()
    }
}

impl From<stellartrail_db::DbError> for ApiError {
    fn from(value: stellartrail_db::DbError) -> Self {
        Self::Internal(value.to_string())
    }
}
