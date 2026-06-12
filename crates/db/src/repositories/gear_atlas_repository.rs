//! Repository for public gear atlas submissions and approved atlas reads.
//!
//! This repository is intentionally separate from personal gear persistence so
//! public queries can only select the atlas table's limited public snapshot
//! columns.

use std::{cmp::Ordering, collections::HashMap};

use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, QueryResult, TransactionTrait,
    Value,
};
use sha2::{Digest, Sha256};
use stellartrail_domain::{
    deletion::DeletedFilter,
    gear::{GearCategory, GearSpecs, GearVariants},
    gear_atlas::{
        GEAR_ATLAS_LOCALIZATION_STATUS_DRAFT, GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED,
        GearAtlasDraft, GearAtlasExternalImportDraft, GearAtlasItem, GearAtlasLocalizationDraft,
        GearAtlasLocalizationReviewState, GearAtlasLocalizationReviewStatus,
        GearAtlasPublicSnapshot, GearAtlasReviewChanges, GearAtlasSort, GearAtlasSourceType,
        GearAtlasStatus, now_atlas_rfc3339, review_changes_between,
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
    pub deleted: DeletedFilter,
    pub q: Option<String>,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearAtlasAdminOptions {
    fn default() -> Self {
        Self {
            status: Some(GearAtlasStatus::Pending),
            category: None,
            deleted: DeletedFilter::Active,
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
    SkippedLessDetailed,
}

impl GearAtlasExternalImportAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::SkippedApproved => "skipped_approved",
            Self::SkippedLessDetailed => "skipped_less_detailed",
        }
    }
}

/// Return value for one external import upsert.
#[derive(Clone, Debug)]
pub struct GearAtlasExternalImportResult {
    pub action: GearAtlasExternalImportAction,
    pub item: GearAtlasItem,
}

#[derive(Clone, Debug)]
struct ImportSourceLookup {
    atlas_item_id: String,
}

#[derive(Clone, Debug)]
struct CanonicalImportMatch {
    atlas_item_id: String,
    detail_score: i32,
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
        let submitted_snapshot_json = snapshot_json(&GearAtlasPublicSnapshot::from_draft(draft))?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"INSERT INTO gear_atlas_items (
                id, category, name, brand, model, description, weight_g,
                official_price_cents, official_price_currency, variants_json, specs_json,
                submitted_snapshot_json, review_changes_json,
                source_type, submitted_by_user_id, source_user_gear_id, status,
                rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            atlas_values(
                &id,
                draft,
                &variants_json,
                &specs_json,
                &submitted_snapshot_json,
                &now,
            ),
        ))
        .await?;
        upsert_localization(
            &tx,
            self.db.get_database_backend(),
            &id,
            &GearAtlasLocalizationDraft {
                locale: Locale::ZhCn,
                name: draft.name.clone(),
                description: draft.description.clone(),
                variants: draft.variants.clone(),
                specs: draft.specs.clone(),
                translation_status: None,
                translation_provider: None,
                translated_at: None,
            },
        )
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
        let canonical_key = canonical_key_for_import(draft);
        let incoming_detail_score = detail_score_for_import(draft);

        if let Some(source) = self.find_import_source(&draft.source_key).await? {
            if let Some(existing) = self.get_any(&source.atlas_item_id).await? {
                if existing.status == GearAtlasStatus::Approved && !existing.is_deleted {
                    self.record_import_source(
                        &existing.id,
                        draft,
                        &canonical_key,
                        GearAtlasExternalImportAction::SkippedApproved.as_str(),
                    )
                    .await?;
                    return Ok(GearAtlasExternalImportResult {
                        action: GearAtlasExternalImportAction::SkippedApproved,
                        item: existing,
                    });
                }
                let item = self
                    .refresh_external_import(
                        &existing.id,
                        draft,
                        &canonical_key,
                        incoming_detail_score,
                        GearAtlasExternalImportAction::Updated.as_str(),
                    )
                    .await?;
                return Ok(GearAtlasExternalImportResult {
                    action: GearAtlasExternalImportAction::Updated,
                    item,
                });
            }
        }

        if let Some(existing) = self.find_by_source_key(&draft.source_key).await? {
            if existing.status == GearAtlasStatus::Approved && !existing.is_deleted {
                self.record_import_source(
                    &existing.id,
                    draft,
                    &canonical_key,
                    GearAtlasExternalImportAction::SkippedApproved.as_str(),
                )
                .await?;
                return Ok(GearAtlasExternalImportResult {
                    action: GearAtlasExternalImportAction::SkippedApproved,
                    item: existing,
                });
            }
            let item = self
                .refresh_external_import(
                    &existing.id,
                    draft,
                    &canonical_key,
                    incoming_detail_score,
                    GearAtlasExternalImportAction::Updated.as_str(),
                )
                .await?;
            return Ok(GearAtlasExternalImportResult {
                action: GearAtlasExternalImportAction::Updated,
                item,
            });
        }

        if let Some(candidate) = self.find_best_import_source(&canonical_key).await?
            && let Some(existing) = self.get_any(&candidate.atlas_item_id).await?
        {
            if existing.status == GearAtlasStatus::Approved && !existing.is_deleted {
                self.record_import_source(
                    &existing.id,
                    draft,
                    &canonical_key,
                    GearAtlasExternalImportAction::SkippedApproved.as_str(),
                )
                .await?;
                return Ok(GearAtlasExternalImportResult {
                    action: GearAtlasExternalImportAction::SkippedApproved,
                    item: existing,
                });
            }
            if incoming_detail_score < candidate.detail_score {
                self.record_import_source(
                    &existing.id,
                    draft,
                    &canonical_key,
                    GearAtlasExternalImportAction::SkippedLessDetailed.as_str(),
                )
                .await?;
                return Ok(GearAtlasExternalImportResult {
                    action: GearAtlasExternalImportAction::SkippedLessDetailed,
                    item: existing,
                });
            }
            let item = self
                .refresh_external_import(
                    &existing.id,
                    draft,
                    &canonical_key,
                    incoming_detail_score,
                    GearAtlasExternalImportAction::Updated.as_str(),
                )
                .await?;
            return Ok(GearAtlasExternalImportResult {
                action: GearAtlasExternalImportAction::Updated,
                item,
            });
        }

        let id = Uuid::new_v4().to_string();
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        let variants_json = variants_json(&draft.variants)?;
        let submitted_snapshot_json = snapshot_json(&snapshot_from_external_import(draft))?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"INSERT INTO gear_atlas_items (
                id, category, name, brand, model, description, weight_g,
                official_price_cents, official_price_currency, variants_json, specs_json,
                submitted_snapshot_json, review_changes_json,
                source_type, submitted_by_user_id, source_user_gear_id, status,
                rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
                source_key, source_name, source_url, source_license_note,
                import_batch_id, imported_at, source_rating_score, source_rating_count,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            external_import_values(
                &id,
                draft,
                &variants_json,
                &specs_json,
                &submitted_snapshot_json,
                &now,
            ),
        ))
        .await?;
        upsert_external_localizations(&tx, self.db.get_database_backend(), &id, draft).await?;
        upsert_import_source(
            &tx,
            self.db.get_database_backend(),
            &id,
            draft,
            &canonical_key,
            incoming_detail_score,
            GearAtlasExternalImportAction::Created.as_str(),
            &now,
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
        let mut clauses = vec![
            "status = 'approved'".to_owned(),
            "is_deleted = FALSE".to_owned(),
        ];
        if let Some(category) = options.category {
            clauses.push("category = ?".to_owned());
            values.push(category.as_str().to_owned().into());
        }
        apply_public_search_filter(&mut clauses, &mut values, options.q.as_deref());
        let sql = format!(
            "{} WHERE {} ORDER BY created_at DESC, id DESC",
            atlas_select_columns(),
            clauses.join(" AND "),
        );
        let mut items = self.query_items(sql, values).await?;
        self.localize_public_items(&mut items, locale).await?;
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
                    "{} WHERE id = ? AND status = 'approved' AND is_deleted = FALSE LIMIT 1",
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

    /// Builds a locale-resolved display copy without changing canonical fields.
    ///
    /// Administrator review endpoints need both the original editable fields and
    /// the locale overlay used by public atlas clients. Returning a copy keeps
    /// the caller from accidentally persisting translated display text back over
    /// the source row.
    pub async fn localized_display_item(
        &self,
        item: &GearAtlasItem,
        locale: Locale,
    ) -> Result<GearAtlasItem, DbErr> {
        let mut display = item.clone();
        self.localize_public_item(&mut display, locale).await?;
        Ok(display)
    }

    /// Lists pending external-import items that need a target-locale display row.
    ///
    /// This is intentionally scoped to localization backfills: it does not expose
    /// source keys and does not mutate the canonical atlas row.
    pub async fn list_external_import_localization_backfill_candidates(
        &self,
        source_locale: Locale,
        target_locale: Locale,
        limit: u64,
    ) -> Result<Vec<GearAtlasItem>, DbErr> {
        let limit = limit.clamp(1, 1_000);
        let sql = format!(
            "{} WHERE status = 'pending' AND is_deleted = FALSE \
             AND EXISTS ( \
                 SELECT 1 FROM gear_atlas_import_sources s \
                 WHERE s.atlas_item_id = gear_atlas_items.id AND s.source_locale = ? \
             ) \
             AND ( \
                 NOT EXISTS ( \
                     SELECT 1 FROM gear_atlas_item_localizations l \
                     WHERE l.atlas_item_id = gear_atlas_items.id AND l.locale = ? \
                 ) \
                 OR EXISTS ( \
                     SELECT 1 FROM gear_atlas_item_localizations l \
                     WHERE l.atlas_item_id = gear_atlas_items.id \
                       AND l.locale = ? \
                       AND l.name = gear_atlas_items.name \
                       AND COALESCE(l.translation_status, '') <> 'needs_review' \
                 ) \
             ) \
             ORDER BY created_at DESC, id DESC LIMIT ?",
            atlas_select_columns()
        );
        self.query_items(
            sql,
            vec![
                source_locale.as_str().to_owned().into(),
                target_locale.as_str().to_owned().into(),
                target_locale.as_str().to_owned().into(),
                (limit as i64).into(),
            ],
        )
        .await
    }

    /// Upserts a single display localization without touching canonical item fields.
    pub async fn upsert_item_localization(
        &self,
        id: &str,
        localization: &GearAtlasLocalizationDraft,
    ) -> Result<(), DbErr> {
        upsert_localization(&self.db, self.db.get_database_backend(), id, localization).await
    }

    /// Reads every stored display localization for one atlas item.
    pub async fn list_item_localizations(
        &self,
        id: &str,
    ) -> Result<Vec<GearAtlasLocalizationDraft>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT locale, name, description, variants_json, specs_json,
                        translation_status, translation_provider, translated_at
                   FROM gear_atlas_item_localizations
                  WHERE atlas_item_id = ?
                  ORDER BY CASE locale WHEN 'zh-CN' THEN 0 WHEN 'en' THEN 1 ELSE 2 END",
                vec![id.to_owned().into()],
            ))
            .await?;
        rows.into_iter().map(localization_from_row).collect()
    }

    /// Reads one stored display localization for an atlas item and locale.
    pub async fn get_item_localization(
        &self,
        id: &str,
        locale: Locale,
    ) -> Result<Option<GearAtlasLocalizationDraft>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT locale, name, description, variants_json, specs_json,
                        translation_status, translation_provider, translated_at
                   FROM gear_atlas_item_localizations
                  WHERE atlas_item_id = ? AND locale = ?
                  LIMIT 1",
                vec![id.to_owned().into(), locale.as_str().to_owned().into()],
            ))
            .await?;
        row.map(localization_from_row).transpose()
    }

    /// Computes the bilingual review status used by admin approval.
    pub async fn localization_review_statuses(
        &self,
        item: &GearAtlasItem,
    ) -> Result<Vec<GearAtlasLocalizationReviewStatus>, DbErr> {
        let localizations = self.list_item_localizations(&item.id).await?;
        let by_locale = localizations
            .iter()
            .map(|localization| (localization.locale, localization))
            .collect::<HashMap<_, _>>();
        Ok([Locale::ZhCn, Locale::En]
            .into_iter()
            .map(|locale| localization_review_status(item, locale, by_locale.get(&locale).copied()))
            .collect())
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
                    "SELECT name, description, variants_json, specs_json \
                     FROM gear_atlas_item_localizations WHERE atlas_item_id = ? AND locale = ?",
                    vec![item.id.clone().into(), candidate.as_str().to_owned().into()],
                ))
                .await?;
            if let Some(row) = row {
                item.name = row.try_get("", "name")?;
                item.description = row.try_get("", "description")?;
                let variants_json: Option<String> = row.try_get("", "variants_json")?;
                if let Some(variants) = variants_json
                    .as_deref()
                    .and_then(|value| serde_json::from_str::<GearVariants>(value).ok())
                {
                    item.variants = variants;
                }
                let specs_json: Option<String> = row.try_get("", "specs_json")?;
                if let Some(specs) = specs_json
                    .as_deref()
                    .and_then(|value| serde_json::from_str::<GearSpecs>(value).ok())
                {
                    item.specs = specs;
                }
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
                     AND is_deleted = FALSE AND status IN ('pending', 'approved') \
                     ORDER BY created_at DESC, id DESC LIMIT 1",
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
            "{} WHERE submitted_by_user_id = ? AND is_deleted = FALSE ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
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
        apply_deleted_filter(&mut clauses, options.deleted);
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
                   WHERE id = ? AND is_deleted = FALSE"#,
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
        upsert_localization(
            &tx,
            self.db.get_database_backend(),
            id,
            &GearAtlasLocalizationDraft {
                locale: Locale::ZhCn,
                name: draft.name.clone(),
                description: draft.description.clone(),
                variants: draft.variants.clone(),
                specs: draft.specs.clone(),
                translation_status: None,
                translation_provider: None,
                translated_at: None,
            },
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
        let Some(existing) = self.get_any(id).await? else {
            return Ok(None);
        };
        if existing.is_deleted {
            return Ok(None);
        }
        let review_changes = review_changes_between(
            &existing.submitted_snapshot,
            &GearAtlasPublicSnapshot::from_item(&existing),
        );
        let now = now_atlas_rfc3339();
        self.update_review(
            id,
            GearAtlasStatus::Approved,
            reviewer_user_id,
            None,
            Some(now),
            Some(&review_changes),
        )
        .await
    }

    /// Rejects a submission and returns the updated review item.
    pub async fn reject(
        &self,
        id: &str,
        reviewer_user_id: &str,
        reason: String,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        self.update_review(
            id,
            GearAtlasStatus::Rejected,
            reviewer_user_id,
            Some(reason),
            None,
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
        review_changes: Option<&GearAtlasReviewChanges>,
    ) -> Result<Option<GearAtlasItem>, DbErr> {
        let now = now_atlas_rfc3339();
        let review_changes_json = review_changes.map(json_string).transpose()?;
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_atlas_items SET status = ?, rejection_reason = ?, \
                 reviewed_by_user_id = ?, reviewed_at = ?, approved_at = ?, \
                 review_changes_json = ?, updated_at = ? \
                 WHERE id = ? AND is_deleted = FALSE",
                vec![
                    status.as_str().to_owned().into(),
                    rejection_reason.into(),
                    reviewer_user_id.to_owned().into(),
                    now.clone().into(),
                    approved_at.into(),
                    review_changes_json.into(),
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
        canonical_key: &str,
        detail_score: i32,
        action: &str,
    ) -> Result<GearAtlasItem, DbErr> {
        let now = now_atlas_rfc3339();
        let specs_json = json_string(&draft.specs)?;
        let variants_json = variants_json(&draft.variants)?;
        let submitted_snapshot_json = snapshot_json(&snapshot_from_external_import(draft))?;
        let tx = self.db.begin().await?;
        tx.execute(statement(
            self.db.get_database_backend(),
            r#"UPDATE gear_atlas_items
               SET category = ?, name = ?, brand = ?, model = ?, description = ?,
                   weight_g = ?, official_price_cents = ?, official_price_currency = ?,
                   variants_json = ?, specs_json = ?, submitted_snapshot_json = ?,
                   review_changes_json = NULL, source_type = ?, submitted_by_user_id = ?,
                   source_user_gear_id = NULL, status = ?, rejection_reason = NULL,
                   reviewed_by_user_id = NULL, reviewed_at = NULL, approved_at = NULL,
                   source_key = ?, source_name = ?, source_url = ?,
                   source_license_note = ?, import_batch_id = ?, imported_at = ?,
                   source_rating_score = ?, source_rating_count = ?, is_deleted = FALSE, updated_at = ?
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
                submitted_snapshot_json.into(),
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
                now.clone().into(),
                id.to_owned().into(),
            ],
        ))
        .await?;
        upsert_external_localizations(&tx, self.db.get_database_backend(), id, draft).await?;
        upsert_import_source(
            &tx,
            self.db.get_database_backend(),
            id,
            draft,
            canonical_key,
            detail_score,
            action,
            &now,
        )
        .await?;
        tx.commit().await?;
        self.get_any(id)
            .await?
            .ok_or_else(|| DbErr::Custom("updated gear atlas import not found".to_owned()))
    }

    async fn find_import_source(
        &self,
        source_key: &str,
    ) -> Result<Option<ImportSourceLookup>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT atlas_item_id FROM gear_atlas_import_sources WHERE source_key = ? LIMIT 1",
                vec![source_key.to_owned().into()],
            ))
            .await?;
        row.map(|row| {
            Ok(ImportSourceLookup {
                atlas_item_id: row.try_get("", "atlas_item_id")?,
            })
        })
        .transpose()
    }

    async fn find_best_import_source(
        &self,
        canonical_key: &str,
    ) -> Result<Option<CanonicalImportMatch>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT atlas_item_id, detail_score FROM gear_atlas_import_sources \
                 WHERE canonical_key = ? ORDER BY detail_score DESC, updated_at DESC LIMIT 1",
                vec![canonical_key.to_owned().into()],
            ))
            .await?;
        row.map(|row| {
            Ok(CanonicalImportMatch {
                atlas_item_id: row.try_get("", "atlas_item_id")?,
                detail_score: row.try_get("", "detail_score")?,
            })
        })
        .transpose()
    }

    async fn record_import_source(
        &self,
        atlas_item_id: &str,
        draft: &GearAtlasExternalImportDraft,
        canonical_key: &str,
        action: &str,
    ) -> Result<(), DbErr> {
        let now = now_atlas_rfc3339();
        upsert_import_source(
            &self.db,
            self.db.get_database_backend(),
            atlas_item_id,
            draft,
            canonical_key,
            detail_score_for_import(draft),
            action,
            &now,
        )
        .await
    }

    /// Soft-deletes a submission or public atlas record.
    pub async fn soft_delete(&self, id: &str) -> Result<bool, DbErr> {
        let now = now_atlas_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_atlas_items SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
                vec![now.into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Restores a previously soft-deleted atlas record.
    pub async fn restore_deleted(&self, id: &str) -> Result<Option<GearAtlasItem>, DbErr> {
        let now = now_atlas_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_atlas_items SET is_deleted = FALSE, updated_at = ? WHERE id = ? AND is_deleted = TRUE",
                vec![now.into(), id.to_owned().into()],
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

fn atlas_values(
    id: &str,
    draft: &GearAtlasDraft,
    variants_json: &str,
    specs_json: &str,
    submitted_snapshot_json: &str,
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
        submitted_snapshot_json.to_owned().into(),
        Option::<String>::None.into(),
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
    submitted_snapshot_json: &str,
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
        submitted_snapshot_json.to_owned().into(),
        Option::<String>::None.into(),
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
        submitted_snapshot_json, review_changes_json,
        source_type, submitted_by_user_id, source_user_gear_id, status,
        rejection_reason, reviewed_by_user_id, reviewed_at, approved_at,
        source_key, source_name, source_url, source_license_note, import_batch_id,
        imported_at, source_rating_score, source_rating_count,
        is_deleted, created_at, updated_at
       FROM gear_atlas_items"#
}

async fn upsert_external_localizations(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
    id: &str,
    draft: &GearAtlasExternalImportDraft,
) -> Result<(), DbErr> {
    if draft.localizations.is_empty() {
        let localization = GearAtlasLocalizationDraft {
            locale: Locale::ZhCn,
            name: draft.name.clone(),
            description: draft.description.clone(),
            variants: draft.variants.clone(),
            specs: draft.specs.clone(),
            translation_status: None,
            translation_provider: None,
            translated_at: None,
        };
        return upsert_localization(db, backend, id, &localization).await;
    }

    for localization in &draft.localizations {
        upsert_localization(db, backend, id, localization).await?;
    }
    Ok(())
}

async fn upsert_localization(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
    id: &str,
    localization: &GearAtlasLocalizationDraft,
) -> Result<(), DbErr> {
    let variants_json = variants_json(&localization.variants)?;
    let specs_json = json_string(&localization.specs)?;
    db.execute(statement(
        backend,
        "INSERT INTO gear_atlas_item_localizations(
             atlas_item_id, locale, name, description, variants_json, specs_json,
             translation_status, translation_provider, translated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(atlas_item_id, locale) DO UPDATE SET
             name = excluded.name,
             description = excluded.description,
             variants_json = excluded.variants_json,
             specs_json = excluded.specs_json,
             translation_status = excluded.translation_status,
             translation_provider = excluded.translation_provider,
             translated_at = excluded.translated_at",
        vec![
            id.to_owned().into(),
            localization.locale.as_str().to_owned().into(),
            localization.name.clone().into(),
            localization.description.clone().into(),
            variants_json.into(),
            specs_json.into(),
            localization.translation_status.clone().into(),
            localization.translation_provider.clone().into(),
            localization.translated_at.clone().into(),
        ],
    ))
    .await?;
    Ok(())
}

fn localization_from_row(row: QueryResult) -> Result<GearAtlasLocalizationDraft, DbErr> {
    let locale_value: String = row.try_get("", "locale")?;
    let Some(locale) = Locale::parse(&locale_value) else {
        return Err(DbErr::Custom(format!(
            "unsupported gear atlas localization locale: {locale_value}"
        )));
    };
    let variants_json: Option<String> = row.try_get("", "variants_json")?;
    let specs_json: Option<String> = row.try_get("", "specs_json")?;
    Ok(GearAtlasLocalizationDraft {
        locale,
        name: row.try_get("", "name")?,
        description: row.try_get("", "description")?,
        variants: variants_json
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| serde_json::from_str::<GearVariants>(value))
            .transpose()
            .map_err(|error| DbErr::Custom(format!("invalid variants_json: {error}")))?
            .unwrap_or_default(),
        specs: specs_json
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| serde_json::from_str::<GearSpecs>(value))
            .transpose()
            .map_err(|error| DbErr::Custom(format!("invalid specs_json: {error}")))?
            .unwrap_or_default(),
        translation_status: row.try_get("", "translation_status")?,
        translation_provider: row.try_get("", "translation_provider")?,
        translated_at: row.try_get("", "translated_at")?,
    })
}

fn localization_review_status(
    item: &GearAtlasItem,
    locale: Locale,
    localization: Option<&GearAtlasLocalizationDraft>,
) -> GearAtlasLocalizationReviewStatus {
    let Some(localization) = localization else {
        let mut missing_fields = vec!["name".to_owned(), "description".to_owned()];
        if !item.variants.is_empty() {
            missing_fields.push("variants".to_owned());
        }
        if !item.specs.is_empty() {
            missing_fields.push("specs".to_owned());
        }
        return GearAtlasLocalizationReviewStatus {
            locale,
            state: GearAtlasLocalizationReviewState::Missing,
            missing_fields,
            translation_status: None,
        };
    };

    let mut missing_fields = Vec::new();
    if localization.name.trim().is_empty() {
        missing_fields.push("name".to_owned());
    }
    if localization
        .description
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        missing_fields.push("description".to_owned());
    }
    if !item.variants.is_empty() && localization.variants.is_empty() {
        missing_fields.push("variants".to_owned());
    }
    if !item.specs.is_empty() && localization.specs.is_empty() {
        missing_fields.push("specs".to_owned());
    }

    let state = if missing_fields.is_empty()
        && localization.translation_status.as_deref()
            == Some(GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED)
    {
        GearAtlasLocalizationReviewState::Reviewed
    } else if localization.translation_status.as_deref()
        == Some(GEAR_ATLAS_LOCALIZATION_STATUS_DRAFT)
    {
        GearAtlasLocalizationReviewState::Draft
    } else {
        GearAtlasLocalizationReviewState::NeedsReview
    };

    GearAtlasLocalizationReviewStatus {
        locale,
        state,
        missing_fields,
        translation_status: localization.translation_status.clone(),
    }
}

async fn upsert_import_source(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
    atlas_item_id: &str,
    draft: &GearAtlasExternalImportDraft,
    canonical_key: &str,
    detail_score: i32,
    action: &str,
    now: &str,
) -> Result<(), DbErr> {
    let source_locale = draft.source_locale.unwrap_or(Locale::ZhCn);
    db.execute(statement(
        backend,
        "INSERT INTO gear_atlas_import_sources(
             source_key, canonical_key, atlas_item_id, source_name, source_url, source_locale,
             detail_score, last_seen_batch_id, last_seen_at, last_action, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(source_key) DO UPDATE SET
             canonical_key = excluded.canonical_key,
             atlas_item_id = excluded.atlas_item_id,
             source_name = excluded.source_name,
             source_url = excluded.source_url,
             source_locale = excluded.source_locale,
             detail_score = excluded.detail_score,
             last_seen_batch_id = excluded.last_seen_batch_id,
             last_seen_at = excluded.last_seen_at,
             last_action = excluded.last_action,
             updated_at = excluded.updated_at",
        vec![
            draft.source_key.clone().into(),
            canonical_key.to_owned().into(),
            atlas_item_id.to_owned().into(),
            draft.source_name.clone().into(),
            draft.source_url.clone().into(),
            source_locale.as_str().to_owned().into(),
            detail_score.into(),
            draft.import_batch_id.clone().into(),
            now.to_owned().into(),
            action.to_owned().into(),
            now.to_owned().into(),
            now.to_owned().into(),
        ],
    ))
    .await?;
    Ok(())
}

fn canonical_key_for_import(draft: &GearAtlasExternalImportDraft) -> String {
    if let Some(key) = draft.canonical_key.as_deref() {
        return key.to_owned();
    }
    let mut input = format!(
        "{}|{}|{}|{}|{}",
        draft.category.as_str(),
        canonical_part(draft.brand.as_deref()),
        canonical_part(draft.model.as_deref()),
        canonical_part(Some(&draft.name)),
        draft.weight_g.unwrap_or_default(),
    );
    for key in ["capacity", "people_count", "temperature_or_r_value", "type"] {
        if let Some(value) = draft.specs.get(key) {
            input.push('|');
            input.push_str(key);
            input.push('=');
            input.push_str(&canonical_part(Some(value)));
        }
    }
    let digest = Sha256::digest(input.as_bytes());
    format!("external-gear:v1:{}", hex::encode(&digest[..16]))
}

fn canonical_part(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn detail_score_for_import(draft: &GearAtlasExternalImportDraft) -> i32 {
    if let Some(score) = draft.detail_score {
        return score;
    }
    let mut score = 0;
    score += 10;
    if draft.brand.is_some() {
        score += 8;
    }
    if draft.model.is_some() {
        score += 8;
    }
    if draft.description.is_some() {
        score += 3;
    }
    if draft.weight_g.is_some() {
        score += 10;
    }
    if draft.official_price_cents.is_some() {
        score += 6;
    }
    if draft.source_rating_score.is_some() {
        score += 4;
    }
    if draft.source_rating_count.is_some() {
        score += 4;
    }
    score += (draft.specs.len() as i32) * 5;
    score += (draft.variants.len() as i32) * 3;
    score += (draft.localizations.len() as i32) * 5;
    score
}

fn apply_deleted_filter(clauses: &mut Vec<String>, deleted: DeletedFilter) {
    match deleted {
        DeletedFilter::Active => clauses.push("is_deleted = FALSE".to_owned()),
        DeletedFilter::Deleted => clauses.push("is_deleted = TRUE".to_owned()),
        DeletedFilter::All => {}
    }
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

fn apply_public_search_filter(clauses: &mut Vec<String>, values: &mut Vec<Value>, q: Option<&str>) {
    let Some(q) = normalize_query(q) else {
        return;
    };
    clauses.push(
        "(LOWER(name) LIKE ? \
          OR LOWER(COALESCE(description, '')) LIKE ? \
          OR LOWER(COALESCE(brand, '')) LIKE ? \
          OR LOWER(COALESCE(model, '')) LIKE ? \
          OR EXISTS ( \
              SELECT 1 FROM gear_atlas_item_localizations l \
              WHERE l.atlas_item_id = gear_atlas_items.id \
                AND (LOWER(l.name) LIKE ? OR LOWER(COALESCE(l.description, '')) LIKE ?) \
          ) \
          OR EXISTS ( \
              SELECT 1 FROM gear_category_localizations c \
              WHERE c.category = gear_atlas_items.category \
                AND LOWER(c.label) LIKE ? \
          ))"
        .to_owned(),
    );
    for _ in 0..7 {
        values.push(q.clone().into());
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

fn json_string<T: serde::Serialize>(value: &T) -> Result<String, DbErr> {
    serde_json::to_string(value).map_err(|err| DbErr::Custom(err.to_string()))
}

fn variants_json(variants: &GearVariants) -> Result<String, DbErr> {
    serde_json::to_string(variants).map_err(|err| DbErr::Custom(err.to_string()))
}

fn snapshot_json(snapshot: &GearAtlasPublicSnapshot) -> Result<String, DbErr> {
    json_string(snapshot)
}

fn snapshot_from_external_import(draft: &GearAtlasExternalImportDraft) -> GearAtlasPublicSnapshot {
    GearAtlasPublicSnapshot {
        category: draft.category,
        name: draft.name.clone(),
        brand: draft.brand.clone(),
        model: draft.model.clone(),
        description: draft.description.clone(),
        weight_g: draft.weight_g,
        official_price_cents: draft.official_price_cents,
        official_price_currency: draft.official_price_currency.clone(),
        variants: draft.variants.clone(),
        specs: draft.specs.clone(),
    }
}

fn map_atlas_item(row: &sea_orm::QueryResult) -> Result<GearAtlasItem, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let source_type_raw: String = row.try_get("", "source_type")?;
    let status_raw: String = row.try_get("", "status")?;
    let specs_json: String = row.try_get("", "specs_json")?;
    let variants_json: String = row.try_get("", "variants_json")?;
    let submitted_snapshot_json: Option<String> = row.try_get("", "submitted_snapshot_json")?;
    let review_changes_json: Option<String> = row.try_get("", "review_changes_json")?;
    let specs: GearSpecs = serde_json::from_str(&specs_json).unwrap_or_default();
    let variants: GearVariants = serde_json::from_str(&variants_json).unwrap_or_default();
    let category = GearCategory::from_key(&category_raw)
        .ok_or_else(|| DbErr::Custom(format!("invalid gear atlas category: {category_raw}")))?;
    let name: String = row.try_get("", "name")?;
    let brand: Option<String> = row.try_get("", "brand")?;
    let model: Option<String> = row.try_get("", "model")?;
    let description: Option<String> = row.try_get("", "description")?;
    let weight_g: Option<i32> = row.try_get("", "weight_g")?;
    let official_price_cents: Option<i64> = row.try_get("", "official_price_cents")?;
    let official_price_currency: Option<String> = row.try_get("", "official_price_currency")?;
    let submitted_snapshot = submitted_snapshot_json
        .as_deref()
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_else(|| GearAtlasPublicSnapshot {
            category,
            name: name.clone(),
            brand: brand.clone(),
            model: model.clone(),
            description: description.clone(),
            weight_g,
            official_price_cents,
            official_price_currency: official_price_currency.clone(),
            variants: variants.clone(),
            specs: specs.clone(),
        });
    let review_changes: GearAtlasReviewChanges = review_changes_json
        .as_deref()
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or_default();
    Ok(GearAtlasItem {
        id: row.try_get("", "id")?,
        category,
        name,
        brand,
        model,
        description,
        weight_g,
        official_price_cents,
        official_price_currency,
        variants,
        specs,
        submitted_snapshot,
        review_changes,
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
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}
