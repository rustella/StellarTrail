//! Client version service validating public client selectors and administrator release notes.

use stellartrail_db::repositories::{
    ClientVersionDraft, ClientVersionRecord, ClientVersionRepository, ListClientVersionsOptions,
};
use stellartrail_domain::validation::FieldViolation;

use crate::{error::ApiError, state::AppState};

const CLIENT_KEYS: [&str; 5] = ["wechat_miniprogram", "web", "android", "ios", "macos"];
const STATUSES: [&str; 2] = ["draft", "published"];

/// Raw administrator draft before normalization.
#[derive(Clone, Debug)]
pub struct ValidatedClientVersionDraft {
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes: Vec<String>,
    pub status: String,
}

/// Lists published versions for one public client.
pub async fn list_public(
    state: &AppState,
    client_key: Option<String>,
    limit: Option<u64>,
    cursor: Option<String>,
) -> Result<(Vec<ClientVersionRecord>, Option<String>), ApiError> {
    let client_key = normalize_client_key(client_key.unwrap_or_default())?;
    ClientVersionRepository::new(state.db().clone())
        .list_published(&client_key, limit.unwrap_or(20), cursor.as_deref())
        .await
        .map_err(ApiError::from)
}

/// Reads the latest published version for one public client.
pub async fn current_public(
    state: &AppState,
    client_key: Option<String>,
) -> Result<ClientVersionRecord, ApiError> {
    let client_key = normalize_client_key(client_key.unwrap_or_default())?;
    ClientVersionRepository::new(state.db().clone())
        .current_published(&client_key)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Lists versions for administrators.
pub async fn list_admin(
    state: &AppState,
    client_key: Option<String>,
    status: Option<String>,
    limit: Option<u64>,
    cursor: Option<String>,
) -> Result<(Vec<ClientVersionRecord>, Option<String>), ApiError> {
    let options = ListClientVersionsOptions {
        client_key: client_key.map(normalize_client_key).transpose()?,
        status: status.map(normalize_status).transpose()?,
        limit: limit.unwrap_or(50),
        cursor,
    };
    ClientVersionRepository::new(state.db().clone())
        .list_admin(&options)
        .await
        .map_err(ApiError::from)
}

/// Creates a version after validation and duplicate checks.
pub async fn create_admin(
    state: &AppState,
    actor_user_id: &str,
    input: ValidatedClientVersionDraft,
) -> Result<ClientVersionRecord, ApiError> {
    let draft = normalize_draft(input)?;
    let repo = ClientVersionRepository::new(state.db().clone());
    ensure_unique_version(&repo, &draft.client_key, &draft.version, None).await?;
    repo.create(actor_user_id, &draft)
        .await
        .map_err(ApiError::from)
}

/// Updates a version after validation and duplicate checks.
pub async fn update_admin(
    state: &AppState,
    actor_user_id: &str,
    id: &str,
    input: ValidatedClientVersionDraft,
) -> Result<ClientVersionRecord, ApiError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(ApiError::NotFound);
    }
    let draft = normalize_draft(input)?;
    let repo = ClientVersionRepository::new(state.db().clone());
    ensure_unique_version(&repo, &draft.client_key, &draft.version, Some(id)).await?;
    repo.update(id, actor_user_id, &draft)
        .await?
        .ok_or(ApiError::NotFound)
}

async fn ensure_unique_version(
    repo: &ClientVersionRepository,
    client_key: &str,
    version: &str,
    allowed_id: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(existing) = repo.get_by_client_version(client_key, version).await? {
        if Some(existing.id.as_str()) != allowed_id {
            return Err(ApiError::Validation(vec![FieldViolation::new(
                "version",
                "already exists for this client",
            )]));
        }
    }
    Ok(())
}

fn normalize_draft(input: ValidatedClientVersionDraft) -> Result<ClientVersionDraft, ApiError> {
    let mut errors = Vec::new();
    let client_key = normalize_client_key_with_errors(input.client_key, &mut errors);
    let status = normalize_status_with_errors(input.status, &mut errors);
    let version = normalize_version(input.version, &mut errors);
    let title = normalize_text(input.title, 120, "title", &mut errors);
    let release_notes = normalize_release_notes(input.release_notes, &mut errors);
    if !errors.is_empty() {
        return Err(ApiError::Validation(errors));
    }
    let release_notes_json = serde_json::to_string(&release_notes).map_err(ApiError::internal)?;
    Ok(ClientVersionDraft {
        client_key,
        version,
        title,
        release_notes_json,
        status,
    })
}

fn normalize_client_key(value: String) -> Result<String, ApiError> {
    let mut errors = Vec::new();
    let key = normalize_client_key_with_errors(value, &mut errors);
    if errors.is_empty() {
        Ok(key)
    } else {
        Err(ApiError::Validation(errors))
    }
}

fn normalize_client_key_with_errors(value: String, errors: &mut Vec<FieldViolation>) -> String {
    let key = value.trim().to_ascii_lowercase();
    if !CLIENT_KEYS.contains(&key.as_str()) {
        errors.push(FieldViolation::new("client_key", "is not supported"));
    }
    key
}

fn normalize_status(value: String) -> Result<String, ApiError> {
    let mut errors = Vec::new();
    let status = normalize_status_with_errors(value, &mut errors);
    if errors.is_empty() {
        Ok(status)
    } else {
        Err(ApiError::Validation(errors))
    }
}

fn normalize_status_with_errors(value: String, errors: &mut Vec<FieldViolation>) -> String {
    let status = value.trim().to_ascii_lowercase();
    if !STATUSES.contains(&status.as_str()) {
        errors.push(FieldViolation::new("status", "is not supported"));
    }
    status
}

fn normalize_version(value: String, errors: &mut Vec<FieldViolation>) -> String {
    let version = value.trim().to_owned();
    if version.is_empty() {
        errors.push(FieldViolation::new("version", "is required"));
    } else if version.chars().count() > 32 {
        errors.push(FieldViolation::new(
            "version",
            "must be at most 32 characters",
        ));
    } else if !is_semver_like(&version) {
        errors.push(FieldViolation::new(
            "version",
            "must use semantic version format like 0.1.0",
        ));
    }
    version
}

fn is_semver_like(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

fn normalize_text(
    value: String,
    max_chars: usize,
    field: &'static str,
    errors: &mut Vec<FieldViolation>,
) -> String {
    let value = value.trim().to_owned();
    if value.is_empty() {
        errors.push(FieldViolation::new(field, "is required"));
    } else if value.chars().count() > max_chars {
        errors.push(FieldViolation::new(
            field,
            format!("must be at most {max_chars} characters"),
        ));
    }
    value
}

fn normalize_release_notes(values: Vec<String>, errors: &mut Vec<FieldViolation>) -> Vec<String> {
    let notes = values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if notes.is_empty() {
        errors.push(FieldViolation::new("release_notes", "is required"));
    }
    if notes.len() > 20 {
        errors.push(FieldViolation::new(
            "release_notes",
            "must contain at most 20 items",
        ));
    }
    for (index, note) in notes.iter().enumerate() {
        if note.chars().count() > 240 {
            errors.push(FieldViolation::new(
                format!("release_notes[{index}]"),
                "must be at most 240 characters",
            ));
        }
    }
    notes
}
