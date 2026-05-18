//! Profile service for authenticated current-user display data and public avatar uploads.

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use stellartrail_db::repositories::{AuthRepository, UserRecord};
use stellartrail_domain::upload::{detect_image_type, validate_image_upload};
use uuid::Uuid;

use crate::{
    dto::profile::ProfileUserResponse, error::ApiError, object_store::PutObjectRequest,
    services::auth_service, state::AppState,
};

const CACHE_CONTROL_AVATAR: &str = "public, max-age=31536000, immutable";

/// Builds the current authenticated user's profile response.
pub fn current_profile(user: &UserRecord) -> ProfileUserResponse {
    ProfileUserResponse {
        user: auth_service::login_user_response(user.clone()),
    }
}

/// Validates and uploads the authenticated user's avatar, then updates the user row.
pub async fn upload_avatar(
    state: &AppState,
    user: &UserRecord,
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: Vec<u8>,
) -> Result<ProfileUserResponse, ApiError> {
    let max_bytes = state.config().avatar_storage.max_image_bytes;
    if bytes.len() as u64 > max_bytes {
        return Err(ApiError::PayloadTooLarge { max_bytes });
    }
    let fallback_filename;
    let upload_filename = match original_filename.filter(|filename| has_image_extension(filename)) {
        Some(filename) => Some(filename),
        None => {
            let image_type = detect_image_type(&bytes).ok_or_else(|| {
                stellartrail_domain::validation::ValidationError::single(
                    "file",
                    "unsupported or invalid image content",
                )
            })?;
            fallback_filename = format!("wechat-avatar.{}", image_type.safe_extension());
            Some(fallback_filename.as_str())
        }
    };
    let upload_content_type = declared_content_type.filter(|value| is_supported_image_type(value));
    let validated = validate_image_upload(upload_filename, upload_content_type, &bytes)?;
    let sha256 = sha256_hex(&bytes);
    let object_key = format!(
        "users/{}/avatar/{}-{}.{}",
        user.id,
        sha256,
        Uuid::new_v4(),
        validated.safe_extension,
    );
    let public_base_url = state
        .config()
        .avatar_storage
        .public_base_url
        .trim_end_matches('/');
    let avatar_url = format!("{public_base_url}/{object_key}");
    let object_store = state.object_store();
    let put_result = object_store
        .put_object(PutObjectRequest {
            bucket: Some(state.config().avatar_storage.bucket.clone()),
            object_key: object_key.clone(),
            content_type: validated.content_type,
            bytes,
            metadata: HashMap::from([
                (
                    "original_filename".to_owned(),
                    safe_metadata_value(&validated.original_filename),
                ),
                ("sha256".to_owned(), sha256),
                (
                    "image_type".to_owned(),
                    validated.image_type.as_str().to_owned(),
                ),
                ("user_id".to_owned(), user.id.clone()),
            ]),
            cache_control: Some(CACHE_CONTROL_AVATAR.to_owned()),
        })
        .await
        .map_err(ApiError::internal)?;

    let updated = match AuthRepository::new(state.db().clone())
        .update_user_avatar_url(&user.id, &avatar_url)
        .await
    {
        Ok(Some(updated)) => updated,
        Ok(None) => {
            let _ = object_store.delete_object(&object_key).await;
            return Err(ApiError::Unauthorized);
        }
        Err(error) => {
            let _ = object_store.delete_object(&object_key).await;
            return Err(ApiError::from(error));
        }
    };
    tracing::debug!(
        user_id = %user.id,
        size_bytes = put_result.size_bytes,
        "uploaded profile avatar"
    );
    Ok(ProfileUserResponse {
        user: auth_service::login_user_response(updated),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn safe_metadata_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '\0')
        .take(160)
        .collect()
}

fn has_image_extension(filename: &str) -> bool {
    let lower = filename.trim().to_ascii_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
}

fn is_supported_image_type(value: &&str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "image/jpeg" | "image/jpg" | "image/png" | "image/webp"
    )
}
