use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{AuthRepository, GearAtlasExternalImportAction, GearAtlasRepository},
};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs},
    gear_atlas::GearAtlasExternalImportDraft,
};
use stellartrail_migration::Migrator;

fn temp_db_url(name: &str) -> String {
    let unique = format!(
        "stellartrail-gear-atlas-{name}-{}-{}.sqlite",
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

fn external_draft(
    source_key: &str,
    name: &str,
    submitter_user_id: &str,
) -> GearAtlasExternalImportDraft {
    GearAtlasExternalImportDraft {
        category: GearCategory::BackpackSystem,
        name: name.to_owned(),
        brand: None,
        model: None,
        description: Some(
            "来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。".to_owned(),
        ),
        weight_g: None,
        official_price_cents: Some(34_900),
        official_price_currency: Some("CNY".to_owned()),
        specs: GearSpecs::new(),
        submitted_by_user_id: submitter_user_id.to_owned(),
        source_key: source_key.to_owned(),
        source_name: "8264 户外用品点评".to_owned(),
        source_url: Some("https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html".to_owned()),
        source_license_note: Some("facts and source link only".to_owned()),
        import_batch_id: Some("batch-20260521".to_owned()),
        source_rating_score: Some(8.6),
        source_rating_count: Some(7),
    }
}

#[tokio::test]
async fn external_import_upserts_pending_rows_and_skips_approved_rows() {
    let config = DatabaseConfig::new(temp_db_url("external-import")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user("atlas_importer", "atlas-importer@example.test", "hash")
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db);

    let mut first = external_draft("8264:2074165", "探路者38L户外背包", &user.id);
    first.validate_and_normalize().expect("valid first draft");
    let created = repo
        .upsert_external_import(&first)
        .await
        .expect("create import");
    assert_eq!(created.action, GearAtlasExternalImportAction::Created);
    assert_eq!(created.item.source_type.as_str(), "external_import");
    assert_eq!(created.item.source_key.as_deref(), Some("8264:2074165"));
    assert_eq!(
        created.item.source_name.as_deref(),
        Some("8264 户外用品点评")
    );
    assert_eq!(created.item.source_rating_score, Some(8.6));
    assert_eq!(created.item.source_rating_count, Some(7));
    assert_eq!(created.item.status.as_str(), "pending");

    let mut refreshed = external_draft("8264:2074165", "探路者38L户外背包 v2", &user.id);
    refreshed.official_price_cents = Some(39_900);
    refreshed
        .validate_and_normalize()
        .expect("valid refresh draft");
    let updated = repo
        .upsert_external_import(&refreshed)
        .await
        .expect("update import");
    assert_eq!(updated.action, GearAtlasExternalImportAction::Updated);
    assert_eq!(updated.item.id, created.item.id);
    assert_eq!(updated.item.name, "探路者38L户外背包 v2");
    assert_eq!(updated.item.official_price_cents, Some(39_900));
    assert_eq!(updated.item.status.as_str(), "pending");

    repo.approve(&updated.item.id, &user.id)
        .await
        .expect("approve import")
        .expect("approved row");
    let mut after_approval = external_draft("8264:2074165", "不应覆盖已审核条目", &user.id);
    after_approval
        .validate_and_normalize()
        .expect("valid skipped draft");
    let skipped = repo
        .upsert_external_import(&after_approval)
        .await
        .expect("skip approved import");
    assert_eq!(
        skipped.action,
        GearAtlasExternalImportAction::SkippedApproved
    );
    assert_eq!(skipped.item.id, created.item.id);
    assert_eq!(skipped.item.name, "探路者38L户外背包 v2");
    assert_eq!(skipped.item.status.as_str(), "approved");
}
