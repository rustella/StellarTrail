//! Media resource migration adding MinIO-backed reusable media metadata and knot media mappings.

use sea_orm_migration::prelude::*;

/// Migration adding DB metadata for object-storage media resources.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates media resource tables without rewriting existing knot content migrations.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(MEDIA_RESOURCES_SCHEMA_SQL).await?;
        Ok(())
    }

    /// Drops only the media resource tables introduced by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_knot_media_resources_media")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_media_resources_sha")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_media_resources_status")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS knot_media_resources")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS media_resources")
            .await?;
        Ok(())
    }
}

const MEDIA_RESOURCES_SCHEMA_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS media_resources (
        id TEXT PRIMARY KEY,
        provider TEXT NOT NULL,
        storage_profile TEXT NOT NULL,
        bucket TEXT NOT NULL,
        object_key TEXT NOT NULL,
        public_base_url TEXT NOT NULL,
        public_url TEXT NOT NULL,
        mime_type TEXT NOT NULL,
        extension TEXT NOT NULL,
        size_bytes INTEGER NOT NULL,
        sha256_hex TEXT NOT NULL,
        etag TEXT NULL,
        width INTEGER NULL,
        height INTEGER NULL,
        duration_ms INTEGER NULL,
        status TEXT NOT NULL DEFAULT 'active',
        source_name TEXT NULL,
        source_path TEXT NULL,
        uploaded_by_user_id TEXT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(provider, bucket, object_key),
        CHECK (provider IN ('minio')),
        CHECK (status IN ('active', 'deleted'))
    );

    CREATE TABLE IF NOT EXISTS knot_media_resources (
        knot_id TEXT NOT NULL,
        asset_id TEXT NOT NULL,
        media_type TEXT NOT NULL,
        media_resource_id TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        attribution TEXT NULL,
        license_note TEXT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (knot_id, asset_id),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        FOREIGN KEY (media_resource_id) REFERENCES media_resources(id) ON DELETE RESTRICT
    );

    CREATE INDEX IF NOT EXISTS idx_knot_media_resources_media ON knot_media_resources(media_resource_id);
    CREATE INDEX IF NOT EXISTS idx_media_resources_sha ON media_resources(sha256_hex);
    CREATE INDEX IF NOT EXISTS idx_media_resources_status ON media_resources(status);
"#;
