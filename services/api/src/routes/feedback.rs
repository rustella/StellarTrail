//! Authenticated current-user feedback routes.

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use stellartrail_db::repositories::FeedbackRepository;

use crate::{
    dto::feedback::{
        CreateFeedbackRequest, FeedbackResponse, ListAdminFeedbackQuery, ListAdminFeedbackResponse,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::{admin_service, feedback_service},
    state::AppState,
};

/// Feedback route group.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/feedback", post(create_feedback))
        .route("/admin/feedback", get(list_admin_feedback))
        .route("/admin/feedback/:id", delete(delete_admin_feedback))
        .route("/admin/feedback/:id/restore", post(restore_admin_feedback))
        .route(
            "/admin/feedback-images/:id",
            get(download_admin_feedback_image),
        )
}

/// Creates feedback for the authenticated user and binds any provided image IDs.
async fn create_feedback(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Result<(StatusCode, Json<FeedbackResponse>), ApiError> {
    let response = feedback_service::create_feedback(&state, &user.id, payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Soft-deletes a feedback row from administrator dashboards.
async fn delete_admin_feedback(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let deleted = FeedbackRepository::new(state.db().clone())
        .soft_delete(&id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Restores a feedback row previously hidden by administrator soft-delete.
async fn restore_admin_feedback(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let restored = FeedbackRepository::new(state.db().clone())
        .restore_deleted(&id)
        .await?;
    if restored {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Lists user feedback for administrators.
async fn list_admin_feedback(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListAdminFeedbackQuery>,
) -> Result<Json<ListAdminFeedbackResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    Ok(Json(
        feedback_service::list_admin_feedback(&state, query).await?,
    ))
}

/// Streams a feedback image for administrators while preserving bearer-token authorization.
async fn download_admin_feedback_image(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let upload = FeedbackRepository::new(state.db().clone())
        .get_feedback_image_for_admin(&id)
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
