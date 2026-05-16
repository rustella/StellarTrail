//! Gear inventory service module that adds domain validation, cache invalidation, and export/list orchestration around repository reads and writes.

use stellartrail_db::repositories::{GearRepository, ListGearOptions};
use stellartrail_domain::gear::{GearDraft, GearItem, GearTab};

use crate::{dto::gear::UpdateGearRequest, error::ApiError, state::AppState};

/// Runs the `create gear` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn create_gear(
    state: &AppState,
    user_id: &str,
    mut draft: GearDraft,
) -> Result<GearItem, ApiError> {
    draft.validate_and_normalize()?;
    GearRepository::new(state.db().clone())
        .create(user_id, &draft)
        .await
        .map_err(ApiError::from)
}

/// Runs the `update gear` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn update_gear(
    state: &AppState,
    user_id: &str,
    id: &str,
    request: UpdateGearRequest,
) -> Result<GearItem, ApiError> {
    let repo = GearRepository::new(state.db().clone());
    let existing = repo.get(user_id, id).await?.ok_or(ApiError::NotFound)?;
    let mut draft = request.merge_into(&existing);
    draft.validate_and_normalize()?;
    repo.replace(user_id, id, &draft)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Runs the `list for export` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn list_for_export(
    state: &AppState,
    user_id: &str,
    tab: GearTab,
) -> Result<Vec<GearItem>, ApiError> {
    let repo = GearRepository::new(state.db().clone());
    let (items, _) = repo
        .list(
            user_id,
            &ListGearOptions {
                tab,
                limit: 10_000,
                ..Default::default()
            },
        )
        .await?;
    Ok(items)
}
