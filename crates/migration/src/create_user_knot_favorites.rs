//! User knot favorites migration.
//!
//! Favorites are account-scoped user state and therefore live outside the
//! public knot catalog tables. Deleting a favorite is modeled as a soft delete
//! so user intent history can be preserved without leaking deleted rows into
//! normal list queries.

use sea_orm_migration::prelude::*;

/// Single migration type for the user knot favorites table.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260524_000002_create_user_knot_favorites"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the account-scoped knot favorites table and its list index.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_knot_favorites (
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                knot_id TEXT NOT NULL REFERENCES knots(id) ON DELETE CASCADE,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                favorited_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, knot_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_knot_favorites_user_deleted_favorited \
             ON user_knot_favorites(user_id, is_deleted, favorited_at)",
        )
        .await?;
        Ok(())
    }

    /// Removes the user knot favorites table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_user_knot_favorites_user_deleted_favorited",
        )
        .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS user_knot_favorites")
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
        crate::create_users_sessions::Migration
            .up(&manager)
            .await
            .unwrap();
        crate::create_knots_content::Migration
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
    async fn creates_soft_deleted_user_knot_favorites_table() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_dependencies(&db).await;
        let manager = SchemaManager::new(&db);

        Migration.up(&manager).await.unwrap();

        let columns = table_columns(&db, "user_knot_favorites").await;
        let names = columns
            .into_iter()
            .map(|row| row.try_get::<String>("", "name").unwrap())
            .collect::<Vec<_>>();
        for expected in [
            "user_id",
            "knot_id",
            "is_deleted",
            "favorited_at",
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
            r#"INSERT INTO knots (id, source_name, source_slug_en)
               VALUES ('bowline-knot', 'test', 'bowline-knot')"#,
        ))
        .await
        .unwrap();
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"INSERT INTO user_knot_favorites (
                user_id, knot_id, favorited_at, created_at, updated_at
            ) VALUES (
                'user-1', 'bowline-knot', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )"#,
        ))
        .await
        .unwrap();

        let row = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT is_deleted FROM user_knot_favorites",
            ))
            .await
            .unwrap()
            .unwrap();
        assert!(!row.try_get::<bool>("", "is_deleted").unwrap());
    }

    #[tokio::test]
    async fn down_removes_user_knot_favorites_table() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_dependencies(&db).await;
        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.unwrap();
        Migration.down(&manager).await.unwrap();

        let columns = table_columns(&db, "user_knot_favorites").await;
        assert!(columns.is_empty());
    }
}
