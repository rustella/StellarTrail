//! HTTP DTOs for database-backed administrator role management.

use serde::{Deserialize, Serialize};
use stellartrail_domain::admin::AdminRole;

use crate::services::admin_service::AdminUserSelectorInput;

/// Request body or query selector for choosing an existing user.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdminUserSelector {
    pub username: Option<String>,
    pub user_id: Option<String>,
}

impl From<AdminUserSelector> for AdminUserSelectorInput {
    fn from(value: AdminUserSelector) -> Self {
        Self {
            username: value.username,
            user_id: value.user_id,
        }
    }
}

/// Administrator role response returned after grants.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminRoleResponse {
    pub user_id: String,
    pub role: AdminRole,
}
