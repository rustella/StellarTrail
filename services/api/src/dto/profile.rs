//! Profile HTTP DTOs for authenticated current-user profile updates.

use serde::Serialize;

use crate::dto::auth::LoginUserResponse;

/// Response returned after updating the authenticated user's public profile.
#[derive(Debug, Serialize)]
pub struct ProfileUserResponse {
    pub user: LoginUserResponse,
}
