//! Roadmap repository wrapping SQL for product planning items, votes, and subscriptions.
//!
//! The repository keeps Roadmap interaction state account-scoped and idempotent:
//! voting or subscribing twice returns the current active state, while cancelling
//! a missing interaction returns an inactive state for the same published item.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, Value};
use stellartrail_domain::{gear::now_rfc3339, roadmap::RoadmapItemDraft};
use uuid::Uuid;

use super::statement;

/// Persisted roadmap item row.
#[derive(Clone, Debug)]
pub struct RoadmapItemRecord {
    pub id: String,
    pub client_key: String,
    pub title: String,
    pub summary: String,
    pub details: Option<String>,
    pub category: String,
    pub status: String,
    pub priority: i32,
    pub sort_order: i32,
    pub is_published: bool,
    pub is_deleted: bool,
    pub published_at: Option<String>,
    pub created_by_user_id: Option<String>,
    pub updated_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Roadmap item plus aggregate counts and one user's active interaction state.
#[derive(Clone, Debug)]
pub struct RoadmapListEntry {
    pub item: RoadmapItemRecord,
    pub vote_count: u32,
    pub subscription_count: u32,
    pub is_voted: bool,
    pub is_subscribed: bool,
}

/// Query options shared by public, current-user, and administrator Roadmap lists.
#[derive(Clone, Debug, Default)]
pub struct ListRoadmapOptions {
    pub client_key: Option<String>,
    pub status: Option<String>,
    pub limit: u64,
    pub cursor: Option<String>,
}

/// Repository for Roadmap content and current-user interaction state.
#[derive(Clone)]
pub struct RoadmapRepository {
    db: DatabaseConnection,
}

impl RoadmapRepository {
    /// Creates a repository backed by the shared database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Lists published Roadmap items, optionally decorated with one user's state.
    pub async fn list_public(
        &self,
        options: &ListRoadmapOptions,
        user_id: Option<&str>,
    ) -> Result<(Vec<RoadmapListEntry>, Option<String>), DbErr> {
        self.list_entries(options, user_id, true).await
    }

    /// Lists active Roadmap items for administrator dashboards.
    pub async fn list_admin(
        &self,
        options: &ListRoadmapOptions,
    ) -> Result<(Vec<RoadmapListEntry>, Option<String>), DbErr> {
        self.list_entries(options, None, false).await
    }

    /// Creates a Roadmap item from an administrator-maintained draft.
    pub async fn create(
        &self,
        actor_user_id: &str,
        draft: &RoadmapItemDraft,
    ) -> Result<RoadmapListEntry, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let published_at = draft.is_published.then(|| now.clone());
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO roadmap_items (
                    id, client_key, title, summary, details, category, status, priority,
                    sort_order, is_published, is_deleted, published_at, created_by_user_id,
                    updated_by_user_id, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    draft.client_key.clone().into(),
                    draft.title.clone().into(),
                    draft.summary.clone().into(),
                    draft.details.clone().into(),
                    draft.category.clone().into(),
                    draft.status.clone().into(),
                    draft.priority.into(),
                    draft.sort_order.into(),
                    draft.is_published.into(),
                    published_at.into(),
                    actor_user_id.to_owned().into(),
                    actor_user_id.to_owned().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.entry_by_id(&id, None, false)
            .await?
            .ok_or_else(|| DbErr::Custom("created roadmap item not found".to_owned()))
    }

    /// Updates one active Roadmap item.
    pub async fn update(
        &self,
        id: &str,
        actor_user_id: &str,
        draft: &RoadmapItemDraft,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        let Some(existing) = self.entry_by_id(id, None, false).await? else {
            return Ok(None);
        };
        let now = now_rfc3339();
        let published_at = match (existing.item.is_published, draft.is_published) {
            (true, true) => existing
                .item
                .published_at
                .clone()
                .or_else(|| Some(now.clone())),
            (false, true) => Some(now.clone()),
            (_, false) => None,
        };
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE roadmap_items
                   SET client_key = ?, title = ?, summary = ?, details = ?, category = ?,
                       status = ?, priority = ?, sort_order = ?, is_published = ?,
                       published_at = ?, updated_by_user_id = ?, updated_at = ?
                   WHERE id = ? AND is_deleted = FALSE"#,
                vec![
                    draft.client_key.clone().into(),
                    draft.title.clone().into(),
                    draft.summary.clone().into(),
                    draft.details.clone().into(),
                    draft.category.clone().into(),
                    draft.status.clone().into(),
                    draft.priority.into(),
                    draft.sort_order.into(),
                    draft.is_published.into(),
                    published_at.into(),
                    actor_user_id.to_owned().into(),
                    now.into(),
                    id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.entry_by_id(id, None, false).await
        }
    }

    /// Soft-deletes an active Roadmap item.
    pub async fn soft_delete(&self, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE roadmap_items SET is_deleted = TRUE, updated_at = ? \
                 WHERE id = ? AND is_deleted = FALSE",
                vec![now.into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Idempotently records a current user's vote for a published Roadmap item.
    pub async fn vote(
        &self,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        if self
            .entry_by_id(roadmap_item_id, Some(user_id), true)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        if self
            .stored_interaction("user_roadmap_votes", user_id, roadmap_item_id)
            .await?
            .is_some_and(|active| active)
        {
            return self.entry_by_id(roadmap_item_id, Some(user_id), true).await;
        }
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO user_roadmap_votes (
                    user_id, roadmap_item_id, is_deleted, voted_at, created_at, updated_at
                ) VALUES (
                    ?, ?, FALSE, ?, ?, ?
                ) ON CONFLICT(user_id, roadmap_item_id) DO UPDATE SET
                    is_deleted = FALSE,
                    voted_at = excluded.voted_at,
                    updated_at = excluded.updated_at"#,
                vec![
                    user_id.to_owned().into(),
                    roadmap_item_id.to_owned().into(),
                    now.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.entry_by_id(roadmap_item_id, Some(user_id), true).await
    }

    /// Idempotently removes a current user's vote for a published Roadmap item.
    pub async fn unvote(
        &self,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        if self
            .entry_by_id(roadmap_item_id, Some(user_id), true)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        self.soft_delete_interaction("user_roadmap_votes", user_id, roadmap_item_id)
            .await?;
        self.entry_by_id(roadmap_item_id, Some(user_id), true).await
    }

    /// Idempotently records a current user's subscription for a published Roadmap item.
    pub async fn subscribe(
        &self,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        if self
            .entry_by_id(roadmap_item_id, Some(user_id), true)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        if self
            .stored_interaction("user_roadmap_subscriptions", user_id, roadmap_item_id)
            .await?
            .is_some_and(|active| active)
        {
            return self.entry_by_id(roadmap_item_id, Some(user_id), true).await;
        }
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO user_roadmap_subscriptions (
                    user_id, roadmap_item_id, is_deleted, subscribed_at, created_at, updated_at
                ) VALUES (
                    ?, ?, FALSE, ?, ?, ?
                ) ON CONFLICT(user_id, roadmap_item_id) DO UPDATE SET
                    is_deleted = FALSE,
                    subscribed_at = excluded.subscribed_at,
                    updated_at = excluded.updated_at"#,
                vec![
                    user_id.to_owned().into(),
                    roadmap_item_id.to_owned().into(),
                    now.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.entry_by_id(roadmap_item_id, Some(user_id), true).await
    }

    /// Idempotently removes a current user's subscription for a published Roadmap item.
    pub async fn unsubscribe(
        &self,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        if self
            .entry_by_id(roadmap_item_id, Some(user_id), true)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        self.soft_delete_interaction("user_roadmap_subscriptions", user_id, roadmap_item_id)
            .await?;
        self.entry_by_id(roadmap_item_id, Some(user_id), true).await
    }

    async fn list_entries(
        &self,
        options: &ListRoadmapOptions,
        user_id: Option<&str>,
        public_only: bool,
    ) -> Result<(Vec<RoadmapListEntry>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values = interaction_values(user_id);
        let mut filters = Vec::new();
        if public_only {
            filters.push("r.is_published = TRUE");
        }
        filters.push("r.is_deleted = FALSE");
        if let Some(client_key) = options.client_key.as_deref() {
            filters.push("r.client_key = ?");
            values.push(client_key.to_owned().into());
        }
        if let Some(status) = options.status.as_deref() {
            filters.push("r.status = ?");
            values.push(status.to_owned().into());
        }
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE {} \
                     ORDER BY r.sort_order ASC, r.priority DESC, r.created_at ASC, r.id ASC \
                     LIMIT ? OFFSET ?",
                    roadmap_entry_select_sql(user_id.is_some()),
                    filters.join(" AND ")
                ),
                values,
            ))
            .await?;
        paged_rows(rows, limit, offset)
    }

    async fn entry_by_id(
        &self,
        id: &str,
        user_id: Option<&str>,
        public_only: bool,
    ) -> Result<Option<RoadmapListEntry>, DbErr> {
        let mut values = interaction_values(user_id);
        let mut filters = Vec::new();
        if public_only {
            filters.push("r.is_published = TRUE");
        }
        filters.push("r.is_deleted = FALSE");
        filters.push("r.id = ?");
        values.push(id.to_owned().into());
        self.db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE {} LIMIT 1",
                    roadmap_entry_select_sql(user_id.is_some()),
                    filters.join(" AND ")
                ),
                values,
            ))
            .await?
            .as_ref()
            .map(map_entry)
            .transpose()
    }

    async fn stored_interaction(
        &self,
        table: &str,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<Option<bool>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT is_deleted FROM {table} \
                     WHERE user_id = ? AND roadmap_item_id = ? LIMIT 1"
                ),
                vec![user_id.to_owned().into(), roadmap_item_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| {
            row.try_get::<bool>("", "is_deleted")
                .map(|deleted| !deleted)
        })
        .transpose()
    }

    async fn soft_delete_interaction(
        &self,
        table: &str,
        user_id: &str,
        roadmap_item_id: &str,
    ) -> Result<(), DbErr> {
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                format!(
                    "UPDATE {table} SET is_deleted = TRUE, updated_at = ? \
                     WHERE user_id = ? AND roadmap_item_id = ?"
                ),
                vec![
                    now.into(),
                    user_id.to_owned().into(),
                    roadmap_item_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(())
    }
}

fn roadmap_entry_select_sql(with_user: bool) -> String {
    let vote_state_select = if with_user {
        "CASE WHEN uv.user_id IS NULL THEN FALSE ELSE TRUE END AS is_voted"
    } else {
        "FALSE AS is_voted"
    };
    let subscription_state_select = if with_user {
        "CASE WHEN us.user_id IS NULL THEN FALSE ELSE TRUE END AS is_subscribed"
    } else {
        "FALSE AS is_subscribed"
    };
    let user_joins = if with_user {
        "LEFT JOIN user_roadmap_votes uv \
            ON uv.roadmap_item_id = r.id AND uv.user_id = ? AND uv.is_deleted = FALSE \
         LEFT JOIN user_roadmap_subscriptions us \
            ON us.roadmap_item_id = r.id AND us.user_id = ? AND us.is_deleted = FALSE"
    } else {
        ""
    };
    format!(
        r#"SELECT
            r.id, r.client_key, r.title, r.summary, r.details, r.category, r.status,
            r.priority, r.sort_order, r.is_published, r.is_deleted, r.published_at,
            r.created_by_user_id, r.updated_by_user_id, r.created_at, r.updated_at,
            CAST(COALESCE(v.vote_count, 0) AS BIGINT) AS vote_count,
            CAST(COALESCE(s.subscription_count, 0) AS BIGINT) AS subscription_count,
            {vote_state_select},
            {subscription_state_select}
           FROM roadmap_items r
           LEFT JOIN (
                SELECT roadmap_item_id, COUNT(*) AS vote_count
                FROM user_roadmap_votes
                WHERE is_deleted = FALSE
                GROUP BY roadmap_item_id
           ) v ON v.roadmap_item_id = r.id
           LEFT JOIN (
                SELECT roadmap_item_id, COUNT(*) AS subscription_count
                FROM user_roadmap_subscriptions
                WHERE is_deleted = FALSE
                GROUP BY roadmap_item_id
           ) s ON s.roadmap_item_id = r.id
           {user_joins}"#
    )
}

fn interaction_values(user_id: Option<&str>) -> Vec<Value> {
    match user_id {
        Some(user_id) => vec![user_id.to_owned().into(), user_id.to_owned().into()],
        None => Vec::new(),
    }
}

fn paged_rows(
    rows: Vec<QueryResult>,
    limit: u64,
    offset: i64,
) -> Result<(Vec<RoadmapListEntry>, Option<String>), DbErr> {
    let mut entries = rows.iter().map(map_entry).collect::<Result<Vec<_>, _>>()?;
    let next_cursor = if entries.len() > limit as usize {
        entries.truncate(limit as usize);
        Some((offset + limit as i64).to_string())
    } else {
        None
    };
    Ok((entries, next_cursor))
}

fn map_entry(row: &QueryResult) -> Result<RoadmapListEntry, DbErr> {
    let vote_count = row.try_get::<i64>("", "vote_count")?.max(0) as u32;
    let subscription_count = row.try_get::<i64>("", "subscription_count")?.max(0) as u32;
    Ok(RoadmapListEntry {
        item: RoadmapItemRecord {
            id: row.try_get("", "id")?,
            client_key: row.try_get("", "client_key")?,
            title: row.try_get("", "title")?,
            summary: row.try_get("", "summary")?,
            details: row.try_get("", "details")?,
            category: row.try_get("", "category")?,
            status: row.try_get("", "status")?,
            priority: row.try_get::<i32>("", "priority")?,
            sort_order: row.try_get::<i32>("", "sort_order")?,
            is_published: row.try_get("", "is_published")?,
            is_deleted: row.try_get("", "is_deleted")?,
            published_at: row.try_get("", "published_at")?,
            created_by_user_id: row.try_get("", "created_by_user_id")?,
            updated_by_user_id: row.try_get("", "updated_by_user_id")?,
            created_at: row.try_get("", "created_at")?,
            updated_at: row.try_get("", "updated_at")?,
        },
        vote_count,
        subscription_count,
        is_voted: row.try_get("", "is_voted")?,
        is_subscribed: row.try_get("", "is_subscribed")?,
    })
}

fn parse_cursor(cursor: Option<&str>) -> Result<i64, DbErr> {
    cursor
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<i64>()
                .map_err(|_| DbErr::Custom("invalid roadmap cursor".to_owned()))
        })
        .transpose()
        .map(|value| value.unwrap_or(0).max(0))
}
