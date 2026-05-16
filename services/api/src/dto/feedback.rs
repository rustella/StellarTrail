//! User feedback HTTP DTOs and response conversion helpers.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::FeedbackRecord;
use stellartrail_domain::feedback::{FeedbackDraft, validate_feedback_draft};

use crate::dto::upload::UploadImageResponse;

/// Feedback creation request. image_ids intentionally has no business count limit.
#[derive(Clone, Debug, Deserialize)]
pub struct CreateFeedbackRequest {
    pub category: String,
    pub content: String,
    pub contact: Option<String>,
    pub page: Option<String>,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
    #[serde(default)]
    pub image_ids: Vec<String>,
}

impl CreateFeedbackRequest {
    /// Validates request fields and returns a feedback draft plus ordered image IDs.
    pub fn into_parts(
        self,
    ) -> Result<(FeedbackDraft, Vec<String>), stellartrail_domain::validation::ValidationError>
    {
        let draft = validate_feedback_draft(
            self.category,
            self.content,
            self.contact,
            self.page,
            self.client_platform,
            self.client_version,
            self.device_model,
        )?;
        Ok((draft, self.image_ids))
    }
}

/// Feedback response returned after the current user submits feedback.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeedbackResponse {
    pub id: String,
    pub category: String,
    pub content: String,
    pub contact: Option<String>,
    pub page: Option<String>,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
    pub status: String,
    pub images: Vec<UploadImageResponse>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&FeedbackRecord> for FeedbackResponse {
    /// Converts a feedback record and ordered images to an API response.
    fn from(record: &FeedbackRecord) -> Self {
        Self {
            id: record.id.clone(),
            category: record.category.clone(),
            content: record.content.clone(),
            contact: record.contact.clone(),
            page: record.page.clone(),
            client_platform: record.client_platform.clone(),
            client_version: record.client_version.clone(),
            device_model: record.device_model.clone(),
            status: record.status.clone(),
            images: record
                .images
                .iter()
                .map(UploadImageResponse::from)
                .collect(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}
