//! Roadmap service that validates public filters, administrator drafts, and user interactions.
//!
//! The service intentionally keeps subscription delivery out of scope. A
//! subscription records interest in the database for later in-app surfacing or
//! notification work, but no push, email, or WeChat template message is sent.

use stellartrail_db::repositories::{ListRoadmapOptions, RoadmapListEntry, RoadmapRepository};
use stellartrail_domain::{
    roadmap::{RoadmapItemDraft, is_valid_roadmap_client_key, is_valid_roadmap_status},
    validation::FieldViolation,
};

use crate::{error::ApiError, state::AppState};

const DEFAULT_CLIENT_KEY: &str = "wechat_miniprogram";

/// Lists published Roadmap items without user state.
pub async fn list_public(
    state: &AppState,
    input: RoadmapListInput,
) -> Result<(Vec<RoadmapListEntry>, Option<String>), ApiError> {
    let options = normalize_public_options(input)?;
    RoadmapRepository::new(state.db().clone())
        .list_public(&options, None)
        .await
        .map_err(ApiError::from)
}

/// Lists published Roadmap items decorated with the current user's state.
pub async fn list_for_user(
    state: &AppState,
    user_id: &str,
    input: RoadmapListInput,
) -> Result<(Vec<RoadmapListEntry>, Option<String>), ApiError> {
    let options = normalize_public_options(input)?;
    RoadmapRepository::new(state.db().clone())
        .list_public(&options, Some(user_id))
        .await
        .map_err(ApiError::from)
}

/// Lists active Roadmap items for administrators.
pub async fn list_admin(
    state: &AppState,
    input: RoadmapListInput,
) -> Result<(Vec<RoadmapListEntry>, Option<String>), ApiError> {
    let options = normalize_admin_options(input)?;
    RoadmapRepository::new(state.db().clone())
        .list_admin(&options)
        .await
        .map_err(ApiError::from)
}

/// Creates a Roadmap item after domain validation.
pub async fn create_admin(
    state: &AppState,
    actor_user_id: &str,
    mut draft: RoadmapItemDraft,
) -> Result<RoadmapListEntry, ApiError> {
    draft.validate_and_normalize()?;
    RoadmapRepository::new(state.db().clone())
        .create(actor_user_id, &draft)
        .await
        .map_err(ApiError::from)
}

/// Updates one Roadmap item after domain validation.
pub async fn update_admin(
    state: &AppState,
    actor_user_id: &str,
    id: &str,
    mut draft: RoadmapItemDraft,
) -> Result<RoadmapListEntry, ApiError> {
    let id = normalize_id(id)?;
    draft.validate_and_normalize()?;
    RoadmapRepository::new(state.db().clone())
        .update(&id, actor_user_id, &draft)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Soft-deletes one Roadmap item.
pub async fn delete_admin(state: &AppState, id: &str) -> Result<(), ApiError> {
    let id = normalize_id(id)?;
    if RoadmapRepository::new(state.db().clone())
        .soft_delete(&id)
        .await?
    {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Records the current user's vote.
pub async fn vote(
    state: &AppState,
    user_id: &str,
    roadmap_item_id: &str,
) -> Result<RoadmapListEntry, ApiError> {
    let id = normalize_id(roadmap_item_id)?;
    RoadmapRepository::new(state.db().clone())
        .vote(user_id, &id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Removes the current user's vote.
pub async fn unvote(
    state: &AppState,
    user_id: &str,
    roadmap_item_id: &str,
) -> Result<RoadmapListEntry, ApiError> {
    let id = normalize_id(roadmap_item_id)?;
    RoadmapRepository::new(state.db().clone())
        .unvote(user_id, &id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Records the current user's in-app subscription.
pub async fn subscribe(
    state: &AppState,
    user_id: &str,
    roadmap_item_id: &str,
) -> Result<RoadmapListEntry, ApiError> {
    let id = normalize_id(roadmap_item_id)?;
    RoadmapRepository::new(state.db().clone())
        .subscribe(user_id, &id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Removes the current user's in-app subscription.
pub async fn unsubscribe(
    state: &AppState,
    user_id: &str,
    roadmap_item_id: &str,
) -> Result<RoadmapListEntry, ApiError> {
    let id = normalize_id(roadmap_item_id)?;
    RoadmapRepository::new(state.db().clone())
        .unsubscribe(user_id, &id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Raw list filters after HTTP decoding.
#[derive(Clone, Debug, Default)]
pub struct RoadmapListInput {
    pub client_key: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

fn normalize_public_options(input: RoadmapListInput) -> Result<ListRoadmapOptions, ApiError> {
    let client_key = normalize_client_key(
        input
            .client_key
            .unwrap_or_else(|| DEFAULT_CLIENT_KEY.to_owned()),
    )?;
    let status = input.status.map(normalize_status).transpose()?;
    Ok(ListRoadmapOptions {
        client_key: Some(client_key),
        status,
        limit: input.limit.unwrap_or(50),
        cursor: normalize_cursor(input.cursor)?,
    })
}

fn normalize_admin_options(input: RoadmapListInput) -> Result<ListRoadmapOptions, ApiError> {
    Ok(ListRoadmapOptions {
        client_key: input.client_key.map(normalize_client_key).transpose()?,
        status: input.status.map(normalize_status).transpose()?,
        limit: input.limit.unwrap_or(50),
        cursor: normalize_cursor(input.cursor)?,
    })
}

fn normalize_client_key(value: String) -> Result<String, ApiError> {
    let value = value.trim().to_ascii_lowercase();
    if is_valid_roadmap_client_key(&value) {
        Ok(value)
    } else {
        Err(ApiError::Validation(vec![FieldViolation::new(
            "client_key",
            "is not supported",
        )]))
    }
}

fn normalize_status(value: String) -> Result<String, ApiError> {
    let value = value.trim().to_ascii_lowercase();
    if is_valid_roadmap_status(&value) {
        Ok(value)
    } else {
        Err(ApiError::Validation(vec![FieldViolation::new(
            "status",
            "is not supported",
        )]))
    }
}

fn normalize_cursor(value: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.parse::<u64>().is_ok() {
        Ok(Some(value.to_owned()))
    } else {
        Err(ApiError::Validation(vec![FieldViolation::new(
            "cursor",
            "must be a non-negative integer",
        )]))
    }
}

fn normalize_id(value: &str) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 128 {
        Err(ApiError::NotFound)
    } else {
        Ok(value.to_owned())
    }
}
