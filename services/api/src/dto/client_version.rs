//! Client version HTTP DTOs for public release notes and administrator maintenance.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::ClientVersionRecord;

use crate::services::client_version_service::ValidatedClientVersionDraft;

/// Public and administrator client version response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientVersionResponse {
    pub id: String,
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes: Vec<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated client version list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListClientVersionsResponse {
    pub items: Vec<ClientVersionResponse>,
    pub next_cursor: Option<String>,
}

/// Public current/list query.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClientVersionPublicQuery {
    pub client_key: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Administrator list query.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClientVersionAdminQuery {
    pub client_key: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

/// Administrator create/update request.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientVersionRequest {
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes: Vec<String>,
    pub status: String,
}

impl From<ClientVersionRequest> for ValidatedClientVersionDraft {
    fn from(value: ClientVersionRequest) -> Self {
        Self {
            client_key: value.client_key,
            version: value.version,
            title: value.title,
            release_notes: value.release_notes,
            status: value.status,
        }
    }
}

impl ClientVersionResponse {
    /// Converts a persisted record to an API response.
    pub fn from_record(record: &ClientVersionRecord) -> Self {
        Self {
            id: record.id.clone(),
            client_key: record.client_key.clone(),
            version: record.version.clone(),
            title: record.title.clone(),
            release_notes: serde_json::from_str(&record.release_notes_json).unwrap_or_default(),
            status: record.status.clone(),
            published_at: record.published_at.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}
