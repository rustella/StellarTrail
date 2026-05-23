//! Authenticated private upload routes for feedback images.

use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    dto::upload::UploadImageResponse, error::ApiError, extractors::AuthenticatedUser,
    services::upload_service, state::AppState,
};

/// Upload route group.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/uploads", post(upload_feedback_image))
        .route("/me/uploads/:id", get(download_upload))
}

/// Accepts a multipart feedback image upload for the authenticated user.
async fn upload_feedback_image(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadImageResponse>), ApiError> {
    let mut file_name = None;
    let mut content_type = None;
    let mut bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::BadRequest(format!("invalid multipart body: {error}")))?
    {
        if field.name() == Some("file") {
            file_name = field.file_name().map(ToOwned::to_owned);
            content_type = field.content_type().map(ToOwned::to_owned);
            let field_bytes = field.bytes().await.map_err(|error| {
                ApiError::BadRequest(format!("invalid multipart file: {error}"))
            })?;
            bytes = Some(field_bytes.to_vec());
            break;
        }
    }

    let bytes = bytes.ok_or_else(|| {
        ApiError::Validation(vec![stellartrail_domain::validation::FieldViolation::new(
            "file",
            "is required",
        )])
    })?;
    let response = upload_service::upload_feedback_image(
        &state,
        &user.id,
        file_name.as_deref(),
        content_type.as_deref(),
        bytes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Streams a private uploaded image through API authentication/ownership checks.
async fn download_upload(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let upload = upload_service::get_upload_for_user(&state, &user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let object = state
        .object_store()
        .get_image(&upload.object_key)
        .await
        .map_err(ApiError::internal)?
        .ok_or(ApiError::NotFound)?;
    let disposition = format!(
        "inline; filename=\"{}\"",
        safe_header_filename(&upload.original_filename)
    );
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, object.content_type),
            (header::CONTENT_DISPOSITION, disposition),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        Body::from(object.bytes),
    )
        .into_response())
}

fn safe_header_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '"' && *ch != '\\')
        .collect::<String>()
}
