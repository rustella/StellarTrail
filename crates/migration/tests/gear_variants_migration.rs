use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use serde_json::Value;
use stellartrail_migration::Migrator;

#[tokio::test]
async fn migrates_legacy_size_specs_into_variants_and_selected_size() {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    Migrator::up(&db, Some(18))
        .await
        .expect("pre-variant schema");

    db.execute_unprepared(
        "INSERT INTO users(id, created_at, updated_at) VALUES ('user-1', '2026-05-22T00:00:00Z', '2026-05-22T00:00:00Z')",
    )
    .await
    .expect("insert user");
    db.execute_unprepared(
        "INSERT INTO gear_atlas_items(
            id, category, name, specs_json, source_type, submitted_by_user_id,
            status, created_at, updated_at
         ) VALUES (
            'atlas-1', 'sleep_system', '睡袋', '{\"size\":\"M 75*195 L 80*205\",\"fill_weight\":\"700g\"}',
            'external_import', 'user-1', 'pending', '2026-05-22T00:00:00Z', '2026-05-22T00:00:00Z'
         )",
    )
    .await
    .expect("insert atlas");
    db.execute_unprepared(
        "INSERT INTO user_gear_items(
            id, user_id, category, name, status, specs_json, tags_json,
            share_enabled, share_status, created_at, updated_at
         ) VALUES (
            'gear-1', 'user-1', 'backpack_system', '背包', 'available',
            '{\"backpack_size\":\"M\",\"back_length\":\"48 cm\"}', '[]',
            FALSE, 'not_shared', '2026-05-22T00:00:00Z', '2026-05-22T00:00:00Z'
         )",
    )
    .await
    .expect("insert gear");

    Migrator::up(&db, None).await.expect("variant migration");

    let backend = db.get_database_backend();
    let atlas = db
        .query_one(Statement::from_string(
            backend,
            "SELECT specs_json, variants_json FROM gear_atlas_items WHERE id = 'atlas-1'"
                .to_owned(),
        ))
        .await
        .expect("query atlas")
        .expect("atlas row");
    let atlas_specs: String = atlas.try_get("", "specs_json").expect("atlas specs");
    let atlas_variants: String = atlas.try_get("", "variants_json").expect("atlas variants");
    let atlas_specs: Value = serde_json::from_str(&atlas_specs).expect("atlas specs json");
    let atlas_variants: Value = serde_json::from_str(&atlas_variants).expect("atlas variants json");
    assert!(atlas_specs.get("size").is_none());
    assert_eq!(atlas_specs["fill_weight"], "700g");
    assert_eq!(atlas_variants[0]["label"], "M 75*195");
    assert_eq!(atlas_variants[1]["label"], "L 80*205");

    let gear = db
        .query_one(Statement::from_string(
            backend,
            "SELECT specs_json, selected_variant_label FROM user_gear_items WHERE id = 'gear-1'"
                .to_owned(),
        ))
        .await
        .expect("query gear")
        .expect("gear row");
    let gear_specs: String = gear.try_get("", "specs_json").expect("gear specs");
    let selected_variant_label: Option<String> = gear
        .try_get("", "selected_variant_label")
        .expect("selected variant label");
    let gear_specs: Value = serde_json::from_str(&gear_specs).expect("gear specs json");
    assert!(gear_specs.get("backpack_size").is_none());
    assert_eq!(gear_specs["back_length"], "48 cm");
    assert_eq!(selected_variant_label.as_deref(), Some("M"));
}
