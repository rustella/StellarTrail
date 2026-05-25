//! Repository for object-storage media resource metadata and knot media mappings.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, TransactionTrait};
use stellartrail_domain::skill::KnotMediaAsset;

use super::statement;

/// Draft for inserting or updating one media resource metadata row.
#[derive(Clone, Debug)]
pub struct MediaResourceDraft {
    pub id: String,
    pub provider: String,
    pub storage_profile: String,
    pub bucket: String,
    pub object_key: String,
    pub public_base_url: String,
    pub public_url: String,
    pub mime_type: String,
    pub extension: String,
    pub size_bytes: i64,
    pub sha256_hex: String,
    pub etag: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub source_name: Option<String>,
    pub source_path: Option<String>,
    pub uploaded_by_user_id: Option<String>,
}

/// Persisted media resource row.
#[derive(Clone, Debug)]
pub struct MediaResourceRecord {
    pub id: String,
    pub provider: String,
    pub storage_profile: String,
    pub bucket: String,
    pub object_key: String,
    pub public_base_url: String,
    pub public_url: String,
    pub mime_type: String,
    pub extension: String,
    pub size_bytes: i64,
    pub sha256_hex: String,
    pub etag: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i32>,
    pub status: String,
    pub source_name: Option<String>,
    pub source_path: Option<String>,
    pub uploaded_by_user_id: Option<String>,
}

/// Draft linking one media resource to one knot public media asset.
#[derive(Clone, Debug)]
pub struct KnotMediaLinkDraft {
    pub knot_id: String,
    pub asset_id: String,
    pub media_type: String,
    pub media_resource_id: String,
    pub sort_order: i32,
    pub attribution: Option<String>,
    pub license_note: Option<String>,
}

/// Persistence object for media_resources and knot_media_resources.
#[derive(Clone)]
pub struct MediaResourceRepository {
    db: DatabaseConnection,
}

impl MediaResourceRepository {
    /// Creates a media resource repository using the shared database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Returns whether the knot exists.
    pub async fn knot_exists(&self, knot_id: &str) -> Result<bool, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT 1 AS exists_flag FROM knots WHERE id = ?",
                vec![knot_id.to_owned().into()],
            ))
            .await?;
        Ok(row.is_some())
    }

    /// Upserts resource metadata by stable id and returns the stored row.
    pub async fn upsert_media_resource(
        &self,
        draft: &MediaResourceDraft,
    ) -> Result<MediaResourceRecord, DbErr> {
        let backend = self.db.get_database_backend();
        self.db
            .execute(statement(
                backend,
                r#"INSERT INTO media_resources (
                    id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
                    mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
                    status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                ON CONFLICT(id) DO UPDATE SET
                    provider = excluded.provider,
                    storage_profile = excluded.storage_profile,
                    bucket = excluded.bucket,
                    object_key = excluded.object_key,
                    public_base_url = excluded.public_base_url,
                    public_url = excluded.public_url,
                    mime_type = excluded.mime_type,
                    extension = excluded.extension,
                    size_bytes = excluded.size_bytes,
                    sha256_hex = excluded.sha256_hex,
                    etag = excluded.etag,
                    width = excluded.width,
                    height = excluded.height,
                    duration_ms = excluded.duration_ms,
                    status = excluded.status,
                    source_name = excluded.source_name,
                    source_path = excluded.source_path,
                    uploaded_by_user_id = excluded.uploaded_by_user_id,
                    updated_at = CURRENT_TIMESTAMP"#,
                vec![
                    draft.id.clone().into(),
                    draft.provider.clone().into(),
                    draft.storage_profile.clone().into(),
                    draft.bucket.clone().into(),
                    draft.object_key.clone().into(),
                    draft.public_base_url.clone().into(),
                    draft.public_url.clone().into(),
                    draft.mime_type.clone().into(),
                    draft.extension.clone().into(),
                    draft.size_bytes.into(),
                    draft.sha256_hex.clone().into(),
                    draft.etag.clone().into(),
                    draft.width.into(),
                    draft.height.into(),
                    draft.duration_ms.into(),
                    draft.status.clone().into(),
                    draft.source_name.clone().into(),
                    draft.source_path.clone().into(),
                    draft.uploaded_by_user_id.clone().into(),
                ],
            ))
            .await?;
        self.get_media_resource(&draft.id)
            .await?
            .ok_or_else(|| DbErr::Custom("upserted media resource not found".to_owned()))
    }

    /// Upserts a knot/media mapping.
    pub async fn upsert_knot_media_link(&self, draft: &KnotMediaLinkDraft) -> Result<(), DbErr> {
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO knot_media_resources (
                    knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                ON CONFLICT(knot_id, asset_id) DO UPDATE SET
                    media_type = excluded.media_type,
                    media_resource_id = excluded.media_resource_id,
                    sort_order = excluded.sort_order,
                    attribution = excluded.attribution,
                    license_note = excluded.license_note,
                    updated_at = CURRENT_TIMESTAMP"#,
                vec![
                    draft.knot_id.clone().into(),
                    draft.asset_id.clone().into(),
                    draft.media_type.clone().into(),
                    draft.media_resource_id.clone().into(),
                    draft.sort_order.into(),
                    draft.attribution.clone().into(),
                    draft.license_note.clone().into(),
                ],
            ))
            .await?;
        Ok(())
    }

    /// Upserts a media resource and its knot mapping in one transaction.
    pub async fn upsert_knot_media(
        &self,
        resource: &MediaResourceDraft,
        link: &KnotMediaLinkDraft,
    ) -> Result<MediaResourceRecord, DbErr> {
        let backend = self.db.get_database_backend();
        let tx = self.db.begin().await?;
        tx.execute(statement(
            backend,
            r#"INSERT INTO media_resources (
                id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
                mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
                status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                provider = excluded.provider, storage_profile = excluded.storage_profile, bucket = excluded.bucket,
                object_key = excluded.object_key, public_base_url = excluded.public_base_url, public_url = excluded.public_url,
                mime_type = excluded.mime_type, extension = excluded.extension, size_bytes = excluded.size_bytes,
                sha256_hex = excluded.sha256_hex, etag = excluded.etag, width = excluded.width, height = excluded.height,
                duration_ms = excluded.duration_ms, status = excluded.status, source_name = excluded.source_name,
                source_path = excluded.source_path, uploaded_by_user_id = excluded.uploaded_by_user_id, updated_at = CURRENT_TIMESTAMP"#,
            vec![
                resource.id.clone().into(), resource.provider.clone().into(), resource.storage_profile.clone().into(),
                resource.bucket.clone().into(), resource.object_key.clone().into(), resource.public_base_url.clone().into(),
                resource.public_url.clone().into(), resource.mime_type.clone().into(), resource.extension.clone().into(),
                resource.size_bytes.into(), resource.sha256_hex.clone().into(), resource.etag.clone().into(),
                resource.width.into(), resource.height.into(), resource.duration_ms.into(), resource.status.clone().into(),
                resource.source_name.clone().into(), resource.source_path.clone().into(), resource.uploaded_by_user_id.clone().into(),
            ],
        )).await?;
        tx.execute(statement(
            backend,
            r#"INSERT INTO knot_media_resources (
                knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(knot_id, asset_id) DO UPDATE SET
                media_type = excluded.media_type, media_resource_id = excluded.media_resource_id, sort_order = excluded.sort_order,
                attribution = excluded.attribution, license_note = excluded.license_note, updated_at = CURRENT_TIMESTAMP"#,
            vec![
                link.knot_id.clone().into(), link.asset_id.clone().into(), link.media_type.clone().into(),
                link.media_resource_id.clone().into(), link.sort_order.into(), link.attribution.clone().into(), link.license_note.clone().into(),
            ],
        )).await?;
        tx.commit().await?;
        self.get_media_resource(&resource.id)
            .await?
            .ok_or_else(|| DbErr::Custom("upserted media resource not found".to_owned()))
    }

    /// Fetches one media resource by id.
    pub async fn get_media_resource(&self, id: &str) -> Result<Option<MediaResourceRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, provider, storage_profile, bucket, object_key, public_base_url, public_url, \
                 mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms, \
                 status, source_name, source_path, uploaded_by_user_id \
                 FROM media_resources WHERE id = ?",
                vec![id.to_owned().into()],
            ))
            .await?;
        row.map(map_media_resource).transpose()
    }

    /// Lists active public media assets linked to one knot.
    pub async fn list_knot_media_assets(
        &self,
        knot_id: &str,
    ) -> Result<Vec<KnotMediaAsset>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                r#"SELECT kmr.asset_id, kmr.media_type, mr.public_url, mr.mime_type, mr.width, mr.height,
                          mr.size_bytes, kmr.attribution, kmr.license_note
                   FROM knot_media_resources kmr
                   JOIN media_resources mr ON mr.id = kmr.media_resource_id
                   WHERE kmr.knot_id = ? AND mr.status = 'active'
                   ORDER BY CASE kmr.asset_id
                       WHEN 'thumbnail' THEN 0
                       WHEN 'preview' THEN 1
                       WHEN 'draw_gif' THEN 2
                       WHEN 'turntable_gif' THEN 3
                       WHEN 'draw_mp4' THEN 4
                       WHEN 'turntable_mp4' THEN 5
                       ELSE 1000 + kmr.sort_order
                   END ASC, kmr.sort_order ASC, kmr.asset_id ASC"#,
                vec![knot_id.to_owned().into()],
            ))
            .await?;
        rows.into_iter().map(map_knot_media_asset).collect()
    }
}

fn map_media_resource(row: QueryResult) -> Result<MediaResourceRecord, DbErr> {
    Ok(MediaResourceRecord {
        id: row.try_get("", "id")?,
        provider: row.try_get("", "provider")?,
        storage_profile: row.try_get("", "storage_profile")?,
        bucket: row.try_get("", "bucket")?,
        object_key: row.try_get("", "object_key")?,
        public_base_url: row.try_get("", "public_base_url")?,
        public_url: row.try_get("", "public_url")?,
        mime_type: row.try_get("", "mime_type")?,
        extension: row.try_get("", "extension")?,
        size_bytes: row.try_get("", "size_bytes")?,
        sha256_hex: row.try_get("", "sha256_hex")?,
        etag: row.try_get("", "etag")?,
        width: row.try_get("", "width")?,
        height: row.try_get("", "height")?,
        duration_ms: row.try_get("", "duration_ms")?,
        status: row.try_get("", "status")?,
        source_name: row.try_get("", "source_name")?,
        source_path: row.try_get("", "source_path")?,
        uploaded_by_user_id: row.try_get("", "uploaded_by_user_id")?,
    })
}

fn map_knot_media_asset(row: QueryResult) -> Result<KnotMediaAsset, DbErr> {
    Ok(KnotMediaAsset {
        id: row.try_get("", "asset_id")?,
        media_type: row.try_get("", "media_type")?,
        url: row.try_get("", "public_url")?,
        mime_type: row.try_get("", "mime_type")?,
        width: row.try_get("", "width")?,
        height: row.try_get("", "height")?,
        size_bytes: row.try_get("", "size_bytes")?,
        attribution: row.try_get("", "attribution")?,
        license_note: row.try_get("", "license_note")?,
    })
}
