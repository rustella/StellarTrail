//! Gear template repository for DB-backed public equipment checklist templates.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, TransactionTrait};
use stellartrail_domain::gear_template::{GearTemplate, GearTemplateCategory, GearTemplateSeed};

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
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Lists active gear templates with nested categories and item labels.
    pub async fn list_templates(&self) -> Result<Vec<GearTemplate>, DbErr> {
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
            templates.push(GearTemplate {
                title: row.try_get("", "title")?,
                categories: self.fetch_categories(&id).await?,
                id,
            });
        }
        Ok(templates)
    }

    /// Fetches one active gear template by id.
    pub async fn get_template(&self, id: &str) -> Result<Option<GearTemplate>, DbErr> {
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
        Ok(Some(GearTemplate {
            title: row.try_get("", "title")?,
            categories: self.fetch_categories(&id).await?,
            id,
        }))
    }

    async fn fetch_categories(
        &self,
        template_id: &str,
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
            let item_rows = self
                .db
                .query_all(statement(
                    backend,
                    "SELECT name FROM gear_template_items WHERE template_id = ? AND category_id = ? ORDER BY sort_order ASC, id ASC",
                    vec![template_id.to_owned().into(), category_id.clone().into()],
                ))
                .await?;
            let items = item_rows
                .into_iter()
                .map(|row| row.try_get::<String>("", "name"))
                .collect::<Result<Vec<_>, _>>()?;
            categories.push(GearTemplateCategory {
                id: category_id,
                name: row.try_get("", "name")?,
                items,
            });
        }
        Ok(categories)
    }
}
