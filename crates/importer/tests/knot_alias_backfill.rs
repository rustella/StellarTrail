use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_importer::{
    knot_alias_backfill::{
        AliasBackfillExpectations, AliasBackfillOptions, AliasBackfillSource,
        backfill_knot_localization_aliases,
    },
    parse_knots3d_metadata,
};
use stellartrail_migration::Migrator;

fn temp_db_url(name: &str) -> String {
    let unique = format!(
        "stellartrail-importer-{name}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    format!(
        "sqlite://{}?mode=rwc",
        std::env::temp_dir().join(unique).display()
    )
}

#[tokio::test]
async fn backfills_aliases_from_raw_metadata_without_touching_media_mappings() {
    let database_url = temp_db_url("alias-backfill");
    let db = connect_database(&DatabaseConfig::new(database_url.clone()).expect("db config"))
        .await
        .expect("connect");
    Migrator::up(&db, None).await.expect("migrate");

    let seeds = parse_knots3d_metadata(
        r#"{
            "items": [{
                "id": "adjustable-grip-hitch-knot",
                "english_slug": "adjustable-grip-hitch-knot",
                "zh_slug": "ke-tiao-jie-sheng-jie",
                "english_name": "Adjustable Grip Hitch",
                "chinese_name": "可调节绳结",
                "languages": {
                    "en": {"aliases": ["Adjustable Loop", "Cawley Adjustable Hitch"]},
                    "zh-CN": {"aliases": ["可调节活结"]}
                }
            }]
        }"#,
    )
    .expect("parse seed");
    KnotRepository::new(db.clone())
        .replace_all_knots("alias-backfill-test", &seeds)
        .await
        .expect("seed knots");
    db.execute_unprepared("UPDATE knot_localizations SET aliases_json = '[]'")
        .await
        .expect("clear imported aliases");
    insert_media_mapping(&db).await;

    let expectations = AliasBackfillExpectations {
        items: 1,
        localizations: 2,
        media_resources: 1,
        knot_media_resources: 1,
    };
    let dry_run = backfill_knot_localization_aliases(AliasBackfillOptions {
        database_url: database_url.clone(),
        source: AliasBackfillSource::RawDb,
        dry_run: true,
        expectations: expectations.clone(),
    })
    .await
    .expect("dry run");
    assert_eq!(dry_run.source_items, 1);
    assert_eq!(dry_run.total_alias_rows, 3);
    assert_eq!(dry_run.would_update_localization_rows, 2);
    assert_eq!(
        read_aliases_json(&db, "adjustable-grip-hitch-knot", "en").await,
        "[]"
    );

    let applied = backfill_knot_localization_aliases(AliasBackfillOptions {
        database_url,
        source: AliasBackfillSource::RawDb,
        dry_run: false,
        expectations,
    })
    .await
    .expect("apply");
    assert_eq!(applied.updated_localization_rows, Some(2));
    assert_eq!(applied.media_resources_before, 1);
    assert_eq!(applied.media_resources_after, Some(1));
    assert_eq!(applied.knot_media_resources_before, 1);
    assert_eq!(applied.knot_media_resources_after, Some(1));
    assert_eq!(
        read_aliases_json(&db, "adjustable-grip-hitch-knot", "en").await,
        r#"["Adjustable Loop","Cawley Adjustable Hitch"]"#
    );
    assert_eq!(
        read_aliases_json(&db, "adjustable-grip-hitch-knot", "zh-CN").await,
        r#"["可调节活结"]"#
    );
}

async fn insert_media_mapping(db: &sea_orm::DatabaseConnection) {
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO media_resources (
            id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
            mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
            status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
        ) VALUES ('media-1', 'minio', 'knots-public', 'stellartrail-knots-media', 'object.webp',
            'https://media.example.test', 'https://media.example.test/object.webp',
            'image/webp', 'webp', 123, 'sha-media-1', NULL, 320, 180, NULL, 'active',
            'unit-test', 'source.webp', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![],
    ))
    .await
    .expect("insert media resource");
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO knot_media_resources (
            knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note,
            created_at, updated_at
        ) VALUES ('adjustable-grip-hitch-knot', 'thumbnail', 'thumbnail', 'media-1', 0,
            'Knots3D', 'authorization required', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![],
    ))
    .await
    .expect("insert knot media resource");
}

async fn read_aliases_json(
    db: &sea_orm::DatabaseConnection,
    knot_id: &str,
    locale: &str,
) -> String {
    db.query_one(Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT aliases_json FROM knot_localizations WHERE knot_id = ? AND locale = ?",
        vec![knot_id.into(), locale.into()],
    ))
    .await
    .expect("query aliases")
    .expect("alias row")
    .try_get("", "aliases_json")
    .expect("aliases_json")
}
