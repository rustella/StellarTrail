//! Authenticated profile routes for current-user display data.

use axum::{
    Json, Router,
    extract::{Multipart, State},
    http::StatusCode,
    routing::{get, put},
};

use crate::{
    dto::profile::ProfileUserResponse, error::ApiError, extractors::AuthenticatedUser,
    services::profile_service, state::AppState,
};

/// Profile route group.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/me/profile", get(current_profile))
        .route(
            "/api/me/profile/avatar",
            put(upload_avatar).post(upload_avatar),
        )
}

/// Returns the authenticated user's current profile snapshot.
async fn current_profile(
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<ProfileUserResponse>, ApiError> {
    Ok(Json(profile_service::current_profile(&user)))
}

/// Accepts a multipart avatar image upload for the authenticated user.
async fn upload_avatar(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ProfileUserResponse>), ApiError> {
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
    let response = profile_service::upload_avatar(
        &state,
        &user,
        file_name.as_deref(),
        content_type.as_deref(),
        bytes,
    )
    .await?;
    Ok((StatusCode::OK, Json(response)))
}
