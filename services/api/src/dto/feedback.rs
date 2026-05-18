//! User feedback HTTP DTOs and response conversion helpers.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::{
    AdminFeedbackRecord, FeedbackAuthorRecord, FeedbackRecord, ListAdminFeedbackOptions,
};
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

/// Administrator feedback list query.
#[derive(Clone, Debug, Deserialize)]
pub struct ListAdminFeedbackQuery {
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

impl ListAdminFeedbackQuery {
    /// Converts HTTP query parameters to repository filters.
    pub fn into_options(self) -> ListAdminFeedbackOptions {
        ListAdminFeedbackOptions {
            status: self
                .status
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            limit: self.limit.unwrap_or(50),
            cursor: self.cursor,
        }
    }
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

/// User projection shown with administrator feedback entries.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeedbackAuthorResponse {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// Feedback response for administrator dashboards.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminFeedbackResponse {
    pub id: String,
    pub user: FeedbackAuthorResponse,
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

/// Administrator feedback list response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListAdminFeedbackResponse {
    pub items: Vec<AdminFeedbackResponse>,
    pub next_cursor: Option<String>,
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

impl From<&FeedbackAuthorRecord> for FeedbackAuthorResponse {
    /// Converts a feedback author database projection to an API response.
    fn from(record: &FeedbackAuthorRecord) -> Self {
        Self {
            id: record.id.clone(),
            username: record.username.clone(),
            email: record.email.clone(),
            nickname: record.nickname.clone(),
            avatar_url: record.avatar_url.clone(),
        }
    }
}

impl From<&AdminFeedbackRecord> for AdminFeedbackResponse {
    /// Converts an administrator feedback record to an API response.
    fn from(record: &AdminFeedbackRecord) -> Self {
        let feedback = &record.feedback;
        Self {
            id: feedback.id.clone(),
            user: FeedbackAuthorResponse::from(&record.author),
            category: feedback.category.clone(),
            content: feedback.content.clone(),
            contact: feedback.contact.clone(),
            page: feedback.page.clone(),
            client_platform: feedback.client_platform.clone(),
            client_version: feedback.client_version.clone(),
            device_model: feedback.device_model.clone(),
            status: feedback.status.clone(),
            images: feedback
                .images
                .iter()
                .map(|image| {
                    UploadImageResponse::from_record_with_download_url(
                        image,
                        format!("/api/admin/feedback-images/{}", image.id),
                    )
                })
                .collect(),
            created_at: feedback.created_at.clone(),
            updated_at: feedback.updated_at.clone(),
        }
    }
}
