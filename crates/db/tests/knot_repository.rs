use stellartrail_db::{DatabaseConfig, KnotRepository};
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
};

fn temp_db_url(name: &str) -> String {
    let unique = format!(
        "stellartrail-db-{name}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    format!("sqlite://{}", std::env::temp_dir().join(unique).display())
}

fn seed() -> KnotSeed {
    KnotSeed {
        id: "adjustable-grip-hitch-knot".to_owned(),
        source_name: "Knots 3D".to_owned(),
        source_url: Some("https://knots3d.com/en/adjustable-grip-hitch-knot".to_owned()),
        source_slug_en: "adjustable-grip-hitch-knot".to_owned(),
        source_slug_zh: Some("可调节绳结".to_owned()),
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
        raw_metadata: serde_json::json!({"english_slug":"adjustable-grip-hitch-knot","zh_slug":"可调节绳结"}),
    }
}

#[test]
fn repository_migrates_upserts_and_queries_locale_resolved_knots_with_offset() {
    let config = DatabaseConfig::new(temp_db_url("repo")).expect("db config");
    let repo = KnotRepository::connect(&config)
        .expect("connect")
        .migrate()
        .expect("migrate");
    repo.replace_all_knots("unit-test", &[seed()])
        .expect("seed");

    let categories = repo
        .list_skill_categories(Locale::ZhCn)
        .expect("categories");
    assert_eq!(categories[0].id, "knots");
    assert_eq!(categories[0].title, "绳结");
    assert_eq!(categories[0].item_count, 1);

    let page = repo
        .list_knots(Locale::En, 0, 20, None, None)
        .expect("list");
    assert_eq!(page.page.offset, 0);
    assert_eq!(page.page.next_offset, None);
    assert_eq!(page.items[0].title, "Adjustable Grip Hitch");
    assert_eq!(page.items[0].slug, "adjustable-grip-hitch-knot");

    let detail = repo
        .get_knot_detail("adjustable-grip-hitch-knot", Locale::ZhCn)
        .expect("query")
        .expect("detail");
    assert_eq!(detail.title, "可调节绳结");
    assert_eq!(detail.categories[0].title, "露营");
    assert_eq!(
        detail.media[0].url,
        "/assets/skills/knots/adjustable-grip-hitch-knot/thumbnail.webp"
    );
}
