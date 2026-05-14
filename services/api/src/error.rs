use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "resource not found"),
        };

        (
            status,
            Json(ErrorBody {
                code,
                message: message.to_owned(),
            }),
        )
            .into_response()
    }
}
