//! Account-scoped disclaimer acceptance persistence.
//!
//! Legal and safety disclaimer acceptances are stored per user, disclaimer key,
//! and version. The repository keeps the SQL boundary isolated from API routes
//! so future disclaimer families can reuse the same table.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};
use stellartrail_domain::gear::now_rfc3339;

use super::statement;

/// Stored disclaimer acceptance row.
#[derive(Clone, Debug)]
pub struct DisclaimerAcceptanceRecord {
    pub user_id: String,
    pub disclaimer_key: String,
    pub version: String,
    pub title: String,
    pub content: String,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
    pub accepted_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Normalized disclaimer acceptance input.
#[derive(Clone, Debug)]
pub struct DisclaimerAcceptanceDraft {
    pub disclaimer_key: String,
    pub version: String,
    pub title: String,
    pub content: String,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
}

/// Persistence object for account-scoped disclaimer acceptances.
#[derive(Clone)]
pub struct DisclaimerAcceptanceRepository {
    db: DatabaseConnection,
}

impl DisclaimerAcceptanceRepository {
    /// Creates a repository using the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Reads the current user's acceptance for one disclaimer version.
    pub async fn get(
        &self,
        user_id: &str,
        disclaimer_key: &str,
        version: &str,
    ) -> Result<Option<DisclaimerAcceptanceRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                acceptance_select_sql(
                    "WHERE user_id = ? AND disclaimer_key = ? AND version = ? LIMIT 1",
                ),
                vec![
                    user_id.to_owned().into(),
                    disclaimer_key.to_owned().into(),
                    version.to_owned().into(),
                ],
            ))
            .await?;
        row.map(|row| map_acceptance(&row)).transpose()
    }

    /// Idempotently records or refreshes one user's disclaimer acceptance.
    pub async fn accept(
        &self,
        user_id: &str,
        draft: &DisclaimerAcceptanceDraft,
    ) -> Result<DisclaimerAcceptanceRecord, DbErr> {
        let now = now_rfc3339();
        if self
            .get(user_id, &draft.disclaimer_key, &draft.version)
            .await?
            .is_some()
        {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    r#"UPDATE user_disclaimer_acceptances
                       SET title = ?, content = ?, client_platform = ?,
                           client_version = ?, device_model = ?, updated_at = ?
                       WHERE user_id = ? AND disclaimer_key = ? AND version = ?"#,
                    vec![
                        draft.title.clone().into(),
                        draft.content.clone().into(),
                        draft.client_platform.clone().into(),
                        draft.client_version.clone().into(),
                        draft.device_model.clone().into(),
                        now.clone().into(),
                        user_id.to_owned().into(),
                        draft.disclaimer_key.clone().into(),
                        draft.version.clone().into(),
                    ],
                ))
                .await?;
        } else {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    r#"INSERT INTO user_disclaimer_acceptances (
                        user_id, disclaimer_key, version, title, content,
                        client_platform, client_version, device_model,
                        accepted_at, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                    vec![
                        user_id.to_owned().into(),
                        draft.disclaimer_key.clone().into(),
                        draft.version.clone().into(),
                        draft.title.clone().into(),
                        draft.content.clone().into(),
                        draft.client_platform.clone().into(),
                        draft.client_version.clone().into(),
                        draft.device_model.clone().into(),
                        now.clone().into(),
                        now.clone().into(),
                        now.into(),
                    ],
                ))
                .await?;
        }

        self.get(user_id, &draft.disclaimer_key, &draft.version)
            .await?
            .ok_or_else(|| DbErr::Custom("accepted disclaimer not found after upsert".to_owned()))
    }
}

fn acceptance_select_sql(where_clause: &str) -> String {
    format!(
        "SELECT user_id, disclaimer_key, version, title, content, client_platform, \
         client_version, device_model, accepted_at, created_at, updated_at \
         FROM user_disclaimer_acceptances {where_clause}"
    )
}

fn map_acceptance(row: &sea_orm::QueryResult) -> Result<DisclaimerAcceptanceRecord, DbErr> {
    Ok(DisclaimerAcceptanceRecord {
        user_id: row.try_get("", "user_id")?,
        disclaimer_key: row.try_get("", "disclaimer_key")?,
        version: row.try_get("", "version")?,
        title: row.try_get("", "title")?,
        content: row.try_get("", "content")?,
        client_platform: row.try_get("", "client_platform")?,
        client_version: row.try_get("", "client_version")?,
        device_model: row.try_get("", "device_model")?,
        accepted_at: row.try_get("", "accepted_at")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, Statement};

    async fn test_repository() -> DisclaimerAcceptanceRepository {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"CREATE TABLE user_disclaimer_acceptances (
                user_id TEXT NOT NULL,
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
        ))
        .await
        .unwrap();
        DisclaimerAcceptanceRepository::new(db)
    }

    fn draft(client_version: &str) -> DisclaimerAcceptanceDraft {
        DisclaimerAcceptanceDraft {
            disclaimer_key: "knot_tutorial".to_owned(),
            version: "v1".to_owned(),
            title: "绳结免责声明".to_owned(),
            content: "仅供参考".to_owned(),
            client_platform: Some("wechat_miniprogram".to_owned()),
            client_version: Some(client_version.to_owned()),
            device_model: Some("iPhone".to_owned()),
        }
    }

    #[tokio::test]
    async fn accept_creates_and_updates_without_duplicates() {
        let repo = test_repository().await;

        let first = repo.accept("user-1", &draft("1.0.0")).await.unwrap();
        assert_eq!(first.user_id, "user-1");
        assert_eq!(first.client_version.as_deref(), Some("1.0.0"));

        let second = repo.accept("user-1", &draft("1.0.1")).await.unwrap();
        assert_eq!(second.client_version.as_deref(), Some("1.0.1"));
        assert_eq!(second.accepted_at, first.accepted_at);

        let stored = repo
            .get("user-1", "knot_tutorial", "v1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.client_version.as_deref(), Some("1.0.1"));
    }
}
