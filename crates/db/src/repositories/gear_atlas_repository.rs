//! Repository for public gear atlas submissions and approved atlas reads.
//!
//! This repository is intentionally separate from personal gear persistence so
//! public queries can only select the atlas table's limited public snapshot
//! columns.

use std::cmp::Ordering;

use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, TransactionTrait, Value,
};
use stellartrail_domain::{
    gear::{GearCategory, GearSpecs, GearVariants},
    gear_atlas::{
        GearAtlasDraft, GearAtlasExternalImportDraft, GearAtlasItem, GearAtlasSort,
        GearAtlasSourceType, GearAtlasStatus, now_atlas_rfc3339,
    },
    locale::Locale,
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

/// Result action after an idempotent external gear atlas import.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GearAtlasExternalImportAction {
    Created,
    Updated,
    SkippedApproved,
}

impl GearAtlasExternalImportAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::SkippedApproved => "skipped_approved",
        }
    }
}

/// Return value for one external import upsert.
#[derive(Clone, Debug)]
pub struct GearAtlasExternalImportResult {
    pub action: GearAtlasExternalImportAction,
    pub item: GearAtlasItem,
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
        let variants_json = variants_json(&draft.variants)?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"INSERT INTO gear_atlas_items (
                id, category, name, brand, model, description, weight_g,
                official_price_cents, official_price_currency, variants_json, specs_json,
                source_type, submitted_by_user_id, source_user_gear_id, status,
                rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            atlas_values(&id, draft, &variants_json, &specs_json, &now),
        ))
        .await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            "INSERT INTO gear_atlas_item_localizations(atlas_item_id, locale, name, description) \
             VALUES (?, ?, ?, ?) ON CONFLICT(atlas_item_id, locale) DO UPDATE SET name = excluded.name, description = excluded.description",
            vec![
                id.clone().into(),
                Locale::ZhCn.as_str().to_owned().into(),
                draft.name.clone().into(),
                draft.description.clone().into(),
            ],
        ))
        .await?;
        tx.commit().await?;
        self.get_any(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created gear atlas item not found".to_owned()))
    }

    /// Creates or refreshes a pending atlas submission from an external source record.
    ///
    /// Approved rows are returned unchanged so a later import cannot silently
    /// rewrite public reviewed content. Pending and rejected rows are refreshed
    /// back to `pending` for another administrator review pass.
    pub async fn upsert_external_import(
        &self,
        draft: &GearAtlasExternalImportDraft,
    ) -> Result<GearAtlasExternalImportResult, DbErr> {
        if let Some(existing) = self.find_by_source_key(&draft.source_key).await? {
            if existing.status == GearAtlasStatus::Approved {
                return Ok(GearAtlasExternalImportResult {
                    action: GearAtlasExternalImportAction::SkippedApproved,
                    item: existing,
                });
            }
            let item = self.refresh_external_import(&existing.id, draft).await?;
            return Ok(GearAtlasExternalImportResult {
                action: GearAtlasExternalImportAction::Updated,
                item,
            });
        }

        let id = Uuid::new_v4().to_string();
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        let variants_json = variants_json(&draft.variants)?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"INSERT INTO gear_atlas_items (
                id, category, name, brand, model, description, weight_g,
                official_price_cents, official_price_currency, variants_json, specs_json,
                source_type, submitted_by_user_id, source_user_gear_id, status,
                rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
                source_key, source_name, source_url, source_license_note,
                import_batch_id, imported_at, source_rating_score, source_rating_count,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            external_import_values(&id, draft, &variants_json, &specs_json, &now),
        ))
        .await?;
        upsert_zh_localization(
            &tx,
            self.db.get_database_backend(),
            &id,
            &draft.name,
            draft.description.clone(),
        )
        .await?;
        tx.commit().await?;
        let item = self
            .get_any(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created gear atlas import not found".to_owned()))?;
        Ok(GearAtlasExternalImportResult {
            action: GearAtlasExternalImportAction::Created,
            item,
        })
    }

    /// Lists approved public atlas entries with locale-resolved public text.
    pub async fn list_public(
        &self,
        options: &ListGearAtlasOptions,
        locale: Locale,
    ) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = Vec::new();
        let mut clauses = vec!["status = 'approved'".to_owned()];
        if let Some(category) = options.category {
            clauses.push("category = ?".to_owned());
            values.push(category.as_str().to_owned().into());
        }
        let sql = format!(
            "{} WHERE {} ORDER BY created_at DESC, id DESC",
            atlas_select_columns(),
            clauses.join(" AND "),
        );
        let mut items = self.query_items(sql, values).await?;
        self.localize_public_items(&mut items, locale).await?;
        if let Some(query) = normalize_plain_query(options.q.as_deref()) {
            items.retain(|item| public_item_matches_query(item, &query));
        }
        sort_public_items(&mut items, options.sort);
        page_in_memory_items(items, limit, offset)
    }

    /// Fetches one approved public atlas item with locale-resolved public text.
    pub async fn get_public(
        &self,
        id: &str,
        locale: Locale,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
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
        let mut item = match row.map(|row| map_atlas_item(&row)).transpose()? {
            Some(item) => item,
            None => return Ok(None),
        };
        self.localize_public_item(&mut item, locale).await?;
        Ok(Some(item))
    }

    /// Resolves a public gear category label from DB-backed locale rows.
    pub async fn category_label(
        &self,
        category: GearCategory,
        locale: Locale,
    ) -> Result<String, DbErr> {
        for candidate in locale.fallbacks() {
            let row = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT label FROM gear_category_localizations WHERE category = ? AND locale = ?",
                    vec![
                        category.as_str().to_owned().into(),
                        candidate.as_str().to_owned().into(),
                    ],
                ))
                .await?;
            if let Some(row) = row {
                return row.try_get("", "label");
            }
        }
        Ok(category.label().to_owned())
    }

    async fn localize_public_items(
        &self,
        items: &mut [GearAtlasItem],
        locale: Locale,
    ) -> Result<(), DbErr> {
        for item in items {
            self.localize_public_item(item, locale).await?;
        }
        Ok(())
    }

    async fn localize_public_item(
        &self,
        item: &mut GearAtlasItem,
        locale: Locale,
    ) -> Result<(), DbErr> {
        for candidate in locale.fallbacks() {
            let row = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT name, description FROM gear_atlas_item_localizations WHERE atlas_item_id = ? AND locale = ?",
                    vec![
                        item.id.clone().into(),
                        candidate.as_str().to_owned().into(),
                    ],
                ))
                .await?;
            if let Some(row) = row {
                item.name = row.try_get("", "name")?;
                item.description = row.try_get("", "description")?;
                return Ok(());
            }
        }
        Ok(())
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

    /// Updates editable public fields on a submission before or after review.
    pub async fn update_submission(
        &self,
        id: &str,
        draft: &GearAtlasDraft,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        let variants_json = variants_json(&draft.variants)?;
        let tx = self.db.begin().await?;
        let result = tx
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE gear_atlas_items
                   SET category = ?, name = ?, brand = ?, model = ?, description = ?,
                       weight_g = ?, official_price_cents = ?, official_price_currency = ?,
                       variants_json = ?, specs_json = ?, updated_at = ?
                   WHERE id = ?"#,
                vec![
                    draft.category.as_str().to_owned().into(),
                    draft.name.clone().into(),
                    draft.brand.clone().into(),
                    draft.model.clone().into(),
                    draft.description.clone().into(),
                    draft.weight_g.into(),
                    draft.official_price_cents.into(),
                    draft.official_price_currency.clone().into(),
                    variants_json.into(),
                    specs_json.into(),
                    now.into(),
                    id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(None);
        }
        upsert_zh_localization(
            &tx,
            self.db.get_database_backend(),
            id,
            &draft.name,
            draft.description.clone(),
        )
        .await?;
        tx.commit().await?;
        self.get_any(id).await
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

    async fn find_by_source_key(&self, source_key: &str) -> Result<Option<GearAtlasItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("{} WHERE source_key = ? LIMIT 1", atlas_select_columns()),
                vec![source_key.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_atlas_item(&row)).transpose()
    }

    async fn refresh_external_import(
        &self,
        id: &str,
        draft: &GearAtlasExternalImportDraft,
    ) -> Result<GearAtlasItem, DbErr> {
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        let variants_json = variants_json(&draft.variants)?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"UPDATE gear_atlas_items
               SET category = ?, name = ?, brand = ?, model = ?, description = ?,
                   weight_g = ?, official_price_cents = ?, official_price_currency = ?,
                   variants_json = ?, specs_json = ?, source_type = ?, submitted_by_user_id = ?,
                   source_user_gear_id = NULL, status = ?, rejection_reason = NULL,
                   reviewed_by_user_id = NULL, reviewed_at = NULL, approved_at = NULL,
                   source_key = ?, source_name = ?, source_url = ?,
                   source_license_note = ?, import_batch_id = ?, imported_at = ?,
                   source_rating_score = ?, source_rating_count = ?, updated_at = ?
               WHERE id = ?"#,
            vec![
                draft.category.as_str().to_owned().into(),
                draft.name.clone().into(),
                draft.brand.clone().into(),
                draft.model.clone().into(),
                draft.description.clone().into(),
                draft.weight_g.into(),
                draft.official_price_cents.into(),
                draft.official_price_currency.clone().into(),
                variants_json.into(),
                specs_json.into(),
                GearAtlasSourceType::ExternalImport
                    .as_str()
                    .to_owned()
                    .into(),
                draft.submitted_by_user_id.clone().into(),
                GearAtlasStatus::Pending.as_str().to_owned().into(),
                draft.source_key.clone().into(),
                draft.source_name.clone().into(),
                draft.source_url.clone().into(),
                draft.source_license_note.clone().into(),
                draft.import_batch_id.clone().into(),
                now.clone().into(),
                draft.source_rating_score.into(),
                draft.source_rating_count.into(),
                now.into(),
                id.to_owned().into(),
            ],
        ))
        .await?;
        upsert_zh_localization(
            &tx,
            self.db.get_database_backend(),
            id,
            &draft.name,
            draft.description.clone(),
        )
        .await?;
        tx.commit().await?;
        self.get_any(id)
            .await?
            .ok_or_else(|| DbErr::Custom("updated gear atlas import not found".to_owned()))
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

fn atlas_values(
    id: &str,
    draft: &GearAtlasDraft,
    variants_json: &str,
    specs_json: &str,
    now: &str,
) -> Vec<Value> {
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
        variants_json.to_owned().into(),
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

fn external_import_values(
    id: &str,
    draft: &GearAtlasExternalImportDraft,
    variants_json: &str,
    specs_json: &str,
    now: &str,
) -> Vec<Value> {
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
        variants_json.to_owned().into(),
        specs_json.to_owned().into(),
        GearAtlasSourceType::ExternalImport
            .as_str()
            .to_owned()
            .into(),
        draft.submitted_by_user_id.clone().into(),
        Option::<String>::None.into(),
        GearAtlasStatus::Pending.as_str().to_owned().into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        Option::<String>::None.into(),
        draft.source_key.clone().into(),
        draft.source_name.clone().into(),
        draft.source_url.clone().into(),
        draft.source_license_note.clone().into(),
        draft.import_batch_id.clone().into(),
        now.to_owned().into(),
        draft.source_rating_score.into(),
        draft.source_rating_count.into(),
        now.to_owned().into(),
        now.to_owned().into(),
    ]
}

fn atlas_select_columns() -> &'static str {
    r#"SELECT id, category, name, brand, model, description, weight_g,
        official_price_cents, official_price_currency, variants_json, specs_json,
        source_type, submitted_by_user_id, source_user_gear_id, status,
        rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
        source_key, source_name, source_url, source_license_note, import_batch_id,
        imported_at, source_rating_score, source_rating_count,
        created_at, updated_at
       FROM gear_atlas_items"#
}

async fn upsert_zh_localization(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
    id: &str,
    name: &str,
    description: Option<String>,
) -> Result<(), DbErr> {
    db.execute(statement(
        backend,
        "INSERT INTO gear_atlas_item_localizations(atlas_item_id, locale, name, description) \
         VALUES (?, ?, ?, ?) ON CONFLICT(atlas_item_id, locale) DO UPDATE SET name = excluded.name, description = excluded.description",
        vec![
            id.to_owned().into(),
            Locale::ZhCn.as_str().to_owned().into(),
            name.to_owned().into(),
            description.into(),
        ],
    ))
    .await?;
    Ok(())
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

fn page_in_memory_items(
    items: Vec<GearAtlasItem>,
    limit: u64,
    offset: i64,
) -> Result<(Vec<GearAtlasItem>, Option<String>), DbErr> {
    let start = offset as usize;
    if start >= items.len() {
        return Ok((Vec::new(), None));
    }
    let end = (start + limit as usize).min(items.len());
    let next_cursor = if end < items.len() {
        Some((offset + limit as i64).to_string())
    } else {
        None
    };
    Ok((items[start..end].to_vec(), next_cursor))
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

fn normalize_plain_query(q: Option<&str>) -> Option<String> {
    let q = q?.trim().to_lowercase();
    if q.is_empty() { None } else { Some(q) }
}

fn public_item_matches_query(item: &GearAtlasItem, query: &str) -> bool {
    item.name.to_lowercase().contains(query)
        || item
            .description
            .as_deref()
            .map(|value| value.to_lowercase().contains(query))
            .unwrap_or(false)
        || item
            .brand
            .as_deref()
            .map(|value| value.to_lowercase().contains(query))
            .unwrap_or(false)
        || item
            .model
            .as_deref()
            .map(|value| value.to_lowercase().contains(query))
            .unwrap_or(false)
}

fn sort_public_items(items: &mut [GearAtlasItem], sort: GearAtlasSort) {
    match sort {
        GearAtlasSort::ApprovedAtDesc => items.sort_by(cmp_approved_desc),
        GearAtlasSort::NameAsc => items.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| cmp_approved_desc(left, right))
        }),
        GearAtlasSort::WeightDesc => items.sort_by(|left, right| {
            cmp_option_desc(left.weight_g, right.weight_g)
                .then_with(|| cmp_approved_desc(left, right))
        }),
        GearAtlasSort::OfficialPriceDesc => items.sort_by(|left, right| {
            cmp_option_desc(left.official_price_cents, right.official_price_cents)
                .then_with(|| cmp_approved_desc(left, right))
        }),
    }
}

fn cmp_approved_desc(left: &GearAtlasItem, right: &GearAtlasItem) -> Ordering {
    right
        .approved_at
        .cmp(&left.approved_at)
        .then_with(|| right.created_at.cmp(&left.created_at))
        .then_with(|| right.id.cmp(&left.id))
}

fn cmp_option_desc<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn json_string(specs: &GearSpecs) -> Result<String, DbErr> {
    serde_json::to_string(specs).map_err(|err| DbErr::Custom(err.to_string()))
}

fn variants_json(variants: &GearVariants) -> Result<String, DbErr> {
    serde_json::to_string(variants).map_err(|err| DbErr::Custom(err.to_string()))
}

fn map_atlas_item(row: &sea_orm::QueryResult) -> Result<GearAtlasItem, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let source_type_raw: String = row.try_get("", "source_type")?;
    let status_raw: String = row.try_get("", "status")?;
    let specs_json: String = row.try_get("", "specs_json")?;
    let variants_json: String = row.try_get("", "variants_json")?;
    let specs: GearSpecs = serde_json::from_str(&specs_json).unwrap_or_default();
    let variants: GearVariants = serde_json::from_str(&variants_json).unwrap_or_default();
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
        variants,
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
        source_key: row.try_get("", "source_key")?,
        source_name: row.try_get("", "source_name")?,
        source_url: row.try_get("", "source_url")?,
        source_license_note: row.try_get("", "source_license_note")?,
        import_batch_id: row.try_get("", "import_batch_id")?,
        imported_at: row.try_get("", "imported_at")?,
        source_rating_score: row.try_get("", "source_rating_score")?,
        source_rating_count: row.try_get("", "source_rating_count")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}
