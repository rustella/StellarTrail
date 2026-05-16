//! User domain model describing the core profile fields that flow between the API and database.

use serde::{Deserialize, Serialize};

/// Stable data boundary for `UserProfile`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
