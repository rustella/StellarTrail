//! Profile HTTP DTOs for authenticated current-user profile updates.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use stellartrail_domain::outdoor_profile::OutdoorProfile;

use crate::dto::auth::LoginUserResponse;

/// Response returned after updating the authenticated user's public profile.
#[derive(Debug, Serialize)]
pub struct ProfileUserResponse {
    pub user: LoginUserResponse,
}

/// Response returned for the authenticated user's reusable outdoor profile.
#[derive(Debug, Serialize)]
pub struct OutdoorProfileResponse {
    pub profile: OutdoorProfile,
}

/// Sparse PATCH payload for account-level outdoor profile defaults.
#[derive(Debug, Deserialize)]
pub struct UpdateOutdoorProfileRequest {
    #[serde(flatten)]
    pub fields: BTreeMap<String, JsonValue>,
}
