use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_migration::Migrator;

#[tokio::test]
async fn knot_risk_copy_migration_removes_critical_public_phrasing() {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    let repair_index = Migrator::migrations()
        .iter()
        .position(|migration| migration.name() == "m20260524_000006_sanitize_knot_risk_copy")
        .expect("copy repair migration is registered");
    Migrator::up(&db, Some(repair_index as u32))
        .await
        .expect("migrate before copy repair");

    for (id, field, critical_value) in [
        (
            "figure-eight-follow-through-knot",
            "description",
            "特别的，它可用于将绳子系在攀岩安全带上。",
        ),
        (
            "firemans-chair-knot",
            "description",
            "可以用作支撑人员的救援安全带。",
        ),
        ("handcuff-knot", "description", "可用于将人员拉出狭小空间。"),
        (
            "shear-lashing-knot",
            "summary",
            "将两根杆子绑为支点，用于桥梁、杠杆或吊装。",
        ),
        (
            "spanish-bowline-knot",
            "summary",
            "用于双点吊装或固定的双环结。",
        ),
    ] {
        insert_knot_localization(&db, id, field, critical_value).await;
    }

    Migrator::up(&db, None)
        .await
        .expect("run copy repair migration");

    for id in [
        "figure-eight-follow-through-knot",
        "firemans-chair-knot",
        "handcuff-knot",
        "shear-lashing-knot",
        "spanish-bowline-knot",
    ] {
        let public_copy = read_public_copy(&db, id).await;
        for critical in ["救援安全带", "吊装", "系在攀岩安全带", "支撑人员"] {
            assert!(
                !public_copy.contains(critical),
                "{id} still contains critical copy: {critical}"
            );
        }
    }
}

async fn insert_knot_localization(
    db: &sea_orm::DatabaseConnection,
    id: &str,
    critical_field: &str,
    critical_value: &str,
) {
    let backend = db.get_database_backend();
    db.execute(Statement::from_sql_and_values(
        backend,
        "INSERT INTO knots(id, source_name, source_url, source_slug_en, source_slug_zh) \
         VALUES (?, 'Knots 3D', NULL, ?, NULL)",
        vec![id.into(), id.into()],
    ))
    .await
    .expect("insert knot");

    let summary = if critical_field == "summary" {
        critical_value
    } else {
        "用于结构学习的绳结。"
    };
    let description = if critical_field == "description" {
        critical_value
    } else {
        "仅供学习。"
    };
    db.execute(Statement::from_sql_and_values(
        backend,
        "INSERT INTO knot_localizations(knot_id, locale, slug, title, summary, description, steps_json) \
         VALUES (?, 'zh-CN', ?, ?, ?, ?, '[]')",
        vec![
            id.into(),
            id.into(),
            id.into(),
            summary.into(),
            description.into(),
        ],
    ))
    .await
    .expect("insert knot localization");
}

async fn read_public_copy(db: &sea_orm::DatabaseConnection, id: &str) -> String {
    let backend = db.get_database_backend();
    let row = db
        .query_one(Statement::from_sql_and_values(
            backend,
            "SELECT summary, description FROM knot_localizations WHERE knot_id = ? AND locale = 'zh-CN'",
            vec![id.into()],
        ))
        .await
        .expect("query copy")
        .expect("copy row");
    let summary: String = row.try_get("", "summary").expect("summary");
    let description: Option<String> = row.try_get("", "description").expect("description");
    format!("{summary}\n{}", description.unwrap_or_default())
}
