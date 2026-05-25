//! Migration creating database-backed administrator roles.
//!
//! The bootstrap seed only upgrades an already-existing, non-deleted
//! `stellarisw` account to `super_admin`. It never creates placeholder users or
//! reserves that username, keeping account ownership under normal auth flows.

use sea_orm_migration::prelude::*;

/// Single migration type for the administrator role table.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260518_000006_create_admin_roles"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates `admin_roles` and seeds an existing `stellarisw` user when present.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS admin_roles (
                user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
                role TEXT NOT NULL CHECK (role IN ('admin', 'super_admin')),
                granted_by_user_id TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_admin_roles_role ON admin_roles(role)",
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO admin_roles (
                user_id, role, granted_by_user_id, created_at, updated_at
            )
            SELECT id, 'super_admin', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            FROM users
            WHERE username = 'stellarisw'
              AND deleted_at IS NULL
            ON CONFLICT (user_id) DO UPDATE SET
                role = 'super_admin',
                updated_at = excluded.updated_at"#,
        )
        .await?;
        Ok(())
    }

    /// Drops administrator role state.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_admin_roles_role")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS admin_roles")
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, Statement};

    async fn run_user_migrations(db: &sea_orm::DatabaseConnection) {
        let manager = SchemaManager::new(db);
        crate::create_users_sessions::Migration
            .up(&manager)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn seeds_existing_stellarisw_as_super_admin() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_user_migrations(&db).await;
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"INSERT INTO users (
                id, username, email, password_hash, nickname, created_at, updated_at
            ) VALUES (
                'user-stellarisw', 'stellarisw', 'stellarisw@example.test',
                'hash', 'stellarisw', '2026-05-18T00:00:00Z', '2026-05-18T00:00:00Z'
            )"#,
        ))
        .await
        .unwrap();

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.unwrap();

        let role = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT role FROM admin_roles WHERE user_id = 'user-stellarisw'",
            ))
            .await
            .unwrap()
            .unwrap();
        let role: String = role.try_get("", "role").unwrap();
        assert_eq!(role, "super_admin");
    }

    #[tokio::test]
    async fn missing_stellarisw_does_not_create_user_or_role() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        run_user_migrations(&db).await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.unwrap();

        let user_count = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM users WHERE username = 'stellarisw'",
            ))
            .await
            .unwrap()
            .unwrap();
        let user_count: i64 = user_count.try_get("", "count").unwrap();
        let role_count = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM admin_roles",
            ))
            .await
            .unwrap()
            .unwrap();
        let role_count: i64 = role_count.try_get("", "count").unwrap();
        assert_eq!(user_count, 0);
        assert_eq!(role_count, 0);
    }
}
