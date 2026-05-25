//! Shared soft-delete filter values used by list endpoints.

use serde::{Deserialize, Serialize};

/// Soft-delete visibility filter shared by API query parameters and repository options.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletedFilter {
    #[default]
    Active,
    Deleted,
    All,
}

impl DeletedFilter {
    /// Returns the stable wire key used by API query parameters.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deleted => "deleted",
            Self::All => "all",
        }
    }

    /// Parses a stable wire key into a soft-delete filter.
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "deleted" => Some(Self::Deleted),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}
