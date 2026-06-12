use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{
    DatabaseConfig, connect_database,
    repositories::{
        AuthRepository, GearAtlasExternalImportAction, GearAtlasRepository,
        ListGearAtlasAdminOptions, ListGearAtlasOptions,
    },
};
use stellartrail_domain::{
    deletion::DeletedFilter,
    gear::{GearCategory, GearSpecs},
    gear_atlas::{
        GearAtlasExternalImportDraft, GearAtlasLocalizationDraft, GearAtlasLocalizationReviewState,
        GearAtlasStatus,
    },
    locale::Locale,
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
        variants: Vec::new(),
        specs: GearSpecs::new(),
        submitted_by_user_id: submitter_user_id.to_owned(),
        source_key: source_key.to_owned(),
        source_name: "8264 户外用品点评".to_owned(),
        source_url: Some("https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html".to_owned()),
        source_license_note: Some("facts and source link only".to_owned()),
        import_batch_id: Some("batch-20260521".to_owned()),
        source_rating_score: Some(8.6),
        source_rating_count: Some(7),
        canonical_key: None,
        source_locale: Some(Locale::ZhCn),
        detail_score: None,
        localizations: vec![
            GearAtlasLocalizationDraft {
                locale: Locale::ZhCn,
                name: name.to_owned(),
                description: Some(
                    "来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。".to_owned(),
                ),
                variants: Vec::new(),
                specs: GearSpecs::new(),
                translation_status: Some("source".to_owned()),
                translation_provider: None,
                translated_at: None,
            },
            GearAtlasLocalizationDraft {
                locale: Locale::En,
                name: format!("{name} EN"),
                description: Some(
                    "Pending external-import gear atlas item from public fact fields.".to_owned(),
                ),
                variants: Vec::new(),
                specs: GearSpecs::new(),
                translation_status: Some("translated".to_owned()),
                translation_provider: Some("test".to_owned()),
                translated_at: None,
            },
        ],
    }
}

#[tokio::test]
async fn public_list_searches_all_localized_atlas_text_and_keeps_response_locale() {
    let config = DatabaseConfig::new(temp_db_url("localized-search")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user("atlas_locale", "atlas-locale@example.test", "hash")
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db.clone());

    let mut draft = external_draft("locale:headlamp", "中文头灯", &user.id);
    draft.category = GearCategory::LightingSystem;
    draft.description = Some("适合夜间徒步的照明装备".to_owned());
    draft.validate_and_normalize().expect("valid draft");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("create import")
        .item;
    repo.approve(&imported.id, &user.id)
        .await
        .expect("approve")
        .expect("approved item");

    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        "INSERT INTO gear_atlas_item_localizations(atlas_item_id, locale, name, description) \
         VALUES (?, 'en', ?, ?) \
         ON CONFLICT(atlas_item_id, locale) DO UPDATE SET \
         name = excluded.name, description = excluded.description",
        vec![
            imported.id.clone().into(),
            "Public headlamp".to_owned().into(),
            "Lighting tool for night hiking".to_owned().into(),
        ],
    ))
    .await
    .expect("insert english localization");

    let (zh_from_english, _) = repo
        .list_public(
            &ListGearAtlasOptions {
                q: Some("headlamp".to_owned()),
                ..Default::default()
            },
            Locale::ZhCn,
        )
        .await
        .expect("list zh from english query");
    assert_eq!(zh_from_english.len(), 1);
    assert_eq!(zh_from_english[0].name, "中文头灯");

    let (en_from_chinese, _) = repo
        .list_public(
            &ListGearAtlasOptions {
                q: Some("头灯".to_owned()),
                ..Default::default()
            },
            Locale::En,
        )
        .await
        .expect("list en from chinese query");
    assert_eq!(en_from_chinese.len(), 1);
    assert_eq!(en_from_chinese[0].name, "Public headlamp");

    let (zh_from_category_en, _) = repo
        .list_public(
            &ListGearAtlasOptions {
                q: Some("Lighting".to_owned()),
                ..Default::default()
            },
            Locale::ZhCn,
        )
        .await
        .expect("list zh from english category");
    assert_eq!(zh_from_category_en.len(), 1);
    assert_eq!(zh_from_category_en[0].name, "中文头灯");

    let (en_from_category_zh, _) = repo
        .list_public(
            &ListGearAtlasOptions {
                q: Some("照明".to_owned()),
                ..Default::default()
            },
            Locale::En,
        )
        .await
        .expect("list en from chinese category");
    assert_eq!(en_from_category_zh.len(), 1);
    assert_eq!(en_from_category_zh[0].name, "Public headlamp");

    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        "DELETE FROM gear_atlas_item_localizations WHERE atlas_item_id = ? AND locale = 'en'",
        vec![imported.id.clone().into()],
    ))
    .await
    .expect("delete english localization");
    let (fallback, _) = repo
        .list_public(
            &ListGearAtlasOptions {
                q: Some("头灯".to_owned()),
                ..Default::default()
            },
            Locale::En,
        )
        .await
        .expect("list fallback");
    assert_eq!(fallback.len(), 1);
    assert_eq!(fallback[0].name, "中文头灯");
}

#[tokio::test]
async fn localization_review_statuses_require_reviewed_bilingual_rows() {
    let config = DatabaseConfig::new(temp_db_url("localization-review-status")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user(
            "atlas_localization_review",
            "atlas-localization-review@example.test",
            "hash",
        )
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db.clone());

    let mut draft = external_draft("locale-review:tent", "测试帐篷", &user.id);
    draft.validate_and_normalize().expect("valid draft");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("create import")
        .item;

    let initial = repo
        .localization_review_statuses(&imported)
        .await
        .expect("initial statuses");
    assert_eq!(initial.len(), 2);
    assert!(
        initial
            .iter()
            .all(|status| status.state == GearAtlasLocalizationReviewState::NeedsReview)
    );

    for locale in [Locale::ZhCn, Locale::En] {
        repo.upsert_item_localization(
            &imported.id,
            &GearAtlasLocalizationDraft {
                locale,
                name: if locale == Locale::ZhCn {
                    "测试帐篷".to_owned()
                } else {
                    "Test Tent".to_owned()
                },
                description: Some(if locale == Locale::ZhCn {
                    "人工审核后的中文展示描述。".to_owned()
                } else {
                    "Reviewed English display description.".to_owned()
                }),
                variants: Vec::new(),
                specs: GearSpecs::new(),
                translation_status: Some("reviewed".to_owned()),
                translation_provider: Some("admin-manual".to_owned()),
                translated_at: None,
            },
        )
        .await
        .expect("upsert reviewed localization");
    }

    let reviewed = repo
        .localization_review_statuses(&imported)
        .await
        .expect("reviewed statuses");
    assert!(
        reviewed
            .iter()
            .all(|status| status.state == GearAtlasLocalizationReviewState::Reviewed)
    );
}

#[tokio::test]
async fn localized_display_copy_keeps_canonical_admin_fields_unchanged() {
    let config = DatabaseConfig::new(temp_db_url("admin-display")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user(
            "atlas_admin_display",
            "atlas-admin-display@example.test",
            "hash",
        )
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db.clone());

    let mut draft = external_draft(
        "outdoor:admin-display",
        "Sierra Designs Backcountry Bivy",
        &user.id,
    );
    draft.category = GearCategory::SleepSystem;
    draft.source_locale = Some(Locale::En);
    draft.localizations = vec![
        GearAtlasLocalizationDraft {
            locale: Locale::En,
            name: "Sierra Designs Backcountry Bivy".to_owned(),
            description: Some("Source English description".to_owned()),
            variants: Vec::new(),
            specs: GearSpecs::from([("type".to_owned(), "Bivy".to_owned())]),
            translation_status: Some("source".to_owned()),
            translation_provider: None,
            translated_at: None,
        },
        GearAtlasLocalizationDraft {
            locale: Locale::ZhCn,
            name: "Sierra Designs Backcountry 露营袋".to_owned(),
            description: Some("待审核的外部导入装备条目，规格来自公开来源事实字段。".to_owned()),
            variants: Vec::new(),
            specs: GearSpecs::from([("type".to_owned(), "露营袋".to_owned())]),
            translation_status: Some("needs_review".to_owned()),
            translation_provider: Some("test".to_owned()),
            translated_at: None,
        },
    ];
    draft.validate_and_normalize().expect("valid draft");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("create import")
        .item;

    let display = repo
        .localized_display_item(&imported, Locale::ZhCn)
        .await
        .expect("display item");
    assert_eq!(imported.name, "Sierra Designs Backcountry Bivy");
    assert_eq!(display.name, "Sierra Designs Backcountry 露营袋");
    assert_eq!(
        display.specs.get("type").map(String::as_str),
        Some("露营袋")
    );

    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        "DELETE FROM gear_atlas_item_localizations WHERE atlas_item_id = ? AND locale = 'zh-CN'",
        vec![imported.id.clone().into()],
    ))
    .await
    .expect("delete zh localization");
    let fallback = repo
        .localized_display_item(&imported, Locale::ZhCn)
        .await
        .expect("fallback display item");
    assert_eq!(fallback.name, "Sierra Designs Backcountry Bivy");
}

#[tokio::test]
async fn localization_backfill_candidates_are_idempotent_and_keep_canonical_name() {
    let config = DatabaseConfig::new(temp_db_url("localization-backfill")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user("atlas_backfill", "atlas-backfill@example.test", "hash")
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db);

    let mut draft = external_draft(
        "outdoor:backfill-display",
        "Sierra Designs Backcountry Bivy",
        &user.id,
    );
    draft.category = GearCategory::SleepSystem;
    draft.source_locale = Some(Locale::En);
    draft.localizations = vec![GearAtlasLocalizationDraft {
        locale: Locale::En,
        name: "Sierra Designs Backcountry Bivy".to_owned(),
        description: Some("Source English description".to_owned()),
        variants: Vec::new(),
        specs: GearSpecs::from([("type".to_owned(), "Bivy".to_owned())]),
        translation_status: Some("source".to_owned()),
        translation_provider: None,
        translated_at: None,
    }];
    draft.validate_and_normalize().expect("valid draft");
    let imported = repo
        .upsert_external_import(&draft)
        .await
        .expect("create import")
        .item;

    let candidates = repo
        .list_external_import_localization_backfill_candidates(Locale::En, Locale::ZhCn, 20)
        .await
        .expect("list candidates");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, imported.id);

    repo.upsert_item_localization(
        &imported.id,
        &GearAtlasLocalizationDraft {
            locale: Locale::ZhCn,
            name: "Sierra Designs Backcountry 露营袋".to_owned(),
            description: Some("待审核的外部导入装备条目，规格来自公开来源事实字段。".to_owned()),
            variants: Vec::new(),
            specs: GearSpecs::from([("type".to_owned(), "露营袋".to_owned())]),
            translation_status: Some("needs_review".to_owned()),
            translation_provider: Some("test".to_owned()),
            translated_at: None,
        },
    )
    .await
    .expect("upsert zh localization");

    let candidates_after = repo
        .list_external_import_localization_backfill_candidates(Locale::En, Locale::ZhCn, 20)
        .await
        .expect("list candidates after upsert");
    assert!(candidates_after.is_empty());

    let canonical = repo
        .get_any(&imported.id)
        .await
        .expect("get canonical")
        .expect("canonical item");
    let display = repo
        .localized_display_item(&canonical, Locale::ZhCn)
        .await
        .expect("display item");
    assert_eq!(canonical.name, "Sierra Designs Backcountry Bivy");
    assert_eq!(display.name, "Sierra Designs Backcountry 露营袋");
}

#[tokio::test]
async fn external_import_upserts_pending_rows_and_skips_approved_rows() {
    let config = DatabaseConfig::new(temp_db_url("external-import")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user("atlas_source", "atlas-source@example.test", "hash")
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db);

    let mut first = external_draft("8264:2074165", "探路者38L户外背包", &user.id);
    first.weight_g = Some(980);
    first.specs = GearSpecs::from([("capacity".to_owned(), "38L".to_owned())]);
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
    assert_eq!(created.item.weight_g, Some(980));
    assert_eq!(
        created.item.specs.get("capacity").map(String::as_str),
        Some("38L")
    );
    assert_eq!(created.item.status.as_str(), "pending");

    let mut refreshed = external_draft("8264:2074165", "探路者38L户外背包 v2", &user.id);
    refreshed.official_price_cents = Some(39_900);
    refreshed.weight_g = Some(1_050);
    refreshed.specs = GearSpecs::from([
        ("capacity".to_owned(), "45L".to_owned()),
        ("recommended_load".to_owned(), "12kg".to_owned()),
    ]);
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
    assert_eq!(updated.item.weight_g, Some(1_050));
    assert_eq!(
        updated.item.specs.get("capacity").map(String::as_str),
        Some("45L")
    );
    assert_eq!(
        updated
            .item
            .specs
            .get("recommended_load")
            .map(String::as_str),
        Some("12kg")
    );
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

    assert!(
        repo.soft_delete(&created.item.id)
            .await
            .expect("delete atlas")
    );
    let (public_items, _) = repo
        .list_public(&ListGearAtlasOptions::default(), Locale::ZhCn)
        .await
        .expect("list public");
    assert!(public_items.iter().all(|item| item.id != created.item.id));
    let (deleted_items, _) = repo
        .list_admin(&ListGearAtlasAdminOptions {
            status: Some(GearAtlasStatus::Approved),
            deleted: DeletedFilter::Deleted,
            ..Default::default()
        })
        .await
        .expect("list deleted admin");
    assert_eq!(deleted_items.len(), 1);
    assert!(deleted_items[0].is_deleted);

    let restored_by_import = repo
        .upsert_external_import(&after_approval)
        .await
        .expect("restore deleted import");
    assert_eq!(
        restored_by_import.action,
        GearAtlasExternalImportAction::Updated
    );
    assert_eq!(restored_by_import.item.id, created.item.id);
    assert_eq!(restored_by_import.item.name, "不应覆盖已审核条目");
    assert_eq!(restored_by_import.item.status.as_str(), "pending");
    assert!(!restored_by_import.item.is_deleted);
    assert!(
        repo.soft_delete(&created.item.id)
            .await
            .expect("delete pending")
    );
    let restored = repo
        .restore_deleted(&created.item.id)
        .await
        .expect("restore atlas")
        .expect("restored item");
    assert!(!restored.is_deleted);
}

#[tokio::test]
async fn external_import_deduplicates_by_canonical_key_and_keeps_more_detailed_pending_row() {
    let config = DatabaseConfig::new(temp_db_url("canonical-dedupe")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let user = AuthRepository::new(db.clone())
        .create_password_user("atlas_dedupe", "atlas-dedupe@example.test", "hash")
        .await
        .expect("create import user");
    let repo = GearAtlasRepository::new(db.clone());

    let mut first = external_draft("packwizard:tent-a", "Big Agnes Test Tent", &user.id);
    first.category = GearCategory::SleepSystem;
    first.canonical_key = Some("external-gear:test:big-agnes-test-tent".to_owned());
    first.source_locale = Some(Locale::En);
    first.detail_score = Some(30);
    first.specs = GearSpecs::from([("people_count".to_owned(), "2".to_owned())]);
    first.validate_and_normalize().expect("valid first draft");
    let created = repo
        .upsert_external_import(&first)
        .await
        .expect("create canonical import");
    assert_eq!(created.action, GearAtlasExternalImportAction::Created);

    let mut less_detailed = external_draft("trailspace:tent-a", "Less Detailed Tent", &user.id);
    less_detailed.category = GearCategory::SleepSystem;
    less_detailed.canonical_key = first.canonical_key.clone();
    less_detailed.source_locale = Some(Locale::En);
    less_detailed.detail_score = Some(10);
    less_detailed
        .validate_and_normalize()
        .expect("valid less detailed draft");
    let skipped = repo
        .upsert_external_import(&less_detailed)
        .await
        .expect("skip less detailed duplicate");
    assert_eq!(
        skipped.action,
        GearAtlasExternalImportAction::SkippedLessDetailed
    );
    assert_eq!(skipped.item.id, created.item.id);
    assert_eq!(skipped.item.name, "Big Agnes Test Tent");

    let mut more_detailed = external_draft("trailspace:tent-b", "More Detailed Tent", &user.id);
    more_detailed.category = GearCategory::SleepSystem;
    more_detailed.canonical_key = first.canonical_key.clone();
    more_detailed.source_locale = Some(Locale::En);
    more_detailed.detail_score = Some(45);
    more_detailed.weight_g = Some(1_280);
    more_detailed.specs = GearSpecs::from([
        ("people_count".to_owned(), "2".to_owned()),
        ("packed_size".to_owned(), "48 x 15 cm".to_owned()),
    ]);
    more_detailed
        .validate_and_normalize()
        .expect("valid more detailed draft");
    let updated = repo
        .upsert_external_import(&more_detailed)
        .await
        .expect("update duplicate with more detail");
    assert_eq!(updated.action, GearAtlasExternalImportAction::Updated);
    assert_eq!(updated.item.id, created.item.id);
    assert_eq!(updated.item.name, "More Detailed Tent");
    assert_eq!(updated.item.weight_g, Some(1_280));

    let source_rows: i64 = db
        .query_one(Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT COUNT(*) AS count FROM gear_atlas_import_sources WHERE canonical_key = ?",
            vec!["external-gear:test:big-agnes-test-tent".to_owned().into()],
        ))
        .await
        .expect("count source rows")
        .expect("count row")
        .try_get("", "count")
        .expect("count value");
    assert_eq!(source_rows, 3);

    repo.approve(&updated.item.id, &user.id)
        .await
        .expect("approve updated row")
        .expect("approved item");
    let mut after_approval = external_draft("packwizard:tent-c", "Approved should stay", &user.id);
    after_approval.category = GearCategory::SleepSystem;
    after_approval.canonical_key = first.canonical_key.clone();
    after_approval.detail_score = Some(100);
    after_approval
        .validate_and_normalize()
        .expect("valid approved duplicate");
    let approved_skip = repo
        .upsert_external_import(&after_approval)
        .await
        .expect("skip approved duplicate");
    assert_eq!(
        approved_skip.action,
        GearAtlasExternalImportAction::SkippedApproved
    );
    assert_eq!(approved_skip.item.name, "More Detailed Tent");
}
