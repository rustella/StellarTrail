//! Profile service for authenticated current-user display data and public avatar uploads.

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use stellartrail_db::repositories::{AuthRepository, UserRecord};
use stellartrail_domain::upload::validate_image_upload;
use uuid::Uuid;

use crate::{
    dto::profile::ProfileUserResponse, error::ApiError, object_store::PutObjectRequest,
    services::auth_service, state::AppState,
};

const CACHE_CONTROL_AVATAR: &str = "public, max-age=31536000, immutable";

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
    let validated = validate_image_upload(original_filename, declared_content_type, &bytes)?;
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
