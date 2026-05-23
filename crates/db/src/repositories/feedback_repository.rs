//! User feedback repository storing feedback rows and unlimited image associations.

use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};
use stellartrail_domain::{deletion::DeletedFilter, feedback::FeedbackDraft, gear::now_rfc3339};
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
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Feedback author projection for administrator review screens.
#[derive(Clone, Debug)]
pub struct FeedbackAuthorRecord {
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

/// Feedback row plus the submitting user, used by administrator reads.
#[derive(Clone, Debug)]
pub struct AdminFeedbackRecord {
    pub feedback: FeedbackRecord,
    pub author: FeedbackAuthorRecord,
}

/// Administrator feedback list filters.
#[derive(Clone, Debug)]
pub struct ListAdminFeedbackOptions {
    pub status: Option<String>,
    pub deleted: DeletedFilter,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListAdminFeedbackOptions {
    fn default() -> Self {
        Self {
            status: None,
            deleted: DeletedFilter::Active,
            limit: 50,
            cursor: None,
        }
    }
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
                    client_version, device_model, status, is_deleted, created_at, updated_at
                   FROM user_feedback WHERE user_id = ? AND id = ? AND is_deleted = FALSE LIMIT 1"#,
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_feedback(&row, images.to_vec()))
            .transpose()
    }

    /// Lists feedback for administrators, including author and ordered image metadata.
    pub async fn list_admin(
        &self,
        options: &ListAdminFeedbackOptions,
    ) -> Result<(Vec<AdminFeedbackRecord>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = Vec::new();
        let mut clauses = Vec::new();
        apply_deleted_filter(&mut clauses, options.deleted);
        if let Some(status) = options
            .status
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            clauses.push("f.status = ?".to_owned());
            values.push(status.to_owned().into());
        }
        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    r#"SELECT f.id, f.user_id, f.category, f.content, f.contact, f.page,
                        f.client_platform, f.client_version, f.device_model, f.status,
                        f.is_deleted, f.created_at, f.updated_at,
                        u.username AS author_username,
                        u.email AS author_email,
                        u.nickname AS author_nickname,
                        u.avatar_url AS author_avatar_url
                       FROM user_feedback f
                       LEFT JOIN users u ON u.id = f.user_id
                       {where_clause}
                       ORDER BY f.created_at DESC, f.id DESC
                       LIMIT ? OFFSET ?"#
                ),
                values,
            ))
            .await?;
        let mut feedback_ids = Vec::new();
        let mut records = rows
            .iter()
            .map(|row| {
                let record = map_admin_feedback_without_images(row)?;
                feedback_ids.push(record.feedback.id.clone());
                Ok(record)
            })
            .collect::<Result<Vec<_>, DbErr>>()?;
        let mut images_by_feedback = self.list_images_for_feedback_ids(&feedback_ids).await?;
        for record in &mut records {
            record.feedback.images = images_by_feedback
                .remove(&record.feedback.id)
                .unwrap_or_default();
        }
        let next_cursor = if records.len() > limit as usize {
            records.truncate(limit as usize);
            Some((offset + limit as i64).to_string())
        } else {
            None
        };
        Ok((records, next_cursor))
    }

    /// Reads a feedback image for administrator downloads only if it is attached to feedback.
    pub async fn get_feedback_image_for_admin(
        &self,
        upload_image_id: &str,
    ) -> Result<Option<UploadImageRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} INNER JOIN user_feedback_images fi ON fi.upload_image_id = upload_images.id \
                       INNER JOIN user_feedback f ON f.id = fi.feedback_id \
                       WHERE upload_images.id = ? AND upload_images.is_deleted = FALSE AND f.is_deleted = FALSE LIMIT 1",
                    upload_select_columns()
                ),
                vec![upload_image_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_upload_image(&row)).transpose()
    }

    async fn list_images_for_feedback_ids(
        &self,
        feedback_ids: &[String],
    ) -> Result<HashMap<String, Vec<UploadImageRecord>>, DbErr> {
        let mut images_by_feedback: HashMap<String, Vec<UploadImageRecord>> = HashMap::new();
        for chunk in feedback_ids.chunks(400) {
            if chunk.is_empty() {
                continue;
            }
            let placeholders = vec!["?"; chunk.len()].join(", ");
            let rows = self
                .db
                .query_all(statement(
                    self.db.get_database_backend(),
                    format!(
                        r#"SELECT fi.feedback_id, upload_images.id, upload_images.user_id,
                            upload_images.purpose, upload_images.original_filename,
                            upload_images.bucket, upload_images.object_key,
                            upload_images.image_type, upload_images.content_type,
                            upload_images.size_bytes, upload_images.sha256,
                            upload_images.etag, upload_images.is_deleted,
                            upload_images.created_at
                           FROM user_feedback_images fi
                           INNER JOIN upload_images ON upload_images.id = fi.upload_image_id
                           WHERE fi.feedback_id IN ({placeholders}) AND upload_images.is_deleted = FALSE
                           ORDER BY fi.feedback_id, fi.sort_order"#
                    ),
                    chunk.iter().cloned().map(Into::into).collect(),
                ))
                .await?;
            for row in rows {
                let feedback_id: String = row.try_get("", "feedback_id")?;
                images_by_feedback
                    .entry(feedback_id)
                    .or_default()
                    .push(map_upload_image(&row)?);
            }
        }
        Ok(images_by_feedback)
    }

    /// Soft-deletes a feedback row without removing image associations.
    pub async fn soft_delete(&self, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_feedback SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
                vec![now.into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Restores a previously soft-deleted feedback row.
    pub async fn restore_deleted(&self, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_feedback SET is_deleted = FALSE, updated_at = ? WHERE id = ? AND is_deleted = TRUE",
                vec![now.into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
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
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_admin_feedback_without_images(
    row: &sea_orm::QueryResult,
) -> Result<AdminFeedbackRecord, DbErr> {
    Ok(AdminFeedbackRecord {
        feedback: map_feedback(row, Vec::new())?,
        author: FeedbackAuthorRecord {
            id: row.try_get("", "user_id")?,
            username: row.try_get("", "author_username")?,
            email: row.try_get("", "author_email")?,
            nickname: row.try_get("", "author_nickname")?,
            avatar_url: row.try_get("", "author_avatar_url")?,
        },
    })
}

fn upload_select_columns() -> &'static str {
    r#"SELECT upload_images.id, upload_images.user_id, upload_images.purpose,
        upload_images.original_filename, upload_images.bucket, upload_images.object_key,
        upload_images.image_type, upload_images.content_type, upload_images.size_bytes,
        upload_images.sha256, upload_images.etag, upload_images.is_deleted, upload_images.created_at
       FROM upload_images"#
}

fn apply_deleted_filter(clauses: &mut Vec<String>, deleted: DeletedFilter) {
    match deleted {
        DeletedFilter::Active => clauses.push("f.is_deleted = FALSE".to_owned()),
        DeletedFilter::Deleted => clauses.push("f.is_deleted = TRUE".to_owned()),
        DeletedFilter::All => {}
    }
}

fn map_upload_image(row: &sea_orm::QueryResult) -> Result<UploadImageRecord, DbErr> {
    Ok(UploadImageRecord {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        purpose: row.try_get("", "purpose")?,
        original_filename: row.try_get("", "original_filename")?,
        bucket: row.try_get("", "bucket")?,
        object_key: row.try_get("", "object_key")?,
        image_type: row.try_get("", "image_type")?,
        content_type: row.try_get("", "content_type")?,
        size_bytes: row.try_get("", "size_bytes")?,
        sha256: row.try_get("", "sha256")?,
        etag: row.try_get("", "etag")?,
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
    })
}

fn parse_cursor(cursor: Option<&str>) -> Result<i64, DbErr> {
    match cursor {
        Some(value) if !value.trim().is_empty() => value
            .trim()
            .parse::<i64>()
            .map_err(|_| DbErr::Custom("invalid feedback cursor".to_owned()))
            .map(|offset| offset.max(0)),
        _ => Ok(0),
    }
}
