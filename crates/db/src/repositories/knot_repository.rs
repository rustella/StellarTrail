//! Knot repository for DB-backed outdoor skill metadata imported from Knots3D.

use std::collections::{HashMap, HashSet};

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};
use stellartrail_domain::skill::{
    KnotDetail, KnotFilterOption, KnotFiltersResponse, KnotListResponse, KnotMediaAsset,
    KnotOfflineManifestResponse, KnotSeed, KnotSummary, KnotTaxonomyItem, Locale, PageInfo,
    SkillCategorySummary,
};

use super::{MediaResourceRepository, statement};

const API_PREFIX: &str = "/api/v1";

/// Grouping key for taxonomy rows that belong to the same knot and taxonomy item.
type KnotTaxonomyCandidateKey = (String, String);

/// Localized taxonomy row collected before locale fallback selection.
struct LocalizedKnotTaxonomyCandidate {
    locale: Locale,
    slug: String,
    title: String,
}

/// Intermediate taxonomy lookup used to select the best localized item per knot.
type KnotTaxonomyCandidates =
    HashMap<KnotTaxonomyCandidateKey, Vec<LocalizedKnotTaxonomyCandidate>>;

/// Persistence object for skill categories and knot metadata.
#[derive(Clone)]
pub struct KnotRepository {
    db: DatabaseConnection,
}

impl KnotRepository {
    /// Creates a repository using the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Replaces all imported knots in a single transaction.
    pub async fn replace_all_knots(
        &self,
        import_source: &str,
        knots: &[KnotSeed],
    ) -> Result<(), DbErr> {
        let backend = self.db.get_database_backend();
        let tx = self.db.begin().await?;
        for table in [
            "knot_raw_metadata",
            "knot_media_assets",
            "knot_type_memberships",
            "knot_category_memberships",
            "knot_type_localizations",
            "knot_types",
            "knot_category_localizations",
            "knot_categories",
            "knot_localizations",
            "knots",
        ] {
            tx.execute(statement(backend, format!("DELETE FROM {table}"), vec![]))
                .await?;
        }
        tx.execute(statement(
            backend,
            "INSERT INTO knot_import_runs(source, item_count) VALUES (?, ?)",
            vec![import_source.to_owned().into(), (knots.len() as i64).into()],
        ))
        .await?;

        for knot in knots {
            tx.execute(statement(
                backend,
                "INSERT INTO knots(id, source_name, source_url, source_slug_en, source_slug_zh, difficulty) \
                 VALUES (?, ?, ?, ?, ?, ?)",
                vec![
                    knot.id.clone().into(),
                    knot.source_name.clone().into(),
                    knot.source_url.clone().into(),
                    knot.source_slug_en.clone().into(),
                    knot.source_slug_zh.clone().into(),
                    knot.difficulty.clone().into(),
                ],
            ))
            .await?;

            for localization in &knot.localizations {
                let steps_json = serde_json::to_string(&localization.steps)
                    .map_err(|err| DbErr::Custom(err.to_string()))?;
                tx.execute(statement(
                    backend,
                    "INSERT INTO knot_localizations(knot_id, locale, slug, title, summary, description, steps_json) \
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    vec![
                        knot.id.clone().into(),
                        localization.locale.as_str().to_owned().into(),
                        localization.slug.clone().into(),
                        localization.title.clone().into(),
                        localization.summary.clone().into(),
                        localization.description.clone().into(),
                        steps_json.into(),
                    ],
                ))
                .await?;
            }

            for category in &knot.categories {
                tx.execute(statement(
                    backend,
                    insert_ignore_sql("knot_categories", "id"),
                    vec![category.id.clone().into()],
                ))
                .await?;
                for (locale, slug, title) in &category.localizations {
                    tx.execute(statement(
                        backend,
                        "INSERT INTO knot_category_localizations(category_id, locale, slug, title) \
                         VALUES (?, ?, ?, ?) ON CONFLICT(category_id, locale) DO UPDATE SET slug = excluded.slug, title = excluded.title",
                        vec![
                            category.id.clone().into(),
                            locale.as_str().to_owned().into(),
                            slug.clone().into(),
                            title.clone().into(),
                        ],
                    ))
                    .await?;
                }
                tx.execute(statement(
                    backend,
                    "INSERT INTO knot_category_memberships(knot_id, category_id) VALUES (?, ?) ON CONFLICT(knot_id, category_id) DO NOTHING",
                    vec![knot.id.clone().into(), category.id.clone().into()],
                ))
                .await?;
            }

            for knot_type in &knot.types {
                tx.execute(statement(
                    backend,
                    insert_ignore_sql("knot_types", "id"),
                    vec![knot_type.id.clone().into()],
                ))
                .await?;
                for (locale, slug, title) in &knot_type.localizations {
                    tx.execute(statement(
                        backend,
                        "INSERT INTO knot_type_localizations(type_id, locale, slug, title) \
                         VALUES (?, ?, ?, ?) ON CONFLICT(type_id, locale) DO UPDATE SET slug = excluded.slug, title = excluded.title",
                        vec![
                            knot_type.id.clone().into(),
                            locale.as_str().to_owned().into(),
                            slug.clone().into(),
                            title.clone().into(),
                        ],
                    ))
                    .await?;
                }
                tx.execute(statement(
                    backend,
                    "INSERT INTO knot_type_memberships(knot_id, type_id) VALUES (?, ?) ON CONFLICT(knot_id, type_id) DO NOTHING",
                    vec![knot.id.clone().into(), knot_type.id.clone().into()],
                ))
                .await?;
            }

            for media in &knot.media {
                tx.execute(statement(
                    backend,
                    "INSERT INTO knot_media_assets(knot_id, asset_id, media_type, path, mime_type, width, height, attribution, license_note) \
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    vec![
                        knot.id.clone().into(),
                        media.id.clone().into(),
                        media.media_type.clone().into(),
                        media.path.clone().into(),
                        media.mime_type.clone().into(),
                        media.width.into(),
                        media.height.into(),
                        media.attribution.clone().into(),
                        media.license_note.clone().into(),
                    ],
                ))
                .await?;
            }

            let raw_json = serde_json::to_string(&knot.raw_metadata)
                .map_err(|err| DbErr::Custom(err.to_string()))?;
            tx.execute(statement(
                backend,
                "INSERT INTO knot_raw_metadata(knot_id, raw_json) VALUES (?, ?)",
                vec![knot.id.clone().into(), raw_json.into()],
            ))
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Lists outdoor skill categories. Phase 1 has only the knots category.
    pub async fn list_skill_categories(
        &self,
        locale: Locale,
    ) -> Result<Vec<SkillCategorySummary>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM knots",
                vec![],
            ))
            .await?;
        let count = row
            .map(|row| row.try_get::<i64>("", "count"))
            .transpose()?
            .unwrap_or_default();
        let (title, summary) = match locale {
            Locale::ZhCn => ("绳结", "户外、露营、钓鱼、航海等场景常用绳结技能。"),
            Locale::En => (
                "Knots",
                "Outdoor knots for camping, fishing, sailing, and field skills.",
            ),
        };
        Ok(vec![SkillCategorySummary {
            id: "knots".to_owned(),
            slug: "knots".to_owned(),
            title: title.to_owned(),
            summary: summary.to_owned(),
            item_count: count.max(0) as u32,
            href: format!("{API_PREFIX}/skills/knots/list"),
        }])
    }

    /// Lists available knot filter options with locale-resolved labels and full-catalog counts.
    pub async fn list_knot_filters(&self, locale: Locale) -> Result<KnotFiltersResponse, DbErr> {
        let backend = self.db.get_database_backend();
        let category_rows = self
            .db
            .query_all(statement(
                backend,
                "SELECT kc.id AS id, COUNT(kcm.knot_id) AS count \
                 FROM knot_categories kc \
                 JOIN knot_category_memberships kcm ON kcm.category_id = kc.id \
                 GROUP BY kc.id ORDER BY kc.id ASC",
                vec![],
            ))
            .await?;
        let mut categories = Vec::with_capacity(category_rows.len());
        for row in category_rows {
            let id: String = row.try_get("", "id")?;
            let count: i64 = row.try_get("", "count")?;
            categories.push(fetch_category_filter_option(&self.db, &id, count, locale).await?);
        }

        let difficulty_rows = self
            .db
            .query_all(statement(
                backend,
                "SELECT difficulty AS id, COUNT(*) AS count FROM knots \
                 WHERE difficulty IS NOT NULL AND TRIM(difficulty) <> '' \
                 GROUP BY difficulty \
                 ORDER BY CASE difficulty \
                    WHEN 'leisure' THEN 0 \
                    WHEN 'beginner' THEN 1 \
                    WHEN 'intermediate' THEN 2 \
                    WHEN 'advanced' THEN 3 \
                    WHEN 'technical' THEN 4 \
                    ELSE 100 \
                 END ASC, difficulty ASC",
                vec![],
            ))
            .await?;
        let difficulties = difficulty_rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("", "id")?;
                let count: i64 = row.try_get("", "count")?;
                Ok(KnotFilterOption {
                    title: difficulty_label(&id, locale).to_owned(),
                    id,
                    slug: None,
                    count: count.max(0) as u32,
                })
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(KnotFiltersResponse {
            locale,
            categories,
            difficulties,
        })
    }

    /// Lists knots with offset pagination and locale-resolved public fields.
    pub async fn list_knots(
        &self,
        locale: Locale,
        offset: u32,
        limit: u32,
        category: Option<&str>,
        difficulty: Option<&str>,
        q: Option<&str>,
    ) -> Result<KnotListResponse, DbErr> {
        let backend = self.db.get_database_backend();
        let rows = if let Some(category) = category {
            self.db
                .query_all(statement(
                    backend,
                    "SELECT k.id AS id FROM knots k \
                     JOIN knot_category_memberships kcm ON kcm.knot_id = k.id \
                     WHERE kcm.category_id = ? ORDER BY k.id ASC",
                    vec![category.to_owned().into()],
                ))
                .await?
        } else {
            self.db
                .query_all(statement(
                    backend,
                    "SELECT id AS id FROM knots ORDER BY id ASC",
                    vec![],
                ))
                .await?
        };
        let ids = rows
            .into_iter()
            .map(|row| row.try_get::<String>("", "id"))
            .collect::<Result<Vec<_>, _>>()?;

        let difficulty = difficulty.map(str::trim).filter(|value| !value.is_empty());
        let query = q
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let mut summaries = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(detail) = self.detail(&id, locale).await? {
                let matches_query = match query.as_ref() {
                    Some(needle) => {
                        detail.title.to_lowercase().contains(needle)
                            || detail.summary.to_lowercase().contains(needle)
                            || detail.id.to_lowercase().contains(needle)
                    }
                    None => true,
                };
                let matches_difficulty = match difficulty {
                    Some(value) => detail.difficulty.as_deref() == Some(value),
                    None => true,
                };
                if matches_query && matches_difficulty {
                    summaries.push(KnotSummary {
                        id: detail.id.clone(),
                        slug: detail.slug.clone(),
                        title: detail.title.clone(),
                        summary: detail.summary.clone(),
                        difficulty: detail.difficulty.clone(),
                        categories: detail.categories.clone(),
                        types: detail.types.clone(),
                        media: detail.media.clone(),
                        href: format!("{API_PREFIX}/skills/knots/detail/{}", detail.id),
                    });
                }
            }
        }

        let limit = limit.clamp(1, 100);
        let start = offset as usize;
        let end = (start + limit as usize).min(summaries.len());
        let items = if start >= summaries.len() {
            Vec::new()
        } else {
            summaries[start..end].to_vec()
        };
        let next_offset = if end < summaries.len() {
            Some(end as u32)
        } else {
            None
        };

        Ok(KnotListResponse {
            locale,
            items,
            page: PageInfo {
                limit,
                offset,
                next_offset,
            },
        })
    }

    /// Returns one knot detail by canonical id.
    pub async fn get_knot_detail(
        &self,
        id: &str,
        locale: Locale,
    ) -> Result<Option<KnotDetail>, DbErr> {
        self.detail(id, locale).await
    }

    /// Returns the complete public knot detail set for offline client pre-cache flows.
    pub async fn offline_manifest(
        &self,
        locale: Locale,
    ) -> Result<KnotOfflineManifestResponse, DbErr> {
        let backend = self.db.get_database_backend();
        let knot_rows = self
            .db
            .query_all(statement(
                backend,
                "SELECT id, difficulty FROM knots ORDER BY id ASC",
                vec![],
            ))
            .await?;
        let knots = knot_rows
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get::<String>("", "id")?,
                    row.try_get::<Option<String>>("", "difficulty")?,
                ))
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        let localizations = fetch_all_knot_localizations(&self.db, locale).await?;
        let categories = fetch_all_taxonomy(
            &self.db,
            locale,
            "knot_category_memberships",
            "category_id",
            "knot_category_localizations",
        )
        .await?;
        let types = fetch_all_taxonomy(
            &self.db,
            locale,
            "knot_type_memberships",
            "type_id",
            "knot_type_localizations",
        )
        .await?;
        let media = fetch_all_media(&self.db).await?;

        let mut items = Vec::with_capacity(knots.len());
        let mut seen_media_urls = HashSet::new();
        let mut media_count = 0u32;
        let mut estimated_bytes = 0i64;
        for (id, difficulty) in knots {
            let Some(localization) = localizations.get(&id) else {
                continue;
            };
            let item_media = media.get(&id).cloned().unwrap_or_default();
            for asset in &item_media {
                if seen_media_urls.insert(asset.url.clone()) {
                    media_count += 1;
                    estimated_bytes += asset.size_bytes.max(0);
                }
            }
            items.push(KnotDetail {
                id: id.clone(),
                slug: localization.slug.clone(),
                title: localization.title.clone(),
                summary: localization.summary.clone(),
                description: localization.description.clone(),
                steps: localization.steps.clone(),
                difficulty,
                categories: categories.get(&id).cloned().unwrap_or_default(),
                types: types.get(&id).cloned().unwrap_or_default(),
                media: item_media,
                locale,
            });
        }

        Ok(KnotOfflineManifestResponse {
            locale,
            item_count: items.len() as u32,
            media_count,
            estimated_bytes,
            items,
        })
    }

    async fn detail(&self, id: &str, locale: Locale) -> Result<Option<KnotDetail>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT difficulty FROM knots WHERE id = ?",
                vec![id.to_owned().into()],
            ))
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let difficulty = row.try_get::<Option<String>>("", "difficulty")?;
        let Some(localization) = fetch_knot_localization(&self.db, id, locale).await? else {
            return Ok(None);
        };
        Ok(Some(KnotDetail {
            id: id.to_owned(),
            slug: localization.slug,
            title: localization.title,
            summary: localization.summary,
            description: localization.description,
            steps: localization.steps,
            difficulty,
            categories: fetch_categories(&self.db, id, locale).await?,
            types: fetch_types(&self.db, id, locale).await?,
            media: fetch_media(&self.db, id).await?,
            locale,
        }))
    }
}

#[derive(Clone)]
struct KnotLocalizationRow {
    slug: String,
    title: String,
    summary: String,
    description: Option<String>,
    steps: Vec<String>,
}

async fn fetch_knot_localization(
    db: &DatabaseConnection,
    id: &str,
    locale: Locale,
) -> Result<Option<KnotLocalizationRow>, DbErr> {
    for candidate in locale.fallbacks() {
        let row = db
            .query_one(statement(
                db.get_database_backend(),
                "SELECT slug, title, summary, description, steps_json \
                 FROM knot_localizations WHERE knot_id = ? AND locale = ?",
                vec![id.to_owned().into(), candidate.as_str().to_owned().into()],
            ))
            .await?;
        if let Some(row) = row {
            let steps_json: String = row.try_get("", "steps_json")?;
            let steps = serde_json::from_str::<Vec<String>>(&steps_json)
                .map_err(|err| DbErr::Custom(err.to_string()))?;
            return Ok(Some(KnotLocalizationRow {
                slug: row.try_get("", "slug")?,
                title: row.try_get("", "title")?,
                summary: row.try_get("", "summary")?,
                description: row.try_get("", "description")?,
                steps,
            }));
        }
    }
    Ok(None)
}

async fn fetch_all_knot_localizations(
    db: &DatabaseConnection,
    locale: Locale,
) -> Result<HashMap<String, KnotLocalizationRow>, DbErr> {
    let fallbacks = locale.fallbacks();
    let rows = db
        .query_all(statement(
            db.get_database_backend(),
            "SELECT knot_id, locale, slug, title, summary, description, steps_json \
             FROM knot_localizations WHERE locale IN (?, ?) ORDER BY knot_id ASC",
            vec![
                fallbacks[0].as_str().to_owned().into(),
                fallbacks[1].as_str().to_owned().into(),
            ],
        ))
        .await?;
    let mut candidates: HashMap<String, Vec<(Locale, KnotLocalizationRow)>> = HashMap::new();
    for row in rows {
        let raw_locale: String = row.try_get("", "locale")?;
        let Some(row_locale) = Locale::parse(&raw_locale) else {
            continue;
        };
        let steps_json: String = row.try_get("", "steps_json")?;
        let steps = serde_json::from_str::<Vec<String>>(&steps_json)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        candidates
            .entry(row.try_get("", "knot_id")?)
            .or_default()
            .push((
                row_locale,
                KnotLocalizationRow {
                    slug: row.try_get("", "slug")?,
                    title: row.try_get("", "title")?,
                    summary: row.try_get("", "summary")?,
                    description: row.try_get("", "description")?,
                    steps,
                },
            ));
    }

    let mut selected = HashMap::with_capacity(candidates.len());
    for (knot_id, rows) in candidates {
        for fallback in fallbacks {
            if let Some((_, row)) = rows
                .iter()
                .find(|(candidate_locale, _)| *candidate_locale == fallback)
            {
                selected.insert(knot_id, row.clone());
                break;
            }
        }
    }
    Ok(selected)
}

async fn fetch_category_filter_option(
    db: &DatabaseConnection,
    id: &str,
    count: i64,
    locale: Locale,
) -> Result<KnotFilterOption, DbErr> {
    for candidate in locale.fallbacks() {
        let row = db
            .query_one(statement(
                db.get_database_backend(),
                "SELECT slug, title FROM knot_category_localizations WHERE category_id = ? AND locale = ?",
                vec![id.to_owned().into(), candidate.as_str().to_owned().into()],
            ))
            .await?;
        if let Some(row) = row {
            return Ok(KnotFilterOption {
                id: id.to_owned(),
                slug: Some(row.try_get("", "slug")?),
                title: row.try_get("", "title")?,
                count: count.max(0) as u32,
            });
        }
    }
    Ok(KnotFilterOption {
        id: id.to_owned(),
        slug: None,
        title: id.to_owned(),
        count: count.max(0) as u32,
    })
}

fn difficulty_label(value: &str, locale: Locale) -> &'static str {
    match (locale, value) {
        (Locale::ZhCn, "leisure") => "入门",
        (Locale::ZhCn, "beginner") => "新手",
        (Locale::ZhCn, "intermediate") => "进阶",
        (Locale::ZhCn, "advanced") => "熟练",
        (Locale::ZhCn, "technical") => "技术",
        (Locale::En, "leisure") => "Leisure",
        (Locale::En, "beginner") => "Beginner",
        (Locale::En, "intermediate") => "Intermediate",
        (Locale::En, "advanced") => "Advanced",
        (Locale::En, "technical") => "Technical",
        _ => "Common",
    }
}

async fn fetch_categories(
    db: &DatabaseConnection,
    knot_id: &str,
    locale: Locale,
) -> Result<Vec<KnotTaxonomyItem>, DbErr> {
    fetch_taxonomy(
        db,
        knot_id,
        locale,
        "knot_category_memberships",
        "category_id",
        "knot_category_localizations",
    )
    .await
}

async fn fetch_types(
    db: &DatabaseConnection,
    knot_id: &str,
    locale: Locale,
) -> Result<Vec<KnotTaxonomyItem>, DbErr> {
    fetch_taxonomy(
        db,
        knot_id,
        locale,
        "knot_type_memberships",
        "type_id",
        "knot_type_localizations",
    )
    .await
}

async fn fetch_taxonomy(
    db: &DatabaseConnection,
    knot_id: &str,
    locale: Locale,
    membership_table: &str,
    item_id_col: &str,
    localization_table: &str,
) -> Result<Vec<KnotTaxonomyItem>, DbErr> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT {item_id_col} AS item_id FROM {membership_table} WHERE knot_id = ? ORDER BY {item_id_col} ASC"
    );
    let rows = db
        .query_all(statement(backend, sql, vec![knot_id.to_owned().into()]))
        .await?;
    let ids = rows
        .into_iter()
        .map(|row| row.try_get::<String>("", "item_id"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut items = Vec::new();
    for id in ids {
        for candidate in locale.fallbacks() {
            let sql = format!(
                "SELECT slug, title FROM {localization_table} WHERE {item_id_col} = ? AND locale = ?"
            );
            let row = db
                .query_one(statement(
                    backend,
                    sql,
                    vec![id.clone().into(), candidate.as_str().to_owned().into()],
                ))
                .await?;
            if let Some(row) = row {
                items.push(KnotTaxonomyItem {
                    id: id.clone(),
                    slug: row.try_get("", "slug")?,
                    title: row.try_get("", "title")?,
                });
                break;
            }
        }
    }
    Ok(items)
}

async fn fetch_all_taxonomy(
    db: &DatabaseConnection,
    locale: Locale,
    membership_table: &str,
    item_id_col: &str,
    localization_table: &str,
) -> Result<HashMap<String, Vec<KnotTaxonomyItem>>, DbErr> {
    let fallbacks = locale.fallbacks();
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT m.knot_id, m.{item_id_col} AS item_id, l.locale, l.slug, l.title \
         FROM {membership_table} m \
         JOIN {localization_table} l ON l.{item_id_col} = m.{item_id_col} \
         WHERE l.locale IN (?, ?) \
         ORDER BY m.knot_id ASC, m.{item_id_col} ASC"
    );
    let rows = db
        .query_all(statement(
            backend,
            sql,
            vec![
                fallbacks[0].as_str().to_owned().into(),
                fallbacks[1].as_str().to_owned().into(),
            ],
        ))
        .await?;

    let mut candidates: KnotTaxonomyCandidates = HashMap::new();
    for row in rows {
        let raw_locale: String = row.try_get("", "locale")?;
        let Some(row_locale) = Locale::parse(&raw_locale) else {
            continue;
        };
        candidates
            .entry((row.try_get("", "knot_id")?, row.try_get("", "item_id")?))
            .or_default()
            .push(LocalizedKnotTaxonomyCandidate {
                locale: row_locale,
                slug: row.try_get("", "slug")?,
                title: row.try_get("", "title")?,
            });
    }

    let mut keys = candidates.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    let mut grouped: HashMap<String, Vec<KnotTaxonomyItem>> = HashMap::new();
    for (knot_id, item_id) in keys {
        let Some(rows) = candidates.get(&(knot_id.clone(), item_id.clone())) else {
            continue;
        };
        for fallback in fallbacks {
            if let Some(candidate) = rows.iter().find(|candidate| candidate.locale == fallback) {
                grouped
                    .entry(knot_id.clone())
                    .or_default()
                    .push(KnotTaxonomyItem {
                        id: item_id.clone(),
                        slug: candidate.slug.clone(),
                        title: candidate.title.clone(),
                    });
                break;
            }
        }
    }
    Ok(grouped)
}

async fn fetch_media(db: &DatabaseConnection, knot_id: &str) -> Result<Vec<KnotMediaAsset>, DbErr> {
    MediaResourceRepository::new(db.clone())
        .list_knot_media_assets(knot_id)
        .await
}

async fn fetch_all_media(
    db: &DatabaseConnection,
) -> Result<HashMap<String, Vec<KnotMediaAsset>>, DbErr> {
    let rows = db
        .query_all(statement(
            db.get_database_backend(),
            r#"SELECT kmr.knot_id, kmr.asset_id, kmr.media_type, mr.public_url, mr.mime_type, mr.width, mr.height,
                      mr.size_bytes, kmr.attribution, kmr.license_note
               FROM knot_media_resources kmr
               JOIN media_resources mr ON mr.id = kmr.media_resource_id
               WHERE mr.status = 'active'
               ORDER BY kmr.knot_id ASC,
                   CASE kmr.asset_id
                       WHEN 'thumbnail' THEN 0
                       WHEN 'preview' THEN 1
                       WHEN 'draw_gif' THEN 2
                       WHEN 'turntable_gif' THEN 3
                       WHEN 'draw_mp4' THEN 4
                       WHEN 'turntable_mp4' THEN 5
                       ELSE 1000 + kmr.sort_order
                   END ASC, kmr.sort_order ASC, kmr.asset_id ASC"#,
            vec![],
        ))
        .await?;
    let mut grouped: HashMap<String, Vec<KnotMediaAsset>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.try_get("", "knot_id")?)
            .or_default()
            .push(KnotMediaAsset {
                id: row.try_get("", "asset_id")?,
                media_type: row.try_get("", "media_type")?,
                url: row.try_get("", "public_url")?,
                mime_type: row.try_get("", "mime_type")?,
                width: row.try_get("", "width")?,
                height: row.try_get("", "height")?,
                size_bytes: row.try_get("", "size_bytes")?,
                attribution: row.try_get("", "attribution")?,
                license_note: row.try_get("", "license_note")?,
            });
    }
    Ok(grouped)
}

fn insert_ignore_sql(table: &str, column: &str) -> String {
    format!("INSERT INTO {table}({column}) VALUES (?) ON CONFLICT({column}) DO NOTHING")
}
