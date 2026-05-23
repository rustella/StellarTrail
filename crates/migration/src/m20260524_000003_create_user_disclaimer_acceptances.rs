//! User disclaimer acceptance migration.
//!
//! Disclaimer acceptances are account-scoped audit records. Each disclaimer
//! version is stored separately so future legal copy changes can require a new
//! acknowledgement without losing the original consent trail.

use sea_orm_migration::prelude::*;

/// Single migration type for account-scoped disclaimer acceptances.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the disclaimer acceptance table and user lookup index.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_disclaimer_acceptances (
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                disclaimer_key TEXT NOT NULL,
                version TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                client_platform TEXT NULL,
                client_version TEXT NULL,
                device_model TEXT NULL,
                accepted_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, disclaimer_key, version)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_disclaimer_acceptances_user_updated \
             ON user_disclaimer_acceptances(user_id, updated_at)",
        )
        .await?;
        Ok(())
    }

    /// Removes disclaimer acceptance audit records.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_disclaimer_acceptances_user_updated")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS user_disclaimer_acceptances")
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, QueryResult, Statement};

    async fn run_dependencies(db: &sea_orm::DatabaseConnection) {
        let manager = SchemaManager::new(db);
        crate::m20260516_000001_create_users_sessions::Migration
            .up(&manager)
            .await
            .unwrap();
    }

    async fn table_columns(db: &sea_orm::DatabaseConnection, table: &str) -> Vec<QueryResult> {
        db.query_all(Statement::from_string(
            db.get_database_backend(),
            format!("PRAGMA table_info({table})"),
        ))
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn creates_user_disclaimer_acceptances_table() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_dependencies(&db).await;
        let manager = SchemaManager::new(&db);

        Migration.up(&manager).await.unwrap();

        let columns = table_columns(&db, "user_disclaimer_acceptances").await;
        let names = columns
            .into_iter()
            .map(|row| row.try_get::<String>("", "name").unwrap())
            .collect::<Vec<_>>();
        for expected in [
            "user_id",
            "disclaimer_key",
            "version",
            "title",
            "content",
            "client_platform",
            "client_version",
            "device_model",
            "accepted_at",
            "created_at",
            "updated_at",
        ] {
            assert!(names.contains(&expected.to_owned()), "missing {expected}");
        }

        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"INSERT INTO users (id, wechat_openid, created_at, updated_at)
               VALUES ('user-1', 'openid-1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        ))
        .await
        .unwrap();
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"INSERT INTO user_disclaimer_acceptances (
                user_id, disclaimer_key, version, title, content,
                accepted_at, created_at, updated_at
            ) VALUES (
                'user-1', 'knot_tutorial', 'v1', '声明', '内容',
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )"#,
        ))
        .await
        .unwrap();

        let row = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM user_disclaimer_acceptances",
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.try_get::<i64>("", "count").unwrap(), 1);
    }

    #[tokio::test]
    async fn down_removes_user_disclaimer_acceptances_table() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_dependencies(&db).await;
        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.unwrap();
        Migration.down(&manager).await.unwrap();

        let columns = table_columns(&db, "user_disclaimer_acceptances").await;
        assert!(columns.is_empty());
    }
}
