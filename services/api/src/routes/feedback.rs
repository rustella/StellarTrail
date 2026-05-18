//! Authenticated current-user feedback routes.

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

use crate::{
    dto::feedback::{CreateFeedbackRequest, FeedbackResponse},
    error::ApiError,
    extractors::AuthenticatedUser,
    services::feedback_service,
    state::AppState,
};

/// Feedback route group.
pub fn routes() -> Router<AppState> {
    Router::new().route("/api/me/feedback", post(create_feedback))
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
