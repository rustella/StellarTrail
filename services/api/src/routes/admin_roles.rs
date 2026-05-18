//! Administrator role management routes.
//!
//! These endpoints are intentionally restricted to `super_admin` callers. They
//! can grant or revoke regular `admin` roles for existing users, while
//! `super_admin` lifecycle remains a separate bootstrap or future explicit flow.

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::post,
};

use crate::{
    dto::admin::{AdminRoleResponse, AdminUserSelector},
    error::ApiError,
    extractors::AuthenticatedUser,
    services::admin_service,
    state::AppState,
};

/// Builds administrator role-management routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/api/admin/admins", post(grant_admin).delete(revoke_admin))
}

async fn grant_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(selector): Json<AdminUserSelector>,
) -> Result<(StatusCode, Json<AdminRoleResponse>), ApiError> {
    let result = admin_service::grant_admin(&state, &user, selector.into()).await?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        Json(AdminRoleResponse {
            user_id: result.record.user_id,
            role: result.record.role,
        }),
    ))
}

async fn revoke_admin(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(selector): Query<AdminUserSelector>,
) -> Result<StatusCode, ApiError> {
    admin_service::revoke_admin(&state, &user, selector.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}
