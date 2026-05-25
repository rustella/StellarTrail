//! Privacy-preserving API usage aggregation repository.
//!
//! This module writes daily counters keyed by route templates rather than raw
//! request paths. It never stores headers, tokens, query strings, request bodies,
//! IP addresses, or user agents. The API service supplies a trusted user id when
//! authentication has already succeeded; anonymous and failed-auth requests are
//! grouped under a non-identifying key.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};

use super::statement;

const ANONYMOUS_USER_KEY: &str = "anonymous";

/// One usage increment derived from an already-sanitized API usage event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiUsageIncrement {
    pub bucket_date: String,
    pub user_id: Option<String>,
    pub method: String,
    pub route_pattern: String,
    pub status_code: i32,
    pub occurred_at: String,
    pub call_count: i64,
}

/// Query filters for the administrator API usage list endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiUsageQuery {
    pub from_date: String,
    pub to_date: String,
    pub user_id: Option<String>,
    pub method: Option<String>,
    pub route_pattern: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Aggregated API usage row returned to administrator queries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiUsageRecord {
    pub bucket_date: String,
    pub user_id: Option<String>,
    pub method: String,
    pub route_pattern: String,
    pub status_code: i32,
    pub call_count: i64,
    pub first_called_at: String,
    pub last_called_at: String,
}

/// Repository for daily API usage counters.
#[derive(Clone)]
pub struct ApiUsageRepository {
    db: DatabaseConnection,
}

impl ApiUsageRepository {
    /// Creates a repository bound to the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Increments one daily aggregate row using only sanitized route metadata.
    ///
    /// The conflict key deliberately uses `user_key` instead of `user_id` because
    /// SQL composite primary keys allow multiple `NULL` values in some engines.
    /// `user_id` remains nullable in the row for administrator responses, while
    /// `user_key` is the stable aggregation key for both authenticated and
    /// anonymous traffic.
    pub async fn record_increment(&self, increment: &ApiUsageIncrement) -> Result<(), DbErr> {
        let user_key = user_key(increment.user_id.as_deref());
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO api_usage_daily (
                    bucket_date, user_key, user_id, method, route_pattern, status_code,
                    call_count, first_called_at, last_called_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT (bucket_date, user_key, method, route_pattern, status_code)
                DO UPDATE SET
                    call_count = api_usage_daily.call_count + excluded.call_count,
                    first_called_at = CASE
                        WHEN api_usage_daily.first_called_at <= excluded.first_called_at
                        THEN api_usage_daily.first_called_at
                        ELSE excluded.first_called_at
                    END,
                    last_called_at = CASE
                        WHEN api_usage_daily.last_called_at >= excluded.last_called_at
                        THEN api_usage_daily.last_called_at
                        ELSE excluded.last_called_at
                    END"#,
                vec![
                    increment.bucket_date.clone().into(),
                    user_key.into(),
                    increment.user_id.clone().into(),
                    increment.method.clone().into(),
                    increment.route_pattern.clone().into(),
                    i64::from(increment.status_code).into(),
                    increment.call_count.into(),
                    increment.occurred_at.clone().into(),
                    increment.occurred_at.clone().into(),
                ],
            ))
            .await?;
        Ok(())
    }

    /// Lists daily aggregates with bounded pagination for administrator queries.
    pub async fn list(&self, query: &ApiUsageQuery) -> Result<Vec<ApiUsageRecord>, DbErr> {
        let mut sql = String::from(
            r#"SELECT bucket_date, user_id, method, route_pattern, status_code,
                      SUM(call_count) AS call_count,
                      MIN(first_called_at) AS first_called_at,
                      MAX(last_called_at) AS last_called_at
               FROM api_usage_daily
               WHERE bucket_date >= ? AND bucket_date <= ?"#,
        );
        let mut values: Vec<Value> =
            vec![query.from_date.clone().into(), query.to_date.clone().into()];
        if let Some(user_id) = query.user_id.as_deref() {
            sql.push_str(" AND user_id = ?");
            values.push(user_id.to_owned().into());
        }
        if let Some(method) = query.method.as_deref() {
            sql.push_str(" AND method = ?");
            values.push(method.to_owned().into());
        }
        if let Some(route_pattern) = query.route_pattern.as_deref() {
            sql.push_str(" AND route_pattern = ?");
            values.push(route_pattern.to_owned().into());
        }
        sql.push_str(
            r#" GROUP BY bucket_date, user_id, method, route_pattern, status_code
                ORDER BY bucket_date DESC, call_count DESC, route_pattern ASC, status_code ASC
                LIMIT ? OFFSET ?"#,
        );
        values.push(query.limit.into());
        values.push(query.offset.into());

        let rows = self
            .db
            .query_all(statement(self.db.get_database_backend(), sql, values))
            .await?;
        rows.iter().map(map_api_usage_record).collect()
    }
}

fn user_key(user_id: Option<&str>) -> String {
    user_id
        .map(|id| format!("user:{id}"))
        .unwrap_or_else(|| ANONYMOUS_USER_KEY.to_owned())
}

fn map_api_usage_record(row: &sea_orm::QueryResult) -> Result<ApiUsageRecord, DbErr> {
    let status_code: i64 = row.try_get("", "status_code")?;
    Ok(ApiUsageRecord {
        bucket_date: row.try_get("", "bucket_date")?,
        user_id: row.try_get("", "user_id")?,
        method: row.try_get("", "method")?,
        route_pattern: row.try_get("", "route_pattern")?,
        status_code: i32::try_from(status_code)
            .map_err(|error| DbErr::Custom(error.to_string()))?,
        call_count: row.try_get("", "call_count")?,
        first_called_at: row.try_get("", "first_called_at")?,
        last_called_at: row.try_get("", "last_called_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_migration::Migrator;

    use crate::repositories::AuthRepository;

    async fn test_repo() -> (ApiUsageRepository, AuthRepository) {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        (ApiUsageRepository::new(db.clone()), AuthRepository::new(db))
    }

    fn increment(
        user_id: Option<String>,
        route_pattern: &str,
        status_code: i32,
    ) -> ApiUsageIncrement {
        ApiUsageIncrement {
            bucket_date: "2026-05-18".to_owned(),
            user_id,
            method: "GET".to_owned(),
            route_pattern: route_pattern.to_owned(),
            status_code,
            occurred_at: "2026-05-18T12:34:56Z".to_owned(),
            call_count: 1,
        }
    }

    #[tokio::test]
    async fn record_increment_aggregates_by_user_route_method_and_status() {
        let (usage_repo, auth_repo) = test_repo().await;
        let user = auth_repo
            .upsert_mock_user("mock:usage-repo", Some("Usage".to_owned()), None)
            .await
            .unwrap();

        usage_repo
            .record_increment(&increment(
                Some(user.id.clone()),
                "/api/v1/me/gears/:id",
                200,
            ))
            .await
            .unwrap();
        usage_repo
            .record_increment(&ApiUsageIncrement {
                occurred_at: "2026-05-18T12:35:56Z".to_owned(),
                ..increment(Some(user.id.clone()), "/api/v1/me/gears/:id", 200)
            })
            .await
            .unwrap();

        let rows = usage_repo
            .list(&ApiUsageQuery {
                from_date: "2026-05-18".to_owned(),
                to_date: "2026-05-18".to_owned(),
                user_id: Some(user.id.clone()),
                method: Some("GET".to_owned()),
                route_pattern: Some("/api/v1/me/gears/:id".to_owned()),
                limit: 50,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].user_id.as_deref(), Some(user.id.as_str()));
        assert_eq!(rows[0].route_pattern, "/api/v1/me/gears/:id");
        assert_eq!(rows[0].status_code, 200);
        assert_eq!(rows[0].call_count, 2);
        assert_eq!(rows[0].first_called_at, "2026-05-18T12:34:56Z");
        assert_eq!(rows[0].last_called_at, "2026-05-18T12:35:56Z");
    }

    #[tokio::test]
    async fn anonymous_traffic_is_grouped_without_user_id() {
        let (usage_repo, _auth_repo) = test_repo().await;
        usage_repo
            .record_increment(&increment(None, "/api/v1/skills/knots/detail/:id", 404))
            .await
            .unwrap();

        let rows = usage_repo
            .list(&ApiUsageQuery {
                from_date: "2026-05-18".to_owned(),
                to_date: "2026-05-18".to_owned(),
                user_id: None,
                method: None,
                route_pattern: Some("/api/v1/skills/knots/detail/:id".to_owned()),
                limit: 50,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert!(rows[0].user_id.is_none());
        assert_eq!(rows[0].call_count, 1);
        assert_eq!(rows[0].status_code, 404);
    }

    #[tokio::test]
    async fn list_filters_by_user_method_route_and_date_range() {
        let (usage_repo, auth_repo) = test_repo().await;
        let user_a = auth_repo
            .upsert_mock_user("mock:usage-a", Some("A".to_owned()), None)
            .await
            .unwrap();
        let user_b = auth_repo
            .upsert_mock_user("mock:usage-b", Some("B".to_owned()), None)
            .await
            .unwrap();
        usage_repo
            .record_increment(&increment(Some(user_a.id.clone()), "/api/v1/me/gears", 200))
            .await
            .unwrap();
        usage_repo
            .record_increment(&increment(Some(user_b.id), "/api/v1/me/gears", 200))
            .await
            .unwrap();
        usage_repo
            .record_increment(&ApiUsageIncrement {
                bucket_date: "2026-05-17".to_owned(),
                method: "POST".to_owned(),
                ..increment(Some(user_a.id.clone()), "/api/v1/me/gears", 201)
            })
            .await
            .unwrap();

        let rows = usage_repo
            .list(&ApiUsageQuery {
                from_date: "2026-05-18".to_owned(),
                to_date: "2026-05-18".to_owned(),
                user_id: Some(user_a.id.clone()),
                method: Some("GET".to_owned()),
                route_pattern: Some("/api/v1/me/gears".to_owned()),
                limit: 50,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].user_id.as_deref(), Some(user_a.id.as_str()));
        assert_eq!(rows[0].method, "GET");
        assert_eq!(rows[0].status_code, 200);
    }
}
