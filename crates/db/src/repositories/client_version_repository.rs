//! Client version repository storing public release notes and administrator-managed drafts.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, Value};
use stellartrail_domain::gear::now_rfc3339;
use uuid::Uuid;

use super::statement;

/// Persisted client version row.
#[derive(Clone, Debug)]
pub struct ClientVersionRecord {
    pub id: String,
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes_json: String,
    pub commit_hash: Option<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_by_user_id: Option<String>,
    pub updated_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Draft used for creating or replacing client version content.
#[derive(Clone, Debug)]
pub struct ClientVersionDraft {
    pub client_key: String,
    pub version: String,
    pub title: String,
    pub release_notes_json: String,
    pub commit_hash: Option<String>,
    pub status: String,
}

/// Administrator filters for version rows.
#[derive(Clone, Debug, Default)]
pub struct ListClientVersionsOptions {
    pub client_key: Option<String>,
    pub status: Option<String>,
    pub limit: u64,
    pub cursor: Option<String>,
}

/// Repository for client release versions.
#[derive(Clone)]
pub struct ClientVersionRepository {
    db: DatabaseConnection,
}

impl ClientVersionRepository {
    /// Creates a repository backed by the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Reads the latest published version for one client.
    pub async fn current_published(
        &self,
        client_key: &str,
    ) -> Result<Option<ClientVersionRecord>, DbErr> {
        self.db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE client_key = ? AND status = 'published' ORDER BY published_at DESC, created_at DESC, version DESC LIMIT 1",
                    select_columns()
                ),
                vec![client_key.to_owned().into()],
            ))
            .await?
            .map(|row| map_record(&row))
            .transpose()
    }

    /// Lists published versions for one client with offset cursors.
    pub async fn list_published(
        &self,
        client_key: &str,
        limit: u64,
        cursor: Option<&str>,
    ) -> Result<(Vec<ClientVersionRecord>, Option<String>), DbErr> {
        let limit = limit.clamp(1, 100);
        let offset = parse_cursor(cursor)?;
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE client_key = ? AND status = 'published' ORDER BY published_at DESC, created_at DESC, version DESC LIMIT ? OFFSET ?",
                    select_columns()
                ),
                vec![
                    client_key.to_owned().into(),
                    (limit as i64 + 1).into(),
                    offset.into(),
                ],
            ))
            .await?;
        paged_rows(rows, limit, offset)
    }

    /// Lists versions for administrators with optional filters.
    pub async fn list_admin(
        &self,
        options: &ListClientVersionsOptions,
    ) -> Result<(Vec<ClientVersionRecord>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = Vec::new();
        let mut filters = Vec::new();
        if let Some(client_key) = options.client_key.as_deref() {
            filters.push("client_key = ?");
            values.push(client_key.to_owned().into());
        }
        if let Some(status) = options.status.as_deref() {
            filters.push("status = ?");
            values.push(status.to_owned().into());
        }
        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        };
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    "{}{} ORDER BY updated_at DESC, created_at DESC, version DESC LIMIT ? OFFSET ?",
                    select_columns(),
                    where_clause
                ),
                values,
            ))
            .await?;
        paged_rows(rows, limit, offset)
    }

    /// Creates a client version.
    pub async fn create(
        &self,
        actor_user_id: &str,
        draft: &ClientVersionDraft,
    ) -> Result<ClientVersionRecord, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let published_at = if draft.status == "published" {
            Some(now.clone())
        } else {
            None
        };
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO client_versions (
                    id, client_key, version, title, release_notes_json, commit_hash, status, published_at,
                    created_by_user_id, updated_by_user_id, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    draft.client_key.clone().into(),
                    draft.version.clone().into(),
                    draft.title.clone().into(),
                    draft.release_notes_json.clone().into(),
                    draft.commit_hash.clone().into(),
                    draft.status.clone().into(),
                    published_at.clone().into(),
                    actor_user_id.to_owned().into(),
                    actor_user_id.to_owned().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.get_by_id(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created client version not found".to_owned()))
    }

    /// Updates a client version in place.
    pub async fn update(
        &self,
        id: &str,
        actor_user_id: &str,
        draft: &ClientVersionDraft,
    ) -> Result<Option<ClientVersionRecord>, DbErr> {
        let Some(existing) = self.get_by_id(id).await? else {
            return Ok(None);
        };
        let now = now_rfc3339();
        let published_at = match (existing.status.as_str(), draft.status.as_str()) {
            ("published", "published") => {
                existing.published_at.clone().or_else(|| Some(now.clone()))
            }
            (_, "published") => Some(now.clone()),
            (_, _) => None,
        };
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE client_versions
                   SET client_key = ?, version = ?, title = ?, release_notes_json = ?,
                       commit_hash = ?, status = ?, published_at = ?, updated_by_user_id = ?, updated_at = ?
                   WHERE id = ?"#,
                vec![
                    draft.client_key.clone().into(),
                    draft.version.clone().into(),
                    draft.title.clone().into(),
                    draft.release_notes_json.clone().into(),
                    draft.commit_hash.clone().into(),
                    draft.status.clone().into(),
                    published_at.into(),
                    actor_user_id.to_owned().into(),
                    now.into(),
                    id.to_owned().into(),
                ],
            ))
            .await?;
        self.get_by_id(id).await
    }

    /// Reads one version by id.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<ClientVersionRecord>, DbErr> {
        self.db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("{} WHERE id = ? LIMIT 1", select_columns()),
                vec![id.to_owned().into()],
            ))
            .await?
            .map(|row| map_record(&row))
            .transpose()
    }

    /// Reads one version by client and version string.
    pub async fn get_by_client_version(
        &self,
        client_key: &str,
        version: &str,
    ) -> Result<Option<ClientVersionRecord>, DbErr> {
        self.db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE client_key = ? AND version = ? LIMIT 1",
                    select_columns()
                ),
                vec![client_key.to_owned().into(), version.to_owned().into()],
            ))
            .await?
            .map(|row| map_record(&row))
            .transpose()
    }
}

fn paged_rows(
    rows: Vec<QueryResult>,
    limit: u64,
    offset: i64,
) -> Result<(Vec<ClientVersionRecord>, Option<String>), DbErr> {
    let mut records = rows
        .iter()
        .map(map_record)
        .collect::<Result<Vec<_>, DbErr>>()?;
    let next_cursor = if records.len() > limit as usize {
        records.truncate(limit as usize);
        Some((offset + limit as i64).to_string())
    } else {
        None
    };
    Ok((records, next_cursor))
}

fn select_columns() -> &'static str {
    r#"SELECT id, client_key, version, title, release_notes_json, commit_hash, status, published_at,
        created_by_user_id, updated_by_user_id, created_at, updated_at
       FROM client_versions"#
}

fn map_record(row: &QueryResult) -> Result<ClientVersionRecord, DbErr> {
    Ok(ClientVersionRecord {
        id: row.try_get("", "id")?,
        client_key: row.try_get("", "client_key")?,
        version: row.try_get("", "version")?,
        title: row.try_get("", "title")?,
        release_notes_json: row.try_get("", "release_notes_json")?,
        commit_hash: row.try_get("", "commit_hash")?,
        status: row.try_get("", "status")?,
        published_at: row.try_get("", "published_at")?,
        created_by_user_id: row.try_get("", "created_by_user_id")?,
        updated_by_user_id: row.try_get("", "updated_by_user_id")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn parse_cursor(cursor: Option<&str>) -> Result<i64, DbErr> {
    cursor
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<i64>()
                .map_err(|_| DbErr::Custom("invalid client version cursor".to_owned()))
        })
        .transpose()
        .map(|value| value.unwrap_or(0).max(0))
}
