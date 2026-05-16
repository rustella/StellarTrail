//! Feedback service for authenticated current-user feedback and unlimited ordered image associations.

use std::collections::{HashMap, HashSet};

use stellartrail_db::repositories::{FeedbackRepository, UploadImageRecord, UploadImageRepository};
use stellartrail_domain::validation::FieldViolation;

use crate::{
    dto::feedback::{CreateFeedbackRequest, FeedbackResponse},
    error::ApiError,
    state::AppState,
};

/// Creates user feedback after validating all referenced upload images belong to the current user.
pub async fn create_feedback(
    state: &AppState,
    user_id: &str,
    request: CreateFeedbackRequest,
) -> Result<FeedbackResponse, ApiError> {
    let (draft, image_ids) = request.into_parts()?;
    validate_no_duplicate_image_ids(&image_ids)?;

    let images = load_ordered_user_images(state, user_id, &image_ids).await?;
    let record = FeedbackRepository::new(state.db().clone())
        .create(user_id, &draft, &images)
        .await?;
    Ok(FeedbackResponse::from(&record))
}

async fn load_ordered_user_images(
    state: &AppState,
    user_id: &str,
    image_ids: &[String],
) -> Result<Vec<UploadImageRecord>, ApiError> {
    if image_ids.is_empty() {
        return Ok(Vec::new());
    }
    let records = UploadImageRepository::new(state.db().clone())
        .list_for_user_by_ids(user_id, image_ids)
        .await?;
    if records.len() != image_ids.len() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "image_ids",
            "must reference existing uploads owned by the current user",
        )]));
    }
    let mut by_id: HashMap<String, UploadImageRecord> = records
        .into_iter()
        .map(|record| (record.id.clone(), record))
        .collect();
    image_ids
        .iter()
        .map(|id| {
            by_id.remove(id).ok_or_else(|| {
                ApiError::Validation(vec![FieldViolation::new(
                    "image_ids",
                    "must reference existing uploads owned by the current user",
                )])
            })
        })
        .collect()
}

fn validate_no_duplicate_image_ids(image_ids: &[String]) -> Result<(), ApiError> {
    let mut seen = HashSet::new();
    for image_id in image_ids {
        let trimmed = image_id.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_owned()) {
            return Err(ApiError::Validation(vec![FieldViolation::new(
                "image_ids",
                "must not contain duplicate or empty ids",
            )]));
        }
    }
    Ok(())
}
