use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
};
use stellartrail_migration::Migrator;

fn temp_db_url(name: &str) -> String {
    let unique = format!(
        "stellartrail-db-{name}-{}-{}.sqlite",
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

fn seed() -> KnotSeed {
    KnotSeed {
        id: "adjustable-grip-hitch-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/en/adjustable-grip-hitch-knot".to_owned()),
        source_slug_en: "adjustable-grip-hitch-knot".to_owned(),
        source_slug_zh: Some("ke-tiao-jie-sheng-jie".to_owned()),
        localizations: vec![
            KnotLocalizationSeed {
                locale: Locale::En,
                slug: "adjustable-grip-hitch-knot".to_owned(),
                title: "Adjustable Grip Hitch".to_owned(),
                summary: "Adjust tension on a line.".to_owned(),
                description: None,
                steps: vec![],
            },
            KnotLocalizationSeed {
                locale: Locale::ZhCn,
                slug: "ke-tiao-jie-sheng-jie".to_owned(),
                title: "可调节绳结".to_owned(),
                summary: "调节绳索上的张力。".to_owned(),
                description: None,
                steps: vec![],
            },
        ],
        categories: vec![KnotCategorySeed {
            id: "camping-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "camping-knots".to_owned(),
                    "Camping Knots".to_owned(),
                ),
                (Locale::ZhCn, "lu-ying".to_owned(), "露营".to_owned()),
            ],
        }],
        types: vec![KnotTypeSeed {
            id: "hitch-knots".to_owned(),
            localizations: vec![
                (Locale::En, "hitch-knots".to_owned(), "Hitches".to_owned()),
                (Locale::ZhCn, "tao-jie".to_owned(), "套结".to_owned()),
            ],
        }],
        media: vec![KnotMediaAssetSeed {
            id: "preview".to_owned(),
            media_type: "thumbnail".to_owned(),
            path: "skills/knots/adjustable-grip-hitch-knot/thumbnail.webp".to_owned(),
            mime_type: "image/webp".to_owned(),
            width: Some(320),
            height: Some(180),
            attribution: Some("Knots 3D".to_owned()),
            license_note: Some("authorization required".to_owned()),
        }],
        raw_metadata: serde_json::json!({"english_slug":"adjustable-grip-hitch-knot","zh_slug":"ke-tiao-jie-sheng-jie"}),
    }
}

fn technical_seed() -> KnotSeed {
    KnotSeed {
        id: "figure-eight-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/en/figure-eight-knot".to_owned()),
        source_slug_en: "figure-eight-knot".to_owned(),
        source_slug_zh: Some("ba-zi-jie".to_owned()),
        localizations: vec![
            KnotLocalizationSeed {
                locale: Locale::En,
                slug: "figure-eight-knot".to_owned(),
                title: "Figure Eight Knot".to_owned(),
                summary: "Stopper knot for rope ends.".to_owned(),
                description: None,
                steps: vec![],
            },
            KnotLocalizationSeed {
                locale: Locale::ZhCn,
                slug: "ba-zi-jie".to_owned(),
                title: "八字结".to_owned(),
                summary: "用于绳端防脱的基础结。".to_owned(),
                description: None,
                steps: vec![],
            },
        ],
        categories: vec![KnotCategorySeed {
            id: "basic-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "basic-knots".to_owned(),
                    "Basic Knots".to_owned(),
                ),
                (
                    Locale::ZhCn,
                    "ji-chu-sheng-jie".to_owned(),
                    "基础绳结".to_owned(),
                ),
            ],
        }],
        types: vec![KnotTypeSeed {
            id: "stopper-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "stopper-knots".to_owned(),
                    "Stopper Knots".to_owned(),
                ),
                (Locale::ZhCn, "zhi-dong-jie".to_owned(), "止动结".to_owned()),
            ],
        }],
        media: vec![],
        raw_metadata: serde_json::json!({"english_slug":"figure-eight-knot","zh_slug":"ba-zi-jie"}),
    }
}

async fn insert_uploaded_media(
    db: &sea_orm::DatabaseConnection,
    media_id: &str,
    knot_id: &str,
    asset_id: &str,
    media_type: &str,
    public_url: &str,
    size_bytes: i64,
) {
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO media_resources (
            id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
            mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
            status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
        ) VALUES (?, 'minio', 'knots-public', 'stellartrail-knots-media', ?, 'https://media.example.test', ?, 'image/webp', 'webp', ?, ?, NULL, 320, 180, NULL, 'active', 'unit-test', 'relative/source.webp', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![
            media_id.to_owned().into(),
            format!("skills/knots/{knot_id}/{asset_id}/{media_id}.webp").into(),
            public_url.to_owned().into(),
            size_bytes.into(),
            format!("sha-{media_id}").into(),
        ],
    ))
    .await
    .expect("insert media resource");
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO knot_media_resources (
            knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, 0, 'Knots3D', 'authorization required', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![
            knot_id.to_owned().into(),
            asset_id.to_owned().into(),
            media_type.to_owned().into(),
            media_id.to_owned().into(),
        ],
    ))
    .await
    .expect("insert knot media resource");
}

#[tokio::test]
async fn repository_migrates_upserts_and_queries_locale_resolved_knots_with_offset() {
    let config = DatabaseConfig::new(temp_db_url("repo")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db.clone());
    repo.replace_all_knots("unit-test", &[seed()])
        .await
        .expect("seed");

    let categories = repo
        .list_skill_categories(Locale::ZhCn)
        .await
        .expect("categories");
    assert_eq!(categories[0].id, "knots");
    assert_eq!(categories[0].title, "绳结");
    assert_eq!(categories[0].item_count, 1);

    let page = repo
        .list_knots(Locale::En, 0, 20, None, None)
        .await
        .expect("list");
    assert_eq!(page.page.offset, 0);
    assert_eq!(page.page.next_offset, None);
    assert_eq!(page.items[0].title, "Adjustable Grip Hitch");
    assert_eq!(page.items[0].slug, "adjustable-grip-hitch-knot");

    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO media_resources (
            id, provider, storage_profile, bucket, object_key, public_base_url, public_url,
            mime_type, extension, size_bytes, sha256_hex, etag, width, height, duration_ms,
            status, source_name, source_path, uploaded_by_user_id, created_at, updated_at
        ) VALUES (?, 'minio', 'knots-public', 'stellartrail-knots-media', ?, 'https://media.example.test', ?, 'image/webp', 'webp', 12345, ?, NULL, 320, 180, NULL, 'active', 'unit-test', 'relative/source.webp', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![
            "media-adjustable-grip-hitch-thumbnail".to_owned().into(),
            "skills/knots/adjustable-grip-hitch-knot/thumbnail/hash.webp".to_owned().into(),
            "https://media.example.test/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash.webp".to_owned().into(),
            "0123456789abcdef".to_owned().into(),
        ],
    ))
    .await
    .expect("insert media resource");
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        r#"INSERT INTO knot_media_resources (
            knot_id, asset_id, media_type, media_resource_id, sort_order, attribution, license_note,
            created_at, updated_at
        ) VALUES ('adjustable-grip-hitch-knot', 'thumbnail', 'thumbnail', 'media-adjustable-grip-hitch-thumbnail', 0, 'Knots3D', 'authorization required', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        vec![],
    ))
    .await
    .expect("insert knot media resource");

    let detail = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::ZhCn)
        .await
        .expect("query")
        .expect("detail");
    assert_eq!(detail.title, "可调节绳结");
    assert_eq!(detail.categories[0].title, "露营");
    assert_eq!(
        detail.media[0].url,
        "https://media.example.test/skills/knots/adjustable-grip-hitch-knot/thumbnail/hash.webp"
    );
    assert_eq!(detail.media[0].size_bytes, 12345);
    assert_eq!(detail.media[0].id, "thumbnail");
}

#[tokio::test]
async fn repository_builds_offline_manifest_with_fallback_locale_and_deduped_media() {
    let config = DatabaseConfig::new(temp_db_url("offline-manifest")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db.clone());
    repo.replace_all_knots("unit-test", &[seed(), technical_seed()])
        .await
        .expect("seed");

    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        "DELETE FROM knot_localizations WHERE knot_id = ? AND locale = ?",
        vec![
            "figure-eight-knot".to_owned().into(),
            Locale::ZhCn.as_str().to_owned().into(),
        ],
    ))
    .await
    .expect("delete zh localization");

    insert_uploaded_media(
        &db,
        "media-thumb",
        "adjustable-grip-hitch-knot",
        "thumbnail",
        "thumbnail",
        "https://media.example.test/knots/shared.webp",
        111,
    )
    .await;
    insert_uploaded_media(
        &db,
        "media-preview",
        "adjustable-grip-hitch-knot",
        "preview",
        "preview",
        "https://media.example.test/knots/shared.webp",
        111,
    )
    .await;
    insert_uploaded_media(
        &db,
        "media-draw-gif",
        "adjustable-grip-hitch-knot",
        "draw_gif",
        "draw_gif",
        "https://media.example.test/knots/draw.gif",
        222,
    )
    .await;

    let manifest = repo
        .offline_manifest(Locale::ZhCn)
        .await
        .expect("offline manifest");

    assert_eq!(manifest.locale, Locale::ZhCn);
    assert_eq!(manifest.item_count, 2);
    assert_eq!(manifest.media_count, 2);
    assert_eq!(manifest.estimated_bytes, 333);
    let adjustable = manifest
        .items
        .iter()
        .find(|item| item.id == "adjustable-grip-hitch-knot")
        .expect("adjustable knot");
    assert_eq!(adjustable.title, "可调节绳结");
    assert_eq!(adjustable.categories[0].title, "露营");
    assert_eq!(adjustable.types[0].title, "套结");
    assert_eq!(adjustable.media[0].id, "thumbnail");
    assert_eq!(adjustable.media[1].id, "preview");
    assert_eq!(adjustable.media[2].id, "draw_gif");

    let fallback = manifest
        .items
        .iter()
        .find(|item| item.id == "figure-eight-knot")
        .expect("fallback knot");
    assert_eq!(fallback.title, "Figure Eight Knot");
    assert_eq!(fallback.locale, Locale::ZhCn);
}

#[tokio::test]
async fn repository_lists_filter_options_and_filters_knots_by_category_and_query() {
    let config = DatabaseConfig::new(temp_db_url("filters")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db.clone());
    repo.replace_all_knots("unit-test", &[seed(), technical_seed()])
        .await
        .expect("seed");

    let filters = repo.list_knot_filters(Locale::ZhCn).await.expect("filters");
    assert_eq!(filters.locale, Locale::ZhCn);
    let camping = filters
        .categories
        .iter()
        .find(|option| option.id == "camping-knots")
        .expect("camping option");
    assert_eq!(camping.slug.as_deref(), Some("lu-ying"));
    assert_eq!(camping.title, "露营");
    assert_eq!(camping.count, 1);
    let page = repo
        .list_knots(Locale::ZhCn, 0, 20, Some("camping-knots"), Some("调节"))
        .await
        .expect("filtered list");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "adjustable-grip-hitch-knot");

    let empty = repo
        .list_knots(Locale::ZhCn, 0, 20, Some("camping-knots"), Some("八字"))
        .await
        .expect("empty filtered list");
    assert!(empty.items.is_empty());
    assert_eq!(empty.page.next_offset, None);
}

#[tokio::test]
async fn repository_uses_conservative_labels_for_high_risk_knot_categories() {
    let config = DatabaseConfig::new(temp_db_url("safe-category-labels")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db.clone());
    let mut risk_seed = seed();
    risk_seed.categories = vec![
        KnotCategorySeed {
            id: "climbing-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "climbing-knots".to_owned(),
                    "Climbing".to_owned(),
                ),
                (Locale::ZhCn, "pan-yan".to_owned(), "攀岩".to_owned()),
            ],
        },
        KnotCategorySeed {
            id: "fire-search-rescue-sar-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "fire-search-rescue-sar-knots".to_owned(),
                    "Fire and Rescue".to_owned(),
                ),
                (
                    Locale::ZhCn,
                    "xiao-fang-yu-jiu-yuan".to_owned(),
                    "消防与救援".to_owned(),
                ),
            ],
        },
        KnotCategorySeed {
            id: "boating-knots".to_owned(),
            localizations: vec![
                (Locale::En, "boating-knots".to_owned(), "Boating".to_owned()),
                (Locale::ZhCn, "chuan-ting".to_owned(), "船艇".to_owned()),
            ],
        },
        KnotCategorySeed {
            id: "essential-knots".to_owned(),
            localizations: vec![
                (
                    Locale::En,
                    "essential-knots".to_owned(),
                    "Essential Knots".to_owned(),
                ),
                (
                    Locale::ZhCn,
                    "bi-bei-sheng-jie".to_owned(),
                    "必备绳结".to_owned(),
                ),
            ],
        },
    ];
    repo.replace_all_knots("unit-test", &[risk_seed])
        .await
        .expect("seed");

    let detail = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::ZhCn)
        .await
        .expect("query")
        .expect("detail");
    let labels = detail
        .categories
        .iter()
        .map(|item| (item.id.as_str(), item.title.as_str()))
        .collect::<Vec<_>>();
    assert!(labels.contains(&("climbing-knots", "攀岩知识（仅供学习）")));
    assert!(labels.contains(&("fire-search-rescue-sar-knots", "消防与救援知识（仅供学习）")));
    assert!(labels.contains(&("boating-knots", "船艇知识（仅供学习）")));
    assert!(labels.contains(&("essential-knots", "基础绳结")));
    assert!(!labels.iter().any(|(_, title)| *title == "必备绳结"));

    let filters = repo.list_knot_filters(Locale::ZhCn).await.expect("filters");
    assert!(filters.categories.iter().any(|option| {
        option.id == "fire-search-rescue-sar-knots" && option.title == "消防与救援知识（仅供学习）"
    }));

    let english = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::En)
        .await
        .expect("query english")
        .expect("english detail");
    assert!(english.categories.iter().any(|item| {
        item.id == "climbing-knots" && item.title == "Climbing Knowledge (Learning Only)"
    }));
    assert!(
        english
            .categories
            .iter()
            .any(|item| item.id == "essential-knots" && item.title == "Basic Knots")
    );
}

#[tokio::test]
async fn repository_ignores_legacy_knot_media_assets_when_no_media_resources_exist() {
    let config = DatabaseConfig::new(temp_db_url("legacy-media")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db.clone());
    repo.replace_all_knots("unit-test", &[seed()])
        .await
        .expect("seed");

    let detail = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::ZhCn)
        .await
        .expect("query")
        .expect("detail");
    assert!(
        detail.media.is_empty(),
        "legacy /assets media must not be returned: {:?}",
        detail.media
    );
}

#[tokio::test]
async fn knots_migration_down_preserves_shared_skill_categories() {
    let config = DatabaseConfig::new(temp_db_url("migration-down")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO skill_categories(id, slug) VALUES ('navigation', 'navigation')".to_owned(),
    ))
    .await
    .expect("insert shared category");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO skill_category_localizations(category_id, locale, title, summary)          VALUES ('navigation', 'en', 'Navigation', 'Map and compass skills')"
            .to_owned(),
    ))
    .await
    .expect("insert shared category localization");

    // Roll back every migration from knots content onward. Keep this dynamic so
    // appending later migrations does not leave the knots seed row in place.
    let migrations_after_base_schema = Migrator::migrations()
        .len()
        .checked_sub(6)
        .expect("knots migration follows six base migrations");
    Migrator::down(&db, Some(migrations_after_base_schema as u32))
        .await
        .expect("roll back knots migration");

    let shared_count = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT COUNT(*) AS count FROM skill_categories WHERE id = 'navigation'".to_owned(),
        ))
        .await
        .expect("query shared category")
        .expect("shared category count row")
        .try_get::<i64>("", "count")
        .expect("shared category count");
    assert_eq!(shared_count, 1);

    let knots_count = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT COUNT(*) AS count FROM skill_categories WHERE id = 'knots'".to_owned(),
        ))
        .await
        .expect("query knots category")
        .expect("knots category count row")
        .try_get::<i64>("", "count")
        .expect("knots category count");
    assert_eq!(knots_count, 0);
}
