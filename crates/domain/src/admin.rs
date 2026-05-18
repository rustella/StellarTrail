//! Administrator role domain types.
//!
//! Roles are intentionally small and database-backed. `super_admin` can manage
//! other administrators, while both roles can access existing administrator
//! capabilities such as media upload, API usage, and gear atlas review.

use serde::{Deserialize, Serialize};

/// Database-backed administrator role.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    Admin,
    SuperAdmin,
}

impl AdminRole {
    /// Returns every role accepted by the database check constraint.
    pub const ALL: [Self; 2] = [Self::Admin, Self::SuperAdmin];

    /// Stable string stored in `admin_roles.role` and exposed by JSON APIs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::SuperAdmin => "super_admin",
        }
    }

    /// Parses a persisted role string.
    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.as_str() == value)
    }

    /// Returns true when this role grants all administrator capabilities.
    pub fn can_administer(self) -> bool {
        matches!(self, Self::Admin | Self::SuperAdmin)
    }

    /// Returns true when this role can grant or revoke regular administrators.
    pub fn can_manage_admins(self) -> bool {
        matches!(self, Self::SuperAdmin)
    }
}

impl std::fmt::Display for AdminRole {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
