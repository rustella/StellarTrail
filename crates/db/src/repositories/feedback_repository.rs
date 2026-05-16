//! User feedback repository storing feedback rows and unlimited image associations.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};
use stellartrail_domain::{feedback::FeedbackDraft, gear::now_rfc3339};
use uuid::Uuid;

use super::{statement, upload_image_repository::UploadImageRecord};

/// Feedback row with ordered image summaries.
#[derive(Clone, Debug)]
pub struct FeedbackRecord {
    pub id: String,
    pub user_id: String,
    pub category: String,
    pub content: String,
    pub contact: Option<String>,
    pub page: Option<String>,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
    pub status: String,
    pub images: Vec<UploadImageRecord>,
    pub created_at: String,
    pub updated_at: String,
}

/// Repository for current-user feedback.
#[derive(Clone)]
pub struct FeedbackRepository {
    db: DatabaseConnection,
}

impl FeedbackRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates feedback and image associations. The caller pre-validates image ownership.
    pub async fn create(
        &self,
        user_id: &str,
        draft: &FeedbackDraft,
        images: &[UploadImageRecord],
    ) -> Result<FeedbackRecord, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO user_feedback (
                    id, user_id, category, content, contact, page, client_platform,
                    client_version, device_model, status, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?, ?)"#,
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.category.as_str().to_owned().into(),
                    draft.content.clone().into(),
                    draft.contact.clone().into(),
                    draft.page.clone().into(),
                    draft.client_platform.clone().into(),
                    draft.client_version.clone().into(),
                    draft.device_model.clone().into(),
                    now.clone().into(),
                    now.clone().into(),
                ],
            ))
            .await?;

        for (index, image) in images.iter().enumerate() {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "INSERT INTO user_feedback_images (feedback_id, upload_image_id, sort_order, created_at) VALUES (?, ?, ?, ?)",
                    vec![
                        id.clone().into(),
                        image.id.clone().into(),
                        (index as i64).into(),
                        now.clone().into(),
                    ],
                ))
                .await?;
        }

        self.get_for_user(user_id, &id, images)
            .await?
            .ok_or_else(|| DbErr::Custom("created feedback not found after insert".to_owned()))
    }

    /// Reads one feedback row for the current user. Images are supplied by the caller to preserve response order.
    pub async fn get_for_user(
        &self,
        user_id: &str,
        id: &str,
        images: &[UploadImageRecord],
    ) -> Result<Option<FeedbackRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                r#"SELECT id, user_id, category, content, contact, page, client_platform,
                    client_version, device_model, status, created_at, updated_at
                   FROM user_feedback WHERE user_id = ? AND id = ? LIMIT 1"#,
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_feedback(&row, images.to_vec()))
            .transpose()
    }
}

fn map_feedback(
    row: &sea_orm::QueryResult,
    images: Vec<UploadImageRecord>,
) -> Result<FeedbackRecord, DbErr> {
    Ok(FeedbackRecord {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        category: row.try_get("", "category")?,
        content: row.try_get("", "content")?,
        contact: row.try_get("", "contact")?,
        page: row.try_get("", "page")?,
        client_platform: row.try_get("", "client_platform")?,
        client_version: row.try_get("", "client_version")?,
        device_model: row.try_get("", "device_model")?,
        status: row.try_get("", "status")?,
        images,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}
