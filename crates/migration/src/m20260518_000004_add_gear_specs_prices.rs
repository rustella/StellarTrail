//! Gear inventory migration adding specs JSON and currency-aware price columns.

use std::collections::BTreeMap;

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};

/// Adds dynamic specs and separate official/purchase currency fields.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds new columns and backfills legacy typed fields into `specs_json`.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute_unprepared(
            "ALTER TABLE user_gear_items ADD COLUMN official_price_cents BIGINT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_gear_items ADD COLUMN official_price_currency TEXT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_gear_items ADD COLUMN purchase_price_currency TEXT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_gear_items ADD COLUMN specs_json TEXT NOT NULL DEFAULT '{}'",
        )
        .await?;
        db.execute_unprepared(
            "UPDATE user_gear_items \
             SET purchase_price_currency = 'CNY' \
             WHERE purchase_price_cents IS NOT NULL \
               AND (purchase_price_currency IS NULL OR purchase_price_currency = '')",
        )
        .await?;

        let rows = db
            .query_all(Statement::from_string(
                backend,
                "SELECT id, color, material, capacity, size, warmth_index, waterproof_index, \
                        expiry_or_warranty_date, specs_json \
                 FROM user_gear_items"
                    .to_owned(),
            ))
            .await?;

        let update_sql = match backend {
            DatabaseBackend::Postgres => "UPDATE user_gear_items SET specs_json = $1 WHERE id = $2",
            _ => "UPDATE user_gear_items SET specs_json = ? WHERE id = ?",
        };

        for row in rows {
            let id: String = row.try_get("", "id")?;
            let specs_json: String = row.try_get("", "specs_json")?;
            let mut specs =
                serde_json::from_str::<BTreeMap<String, String>>(&specs_json).unwrap_or_default();

            insert_legacy_spec(&mut specs, "color", row.try_get("", "color")?);
            insert_legacy_spec(&mut specs, "material", row.try_get("", "material")?);
            insert_legacy_spec(&mut specs, "capacity", row.try_get("", "capacity")?);
            insert_legacy_spec(&mut specs, "size", row.try_get("", "size")?);
            insert_legacy_spec(&mut specs, "warmth_index", row.try_get("", "warmth_index")?);
            insert_legacy_spec(
                &mut specs,
                "waterproof_index",
                row.try_get("", "waterproof_index")?,
            );
            insert_legacy_spec(
                &mut specs,
                "expiry_or_warranty_date",
                row.try_get("", "expiry_or_warranty_date")?,
            );

            db.execute(Statement::from_sql_and_values(
                backend,
                update_sql,
                vec![json_string(&specs)?.into(), id.into()],
            ))
            .await?;
        }

        for column in LEGACY_SPEC_COLUMNS {
            db.execute_unprepared(&format!("ALTER TABLE user_gear_items DROP COLUMN {column}"))
                .await?;
        }

        Ok(())
    }

    /// This migration is not reversible because legacy columns are folded into `specs_json` and dropped.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

const LEGACY_SPEC_COLUMNS: &[&str] = &[
    "color",
    "material",
    "capacity",
    "size",
    "warmth_index",
    "waterproof_index",
    "expiry_or_warranty_date",
];

fn json_string(specs: &BTreeMap<String, String>) -> Result<String, DbErr> {
    serde_json::to_string(specs).map_err(|err| DbErr::Custom(err.to_string()))
}

fn insert_legacy_spec(
    specs: &mut BTreeMap<String, String>,
    key: &'static str,
    value: Option<String>,
) {
    let Some(value) = value else {
        return;
    };
    let value = value.trim();
    if value.is_empty() || specs.contains_key(key) {
        return;
    }
    specs.insert(key.to_owned(), value.to_owned());
}
