//! Knot repository for DB-backed outdoor skill metadata imported from Knots3D.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, TransactionTrait};
use stellartrail_domain::skill::{
    KnotDetail, KnotListResponse, KnotMediaAsset, KnotSeed, KnotSummary, KnotTaxonomyItem, Locale,
    PageInfo, SkillCategorySummary,
};

use super::statement;

/// Persistence object for skill categories and knot metadata.
#[derive(Clone)]
pub struct KnotRepository {
    db: DatabaseConnection,
    media_base_url: String,
}

impl KnotRepository {
    /// Creates a repository using the shared application database connection.
    pub fn new(db: DatabaseConnection, media_base_url: impl Into<String>) -> Self {
        Self {
            db,
            media_base_url: normalize_media_base(media_base_url.into()),
        }
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
            href: "/api/skills/knots/list".to_owned(),
        }])
    }

    /// Lists knots with offset pagination and locale-resolved public fields.
    pub async fn list_knots(
        &self,
        locale: Locale,
        offset: u32,
        limit: u32,
        category: Option<&str>,
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
                if matches_query {
                    summaries.push(KnotSummary {
                        id: detail.id.clone(),
                        slug: detail.slug.clone(),
                        title: detail.title.clone(),
                        summary: detail.summary.clone(),
                        difficulty: detail.difficulty.clone(),
                        categories: detail.categories.clone(),
                        types: detail.types.clone(),
                        media: detail.media.clone(),
                        href: format!("/api/skills/knots/detail/{}", detail.id),
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
            media: fetch_media(&self.db, id, &self.media_base_url).await?,
            locale,
        }))
    }
}

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

async fn fetch_media(
    db: &DatabaseConnection,
    knot_id: &str,
    media_base_url: &str,
) -> Result<Vec<KnotMediaAsset>, DbErr> {
    let rows = db
        .query_all(statement(
            db.get_database_backend(),
            "SELECT asset_id, media_type, path, mime_type, width, height, attribution, license_note \
             FROM knot_media_assets WHERE knot_id = ? ORDER BY id ASC",
            vec![knot_id.to_owned().into()],
        ))
        .await?;
    rows.into_iter()
        .map(|row| map_media(row, media_base_url))
        .collect()
}

fn map_media(row: QueryResult, media_base_url: &str) -> Result<KnotMediaAsset, DbErr> {
    let path: String = row.try_get("", "path")?;
    Ok(KnotMediaAsset {
        id: row.try_get("", "asset_id")?,
        media_type: row.try_get("", "media_type")?,
        url: format!(
            "{}/{}",
            media_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        ),
        mime_type: row.try_get("", "mime_type")?,
        width: row.try_get("", "width")?,
        height: row.try_get("", "height")?,
        attribution: row.try_get("", "attribution")?,
        license_note: row.try_get("", "license_note")?,
    })
}

fn normalize_media_base(media_base_url: String) -> String {
    let trimmed = media_base_url.trim();
    if trimmed.is_empty() {
        "/assets".to_owned()
    } else if trimmed == "/" {
        String::new()
    } else {
        trimmed.trim_end_matches('/').to_owned()
    }
}

fn insert_ignore_sql(table: &str, column: &str) -> String {
    format!("INSERT INTO {table}({column}) VALUES (?) ON CONFLICT({column}) DO NOTHING")
}
