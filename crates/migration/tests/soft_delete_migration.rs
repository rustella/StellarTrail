use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_migration::Migrator;

#[tokio::test]
async fn soft_delete_migration_adds_flags_and_rolls_back() {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");

    for table in [
        "user_gear_items",
        "gear_atlas_items",
        "user_feedback",
        "upload_images",
    ] {
        assert!(table_has_column(&db, table, "is_deleted").await);
    }

    db.execute_unprepared(
        "INSERT INTO users(id, created_at, updated_at) VALUES ('user-1', '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z')",
    )
    .await
    .expect("insert user");
    db.execute_unprepared(
        "INSERT INTO user_gear_items(id, user_id, category, name, status, tags_json, share_enabled, share_status, specs_json, created_at, updated_at) \
         VALUES ('gear-1', 'user-1', 'backpack_system', '背包', 'available', '[]', FALSE, 'not_shared', '{}', '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z')",
    )
    .await
    .expect("insert gear");
    db.execute_unprepared(
        "INSERT INTO gear_atlas_items(id, category, name, variants_json, specs_json, source_type, submitted_by_user_id, status, created_at, updated_at) \
         VALUES ('atlas-1', 'backpack_system', '背包', '[]', '{}', 'manual', 'user-1', 'pending', '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z')",
    )
    .await
    .expect("insert atlas");
    db.execute_unprepared(
        "INSERT INTO user_feedback(id, user_id, category, content, status, created_at, updated_at) \
         VALUES ('feedback-1', 'user-1', 'bug', 'broken', 'open', '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z')",
    )
    .await
    .expect("insert feedback");
    db.execute_unprepared(
        "INSERT INTO upload_images(id, user_id, purpose, original_filename, bucket, object_key, image_type, content_type, size_bytes, sha256, created_at) \
         VALUES ('upload-1', 'user-1', 'feedback', 'a.png', 'bucket', 'object', 'png', 'image/png', 12, 'hash', '2026-05-23T00:00:00Z')",
    )
    .await
    .expect("insert upload");

    for (table, id) in [
        ("user_gear_items", "gear-1"),
        ("gear_atlas_items", "atlas-1"),
        ("user_feedback", "feedback-1"),
        ("upload_images", "upload-1"),
    ] {
        assert!(!read_is_deleted(&db, table, id).await);
    }

    Migrator::down(&db, Some(3)).await.expect("rollback");
    for table in [
        "user_gear_items",
        "gear_atlas_items",
        "user_feedback",
        "upload_images",
    ] {
        assert!(!table_has_column(&db, table, "is_deleted").await);
    }
}

async fn table_has_column(db: &sea_orm::DatabaseConnection, table: &str, column: &str) -> bool {
    let backend = db.get_database_backend();
    let rows = db
        .query_all(Statement::from_string(
            backend,
            format!("PRAGMA table_info({table})"),
        ))
        .await
        .expect("table info");
    rows.into_iter().any(|row| {
        let name: String = row.try_get("", "name").expect("column name");
        name == column
    })
}

async fn read_is_deleted(db: &sea_orm::DatabaseConnection, table: &str, id: &str) -> bool {
    let backend = db.get_database_backend();
    db.query_one(Statement::from_string(
        backend,
        format!("SELECT is_deleted FROM {table} WHERE id = '{id}'"),
    ))
    .await
    .expect("query flag")
    .expect("flag row")
    .try_get("", "is_deleted")
    .expect("is_deleted")
}
