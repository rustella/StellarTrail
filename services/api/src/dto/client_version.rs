//! Client version HTTP DTOs for public release notes and administrator maintenance.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use stellartrail_db::repositories::ClientVersionRecord;

use crate::services::client_version_service::{
    ValidatedClientVersionDraft, ValidatedReleaseNoteSection,
};

const RELEASE_NOTE_SECTION_ORDER: [(&str, &str); 3] = [
    ("feature", "Feature"),
    ("bug_fix", "BugFix"),
    ("notes", "Notes"),
];

/// One grouped section in public release notes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientVersionReleaseNoteSection {
    pub key: String,
    pub title: String,
    pub items: Vec<String>,
}

/// Public and administrator client version response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientVersionResponse {
    pub id: String,
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes: Vec<String>,
    pub release_note_sections: Vec<ClientVersionReleaseNoteSection>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
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
    #[serde(default)]
    pub release_notes: Vec<String>,
    #[serde(default)]
    pub release_note_sections: Vec<ClientVersionReleaseNoteSection>,
    pub status: String,
    #[serde(default)]
    pub commit_hash: Option<String>,
}

impl From<ClientVersionRequest> for ValidatedClientVersionDraft {
    fn from(value: ClientVersionRequest) -> Self {
        Self {
            client_key: value.client_key,
            version: value.version,
            title: value.title,
            release_notes: value.release_notes,
            commit_hash: value.commit_hash,
            release_note_sections: value
                .release_note_sections
                .into_iter()
                .map(|section| ValidatedReleaseNoteSection {
                    key: section.key,
                    title: section.title,
                    items: section.items,
                })
                .collect(),
            status: value.status,
        }
    }
}

impl ClientVersionResponse {
    /// Converts a persisted record to a public API response without internal commit tracking.
    pub fn from_record_public(record: &ClientVersionRecord) -> Self {
        Self::from_record(record, None)
    }

    /// Converts a persisted record to an administrator API response with internal commit tracking.
    pub fn from_record_admin(record: &ClientVersionRecord) -> Self {
        Self::from_record(record, record.commit_hash.clone())
    }

    fn from_record(record: &ClientVersionRecord, commit_hash: Option<String>) -> Self {
        let release_note_sections = parse_release_note_sections(&record.release_notes_json);
        let release_notes = release_note_sections
            .iter()
            .flat_map(|section| section.items.iter().cloned())
            .collect();
        Self {
            id: record.id.clone(),
            client_key: record.client_key.clone(),
            version: record.version.clone(),
            title: record.title.clone(),
            release_notes,
            release_note_sections,
            status: record.status.clone(),
            commit_hash,
            published_at: record.published_at.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

fn parse_release_note_sections(raw: &str) -> Vec<ClientVersionReleaseNoteSection> {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    if let Some(items) = value.as_array() {
        let notes = items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        return section_if_not_empty("feature", "Feature", notes);
    }
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    RELEASE_NOTE_SECTION_ORDER
        .iter()
        .flat_map(|(key, title)| {
            let items = object
                .get(*key)
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            section_if_not_empty(key, title, items)
        })
        .collect()
}

fn section_if_not_empty(
    key: &str,
    title: &str,
    items: Vec<String>,
) -> Vec<ClientVersionReleaseNoteSection> {
    if items.is_empty() {
        return Vec::new();
    }
    vec![ClientVersionReleaseNoteSection {
        key: key.to_owned(),
        title: title.to_owned(),
        items,
    }]
}
