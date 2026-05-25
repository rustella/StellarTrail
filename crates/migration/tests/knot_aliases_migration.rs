use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_migration::Migrator;

#[tokio::test]
async fn knot_aliases_migration_adds_and_removes_localization_column() {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");

    assert!(table_has_column(&db, "knot_localizations", "aliases_json").await);

    Migrator::down(&db, Some(1))
        .await
        .expect("rollback aliases migration");
    assert!(!table_has_column(&db, "knot_localizations", "aliases_json").await);
}

async fn table_has_column(db: &sea_orm::DatabaseConnection, table: &str, column: &str) -> bool {
    let rows = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            format!("PRAGMA table_info({table})"),
        ))
        .await
        .expect("table info");
    rows.into_iter().any(|row| {
        let name: String = row.try_get("", "name").expect("column name");
        name == column
    })
}
