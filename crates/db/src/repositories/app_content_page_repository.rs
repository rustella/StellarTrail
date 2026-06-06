//! Repository for DB-backed application content pages.
//!
//! Content pages store small structured JSON documents used by clients for
//! copy that should be adjustable after deployment.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult};

use super::statement;

/// Persisted application content page row.
#[derive(Clone, Debug)]
pub struct AppContentPageRecord {
    pub id: String,
    pub page_key: String,
    pub client_key: String,
    pub locale: String,
    pub content_json: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Repository for public and future administrator-facing app content pages.
#[derive(Clone)]
pub struct AppContentPageRepository {
    db: DatabaseConnection,
}

impl AppContentPageRepository {
    /// Creates a repository backed by the shared database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Reads one published page for a client and locale.
    pub async fn get_published(
        &self,
        page_key: &str,
        client_key: &str,
        locale: &str,
    ) -> Result<Option<AppContentPageRecord>, DbErr> {
        self.db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE page_key = ? AND client_key = ? AND locale = ? AND status = 'published' LIMIT 1",
                    select_columns()
                ),
                vec![
                    page_key.to_owned().into(),
                    client_key.to_owned().into(),
                    locale.to_owned().into(),
                ],
            ))
            .await?
            .map(|row| map_record(&row))
            .transpose()
    }
}

fn select_columns() -> &'static str {
    "SELECT id, page_key, client_key, locale, content_json, status, created_at, updated_at \
     FROM app_content_pages"
}

fn map_record(row: &QueryResult) -> Result<AppContentPageRecord, DbErr> {
    Ok(AppContentPageRecord {
        id: row.try_get("", "id")?,
        page_key: row.try_get("", "page_key")?,
        client_key: row.try_get("", "client_key")?,
        locale: row.try_get("", "locale")?,
        content_json: row.try_get("", "content_json")?,
        status: row.try_get("", "status")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}
