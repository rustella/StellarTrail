use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{DatabaseConfig, connect_database, repositories::GearTemplateRepository};
use stellartrail_domain::{
    gear_template::{GearTemplateCategorySeed, GearTemplateItemSeed, GearTemplateSeed},
    locale::Locale,
};
use stellartrail_migration::Migrator;

fn temp_db_url(name: &str) -> String {
    let unique = format!(
        "stellartrail-gear-template-{name}-{}-{}.sqlite",
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

fn backpacking_seed(title: &str) -> GearTemplateSeed {
    GearTemplateSeed {
        id: "backpacking-basic".to_owned(),
        title: title.to_owned(),
        localizations: vec![
            (Locale::ZhCn, title.to_owned()),
            (Locale::En, format!("{title} EN")),
        ],
        sort_order: 10,
        categories: vec![
            GearTemplateCategorySeed {
                id: "rain_protection".to_owned(),
                name: "防雨防风".to_owned(),
                localizations: vec![
                    (Locale::ZhCn, "防雨防风".to_owned()),
                    (Locale::En, "Rain protection".to_owned()),
                ],
                sort_order: 20,
                items: vec![
                    GearTemplateItemSeed {
                        id: "rain-shell".to_owned(),
                        name: "雨衣或硬壳".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "雨衣或硬壳".to_owned()),
                            (Locale::En, "Rain shell".to_owned()),
                        ],
                        sort_order: 20,
                    },
                    GearTemplateItemSeed {
                        id: "pack-cover".to_owned(),
                        name: "背包防雨罩".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "背包防雨罩".to_owned()),
                            (Locale::En, "Pack cover".to_owned()),
                        ],
                        sort_order: 30,
                    },
                ],
            },
            GearTemplateCategorySeed {
                id: "navigation".to_owned(),
                name: "导航记录".to_owned(),
                localizations: vec![
                    (Locale::ZhCn, "导航记录".to_owned()),
                    (Locale::En, "Navigation".to_owned()),
                ],
                sort_order: 30,
                items: vec![GearTemplateItemSeed {
                    id: "offline-map".to_owned(),
                    name: "离线地图".to_owned(),
                    localizations: vec![
                        (Locale::ZhCn, "离线地图".to_owned()),
                        (Locale::En, "Offline map".to_owned()),
                    ],
                    sort_order: 10,
                }],
            },
        ],
    }
}

#[tokio::test]
async fn repository_replaces_system_templates_without_touching_custom_rows() {
    let config = DatabaseConfig::new(temp_db_url("replace")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = GearTemplateRepository::new(db.clone());

    repo.replace_system_templates("system_seed", &[backpacking_seed("入门徒步基础装备模板")])
        .await
        .expect("seed templates");
    repo.replace_system_templates(
        "system_seed",
        &[backpacking_seed("入门徒步基础装备模板 v2")],
    )
    .await
    .expect("replace templates");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "INSERT INTO gear_templates(id, title, source, status, sort_order, created_at, updated_at) \
         VALUES ('custom-template', '自定义模板', 'user_seed', 'active', 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
            .to_owned(),
    ))
    .await
    .expect("insert custom template");

    let templates = repo
        .list_templates(Locale::ZhCn)
        .await
        .expect("list templates");

    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].id, "custom-template");
    assert_eq!(templates[1].id, "backpacking-basic");
    assert_eq!(templates[1].title, "入门徒步基础装备模板 v2");
    assert_eq!(templates[1].categories[0].id, "rain_protection");
    assert_eq!(
        templates[1].categories[0].items,
        vec!["雨衣或硬壳", "背包防雨罩"]
    );
}

#[tokio::test]
async fn repository_gets_nested_template_detail_and_missing_id() {
    let config = DatabaseConfig::new(temp_db_url("get")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = GearTemplateRepository::new(db);
    repo.replace_system_templates("system_seed", &[backpacking_seed("入门徒步基础装备模板")])
        .await
        .expect("seed templates");

    let template = repo
        .get_template("backpacking-basic", Locale::ZhCn)
        .await
        .expect("get template")
        .expect("template exists");
    let english = repo
        .get_template("backpacking-basic", Locale::En)
        .await
        .expect("get template")
        .expect("template exists");
    let missing = repo
        .get_template("missing-template", Locale::ZhCn)
        .await
        .expect("missing lookup");

    assert_eq!(template.categories[1].name, "导航记录");
    assert_eq!(template.categories[1].items, vec!["离线地图"]);
    assert_eq!(english.title, "入门徒步基础装备模板 EN");
    assert_eq!(english.categories[1].name, "Navigation");
    assert_eq!(english.categories[1].items, vec!["Offline map"]);
    assert!(missing.is_none());
}

#[tokio::test]
async fn repository_falls_back_to_chinese_when_requested_locale_is_missing() {
    let config = DatabaseConfig::new(temp_db_url("fallback")).expect("db config");
    let db = connect_database(&config).await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");
    let repo = GearTemplateRepository::new(db.clone());
    repo.replace_system_templates("system_seed", &[backpacking_seed("入门徒步基础装备模板")])
        .await
        .expect("seed templates");
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM gear_template_category_localizations \
         WHERE template_id = 'backpacking-basic' AND category_id = 'navigation' AND locale = 'en'"
            .to_owned(),
    ))
    .await
    .expect("delete english category");

    let template = repo
        .get_template("backpacking-basic", Locale::En)
        .await
        .expect("get template")
        .expect("template exists");

    assert_eq!(template.title, "入门徒步基础装备模板 EN");
    assert_eq!(template.categories[1].name, "导航记录");
}
