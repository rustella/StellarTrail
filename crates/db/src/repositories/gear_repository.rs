//! Gear repository wrapping SQL for user gear CRUD, filtered pagination, statistics, archive, and restore operations.

use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};

use super::statement;
use stellartrail_domain::gear::{
    GearCategory, GearCategoryCount, GearDraft, GearItem, GearShareStatus, GearSort, GearSpecs,
    GearStats, GearStatus, GearStatusCount, GearTab, now_rfc3339,
};
use uuid::Uuid;

/// Stable data boundary for `ListGearOptions`, exposed by or reused within this module.
#[derive(Clone, Debug)]
pub struct ListGearOptions {
    pub tab: GearTab,
    pub category: Option<GearCategory>,
    pub status: Option<GearStatus>,
    pub q: Option<String>,
    pub sort: GearSort,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearOptions {
    /// Runs the `default` server-side flow while preserving input validation, error propagation, and state invariants.
    fn default() -> Self {
        Self {
            tab: GearTab::Available,
            category: None,
            status: None,
            q: None,
            sort: GearSort::CreatedAtDesc,
            limit: 20,
            cursor: None,
        }
    }
}

/// Gear persistence object that centralizes SQL for user gear reads, writes, statistics, import, and export.
#[derive(Clone)]
pub struct GearRepository {
    db: DatabaseConnection,
}

impl GearRepository {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates the current resource and triggers follow-up state maintenance when needed.
    pub async fn create(&self, user_id: &str, draft: &GearDraft) -> Result<GearItem, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let tags_json =
            serde_json::to_string(&draft.tags).map_err(|err| DbErr::Custom(err.to_string()))?;
        let specs_json = json_string(&draft.specs)?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO user_gear_items (
                    id, user_id, category, name, brand, model, description, weight_g,
                    official_price_cents, official_price_currency, purchase_date, purchase_price_cents,
                    purchase_price_currency, purchase_location, status, storage_location, specs_json,
                    tags_json, share_enabled, share_status, notes, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                gear_values(&id, user_id, draft, &specs_json, &tags_json, &now, &now),
            ))
            .await?;
        self.get(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created gear not found".to_owned()))
    }

    /// Runs the `get` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn get(&self, user_id: &str, id: &str) -> Result<Option<GearItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                gear_select_sql("WHERE user_id = ? AND id = ?"),
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_gear(&row)).transpose()
    }

    /// Runs the `replace` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn replace(
        &self,
        user_id: &str,
        id: &str,
        draft: &GearDraft,
    ) -> Result<Option<GearItem>, DbErr> {
        let now = now_rfc3339();
        let tags_json =
            serde_json::to_string(&draft.tags).map_err(|err| DbErr::Custom(err.to_string()))?;
        let specs_json = json_string(&draft.specs)?;
        let mut values = vec![
            draft.category.as_str().to_owned().into(),
            draft.name.clone().into(),
            draft.brand.clone().into(),
            draft.model.clone().into(),
            draft.description.clone().into(),
            draft.weight_g.into(),
            draft.official_price_cents.into(),
            draft.official_price_currency.clone().into(),
            draft.purchase_date.clone().into(),
            draft.purchase_price_cents.into(),
            draft.purchase_price_currency.clone().into(),
            draft.purchase_location.clone().into(),
            draft.status.as_str().to_owned().into(),
            draft.storage_location.clone().into(),
            specs_json.into(),
            tags_json.into(),
            draft.share_enabled.into(),
            draft.share_status.as_str().to_owned().into(),
            draft.notes.clone().into(),
            now.into(),
        ];
        values.push(user_id.to_owned().into());
        values.push(id.to_owned().into());
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE user_gear_items SET
                    category = ?, name = ?, brand = ?, model = ?, description = ?, weight_g = ?,
                    official_price_cents = ?, official_price_currency = ?, purchase_date = ?,
                    purchase_price_cents = ?, purchase_price_currency = ?, purchase_location = ?,
                    status = ?, storage_location = ?, specs_json = ?, tags_json = ?, share_enabled = ?,
                    share_status = ?, notes = ?, updated_at = ?
                   WHERE user_id = ? AND id = ?"#,
                values,
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get(user_id, id).await
        }
    }

    /// Queries gear by user, tab, category, status, search term, and sort order, using limit+1 to detect the next page.
    pub async fn list(
        &self,
        user_id: &str,
        options: &ListGearOptions,
    ) -> Result<(Vec<GearItem>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = vec![user_id.to_owned().into()];
        // user_id is always the first filter, keeping every gear query scoped to the current user.
        let mut clauses = vec!["user_id = ?".to_owned()];
        match options.tab {
            GearTab::Available => clauses.push("archived_at IS NULL".to_owned()),
            GearTab::History => clauses.push("archived_at IS NOT NULL".to_owned()),
        }
        if let Some(category) = options.category {
            clauses.push("category = ?".to_owned());
            values.push(category.as_str().to_owned().into());
        }
        if let Some(status) = options.status {
            clauses.push("status = ?".to_owned());
            values.push(status.as_str().to_owned().into());
        }
        // Search text is bound only as a LIKE parameter while SQL fragments stay on a fixed allowlist.
        if let Some(q) = normalize_query(options.q.as_deref()) {
            clauses.push("(LOWER(name) LIKE ? OR LOWER(COALESCE(brand, '')) LIKE ? OR LOWER(COALESCE(model, '')) LIKE ?)".to_owned());
            values.push(q.clone().into());
            values.push(q.clone().into());
            values.push(q.into());
        }
        // Fetch one extra row to detect whether another page exists, then truncate back to the requested limit before responding.
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        // Dynamic SQL concatenates only fragments generated from internal enums or allowlists; all user input is parameter-bound.
        let sql = format!(
            "{} WHERE {} {} LIMIT ? OFFSET ?",
            gear_select_columns(),
            clauses.join(" AND "),
            order_by(options.sort),
        );
        let mut items = self
            .db
            .query_all(statement(self.db.get_database_backend(), sql, values))
            .await?
            .into_iter()
            .map(|row| map_gear(&row))
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if items.len() > limit as usize {
            items.truncate(limit as usize);
            Some((offset + limit as i64).to_string())
        } else {
            None
        };
        Ok((items, next_cursor))
    }

    /// Archives the current resource so default lists no longer show it.
    pub async fn archive(&self, user_id: &str, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET archived_at = ?, updated_at = ? WHERE user_id = ? AND id = ? AND archived_at IS NULL",
                vec![now.clone().into(), now.into(), user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Restores an archived resource so default lists show it again.
    pub async fn restore(&self, user_id: &str, id: &str) -> Result<Option<GearItem>, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET archived_at = NULL, updated_at = ? WHERE user_id = ? AND id = ?",
                vec![now.into(), user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get(user_id, id).await
        }
    }

    /// Runs the `category counts` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn category_counts(
        &self,
        user_id: &str,
        tab: GearTab,
    ) -> Result<Vec<GearCategoryCount>, DbErr> {
        let archived_clause = archived_clause(tab);
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!("SELECT category, COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND {archived_clause} GROUP BY category"),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "category")?;
            let count: i64 = row.try_get("", "count")?;
            if let Some(category) = GearCategory::from_key(&raw) {
                counts.insert(category.as_str(), count);
            }
        }
        Ok(GearCategory::ALL
            .into_iter()
            .map(|category| GearCategoryCount {
                category,
                label: category.label().to_owned(),
                count: *counts.get(category.as_str()).unwrap_or(&0),
            })
            .collect())
    }

    /// Runs the `stats` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn stats(&self, user_id: &str, tab: GearTab) -> Result<GearStats, DbErr> {
        let archived_clause = archived_clause(tab);
        let summary = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT COUNT(*) AS count, \
                     CAST(COALESCE(SUM(CASE WHEN purchase_price_currency = 'CNY' THEN purchase_price_cents ELSE 0 END), 0) AS BIGINT) AS total_value_cents, \
                     CAST(COALESCE(SUM(weight_g), 0) AS BIGINT) AS total_weight_g \
                     FROM user_gear_items WHERE user_id = ? AND {archived_clause}"
                ),
                vec![user_id.to_owned().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("missing stats row".to_owned()))?;
        let archived_count = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND archived_at IS NOT NULL",
                vec![user_id.to_owned().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("missing archived stats row".to_owned()))?
            .try_get("", "count")?;
        let by_category = self.category_counts(user_id, tab).await?;
        let by_status = self.status_counts(user_id, tab).await?;
        Ok(GearStats {
            current_count: summary.try_get("", "count")?,
            archived_count,
            total_value_cents: summary.try_get("", "total_value_cents")?,
            total_weight_g: summary.try_get("", "total_weight_g")?,
            by_category,
            by_status,
        })
    }

    /// Runs the `status counts` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn status_counts(
        &self,
        user_id: &str,
        tab: GearTab,
    ) -> Result<Vec<GearStatusCount>, DbErr> {
        let archived_clause = archived_clause(tab);
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!("SELECT status, COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND {archived_clause} GROUP BY status"),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "status")?;
            let count: i64 = row.try_get("", "count")?;
            if let Some(status) = GearStatus::from_key(&raw) {
                counts.insert(status.as_str(), count);
            }
        }
        Ok(GearStatus::ALL
            .into_iter()
            .map(|status| GearStatusCount {
                status,
                label: status.label().to_owned(),
                count: *counts.get(status.as_str()).unwrap_or(&0),
            })
            .collect())
    }
}

/// Runs the `gear values` server-side flow while preserving input validation, error propagation, and state invariants.
fn gear_values(
    id: &str,
    user_id: &str,
    draft: &GearDraft,
    specs_json: &str,
    tags_json: &str,
    created_at: &str,
    updated_at: &str,
) -> Vec<Value> {
    vec![
        id.to_owned().into(),
        user_id.to_owned().into(),
        draft.category.as_str().to_owned().into(),
        draft.name.clone().into(),
        draft.brand.clone().into(),
        draft.model.clone().into(),
        draft.description.clone().into(),
        draft.weight_g.into(),
        draft.official_price_cents.into(),
        draft.official_price_currency.clone().into(),
        draft.purchase_date.clone().into(),
        draft.purchase_price_cents.into(),
        draft.purchase_price_currency.clone().into(),
        draft.purchase_location.clone().into(),
        draft.status.as_str().to_owned().into(),
        draft.storage_location.clone().into(),
        specs_json.to_owned().into(),
        tags_json.to_owned().into(),
        draft.share_enabled.into(),
        draft.share_status.as_str().to_owned().into(),
        draft.notes.clone().into(),
        created_at.to_owned().into(),
        updated_at.to_owned().into(),
    ]
}

/// Runs the `gear select sql` server-side flow while preserving input validation, error propagation, and state invariants.
fn gear_select_sql(where_clause: &str) -> String {
    format!("{} {where_clause} LIMIT 1", gear_select_columns())
}

/// Runs the `gear select columns` server-side flow while preserving input validation, error propagation, and state invariants.
fn gear_select_columns() -> &'static str {
    r#"SELECT id, user_id, category, name, brand, model, description,
        weight_g, official_price_cents, official_price_currency,
        purchase_date, purchase_price_cents, purchase_price_currency,
        purchase_location, status, storage_location, tags_json,
        specs_json, share_enabled, share_status, notes, archived_at, created_at, updated_at
       FROM user_gear_items"#
}

/// Runs the `order by` server-side flow while preserving input validation, error propagation, and state invariants.
fn order_by(sort: GearSort) -> &'static str {
    match sort {
        GearSort::CreatedAtDesc => "ORDER BY created_at DESC, id DESC",
        GearSort::CreatedAtAsc => "ORDER BY created_at ASC, id ASC",
        GearSort::PurchaseDateDesc => "ORDER BY purchase_date DESC, created_at DESC, id DESC",
        GearSort::NameAsc => "ORDER BY name ASC, created_at DESC, id DESC",
        GearSort::WeightDesc => "ORDER BY weight_g DESC, created_at DESC, id DESC",
        GearSort::PriceDesc => "ORDER BY purchase_price_cents DESC, created_at DESC, id DESC",
    }
}

/// Runs the `archived clause` server-side flow while preserving input validation, error propagation, and state invariants.
fn archived_clause(tab: GearTab) -> &'static str {
    match tab {
        GearTab::Available => "archived_at IS NULL",
        GearTab::History => "archived_at IS NOT NULL",
    }
}

/// Runs the `parse cursor` server-side flow while preserving input validation, error propagation, and state invariants.
fn parse_cursor(cursor: Option<&str>) -> Result<i64, DbErr> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    cursor
        .parse::<i64>()
        .map_err(|_| DbErr::Custom("invalid cursor".to_owned()))
        .and_then(|offset| {
            if offset >= 0 {
                Ok(offset)
            } else {
                Err(DbErr::Custom("invalid cursor".to_owned()))
            }
        })
}

/// Runs the `normalize query` server-side flow while preserving input validation, error propagation, and state invariants.
fn normalize_query(q: Option<&str>) -> Option<String> {
    let q = q?.trim().to_lowercase();
    if q.is_empty() {
        None
    } else {
        Some(format!("%{}%", q.replace('%', "\\%").replace('_', "\\_")))
    }
}

fn json_string(specs: &GearSpecs) -> Result<String, DbErr> {
    serde_json::to_string(specs).map_err(|err| DbErr::Custom(err.to_string()))
}

/// Runs the `map gear` server-side flow while preserving input validation, error propagation, and state invariants.
fn map_gear(row: &sea_orm::QueryResult) -> Result<GearItem, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let status_raw: String = row.try_get("", "status")?;
    let share_status_raw: String = row.try_get("", "share_status")?;
    let tags_json: String = row.try_get("", "tags_json")?;
    let specs_json: String = row.try_get("", "specs_json")?;
    // tags_json is a database text field; deserialization failures are converted to DbErr for consistent upstream handling.
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let specs: GearSpecs = serde_json::from_str(&specs_json).unwrap_or_default();
    Ok(GearItem {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        category: GearCategory::from_key(&category_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear category: {category_raw}")))?,
        name: row.try_get("", "name")?,
        brand: row.try_get("", "brand")?,
        model: row.try_get("", "model")?,
        description: row.try_get("", "description")?,
        weight_g: row.try_get("", "weight_g")?,
        official_price_cents: row.try_get("", "official_price_cents")?,
        official_price_currency: row.try_get("", "official_price_currency")?,
        purchase_date: row.try_get("", "purchase_date")?,
        purchase_price_cents: row.try_get("", "purchase_price_cents")?,
        purchase_price_currency: row.try_get("", "purchase_price_currency")?,
        purchase_location: row.try_get("", "purchase_location")?,
        status: GearStatus::from_key(&status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear status: {status_raw}")))?,
        storage_location: row.try_get("", "storage_location")?,
        specs,
        tags,
        share_enabled: row.try_get("", "share_enabled")?,
        share_status: GearShareStatus::from_key(&share_status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid share status: {share_status_raw}")))?,
        notes: row.try_get("", "notes")?,
        archived_at: row.try_get("", "archived_at")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::AuthRepository;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_migration::Migrator;

    /// Runs the `draft` server-side flow while preserving input validation, error propagation, and state invariants.
    fn draft(name: &str, category: GearCategory) -> GearDraft {
        GearDraft {
            category,
            name: name.to_owned(),
            brand: Some("NITECORE".to_owned()),
            model: Some("SUMMIT 20000".to_owned()),
            description: None,
            weight_g: Some(315),
            official_price_cents: Some(69900),
            official_price_currency: Some("CNY".to_owned()),
            purchase_date: Some("2026-01-22".to_owned()),
            purchase_price_cents: Some(63900),
            purchase_price_currency: Some("CNY".to_owned()),
            purchase_location: None,
            status: GearStatus::Available,
            storage_location: Some("装备柜".to_owned()),
            specs: GearSpecs::from([("battery_capacity".to_owned(), "20000 mAh".to_owned())]),
            tags: vec!["电子".to_owned()],
            share_enabled: false,
            share_status: GearShareStatus::NotShared,
            notes: None,
        }
    }

    /// Runs the `gear repository crud filter stats archive restore` server-side flow while preserving input validation, error propagation, and state invariants.
    #[tokio::test]
    async fn gear_repository_crud_filter_stats_archive_restore() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let auth_repo = AuthRepository::new(db.clone());
        let user = auth_repo
            .upsert_mock_user("mock:gear-repo-user", Some("装备测试用户".to_owned()), None)
            .await
            .unwrap();
        let repo = GearRepository::new(db.clone());
        let user_id = user.id.as_str();
        let created = repo
            .create(
                user_id,
                &draft("NITECORE充电宝", GearCategory::ElectronicsSystem),
            )
            .await
            .unwrap();
        repo.create(user_id, &draft("挪客户灯", GearCategory::LightingSystem))
            .await
            .unwrap();

        let options = ListGearOptions {
            category: Some(GearCategory::ElectronicsSystem),
            q: Some("nitecore".to_owned()),
            ..Default::default()
        };
        let (items, _) = repo.list(user_id, &options).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, created.id);

        let stats = repo.stats(user_id, GearTab::Available).await.unwrap();
        assert_eq!(stats.current_count, 2);
        assert_eq!(stats.total_weight_g, 630);
        assert_eq!(stats.total_value_cents, 127800);

        assert!(repo.archive(user_id, &created.id).await.unwrap());
        let (available, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(available.len(), 1);
        let (history, _) = repo
            .list(
                user_id,
                &ListGearOptions {
                    tab: GearTab::History,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(history.len(), 1);
        repo.restore(user_id, &created.id).await.unwrap().unwrap();
        let (available, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(available.len(), 2);
    }
}
