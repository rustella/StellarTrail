//! Disclaimer HTTP DTOs for account-scoped acknowledgement flows.

use serde::{Deserialize, Serialize};

/// Current-user disclaimer response for the active version.
#[derive(Clone, Debug, Serialize)]
pub struct KnotDisclaimerResponse {
    pub key: &'static str,
    pub version: &'static str,
    pub title: &'static str,
    pub content: &'static str,
    pub accepted: bool,
    pub accepted_at: Option<String>,
}

/// Client metadata attached when the current user accepts a disclaimer.
#[derive(Clone, Debug, Deserialize)]
pub struct AcceptKnotDisclaimerRequest {
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
}
