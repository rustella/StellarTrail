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
        difficulty: Some("beginner".to_owned()),
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

#[tokio::test]
async fn repository_migrates_upserts_and_queries_locale_resolved_knots_with_offset() {
    let config = DatabaseConfig::new(temp_db_url("repo")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = KnotRepository::new(db, "/assets");
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

    let detail = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::ZhCn)
        .await
        .expect("query")
        .expect("detail");
    assert_eq!(detail.title, "可调节绳结");
    assert_eq!(detail.categories[0].title, "露营");
    assert_eq!(
        detail.media[0].url,
        "/assets/skills/knots/adjustable-grip-hitch-knot/thumbnail.webp"
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

    Migrator::down(&db, Some(1))
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
