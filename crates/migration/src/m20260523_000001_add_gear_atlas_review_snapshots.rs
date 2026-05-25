//! Adds review snapshots and administrator change summaries for gear atlas submissions.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};
use serde_json::{Value, json};

/// Stores the original submitted public fields and the final review delta.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds nullable JSON columns and initializes historical rows from current public fields.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        for sql in ADD_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        backfill_submitted_snapshots(db, backend).await?;
        Ok(())
    }

    /// Drops the review snapshot columns.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in DROP_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}

const ADD_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE gear_atlas_items ADD COLUMN submitted_snapshot_json TEXT NULL",
    "ALTER TABLE gear_atlas_items ADD COLUMN review_changes_json TEXT NULL",
];

const DROP_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE gear_atlas_items DROP COLUMN review_changes_json",
    "ALTER TABLE gear_atlas_items DROP COLUMN submitted_snapshot_json",
];

async fn backfill_submitted_snapshots(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            backend,
            "SELECT id, category, name, brand, model, description, weight_g, \
             official_price_cents, official_price_currency, variants_json, specs_json \
             FROM gear_atlas_items"
                .to_owned(),
        ))
        .await?;

    let update_sql = match backend {
        DatabaseBackend::Postgres => {
            "UPDATE gear_atlas_items SET submitted_snapshot_json = $1, review_changes_json = NULL \
             WHERE id = $2"
        }
        _ => {
            "UPDATE gear_atlas_items SET submitted_snapshot_json = ?, review_changes_json = NULL \
             WHERE id = ?"
        }
    };

    for row in rows {
        let id: String = row.try_get("", "id")?;
        let category: String = row.try_get("", "category")?;
        let name: String = row.try_get("", "name")?;
        let brand: Option<String> = row.try_get("", "brand")?;
        let model: Option<String> = row.try_get("", "model")?;
        let description: Option<String> = row.try_get("", "description")?;
        let weight_g: Option<i32> = row.try_get("", "weight_g")?;
        let official_price_cents: Option<i64> = row.try_get("", "official_price_cents")?;
        let official_price_currency: Option<String> = row.try_get("", "official_price_currency")?;
        let variants_json: String = row.try_get("", "variants_json")?;
        let specs_json: String = row.try_get("", "specs_json")?;
        let snapshot = json!({
            "category": category,
            "name": name,
            "brand": brand,
            "model": model,
            "description": description,
            "weight_g": weight_g,
            "official_price_cents": official_price_cents,
            "official_price_currency": official_price_currency,
            "variants": parse_json_array(&variants_json),
            "specs": parse_json_object(&specs_json),
        });

        db.execute(Statement::from_sql_and_values(
            backend,
            update_sql,
            vec![
                serde_json::to_string(&snapshot)
                    .map_err(|err| DbErr::Custom(err.to_string()))?
                    .into(),
                id.into(),
            ],
        ))
        .await?;
    }

    Ok(())
}

fn parse_json_array(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!([]))
}

fn parse_json_object(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!({}))
}
