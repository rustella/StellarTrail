//! Gear template repository for DB-backed public equipment checklist templates.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};
use stellartrail_domain::{
    gear_template::{GearTemplate, GearTemplateCategory, GearTemplateSeed},
    locale::Locale,
};

use super::statement;

/// Persistence object for public gear templates.
#[derive(Clone)]
pub struct GearTemplateRepository {
    db: DatabaseConnection,
}

impl GearTemplateRepository {
    /// Creates a repository using the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Replaces system-owned gear templates while preserving custom rows from other sources.
    pub async fn replace_system_templates(
        &self,
        source: &str,
        templates: &[GearTemplateSeed],
    ) -> Result<(), DbErr> {
        let backend = self.db.get_database_backend();
        let tx = self.db.begin().await?;
        for sql in [
            "DELETE FROM gear_template_items WHERE template_id IN (SELECT id FROM gear_templates WHERE source = ?)",
            "DELETE FROM gear_template_categories WHERE template_id IN (SELECT id FROM gear_templates WHERE source = ?)",
            "DELETE FROM gear_templates WHERE source = ?",
        ] {
            tx.execute(statement(backend, sql, vec![source.to_owned().into()]))
                .await?;
        }

        for template in templates {
            tx.execute(statement(
                backend,
                "INSERT INTO gear_templates(id, title, source, status, sort_order, created_at, updated_at) \
                 VALUES (?, ?, ?, 'active', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                vec![
                    template.id.clone().into(),
                    template.title.clone().into(),
                    source.to_owned().into(),
                    template.sort_order.into(),
                ],
            ))
            .await?;
            for (locale, title) in
                localizations_with_default(&template.localizations, Locale::ZhCn, &template.title)
            {
                tx.execute(statement(
                    backend,
                    "INSERT INTO gear_template_localizations(template_id, locale, title) \
                     VALUES (?, ?, ?) ON CONFLICT(template_id, locale) DO UPDATE SET title = excluded.title",
                    vec![
                        template.id.clone().into(),
                        locale.as_str().to_owned().into(),
                        title.into(),
                    ],
                ))
                .await?;
            }

            for category in &template.categories {
                tx.execute(statement(
                    backend,
                    "INSERT INTO gear_template_categories(template_id, id, name, sort_order, created_at, updated_at) \
                     VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    vec![
                        template.id.clone().into(),
                        category.id.clone().into(),
                        category.name.clone().into(),
                        category.sort_order.into(),
                    ],
                ))
                .await?;
                for (locale, name) in localizations_with_default(
                    &category.localizations,
                    Locale::ZhCn,
                    &category.name,
                ) {
                    tx.execute(statement(
                        backend,
                        "INSERT INTO gear_template_category_localizations(template_id, category_id, locale, name) \
                         VALUES (?, ?, ?, ?) ON CONFLICT(template_id, category_id, locale) DO UPDATE SET name = excluded.name",
                        vec![
                            template.id.clone().into(),
                            category.id.clone().into(),
                            locale.as_str().to_owned().into(),
                            name.into(),
                        ],
                    ))
                    .await?;
                }

                for item in &category.items {
                    tx.execute(statement(
                        backend,
                        "INSERT INTO gear_template_items(template_id, category_id, id, name, sort_order, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                        vec![
                            template.id.clone().into(),
                            category.id.clone().into(),
                            item.id.clone().into(),
                            item.name.clone().into(),
                            item.sort_order.into(),
                        ],
                    ))
                    .await?;
                    for (locale, name) in
                        localizations_with_default(&item.localizations, Locale::ZhCn, &item.name)
                    {
                        tx.execute(statement(
                            backend,
                            "INSERT INTO gear_template_item_localizations(template_id, category_id, item_id, locale, name) \
                             VALUES (?, ?, ?, ?, ?) ON CONFLICT(template_id, category_id, item_id, locale) DO UPDATE SET name = excluded.name",
                            vec![
                                template.id.clone().into(),
                                category.id.clone().into(),
                                item.id.clone().into(),
                                locale.as_str().to_owned().into(),
                                name.into(),
                            ],
                        ))
                        .await?;
                    }
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Lists active gear templates with nested categories and item labels.
    pub async fn list_templates(&self, locale: Locale) -> Result<Vec<GearTemplate>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT id, title FROM gear_templates WHERE status = 'active' ORDER BY sort_order ASC, id ASC",
                vec![],
            ))
            .await?;
        let mut templates = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.try_get("", "id")?;
            let base_title: String = row.try_get("", "title")?;
            templates.push(GearTemplate {
                title: self.fetch_template_title(&id, locale, base_title).await?,
                categories: self.fetch_categories(&id, locale).await?,
                id,
            });
        }
        Ok(templates)
    }

    /// Fetches one active gear template by id.
    pub async fn get_template(
        &self,
        id: &str,
        locale: Locale,
    ) -> Result<Option<GearTemplate>, DbErr> {
        let Some(row) = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, title FROM gear_templates WHERE id = ? AND status = 'active'",
                vec![id.to_owned().into()],
            ))
            .await?
        else {
            return Ok(None);
        };
        let id: String = row.try_get("", "id")?;
        let base_title: String = row.try_get("", "title")?;
        Ok(Some(GearTemplate {
            title: self.fetch_template_title(&id, locale, base_title).await?,
            categories: self.fetch_categories(&id, locale).await?,
            id,
        }))
    }

    async fn fetch_categories(
        &self,
        template_id: &str,
        locale: Locale,
    ) -> Result<Vec<GearTemplateCategory>, DbErr> {
        let backend = self.db.get_database_backend();
        let rows = self
            .db
            .query_all(statement(
                backend,
                "SELECT id, name FROM gear_template_categories WHERE template_id = ? ORDER BY sort_order ASC, id ASC",
                vec![template_id.to_owned().into()],
            ))
            .await?;
        let mut categories = Vec::with_capacity(rows.len());
        for row in rows {
            let category_id: String = row.try_get("", "id")?;
            let base_name: String = row.try_get("", "name")?;
            let item_rows = self
                .db
                .query_all(statement(
                    backend,
                    "SELECT id, name FROM gear_template_items WHERE template_id = ? AND category_id = ? ORDER BY sort_order ASC, id ASC",
                    vec![template_id.to_owned().into(), category_id.clone().into()],
                ))
                .await?;
            let mut items = Vec::with_capacity(item_rows.len());
            for item_row in item_rows {
                let item_id: String = item_row.try_get("", "id")?;
                let base_item_name: String = item_row.try_get("", "name")?;
                items.push(
                    self.fetch_item_name(
                        template_id,
                        &category_id,
                        &item_id,
                        locale,
                        base_item_name,
                    )
                    .await?,
                );
            }
            let name = self
                .fetch_category_name(template_id, &category_id, locale, base_name)
                .await?;
            categories.push(GearTemplateCategory {
                id: category_id,
                name,
                items,
            });
        }
        Ok(categories)
    }

    async fn fetch_template_title(
        &self,
        template_id: &str,
        locale: Locale,
        fallback: String,
    ) -> Result<String, DbErr> {
        for candidate in locale.fallbacks() {
            let row = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT title FROM gear_template_localizations WHERE template_id = ? AND locale = ?",
                    vec![
                        template_id.to_owned().into(),
                        candidate.as_str().to_owned().into(),
                    ],
                ))
                .await?;
            if let Some(row) = row {
                return row.try_get("", "title");
            }
        }
        Ok(fallback)
    }

    async fn fetch_category_name(
        &self,
        template_id: &str,
        category_id: &str,
        locale: Locale,
        fallback: String,
    ) -> Result<String, DbErr> {
        for candidate in locale.fallbacks() {
            let row = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT name FROM gear_template_category_localizations \
                     WHERE template_id = ? AND category_id = ? AND locale = ?",
                    vec![
                        template_id.to_owned().into(),
                        category_id.to_owned().into(),
                        candidate.as_str().to_owned().into(),
                    ],
                ))
                .await?;
            if let Some(row) = row {
                return row.try_get("", "name");
            }
        }
        Ok(fallback)
    }

    async fn fetch_item_name(
        &self,
        template_id: &str,
        category_id: &str,
        item_id: &str,
        locale: Locale,
        fallback: String,
    ) -> Result<String, DbErr> {
        for candidate in locale.fallbacks() {
            let row = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT name FROM gear_template_item_localizations \
                     WHERE template_id = ? AND category_id = ? AND item_id = ? AND locale = ?",
                    vec![
                        template_id.to_owned().into(),
                        category_id.to_owned().into(),
                        item_id.to_owned().into(),
                        candidate.as_str().to_owned().into(),
                    ],
                ))
                .await?;
            if let Some(row) = row {
                return row.try_get("", "name");
            }
        }
        Ok(fallback)
    }
}

fn localizations_with_default(
    localizations: &[(Locale, String)],
    default_locale: Locale,
    default_value: &str,
) -> Vec<(Locale, String)> {
    let mut rows = Vec::new();
    let mut has_default = false;
    for (locale, value) in localizations {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if *locale == default_locale {
            has_default = true;
        }
        rows.push((*locale, value.to_owned()));
    }
    if !has_default {
        rows.push((default_locale, default_value.to_owned()));
    }
    rows
}
