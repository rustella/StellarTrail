//! Upload HTTP DTOs for authenticated feedback image uploads and private downloads.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::UploadImageRecord;

/// Stable upload response returned after an image is safely stored in private object storage.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadImageResponse {
    pub id: String,
    pub purpose: String,
    pub original_filename: String,
    pub image_type: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub download_url: String,
    pub created_at: String,
}

impl From<&UploadImageRecord> for UploadImageResponse {
    /// Converts persisted upload metadata to a current-user API response.
    fn from(record: &UploadImageRecord) -> Self {
        Self {
            id: record.id.clone(),
            purpose: record.purpose.clone(),
            original_filename: record.original_filename.clone(),
            image_type: record.image_type.clone(),
            content_type: record.content_type.clone(),
            size_bytes: record.size_bytes,
            sha256: record.sha256.clone(),
            download_url: format!("/api/me/uploads/{}", record.id),
            created_at: record.created_at.clone(),
        }
    }
}
