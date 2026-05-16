pub mod config;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rusqlite::{params, Connection, OptionalExtension};
use stellartrail_domain::skill::{
    KnotDetail, KnotListResponse, KnotMediaAsset, KnotSeed, KnotSummary, KnotTaxonomyItem, Locale,
    PageInfo, SkillCategorySummary,
};

pub use config::{DatabaseConfig, DatabaseConfigError, DatabaseKind};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database kind {0:?} is not supported for knot storage yet")]
    UnsupportedDatabase(DatabaseKind),
    #[error("sqlite path is empty")]
    EmptySqlitePath,
    #[error("failed to create database directory {path}: {source}")]
    CreateDatabaseDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("database mutex was poisoned")]
    Poisoned,
}

#[derive(Clone)]
pub struct KnotRepository {
    connection: Arc<Mutex<Connection>>,
    media_base_url: Arc<String>,
}

impl KnotRepository {
    pub fn connect(config: &DatabaseConfig) -> Result<Self, DbError> {
        Self::connect_with_media_base_url(config, "/assets")
    }

    pub fn connect_with_media_base_url(
        config: &DatabaseConfig,
        media_base_url: impl Into<String>,
    ) -> Result<Self, DbError> {
        if config.kind != DatabaseKind::Sqlite {
            return Err(DbError::UnsupportedDatabase(config.kind));
        }
        let path = sqlite_path(&config.url)?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|source| DbError::CreateDatabaseDirectory {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            media_base_url: Arc::new(normalize_media_base(media_base_url.into())),
        })
    }

    pub fn migrate(self) -> Result<Self, DbError> {
        let conn = self.connection.lock().map_err(|_| DbError::Poisoned)?;
        conn.execute_batch(KNOTS_SCHEMA_SQL)?;
        seed_skill_categories(&conn)?;
        Ok(self.clone())
    }

    pub fn replace_all_knots(
        &self,
        import_source: &str,
        knots: &[KnotSeed],
    ) -> Result<(), DbError> {
        let mut conn = self.connection.lock().map_err(|_| DbError::Poisoned)?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM knot_raw_metadata", [])?;
        tx.execute("DELETE FROM knot_media_assets", [])?;
        tx.execute("DELETE FROM knot_type_memberships", [])?;
        tx.execute("DELETE FROM knot_category_memberships", [])?;
        tx.execute("DELETE FROM knot_type_localizations", [])?;
        tx.execute("DELETE FROM knot_types", [])?;
        tx.execute("DELETE FROM knot_category_localizations", [])?;
        tx.execute("DELETE FROM knot_categories", [])?;
        tx.execute("DELETE FROM knot_localizations", [])?;
        tx.execute("DELETE FROM knots", [])?;
        tx.execute(
            "INSERT INTO knot_import_runs(source, item_count) VALUES (?1, ?2)",
            params![import_source, knots.len() as i64],
        )?;

        for knot in knots {
            tx.execute(
                "INSERT INTO knots(id, source_name, source_url, source_slug_en, source_slug_zh, difficulty) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    knot.id,
                    knot.source_name,
                    knot.source_url,
                    knot.source_slug_en,
                    knot.source_slug_zh,
                    knot.difficulty,
                ],
            )?;

            for localization in &knot.localizations {
                tx.execute(
                    "INSERT INTO knot_localizations(knot_id, locale, slug, title, summary, description, steps_json) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        knot.id,
                        localization.locale.as_str(),
                        localization.slug,
                        localization.title,
                        localization.summary,
                        localization.description,
                        serde_json::to_string(&localization.steps)?,
                    ],
                )?;
            }

            for category in &knot.categories {
                tx.execute(
                    "INSERT OR IGNORE INTO knot_categories(id) VALUES (?1)",
                    params![category.id],
                )?;
                for (locale, slug, title) in &category.localizations {
                    tx.execute(
                        "INSERT OR REPLACE INTO knot_category_localizations(category_id, locale, slug, title) \
                         VALUES (?1, ?2, ?3, ?4)",
                        params![category.id, locale.as_str(), slug, title],
                    )?;
                }
                tx.execute(
                    "INSERT OR IGNORE INTO knot_category_memberships(knot_id, category_id) VALUES (?1, ?2)",
                    params![knot.id, category.id],
                )?;
            }

            for knot_type in &knot.types {
                tx.execute(
                    "INSERT OR IGNORE INTO knot_types(id) VALUES (?1)",
                    params![knot_type.id],
                )?;
                for (locale, slug, title) in &knot_type.localizations {
                    tx.execute(
                        "INSERT OR REPLACE INTO knot_type_localizations(type_id, locale, slug, title) \
                         VALUES (?1, ?2, ?3, ?4)",
                        params![knot_type.id, locale.as_str(), slug, title],
                    )?;
                }
                tx.execute(
                    "INSERT OR IGNORE INTO knot_type_memberships(knot_id, type_id) VALUES (?1, ?2)",
                    params![knot.id, knot_type.id],
                )?;
            }

            for media in &knot.media {
                tx.execute(
                    "INSERT INTO knot_media_assets(knot_id, asset_id, media_type, path, mime_type, width, height, attribution, license_note) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        knot.id,
                        media.id,
                        media.media_type,
                        media.path,
                        media.mime_type,
                        media.width,
                        media.height,
                        media.attribution,
                        media.license_note,
                    ],
                )?;
            }

            tx.execute(
                "INSERT INTO knot_raw_metadata(knot_id, raw_json) VALUES (?1, ?2)",
                params![knot.id, serde_json::to_string(&knot.raw_metadata)?],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn list_skill_categories(
        &self,
        locale: Locale,
    ) -> Result<Vec<SkillCategorySummary>, DbError> {
        let conn = self.connection.lock().map_err(|_| DbError::Poisoned)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM knots", [], |row| row.get(0))?;
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
            item_count: count as u32,
            href: "/api/skills/knots/list".to_owned(),
        }])
    }

    pub fn list_knots(
        &self,
        locale: Locale,
        offset: u32,
        limit: u32,
        category: Option<&str>,
        q: Option<&str>,
    ) -> Result<KnotListResponse, DbError> {
        let conn = self.connection.lock().map_err(|_| DbError::Poisoned)?;
        let mut ids = if let Some(category) = category {
            let mut stmt = conn.prepare(
                "SELECT k.id FROM knots k \
                 JOIN knot_category_memberships kcm ON kcm.knot_id = k.id \
                 WHERE kcm.category_id = ?1 ORDER BY k.id ASC",
            )?;
            let rows = stmt.query_map(params![category], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare("SELECT id FROM knots ORDER BY id ASC")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let query = q
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let mut summaries = Vec::with_capacity(ids.len());
        for id in ids.drain(..) {
            if let Some(detail) = self.detail_locked(&conn, &id, locale)? {
                let matches_query = query.as_ref().map_or(true, |needle| {
                    detail.title.to_lowercase().contains(needle)
                        || detail.summary.to_lowercase().contains(needle)
                        || detail.id.to_lowercase().contains(needle)
                });
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

    pub fn get_knot_detail(&self, id: &str, locale: Locale) -> Result<Option<KnotDetail>, DbError> {
        let conn = self.connection.lock().map_err(|_| DbError::Poisoned)?;
        self.detail_locked(&conn, id, locale)
    }

    fn detail_locked(
        &self,
        conn: &Connection,
        id: &str,
        locale: Locale,
    ) -> Result<Option<KnotDetail>, DbError> {
        let difficulty = conn
            .query_row(
                "SELECT difficulty FROM knots WHERE id = ?1",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?;
        let Some(difficulty) = difficulty else {
            return Ok(None);
        };
        let Some(localization) = fetch_knot_localization(conn, id, locale)? else {
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
            categories: fetch_categories(conn, id, locale)?,
            types: fetch_types(conn, id, locale)?,
            media: fetch_media(conn, id, &self.media_base_url)?,
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

fn fetch_knot_localization(
    conn: &Connection,
    id: &str,
    locale: Locale,
) -> Result<Option<KnotLocalizationRow>, DbError> {
    for candidate in locale.fallbacks() {
        let row = conn
            .query_row(
                "SELECT slug, title, summary, description, steps_json \
                 FROM knot_localizations WHERE knot_id = ?1 AND locale = ?2",
                params![id, candidate.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;
        if let Some((slug, title, summary, description, steps_json)) = row {
            let steps = serde_json::from_str::<Vec<String>>(&steps_json)?;
            return Ok(Some(KnotLocalizationRow {
                slug,
                title,
                summary,
                description,
                steps,
            }));
        }
    }
    Ok(None)
}

fn fetch_categories(
    conn: &Connection,
    knot_id: &str,
    locale: Locale,
) -> Result<Vec<KnotTaxonomyItem>, DbError> {
    fetch_taxonomy(
        conn,
        knot_id,
        locale,
        "knot_category_memberships",
        "category_id",
        "knot_category_localizations",
    )
}

fn fetch_types(
    conn: &Connection,
    knot_id: &str,
    locale: Locale,
) -> Result<Vec<KnotTaxonomyItem>, DbError> {
    fetch_taxonomy(
        conn,
        knot_id,
        locale,
        "knot_type_memberships",
        "type_id",
        "knot_type_localizations",
    )
}

fn fetch_taxonomy(
    conn: &Connection,
    knot_id: &str,
    locale: Locale,
    membership_table: &str,
    item_id_col: &str,
    localization_table: &str,
) -> Result<Vec<KnotTaxonomyItem>, DbError> {
    let sql = format!(
        "SELECT {item_id_col} FROM {membership_table} WHERE knot_id = ?1 ORDER BY {item_id_col} ASC"
    );
    let ids = conn
        .prepare(&sql)?
        .query_map(params![knot_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut items = Vec::new();
    for id in ids {
        for candidate in locale.fallbacks() {
            let sql = format!(
                "SELECT slug, title FROM {localization_table} WHERE {item_id_col} = ?1 AND locale = ?2"
            );
            let row = conn
                .query_row(&sql, params![id, candidate.as_str()], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .optional()?;
            if let Some((slug, title)) = row {
                items.push(KnotTaxonomyItem {
                    id: id.clone(),
                    slug,
                    title,
                });
                break;
            }
        }
    }
    Ok(items)
}

fn fetch_media(
    conn: &Connection,
    knot_id: &str,
    media_base_url: &str,
) -> Result<Vec<KnotMediaAsset>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT asset_id, media_type, path, mime_type, width, height, attribution, license_note \
         FROM knot_media_assets WHERE knot_id = ?1 ORDER BY id ASC",
    )?;
    let media = stmt
        .query_map(params![knot_id], |row| {
            let path: String = row.get(2)?;
            Ok(KnotMediaAsset {
                id: row.get(0)?,
                media_type: row.get(1)?,
                url: format!(
                    "{}/{}",
                    media_base_url.trim_end_matches('/'),
                    path.trim_start_matches('/')
                ),
                mime_type: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                attribution: row.get(6)?,
                license_note: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(media)
}

fn sqlite_path(url: &str) -> Result<PathBuf, DbError> {
    let raw = url
        .strip_prefix("sqlite://")
        .or_else(|| url.strip_prefix("sqlite:"))
        .ok_or(DbError::UnsupportedDatabase(DatabaseKind::Sqlite))?;
    if raw.is_empty() {
        return Err(DbError::EmptySqlitePath);
    }
    if raw == ":memory:" {
        return Ok(PathBuf::from(raw));
    }
    Ok(PathBuf::from(raw))
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

fn seed_skill_categories(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR IGNORE INTO skill_categories(id, slug) VALUES ('knots', 'knots')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO skill_category_localizations(category_id, locale, title, summary) \
         VALUES ('knots', 'zh-CN', '绳结', '户外、露营、钓鱼、航海等场景常用绳结技能。')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO skill_category_localizations(category_id, locale, title, summary) \
         VALUES ('knots', 'en', 'Knots', 'Outdoor knots for camping, fishing, sailing, and field skills.')",
        [],
    )?;
    Ok(())
}

pub const KNOTS_SCHEMA_SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS skill_categories (
        id TEXT PRIMARY KEY,
        slug TEXT NOT NULL UNIQUE
    );

    CREATE TABLE IF NOT EXISTS skill_category_localizations (
        category_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT NOT NULL,
        PRIMARY KEY (category_id, locale),
        FOREIGN KEY (category_id) REFERENCES skill_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knots (
        id TEXT PRIMARY KEY,
        source_name TEXT NOT NULL,
        source_url TEXT NULL,
        source_slug_en TEXT NOT NULL,
        source_slug_zh TEXT NULL,
        difficulty TEXT NULL
    );

    CREATE TABLE IF NOT EXISTS knot_localizations (
        knot_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT NOT NULL,
        description TEXT NULL,
        steps_json TEXT NOT NULL DEFAULT '[]',
        PRIMARY KEY (knot_id, locale),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_categories (
        id TEXT PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS knot_category_localizations (
        category_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        PRIMARY KEY (category_id, locale),
        FOREIGN KEY (category_id) REFERENCES knot_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_category_memberships (
        knot_id TEXT NOT NULL,
        category_id TEXT NOT NULL,
        PRIMARY KEY (knot_id, category_id),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        FOREIGN KEY (category_id) REFERENCES knot_categories(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_types (
        id TEXT PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS knot_type_localizations (
        type_id TEXT NOT NULL,
        locale TEXT NOT NULL,
        slug TEXT NOT NULL,
        title TEXT NOT NULL,
        PRIMARY KEY (type_id, locale),
        FOREIGN KEY (type_id) REFERENCES knot_types(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_type_memberships (
        knot_id TEXT NOT NULL,
        type_id TEXT NOT NULL,
        PRIMARY KEY (knot_id, type_id),
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE,
        FOREIGN KEY (type_id) REFERENCES knot_types(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_media_assets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        knot_id TEXT NOT NULL,
        asset_id TEXT NOT NULL,
        media_type TEXT NOT NULL,
        path TEXT NOT NULL,
        mime_type TEXT NOT NULL,
        width INTEGER NULL,
        height INTEGER NULL,
        attribution TEXT NULL,
        license_note TEXT NULL,
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS knot_import_runs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source TEXT NOT NULL,
        item_count INTEGER NOT NULL,
        imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS knot_raw_metadata (
        knot_id TEXT PRIMARY KEY,
        raw_json TEXT NOT NULL,
        FOREIGN KEY (knot_id) REFERENCES knots(id) ON DELETE CASCADE
    );
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_url_to_path_supports_relative_absolute_and_memory() {
        assert_eq!(
            sqlite_path("sqlite://stellartrail.db").unwrap(),
            PathBuf::from("stellartrail.db")
        );
        assert_eq!(
            sqlite_path("sqlite:///tmp/stellartrail.db").unwrap(),
            PathBuf::from("/tmp/stellartrail.db")
        );
        assert_eq!(
            sqlite_path("sqlite://:memory:").unwrap(),
            PathBuf::from(":memory:")
        );
    }
}
