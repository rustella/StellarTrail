//! Upload service for authenticated feedback images, including size checks, magic validation, rate limiting, object writes, and metadata persistence.

use std::time::Duration as StdDuration;

use sha2::{Digest, Sha256};
use stellartrail_db::repositories::{UploadImageDraft, UploadImageRecord, UploadImageRepository};
use stellartrail_domain::{upload::validate_image_upload, validation::FieldViolation};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

use crate::{
    dto::upload::UploadImageResponse, error::ApiError, object_store::ObjectMetadata,
    state::AppState,
};

const PURPOSE_FEEDBACK: &str = "feedback";

/// Handles the full safe feedback-image upload flow.
pub async fn upload_feedback_image(
    state: &AppState,
    user_id: &str,
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: Vec<u8>,
) -> Result<UploadImageResponse, ApiError> {
    let max_bytes = state.config().upload.max_image_bytes;
    if bytes.len() as u64 > max_bytes {
        return Err(ApiError::PayloadTooLarge { max_bytes });
    }
    let size_bytes = i64::try_from(bytes.len()).map_err(ApiError::internal)?;
    let validated = validate_image_upload(original_filename, declared_content_type, &bytes)?;
    enforce_upload_rate_limit(state, user_id).await?;

    let sha256 = sha256_hex(&bytes);
    let object_key = format!(
        "feedback-images/{user_id}/{}.{}",
        Uuid::new_v4(),
        validated.safe_extension,
    );
    let object_store = state.object_store();
    let put_result = object_store
        .put_image(
            &object_key,
            &validated.content_type,
            bytes,
            ObjectMetadata {
                original_filename: validated.original_filename.clone(),
                sha256: sha256.clone(),
                image_type: validated.image_type.as_str().to_owned(),
            },
        )
        .await
        .map_err(ApiError::internal)?;

    let draft = UploadImageDraft {
        purpose: PURPOSE_FEEDBACK.to_owned(),
        original_filename: validated.original_filename,
        bucket: state.config().object_storage.bucket.clone(),
        object_key: object_key.clone(),
        image_type: validated.image_type.as_str().to_owned(),
        content_type: validated.content_type,
        size_bytes,
        sha256,
        etag: put_result.etag,
    };
    let repo = UploadImageRepository::new(state.db().clone());
    match repo.create(user_id, &draft).await {
        Ok(record) => Ok(UploadImageResponse::from(&record)),
        Err(error) => {
            let _ = object_store.delete_image(&object_key).await;
            Err(ApiError::from(error))
        }
    }
}

/// Reads one owner-scoped upload metadata row for feedback validation or download.
pub async fn get_upload_for_user(
    state: &AppState,
    user_id: &str,
    upload_id: &str,
) -> Result<Option<UploadImageRecord>, ApiError> {
    Ok(UploadImageRepository::new(state.db().clone())
        .get_for_user(user_id, upload_id)
        .await?)
}

async fn enforce_upload_rate_limit(state: &AppState, user_id: &str) -> Result<(), ApiError> {
    let window_seconds = state.config().upload.rate_limit_window_seconds;
    let max_images = state.config().upload.max_images_per_window;
    let window_started = current_fixed_window(window_seconds);
    let key = format!(
        "upload:feedback:{user_id}:{}",
        window_started.unix_timestamp()
    );
    if let Some(count) = state
        .cache()
        .increment_with_ttl(&key, StdDuration::from_secs(window_seconds + 60))
        .await
    {
        if count > max_images {
            return Err(ApiError::RateLimited {
                retry_after_seconds: retry_after_seconds(window_seconds),
            });
        }
        return Ok(());
    }

    let since = window_started
        .format(&Iso8601::DEFAULT)
        .map_err(ApiError::internal)?;
    let count = UploadImageRepository::new(state.db().clone())
        .count_recent_for_user(user_id, PURPOSE_FEEDBACK, &since)
        .await?;
    if count >= max_images as i64 {
        return Err(ApiError::RateLimited {
            retry_after_seconds: retry_after_seconds(window_seconds),
        });
    }
    Ok(())
}

fn current_fixed_window(window_seconds: u64) -> OffsetDateTime {
    let now = OffsetDateTime::now_utc();
    let unix = now.unix_timestamp().max(0) as u64;
    let start = unix - (unix % window_seconds);
    OffsetDateTime::from_unix_timestamp(start as i64).unwrap_or(now)
}

fn retry_after_seconds(window_seconds: u64) -> u64 {
    let now = OffsetDateTime::now_utc().unix_timestamp().max(0) as u64;
    let remaining = window_seconds - (now % window_seconds);
    remaining.max(1)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Converts missing/invalid referenced upload IDs to field-level validation errors.
pub fn missing_uploads_error() -> ApiError {
    ApiError::Validation(vec![FieldViolation::new(
        "image_ids",
        "must reference existing uploads owned by the current user",
    )])
}
