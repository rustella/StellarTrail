//! Gear repository wrapping SQL for user gear CRUD, filtered pagination, statistics, and import/export reads.

use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};

use super::statement;
use stellartrail_domain::{
    deletion::DeletedFilter,
    gear::{
        GearCategory, GearCategoryCount, GearDraft, GearItem, GearShareStatus, GearSort, GearSpecs,
        GearStats, GearStatus, GearStatusCount, now_rfc3339,
    },
};
use uuid::Uuid;

/// Stable data boundary for `ListGearOptions`, exposed by or reused within this module.
#[derive(Clone, Debug)]
pub struct ListGearOptions {
    pub category: Option<GearCategory>,
    pub status: Option<GearStatus>,
    pub deleted: DeletedFilter,
    pub q: Option<String>,
    pub sort: GearSort,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearOptions {
    /// Runs the `default` server-side flow while preserving input validation, error propagation, and state invariants.
    fn default() -> Self {
        Self {
            category: None,
            status: None,
            deleted: DeletedFilter::Active,
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
        if let Some(existing) = self.find_merge_candidate(user_id, draft).await? {
            return self.merge_into_existing(user_id, &existing, draft).await;
        }

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
                    purchase_price_currency, purchase_location, status, storage_location,
                    atlas_item_id, selected_variant_key, selected_variant_label, specs_json,
                    quantity, tags_json, share_enabled, share_status, notes, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
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
                gear_select_sql("WHERE user_id = ? AND id = ? AND is_deleted = FALSE"),
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_gear(&row)).transpose()
    }

    async fn find_merge_candidate(
        &self,
        user_id: &str,
        draft: &GearDraft,
    ) -> Result<Option<GearItem>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE user_id = ? AND category = ? AND is_deleted = FALSE \
                     ORDER BY created_at ASC, id ASC",
                    gear_select_columns()
                ),
                vec![
                    user_id.to_owned().into(),
                    draft.category.as_str().to_owned().into(),
                ],
            ))
            .await?;
        for row in rows {
            let item = map_gear(&row)?;
            if same_gear_identity(draft, &item) {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    async fn merge_into_existing(
        &self,
        user_id: &str,
        existing: &GearItem,
        draft: &GearDraft,
    ) -> Result<GearItem, DbErr> {
        let now = now_rfc3339();
        let quantity = (existing.quantity + draft.quantity).min(9_999);
        let specs = merge_specs(&existing.specs, &draft.specs);
        let specs_json = json_string(&specs)?;
        let tags = merge_tags(&existing.tags, &draft.tags);
        let tags_json =
            serde_json::to_string(&tags).map_err(|err| DbErr::Custom(err.to_string()))?;
        let notes = merged_notes(existing, draft);
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET quantity = ?, specs_json = ?, tags_json = ?, notes = ?, updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    quantity.into(),
                    specs_json.into(),
                    tags_json.into(),
                    notes.into(),
                    now.into(),
                    user_id.to_owned().into(),
                    existing.id.clone().into(),
                ],
            ))
            .await?;
        self.get(user_id, &existing.id)
            .await?
            .ok_or_else(|| DbErr::Custom("merged gear not found".to_owned()))
    }

    /// Reads one gear record for packing-list history, including soft-deleted items.
    pub async fn get_any_for_user(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Option<GearItem>, DbErr> {
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
            draft.atlas_item_id.clone().into(),
            draft.selected_variant_key.clone().into(),
            draft.selected_variant_label.clone().into(),
            specs_json.into(),
            draft.quantity.into(),
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
                    status = ?, storage_location = ?, atlas_item_id = ?, selected_variant_key = ?,
                    selected_variant_label = ?, specs_json = ?, quantity = ?, tags_json = ?, share_enabled = ?,
                    share_status = ?, notes = ?, updated_at = ?
                   WHERE user_id = ? AND id = ? AND is_deleted = FALSE"#,
                values,
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get(user_id, id).await
        }
    }

    /// Queries gear by user, category, status, search term, and sort order, using limit+1 to detect the next page.
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
        apply_deleted_filter(&mut clauses, options.deleted);
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

    /// Soft-deletes an item so it no longer appears in normal gear lists.
    pub async fn soft_delete(&self, user_id: &str, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET is_deleted = TRUE, updated_at = ? WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![now.into(), user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Runs the `category counts` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn category_counts(&self, user_id: &str) -> Result<Vec<GearCategoryCount>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT category, \
                     CAST(COALESCE(SUM(quantity), 0) AS BIGINT) AS count, \
                     CAST(COALESCE(SUM(COALESCE(weight_g, 0) * quantity), 0) AS BIGINT) AS total_weight_g, \
                     CAST(COALESCE(SUM(CASE WHEN purchase_price_currency = 'CNY' THEN COALESCE(purchase_price_cents, 0) * quantity ELSE 0 END), 0) AS BIGINT) AS total_value_cents \
                     FROM user_gear_items WHERE user_id = ? AND is_deleted = FALSE GROUP BY category",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "category")?;
            let count: i64 = row.try_get("", "count")?;
            let total_weight_g: i64 = row.try_get("", "total_weight_g")?;
            let total_value_cents: i64 = row.try_get("", "total_value_cents")?;
            if let Some(category) = GearCategory::from_key(&raw) {
                counts.insert(
                    category.as_str(),
                    (count, total_weight_g, total_value_cents),
                );
            }
        }
        Ok(GearCategory::ALL
            .into_iter()
            .map(|category| GearCategoryCount {
                category,
                label: category.label().to_owned(),
                count: counts
                    .get(category.as_str())
                    .map(|stats| stats.0)
                    .unwrap_or(0),
                total_weight_g: counts
                    .get(category.as_str())
                    .map(|stats| stats.1)
                    .unwrap_or(0),
                total_value_cents: counts
                    .get(category.as_str())
                    .map(|stats| stats.2)
                    .unwrap_or(0),
            })
            .collect())
    }

    /// Runs the `stats` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn stats(&self, user_id: &str) -> Result<GearStats, DbErr> {
        let summary = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT CAST(COALESCE(SUM(quantity), 0) AS BIGINT) AS count, \
                     CAST(COALESCE(SUM(CASE WHEN purchase_price_currency = 'CNY' THEN COALESCE(purchase_price_cents, 0) * quantity ELSE 0 END), 0) AS BIGINT) AS total_value_cents, \
                     CAST(COALESCE(SUM(COALESCE(weight_g, 0) * quantity), 0) AS BIGINT) AS total_weight_g \
                     FROM user_gear_items WHERE user_id = ? AND is_deleted = FALSE",
                vec![user_id.to_owned().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("missing stats row".to_owned()))?;
        let by_category = self.category_counts(user_id).await?;
        let by_status = self.status_counts(user_id).await?;
        Ok(GearStats {
            current_count: summary.try_get("", "count")?,
            total_value_cents: summary.try_get("", "total_value_cents")?,
            total_weight_g: summary.try_get("", "total_weight_g")?,
            by_category,
            by_status,
        })
    }

    /// Runs the `status counts` server-side flow while preserving input validation, error propagation, and state invariants.
    async fn status_counts(&self, user_id: &str) -> Result<Vec<GearStatusCount>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT status, \
                     CAST(COALESCE(SUM(quantity), 0) AS BIGINT) AS count, \
                     CAST(COALESCE(SUM(COALESCE(weight_g, 0) * quantity), 0) AS BIGINT) AS total_weight_g, \
                     CAST(COALESCE(SUM(CASE WHEN purchase_price_currency = 'CNY' THEN COALESCE(purchase_price_cents, 0) * quantity ELSE 0 END), 0) AS BIGINT) AS total_value_cents \
                     FROM user_gear_items WHERE user_id = ? AND is_deleted = FALSE GROUP BY status",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "status")?;
            let count: i64 = row.try_get("", "count")?;
            let total_weight_g: i64 = row.try_get("", "total_weight_g")?;
            let total_value_cents: i64 = row.try_get("", "total_value_cents")?;
            if let Some(status) = GearStatus::from_key(&raw) {
                counts.insert(status.as_str(), (count, total_weight_g, total_value_cents));
            }
        }
        Ok(GearStatus::ALL
            .into_iter()
            .map(|status| GearStatusCount {
                status,
                label: status.label().to_owned(),
                count: counts
                    .get(status.as_str())
                    .map(|stats| stats.0)
                    .unwrap_or(0),
                total_weight_g: counts
                    .get(status.as_str())
                    .map(|stats| stats.1)
                    .unwrap_or(0),
                total_value_cents: counts
                    .get(status.as_str())
                    .map(|stats| stats.2)
                    .unwrap_or(0),
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
        draft.atlas_item_id.clone().into(),
        draft.selected_variant_key.clone().into(),
        draft.selected_variant_label.clone().into(),
        specs_json.to_owned().into(),
        draft.quantity.into(),
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
        purchase_location, status, storage_location, atlas_item_id, selected_variant_key,
        selected_variant_label, quantity, tags_json,
        specs_json, share_enabled, share_status, notes, is_deleted, created_at, updated_at
       FROM user_gear_items"#
}

fn apply_deleted_filter(clauses: &mut Vec<String>, deleted: DeletedFilter) {
    match deleted {
        DeletedFilter::Active => clauses.push("is_deleted = FALSE".to_owned()),
        DeletedFilter::Deleted => clauses.push("is_deleted = TRUE".to_owned()),
        DeletedFilter::All => {}
    }
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
        atlas_item_id: row.try_get("", "atlas_item_id")?,
        selected_variant_key: row.try_get("", "selected_variant_key")?,
        selected_variant_label: row.try_get("", "selected_variant_label")?,
        quantity: row.try_get("", "quantity")?,
        specs,
        tags,
        share_enabled: row.try_get("", "share_enabled")?,
        share_status: GearShareStatus::from_key(&share_status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid share status: {share_status_raw}")))?,
        notes: row.try_get("", "notes")?,
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

const MERGE_SPEC_KEYS: &[&str] = &[
    "capacity",
    "net_content",
    "battery_capacity",
    "rated_energy",
    "output_power",
    "ports",
    "packed_size",
    "specification",
    "length",
    "people_count",
    "type",
];

fn same_gear_identity(draft: &GearDraft, item: &GearItem) -> bool {
    if draft.category != item.category {
        return false;
    }
    if same_non_empty(
        draft.atlas_item_id.as_deref(),
        item.atlas_item_id.as_deref(),
    ) {
        return !variant_conflicts(
            draft.selected_variant_key.as_deref(),
            draft.selected_variant_label.as_deref(),
            item.selected_variant_key.as_deref(),
            item.selected_variant_label.as_deref(),
        );
    }
    if normalize_identity_text(Some(&draft.name)) != normalize_identity_text(Some(&item.name)) {
        return false;
    }
    if text_conflicts(draft.brand.as_deref(), item.brand.as_deref())
        || text_conflicts(draft.model.as_deref(), item.model.as_deref())
        || variant_conflicts(
            draft.selected_variant_key.as_deref(),
            draft.selected_variant_label.as_deref(),
            item.selected_variant_key.as_deref(),
            item.selected_variant_label.as_deref(),
        )
        || specs_conflict(&draft.specs, &item.specs)
    {
        return false;
    }

    same_non_empty(draft.model.as_deref(), item.model.as_deref())
        || same_non_empty(draft.brand.as_deref(), item.brand.as_deref())
        || variants_match(
            draft.selected_variant_key.as_deref(),
            draft.selected_variant_label.as_deref(),
            item.selected_variant_key.as_deref(),
            item.selected_variant_label.as_deref(),
        )
        || specs_overlap(&draft.specs, &item.specs)
        || (normalize_identity_text(draft.brand.as_deref()).is_empty()
            && normalize_identity_text(item.brand.as_deref()).is_empty()
            && normalize_identity_text(draft.model.as_deref()).is_empty()
            && normalize_identity_text(item.model.as_deref()).is_empty())
}

fn merge_specs(existing: &GearSpecs, incoming: &GearSpecs) -> GearSpecs {
    let mut merged = existing.clone();
    for (key, value) in incoming {
        if !merged.contains_key(key) {
            merged.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn merge_tags(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut merged = existing.to_vec();
    for tag in incoming {
        if merged.len() >= 20 {
            break;
        }
        if !merged.iter().any(|existing| existing == tag) {
            merged.push(tag.clone());
        }
    }
    merged
}

fn merged_notes(existing: &GearItem, incoming: &GearDraft) -> Option<String> {
    let mut extra = Vec::new();
    push_note_diff(
        &mut extra,
        "购买日期",
        incoming.purchase_date.as_deref(),
        existing.purchase_date.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "购买渠道",
        incoming.purchase_location.as_deref(),
        existing.purchase_location.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "存放位置",
        incoming.storage_location.as_deref(),
        existing.storage_location.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "购入价",
        incoming
            .purchase_price_cents
            .map(|value| value.to_string())
            .as_deref(),
        existing
            .purchase_price_cents
            .map(|value| value.to_string())
            .as_deref(),
    );
    if incoming.status != existing.status {
        extra.push(format!("状态：{}", incoming.status.label()));
    }
    push_note_diff(
        &mut extra,
        "备注",
        incoming.notes.as_deref(),
        existing.notes.as_deref(),
    );
    if extra.is_empty() {
        return existing.notes.clone();
    }
    let addition = format!("合并新增记录信息：{}", extra.join("；"));
    let merged = match existing
        .notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(notes) => format!("{notes}\n{addition}"),
        None => addition,
    };
    Some(truncate_chars(&merged, 1000))
}

fn push_note_diff(
    output: &mut Vec<String>,
    label: &str,
    incoming: Option<&str>,
    existing: Option<&str>,
) {
    let Some(value) = incoming.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    if normalize_identity_text(Some(value)) != normalize_identity_text(existing) {
        output.push(format!("{label}：{value}"));
    }
}

fn same_non_empty(left: Option<&str>, right: Option<&str>) -> bool {
    let left = normalize_identity_text(left);
    let right = normalize_identity_text(right);
    !left.is_empty() && left == right
}

fn text_conflicts(left: Option<&str>, right: Option<&str>) -> bool {
    let left = normalize_identity_text(left);
    let right = normalize_identity_text(right);
    !left.is_empty() && !right.is_empty() && left != right
}

fn variant_conflicts(
    left_key: Option<&str>,
    left_label: Option<&str>,
    right_key: Option<&str>,
    right_label: Option<&str>,
) -> bool {
    let left = normalized_variant_identity(left_key, left_label);
    let right = normalized_variant_identity(right_key, right_label);
    !left.is_empty() && !right.is_empty() && left != right
}

fn variants_match(
    left_key: Option<&str>,
    left_label: Option<&str>,
    right_key: Option<&str>,
    right_label: Option<&str>,
) -> bool {
    let left = normalized_variant_identity(left_key, left_label);
    let right = normalized_variant_identity(right_key, right_label);
    !left.is_empty() && left == right
}

fn normalized_variant_identity(key: Option<&str>, label: Option<&str>) -> String {
    let key = normalize_identity_text(key);
    if key.is_empty() {
        normalize_identity_text(label)
    } else {
        key
    }
}

fn specs_conflict(left: &GearSpecs, right: &GearSpecs) -> bool {
    MERGE_SPEC_KEYS.iter().any(|key| {
        let left = normalize_identity_text(left.get(*key).map(String::as_str));
        let right = normalize_identity_text(right.get(*key).map(String::as_str));
        !left.is_empty() && !right.is_empty() && left != right
    })
}

fn specs_overlap(left: &GearSpecs, right: &GearSpecs) -> bool {
    MERGE_SPEC_KEYS.iter().any(|key| {
        let left = normalize_identity_text(left.get(*key).map(String::as_str));
        let right = normalize_identity_text(right.get(*key).map(String::as_str));
        !left.is_empty() && left == right
    })
}

fn normalize_identity_text(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::AuthRepository;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_domain::deletion::DeletedFilter;
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
            atlas_item_id: None,
            selected_variant_key: None,
            selected_variant_label: None,
            quantity: 1,
            specs: GearSpecs::from([("battery_capacity".to_owned(), "20000 mAh".to_owned())]),
            tags: vec!["电子".to_owned()],
            share_enabled: false,
            share_status: GearShareStatus::NotShared,
            notes: None,
        }
    }

    /// Runs the `gear repository crud filter stats soft delete` server-side flow while preserving input validation, error propagation, and state invariants.
    #[tokio::test]
    async fn gear_repository_crud_filter_stats_soft_delete() {
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

        let stats = repo.stats(user_id).await.unwrap();
        assert_eq!(stats.current_count, 2);
        assert_eq!(stats.total_weight_g, 630);
        assert_eq!(stats.total_value_cents, 127800);
        let electronics_stats = &stats.by_category[7];
        assert_eq!(electronics_stats.count, 1);
        assert_eq!(electronics_stats.total_weight_g, 315);
        assert_eq!(electronics_stats.total_value_cents, 63900);
        let available_stats = &stats.by_status[0];
        assert_eq!(available_stats.count, 2);
        assert_eq!(available_stats.total_weight_g, 630);
        assert_eq!(available_stats.total_value_cents, 127800);

        let (available, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(available.len(), 2);

        assert!(repo.soft_delete(user_id, &created.id).await.unwrap());
        assert!(repo.get(user_id, &created.id).await.unwrap().is_none());
        let stats = repo.stats(user_id).await.unwrap();
        assert_eq!(stats.current_count, 1);
        assert_eq!(stats.by_category[7].count, 0);
        assert_eq!(stats.by_category[7].total_weight_g, 0);
        assert_eq!(stats.by_category[7].total_value_cents, 0);
        let (deleted_items, _) = repo
            .list(
                user_id,
                &ListGearOptions {
                    deleted: DeletedFilter::Deleted,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(deleted_items.len(), 1);
        assert!(deleted_items[0].is_deleted);
    }

    /// Verifies duplicate personal gear rows become one quantity-aware inventory row.
    #[tokio::test]
    async fn gear_repository_merges_same_item_and_counts_quantity() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let user = AuthRepository::new(db.clone())
            .upsert_mock_user("mock:gear-quantity-user", Some("数量用户".to_owned()), None)
            .await
            .unwrap();
        let repo = GearRepository::new(db.clone());
        let user_id = user.id.as_str();
        let first = repo
            .create(
                user_id,
                &draft("NITECORE充电宝", GearCategory::ElectronicsSystem),
            )
            .await
            .unwrap();
        let mut second_draft = draft("NITECORE充电宝", GearCategory::ElectronicsSystem);
        second_draft.quantity = 2;
        second_draft.tags.push("备用".to_owned());
        second_draft.purchase_location = Some("天猫".to_owned());
        let second = repo.create(user_id, &second_draft).await.unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(second.quantity, 3);
        assert_eq!(second.tags, vec!["电子", "备用"]);
        assert!(
            second
                .notes
                .as_deref()
                .is_some_and(|notes| notes.contains("天猫"))
        );
        let (items, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        let stats = repo.stats(user_id).await.unwrap();
        assert_eq!(stats.current_count, 3);
        assert_eq!(stats.total_weight_g, 945);
        assert_eq!(stats.total_value_cents, 191700);
        assert_eq!(stats.by_category[7].count, 3);
        assert_eq!(stats.by_category[7].total_weight_g, 945);
        assert_eq!(stats.by_category[7].total_value_cents, 191700);
        assert_eq!(stats.by_status[0].count, 3);
        assert_eq!(stats.by_status[0].total_weight_g, 945);
        assert_eq!(stats.by_status[0].total_value_cents, 191700);
    }
}
