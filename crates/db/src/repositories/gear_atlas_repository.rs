//! Repository for public gear atlas submissions and approved atlas reads.
//!
//! This repository is intentionally separate from personal gear persistence so
//! public queries can only select the atlas table's limited public snapshot
//! columns.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs},
    gear_atlas::{
        GearAtlasDraft, GearAtlasItem, GearAtlasSort, GearAtlasSourceType, GearAtlasStatus,
        now_atlas_rfc3339,
    },
};
use uuid::Uuid;

use super::statement;

/// Public list options for approved gear atlas entries.
#[derive(Clone, Debug)]
pub struct ListGearAtlasOptions {
    pub category: Option<GearCategory>,
    pub q: Option<String>,
    pub sort: GearAtlasSort,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearAtlasOptions {
    fn default() -> Self {
        Self {
            category: None,
            q: None,
            sort: GearAtlasSort::ApprovedAtDesc,
            limit: 20,
            cursor: None,
        }
    }
}

/// Administrator review queue filters.
#[derive(Clone, Debug)]
pub struct ListGearAtlasAdminOptions {
    pub status: Option<GearAtlasStatus>,
    pub category: Option<GearCategory>,
    pub q: Option<String>,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearAtlasAdminOptions {
    fn default() -> Self {
        Self {
            status: Some(GearAtlasStatus::Pending),
            category: None,
            q: None,
            limit: 20,
            cursor: None,
        }
    }
}

/// Gear atlas persistence object for public reads, user submissions, and admin review.
#[derive(Clone)]
pub struct GearAtlasRepository {
    db: DatabaseConnection,
}

impl GearAtlasRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a pending atlas submission from a validated public draft.
    pub async fn create_submission(&self, draft: &GearAtlasDraft) -> Result<GearAtlasItem, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO gear_atlas_items (
                    id, category, name, brand, model, description, weight_g,
                    official_price_cents, official_price_currency, specs_json,
                    source_type, submitted_by_user_id, source_user_gear_id, status,
                    rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                atlas_values(&id, draft, &specs_json, &now),
            ))
            .await?;
        self.get_any(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created gear atlas item not found".to_owned()))
    }

    /// Returns an existing pending or approved submission for a personal gear source.
    pub async fn active_source_submission(
        &self,
        user_id: &str,
        source_user_gear_id: &str,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE submitted_by_user_id = ? AND source_user_gear_id = ? \
                     AND status IN ('pending', 'approved') ORDER BY created_at DESC, id DESC LIMIT 1",
                    atlas_select_columns()
                ),
                vec![
                    user_id.to_owned().into(),
                    source_user_gear_id.to_owned().into(),
                ],
            ))
            .await?;
        row.map(|row| map_atlas_item(&row)).transpose()
    }

    /// Lists approved public atlas entries.
    pub async fn list_public(
        &self,
        options: &ListGearAtlasOptions,
    ) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = Vec::new();
        let mut clauses = vec!["status = 'approved'".to_owned()];
        apply_common_filters(
            &mut clauses,
            &mut values,
            options.category,
            options.q.as_deref(),
        );
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        let sql = format!(
            "{} WHERE {} {} LIMIT ? OFFSET ?",
            atlas_select_columns(),
            clauses.join(" AND "),
            public_order_by(options.sort),
        );
        page_items(self.query_items(sql, values).await?, limit, offset)
    }

    /// Fetches one approved public atlas item.
    pub async fn get_public(&self, id: &str) -> Result<Option<GearAtlasItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE id = ? AND status = 'approved' LIMIT 1",
                    atlas_select_columns()
                ),
                vec![id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_atlas_item(&row)).transpose()
    }

    /// Lists submissions owned by one user for status tracking in clients.
    pub async fn list_user_submissions(
        &self,
        user_id: &str,
        limit: u64,
        cursor: Option<&str>,
    ) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
        let limit = limit.clamp(1, 100);
        let offset = parse_cursor(cursor)?;
        let values = vec![
            user_id.to_owned().into(),
            (limit as i64 + 1).into(),
            offset.into(),
        ];
        let sql = format!(
            "{} WHERE submitted_by_user_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
            atlas_select_columns()
        );
        page_items(self.query_items(sql, values).await?, limit, offset)
    }

    /// Lists submissions for the administrator review queue.
    pub async fn list_admin(
        &self,
        options: &ListGearAtlasAdminOptions,
    ) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = Vec::new();
        let mut clauses = Vec::new();
        if let Some(status) = options.status {
            clauses.push("status = ?".to_owned());
            values.push(status.as_str().to_owned().into());
        }
        apply_common_filters(
            &mut clauses,
            &mut values,
            options.category,
            options.q.as_deref(),
        );
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        let sql = format!(
            "{}{} ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
            atlas_select_columns(),
            where_clause,
        );
        page_items(self.query_items(sql, values).await?, limit, offset)
    }

    /// Fetches a submission regardless of review status for administrator details.
    pub async fn get_any(&self, id: &str) -> Result<Option<GearAtlasItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("{} WHERE id = ? LIMIT 1", atlas_select_columns()),
                vec![id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_atlas_item(&row)).transpose()
    }

    /// Approves a submission and returns the updated public atlas item.
    pub async fn approve(
        &self,
        id: &str,
        reviewer_user_id: &str,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        let now = now_atlas_rfc3339();
        self.update_review(
            id,
            GearAtlasStatus::Approved,
            reviewer_user_id,
            None,
            Some(now),
        )
        .await
    }

    /// Rejects a submission and returns the updated review item.
    pub async fn reject(
        &self,
        id: &str,
        reviewer_user_id: &str,
        reason: Option<String>,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        self.update_review(
            id,
            GearAtlasStatus::Rejected,
            reviewer_user_id,
            reason,
            None,
        )
        .await
    }

    async fn update_review(
        &self,
        id: &str,
        status: GearAtlasStatus,
        reviewer_user_id: &str,
        rejection_reason: Option<String>,
        approved_at: Option<String>,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        let now = now_atlas_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_atlas_items SET status = ?, rejection_reason = ?, \
                 reviewed_by_user_id = ?, reviewed_at = ?, approved_at = ?, updated_at = ? \
                 WHERE id = ?",
                vec![
                    status.as_str().to_owned().into(),
                    rejection_reason.into(),
                    reviewer_user_id.to_owned().into(),
                    now.clone().into(),
                    approved_at.into(),
                    now.into(),
                    id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get_any(id).await
        }
    }

    async fn query_items(
        &self,
        sql: String,
        values: Vec<Value>,
    ) -> Result<Vec<GearAtlasItem>, DbErr> {
        self.db
            .query_all(statement(self.db.get_database_backend(), sql, values))
            .await?
            .into_iter()
            .map(|row| map_atlas_item(&row))
            .collect()
    }
}

fn atlas_values(id: &str, draft: &GearAtlasDraft, specs_json: &str, now: &str) -> Vec<Value> {
    vec![
        id.to_owned().into(),
        draft.category.as_str().to_owned().into(),
        draft.name.clone().into(),
        draft.brand.clone().into(),
        draft.model.clone().into(),
        draft.description.clone().into(),
        draft.weight_g.into(),
        draft.official_price_cents.into(),
        draft.official_price_currency.clone().into(),
        specs_json.to_owned().into(),
        draft.source_type.as_str().to_owned().into(),
        draft.submitted_by_user_id.clone().into(),
        draft.source_user_gear_id.clone().into(),
        GearAtlasStatus::Pending.as_str().to_owned().into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        now.to_owned().into(),
        now.to_owned().into(),
    ]
}

fn atlas_select_columns() -> &'static str {
    r#"SELECT id, category, name, brand, model, description, weight_g,
        official_price_cents, official_price_currency, specs_json,
        source_type, submitted_by_user_id, source_user_gear_id, status,
        rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
        created_at, updated_at
       FROM gear_atlas_items"#
}

fn apply_common_filters(
    clauses: &mut Vec<String>,
    values: &mut Vec<Value>,
    category: Option<GearCategory>,
    q: Option<&str>,
) {
    if let Some(category) = category {
        clauses.push("category = ?".to_owned());
        values.push(category.as_str().to_owned().into());
    }
    if let Some(q) = normalize_query(q) {
        clauses.push(
            "(LOWER(name) LIKE ? OR LOWER(COALESCE(brand, '')) LIKE ? OR LOWER(COALESCE(model, '')) LIKE ?)".to_owned(),
        );
        values.push(q.clone().into());
        values.push(q.clone().into());
        values.push(q.into());
    }
}

fn public_order_by(sort: GearAtlasSort) -> &'static str {
    match sort {
        GearAtlasSort::ApprovedAtDesc => "ORDER BY approved_at DESC, created_at DESC, id DESC",
        GearAtlasSort::NameAsc => "ORDER BY name ASC, approved_at DESC, id DESC",
        GearAtlasSort::WeightDesc => "ORDER BY weight_g DESC, approved_at DESC, id DESC",
        GearAtlasSort::OfficialPriceDesc => {
            "ORDER BY official_price_cents DESC, approved_at DESC, id DESC"
        }
    }
}

fn page_items(
    mut items: Vec<GearAtlasItem>,
    limit: u64,
    offset: i64,
) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
    let next_cursor = if items.len() > limit as usize {
        items.truncate(limit as usize);
        Some((offset + limit as i64).to_string())
    } else {
        None
    };
    Ok((items, next_cursor))
}

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

fn map_atlas_item(row: &sea_orm::QueryResult) -> Result<GearAtlasItem, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let source_type_raw: String = row.try_get("", "source_type")?;
    let status_raw: String = row.try_get("", "status")?;
    let specs_json: String = row.try_get("", "specs_json")?;
    let specs: GearSpecs = serde_json::from_str(&specs_json).unwrap_or_default();
    Ok(GearAtlasItem {
        id: row.try_get("", "id")?,
        category: GearCategory::from_key(&category_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear atlas category: {category_raw}")))?,
        name: row.try_get("", "name")?,
        brand: row.try_get("", "brand")?,
        model: row.try_get("", "model")?,
        description: row.try_get("", "description")?,
        weight_g: row.try_get("", "weight_g")?,
        official_price_cents: row.try_get("", "official_price_cents")?,
        official_price_currency: row.try_get("", "official_price_currency")?,
        specs,
        source_type: GearAtlasSourceType::from_key(&source_type_raw).ok_or_else(|| {
            DbErr::Custom(format!("invalid gear atlas source type: {source_type_raw}"))
        })?,
        submitted_by_user_id: row.try_get("", "submitted_by_user_id")?,
        source_user_gear_id: row.try_get("", "source_user_gear_id")?,
        status: GearAtlasStatus::from_key(&status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear atlas status: {status_raw}")))?,
        rejection_reason: row.try_get("", "rejection_reason")?,
        reviewed_by_user_id: row.try_get("", "reviewed_by_user_id")?,
        reviewed_at: row.try_get("", "reviewed_at")?,
        approved_at: row.try_get("", "approved_at")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}
