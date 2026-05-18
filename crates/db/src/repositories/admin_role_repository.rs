//! Repository for database-backed administrator roles.
//!
//! Administrator authorization is keyed by immutable `users.id` values. Username
//! lookup exists only as an API convenience and always resolves to an existing,
//! non-deleted user before role changes are written.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};
use stellartrail_domain::admin::AdminRole;
use time::{OffsetDateTime, format_description::well_known::Iso8601};

use super::statement;

/// Persisted administrator role row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminRoleRecord {
    pub user_id: String,
    pub role: AdminRole,
    pub granted_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Minimal user projection used while resolving administrator selectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTargetUser {
    pub id: String,
    pub username: Option<String>,
}

/// Result of granting a regular administrator role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantAdminResult {
    pub record: AdminRoleRecord,
    pub created: bool,
}

/// Repository wrapper around administrator role persistence.
#[derive(Clone)]
pub struct AdminRoleRepository {
    db: DatabaseConnection,
}

impl AdminRoleRepository {
    /// Creates a repository bound to the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds an administrator role by user id.
    pub async fn find_role(&self, user_id: &str) -> Result<Option<AdminRoleRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                role_select_sql("WHERE user_id = ? LIMIT 1"),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_admin_role(&row)).transpose()
    }

    /// Resolves a non-deleted user by immutable user id.
    pub async fn find_target_user_by_id(
        &self,
        user_id: &str,
    ) -> Result<Option<AdminTargetUser>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, username FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_target_user(&row)).transpose()
    }

    /// Resolves a non-deleted user by normalized username.
    pub async fn find_target_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AdminTargetUser>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, username FROM users WHERE username = ? AND deleted_at IS NULL LIMIT 1",
                vec![username.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_target_user(&row)).transpose()
    }

    /// Grants `admin` to a user that does not already have an administrator role.
    ///
    /// Existing `admin` or `super_admin` rows are returned unchanged so granting
    /// regular admin never downgrades a super administrator.
    pub async fn grant_admin(
        &self,
        target_user_id: &str,
        granted_by_user_id: &str,
    ) -> Result<GrantAdminResult, DbErr> {
        if let Some(record) = self.find_role(target_user_id).await? {
            return Ok(GrantAdminResult {
                record,
                created: false,
            });
        }

        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO admin_roles (
                    user_id, role, granted_by_user_id, created_at, updated_at
                ) VALUES (?, 'admin', ?, ?, ?)
                ON CONFLICT (user_id) DO NOTHING"#,
                vec![
                    target_user_id.to_owned().into(),
                    granted_by_user_id.to_owned().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        let record = self
            .find_role(target_user_id)
            .await?
            .ok_or_else(|| DbErr::Custom("granted admin role not found".to_owned()))?;
        Ok(GrantAdminResult {
            record,
            created: result.rows_affected() > 0,
        })
    }

    /// Deletes a regular administrator role and leaves super administrators intact.
    pub async fn revoke_admin(&self, target_user_id: &str) -> Result<bool, DbErr> {
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "DELETE FROM admin_roles WHERE user_id = ? AND role = 'admin'",
                vec![target_user_id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

fn role_select_sql(suffix: &str) -> String {
    format!(
        "SELECT user_id, role, granted_by_user_id, created_at, updated_at FROM admin_roles {suffix}"
    )
}

fn map_admin_role(row: &sea_orm::QueryResult) -> Result<AdminRoleRecord, DbErr> {
    let role_key: String = row.try_get("", "role")?;
    let role = AdminRole::from_key(&role_key)
        .ok_or_else(|| DbErr::Custom(format!("unknown admin role `{role_key}`")))?;
    Ok(AdminRoleRecord {
        user_id: row.try_get("", "user_id")?,
        role,
        granted_by_user_id: row.try_get("", "granted_by_user_id")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_target_user(row: &sea_orm::QueryResult) -> Result<AdminTargetUser, DbErr> {
    Ok(AdminTargetUser {
        id: row.try_get("", "id")?,
        username: row.try_get("", "username")?,
    })
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .expect("RFC3339 timestamp formatting should be infallible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_migration::Migrator;

    use crate::repositories::AuthRepository;

    async fn test_repo() -> (AdminRoleRepository, AuthRepository) {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        (
            AdminRoleRepository::new(db.clone()),
            AuthRepository::new(db),
        )
    }

    #[tokio::test]
    async fn grants_and_revokes_regular_admin_without_downgrading_super_admin() {
        let (repo, auth_repo) = test_repo().await;
        let actor = auth_repo
            .create_password_user("owner", "owner@example.test", "hash")
            .await
            .unwrap();
        let target = auth_repo
            .create_password_user("target_admin", "target@example.test", "hash")
            .await
            .unwrap();

        let granted = repo.grant_admin(&target.id, &actor.id).await.unwrap();
        assert!(granted.created);
        assert_eq!(granted.record.role, AdminRole::Admin);
        assert_eq!(
            granted.record.granted_by_user_id.as_deref(),
            Some(actor.id.as_str())
        );

        let repeated = repo.grant_admin(&target.id, &actor.id).await.unwrap();
        assert!(!repeated.created);
        assert_eq!(repeated.record.role, AdminRole::Admin);

        assert!(repo.revoke_admin(&target.id).await.unwrap());
        assert!(repo.find_role(&target.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn resolves_only_existing_non_deleted_targets() {
        let (repo, auth_repo) = test_repo().await;
        let target = auth_repo
            .create_password_user("lookup_admin", "lookup@example.test", "hash")
            .await
            .unwrap();

        assert_eq!(
            repo.find_target_user_by_username("lookup_admin")
                .await
                .unwrap()
                .unwrap()
                .id,
            target.id
        );
        assert!(
            repo.find_target_user_by_id("missing")
                .await
                .unwrap()
                .is_none()
        );
    }
}
