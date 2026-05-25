//! Gear inventory service module that adds domain validation, cache invalidation, and export/list orchestration around repository reads and writes.

use std::collections::HashMap;

use stellartrail_db::repositories::{GearRepository, ListGearOptions};
use stellartrail_domain::gear::{GearDraft, GearItem, GearTab};
use stellartrail_domain::validation::FieldViolation;

use crate::{
    cache::is_supported_gear_tag_color, dto::gear::UpdateGearRequest, error::ApiError,
    state::AppState,
};

/// Runs the `create gear` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn create_gear(
    state: &AppState,
    user_id: &str,
    mut draft: GearDraft,
    tag_colors: HashMap<String, String>,
) -> Result<GearItem, ApiError> {
    validate_tag_colors(&tag_colors)?;
    draft.validate_and_normalize()?;
    let item = GearRepository::new(state.db().clone())
        .create(user_id, &draft)
        .await
        .map_err(ApiError::from)?;
    state
        .cache()
        .record_gear_spec_keys(user_id, item.category, &item.specs)
        .await;
    state
        .cache()
        .record_gear_tags(user_id, &item.tags, &tag_colors)
        .await;
    Ok(item)
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
    let tag_colors = request.tag_colors.clone().unwrap_or_default();
    validate_tag_colors(&tag_colors)?;
    let mut draft = request.merge_into(&existing);
    draft.validate_and_normalize()?;
    let item = repo
        .replace(user_id, id, &draft)
        .await?
        .ok_or(ApiError::NotFound)?;
    state
        .cache()
        .record_gear_spec_keys(user_id, item.category, &item.specs)
        .await;
    state
        .cache()
        .record_gear_tags(user_id, &item.tags, &tag_colors)
        .await;
    Ok(item)
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

/// Validates user-supplied tag color tokens before gear writes reach the database.
fn validate_tag_colors(tag_colors: &HashMap<String, String>) -> Result<(), ApiError> {
    let mut errors = Vec::new();
    for (tag, color) in tag_colors {
        if !is_supported_gear_tag_color(color.trim()) {
            errors.push(FieldViolation::new(
                format!("tag_colors.{tag}"),
                "is not a supported tag color",
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ApiError::Validation(errors))
    }
}
