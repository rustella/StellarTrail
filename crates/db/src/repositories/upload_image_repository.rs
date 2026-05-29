//! Upload image repository storing private object storage metadata and owner-scoped lookup helpers.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};
use stellartrail_domain::gear::now_rfc3339;
use uuid::Uuid;

use super::statement;

/// Draft metadata persisted after an image object has passed validation and been written to MinIO.
#[derive(Clone, Debug)]
pub struct UploadImageDraft {
    pub purpose: String,
    pub original_filename: String,
    pub bucket: String,
    pub object_key: String,
    pub image_type: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub etag: Option<String>,
}

/// Upload image metadata row.
#[derive(Clone, Debug)]
pub struct UploadImageRecord {
    pub id: String,
    pub user_id: String,
    pub purpose: String,
    pub original_filename: String,
    pub bucket: String,
    pub object_key: String,
    pub image_type: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub etag: Option<String>,
    pub is_deleted: bool,
    pub created_at: String,
}

/// Aggregate upload usage for one user and upload purpose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UploadImageUsage {
    pub count: i64,
    pub total_size_bytes: i64,
}

/// Repository for upload image metadata.
#[derive(Clone)]
pub struct UploadImageRepository {
    db: DatabaseConnection,
}

impl UploadImageRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Persists upload metadata scoped to the current user.
    pub async fn create(
        &self,
        user_id: &str,
        draft: &UploadImageDraft,
    ) -> Result<UploadImageRecord, DbErr> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO upload_images (
                    id, user_id, purpose, original_filename, bucket, object_key, image_type,
                    content_type, size_bytes, sha256, etag, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.purpose.clone().into(),
                    draft.original_filename.clone().into(),
                    draft.bucket.clone().into(),
                    draft.object_key.clone().into(),
                    draft.image_type.clone().into(),
                    draft.content_type.clone().into(),
                    draft.size_bytes.into(),
                    draft.sha256.clone().into(),
                    draft.etag.clone().into(),
                    created_at.into(),
                ],
            ))
            .await?;
        self.get_for_user(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created upload image not found".to_owned()))
    }

    /// Reads one upload image if it belongs to the current user.
    pub async fn get_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Option<UploadImageRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                upload_select_sql("WHERE user_id = ? AND id = ? AND is_deleted = FALSE"),
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_upload_image(&row)).transpose()
    }

    /// Lists owner-scoped upload images for arbitrary ID sets without imposing a business count limit.
    pub async fn list_for_user_by_ids(
        &self,
        user_id: &str,
        ids: &[String],
    ) -> Result<Vec<UploadImageRecord>, DbErr> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        for chunk in ids.chunks(400) {
            let placeholders = vec!["?"; chunk.len()].join(", ");
            let mut values: Vec<Value> = vec![user_id.to_owned().into()];
            values.extend(chunk.iter().cloned().map(Into::into));
            let rows = self
                .db
                .query_all(statement(
                    self.db.get_database_backend(),
                    format!(
                        "{} WHERE user_id = ? AND is_deleted = FALSE AND id IN ({placeholders})",
                        upload_select_columns()
                    ),
                    values,
                ))
                .await?;
            records.extend(
                rows.iter()
                    .map(map_upload_image)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(records)
    }

    /// Counts recent successful uploads for fallback rate limiting when Redis is unavailable.
    pub async fn count_recent_for_user(
        &self,
        user_id: &str,
        purpose: &str,
        since_rfc3339: &str,
    ) -> Result<i64, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM upload_images WHERE user_id = ? AND purpose = ? AND is_deleted = FALSE AND created_at >= ?",
                vec![
                    user_id.to_owned().into(),
                    purpose.to_owned().into(),
                    since_rfc3339.to_owned().into(),
                ],
            ))
            .await?;
        Ok(row
            .map(|row| row.try_get("", "count"))
            .transpose()?
            .unwrap_or(0))
    }

    /// Returns cumulative owner-scoped upload usage for a single business purpose.
    pub async fn usage_for_user(
        &self,
        user_id: &str,
        purpose: &str,
    ) -> Result<UploadImageUsage, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count, CAST(COALESCE(SUM(size_bytes), 0) AS BIGINT) AS total_size_bytes FROM upload_images WHERE user_id = ? AND purpose = ? AND is_deleted = FALSE",
                vec![user_id.to_owned().into(), purpose.to_owned().into()],
            ))
            .await?;
        Ok(match row {
            Some(row) => UploadImageUsage {
                count: row.try_get("", "count")?,
                total_size_bytes: row.try_get("", "total_size_bytes")?,
            },
            None => UploadImageUsage {
                count: 0,
                total_size_bytes: 0,
            },
        })
    }
}

fn upload_select_sql(where_clause: &str) -> String {
    format!("{} {where_clause} LIMIT 1", upload_select_columns())
}

fn upload_select_columns() -> &'static str {
    r#"SELECT id, user_id, purpose, original_filename, bucket, object_key, image_type,
        content_type, size_bytes, sha256, etag, is_deleted, created_at
       FROM upload_images"#
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
