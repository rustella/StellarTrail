//! Adds structured public gear variants and personal selected size fields.
//!
//! Historical size-like spec keys are moved out of `specs_json` so atlas
//! records describe available public variants while personal gear records keep
//! only the user's chosen or hand-entered size.

use std::collections::BTreeMap;

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};
use serde_json::{Value, json};

/// Adds structured variant columns and migrates legacy size-like specs.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates variant columns and removes legacy size keys from specs JSON.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        for sql in ADD_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_gear_items_atlas_item \
             ON user_gear_items(atlas_item_id)",
        )
        .await?;

        migrate_atlas_items(db, backend).await?;
        migrate_user_gear_items(db, backend).await?;
        Ok(())
    }

    /// This migration is not reversible because size keys are intentionally removed from specs JSON.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_gear_items_atlas_item")
            .await?;
        for sql in DROP_COLUMNS_SQL {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}

const ADD_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE gear_atlas_items ADD COLUMN variants_json TEXT NOT NULL DEFAULT '[]'",
    "ALTER TABLE user_gear_items ADD COLUMN atlas_item_id TEXT NULL",
    "ALTER TABLE user_gear_items ADD COLUMN selected_variant_key TEXT NULL",
    "ALTER TABLE user_gear_items ADD COLUMN selected_variant_label TEXT NULL",
];

const DROP_COLUMNS_SQL: &[&str] = &[
    "ALTER TABLE user_gear_items DROP COLUMN selected_variant_label",
    "ALTER TABLE user_gear_items DROP COLUMN selected_variant_key",
    "ALTER TABLE user_gear_items DROP COLUMN atlas_item_id",
    "ALTER TABLE gear_atlas_items DROP COLUMN variants_json",
];

const SIZE_SPEC_KEYS: &[&str] = &["size", "backpack_size", "size_or_length"];

async fn migrate_atlas_items(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            backend,
            "SELECT id, specs_json, variants_json FROM gear_atlas_items".to_owned(),
        ))
        .await?;

    let update_sql = match backend {
        DatabaseBackend::Postgres => {
            "UPDATE gear_atlas_items SET specs_json = $1, variants_json = $2 WHERE id = $3"
        }
        _ => "UPDATE gear_atlas_items SET specs_json = ?, variants_json = ? WHERE id = ?",
    };

    for row in rows {
        let id: String = row.try_get("", "id")?;
        let specs_json: String = row.try_get("", "specs_json")?;
        let variants_json: String = row.try_get("", "variants_json")?;
        let mut specs = parse_specs(&specs_json);
        let mut variants = parse_variants(&variants_json);
        let labels = extract_size_labels(&mut specs);
        append_variant_labels(&mut variants, labels);

        db.execute(Statement::from_sql_and_values(
            backend,
            update_sql,
            vec![
                json_string(&specs)?.into(),
                serde_json::to_string(&variants)
                    .map_err(|err| DbErr::Custom(err.to_string()))?
                    .into(),
                id.into(),
            ],
        ))
        .await?;
    }

    Ok(())
}

async fn migrate_user_gear_items(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            backend,
            "SELECT id, specs_json, selected_variant_key, selected_variant_label \
             FROM user_gear_items"
                .to_owned(),
        ))
        .await?;

    let update_sql = match backend {
        DatabaseBackend::Postgres => {
            "UPDATE user_gear_items \
             SET specs_json = $1, \
                 selected_variant_label = CASE \
                     WHEN selected_variant_label IS NULL OR selected_variant_label = '' THEN $2 \
                     ELSE selected_variant_label \
                 END, \
                 selected_variant_key = CASE \
                     WHEN selected_variant_key IS NULL OR selected_variant_key = '' THEN $3 \
                     ELSE selected_variant_key \
                 END \
             WHERE id = $4"
        }
        _ => {
            "UPDATE user_gear_items \
             SET specs_json = ?, \
                 selected_variant_label = CASE \
                     WHEN selected_variant_label IS NULL OR selected_variant_label = '' THEN ? \
                     ELSE selected_variant_label \
                 END, \
                 selected_variant_key = CASE \
                     WHEN selected_variant_key IS NULL OR selected_variant_key = '' THEN ? \
                     ELSE selected_variant_key \
                 END \
             WHERE id = ?"
        }
    };

    for row in rows {
        let id: String = row.try_get("", "id")?;
        let specs_json: String = row.try_get("", "specs_json")?;
        let selected_variant_label: Option<String> = row.try_get("", "selected_variant_label")?;
        let selected_variant_key: Option<String> = row.try_get("", "selected_variant_key")?;
        let mut specs = parse_specs(&specs_json);
        let label = extract_first_size_label(&mut specs);
        let label = if selected_variant_label
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            None
        } else {
            label
        };
        let key = if selected_variant_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            None
        } else {
            label
                .as_deref()
                .map(|value| variant_key_from_label(value, 0))
        };

        db.execute(Statement::from_sql_and_values(
            backend,
            update_sql,
            vec![
                json_string(&specs)?.into(),
                label.into(),
                key.into(),
                id.into(),
            ],
        ))
        .await?;
    }

    Ok(())
}

fn parse_specs(value: &str) -> BTreeMap<String, String> {
    serde_json::from_str(value).unwrap_or_default()
}

fn parse_variants(value: &str) -> Vec<Value> {
    serde_json::from_str(value).unwrap_or_default()
}

fn extract_size_labels(specs: &mut BTreeMap<String, String>) -> Vec<String> {
    let mut labels = Vec::new();
    for key in SIZE_SPEC_KEYS {
        let Some(value) = specs.remove(*key) else {
            continue;
        };
        labels.extend(split_variant_labels(&value));
    }
    dedupe(labels)
}

fn extract_first_size_label(specs: &mut BTreeMap<String, String>) -> Option<String> {
    for key in SIZE_SPEC_KEYS {
        let Some(value) = specs.remove(*key) else {
            continue;
        };
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

fn append_variant_labels(variants: &mut Vec<Value>, labels: Vec<String>) {
    let mut index = variants.len();
    for label in labels {
        if variants.iter().any(|variant| {
            variant
                .get("label")
                .and_then(Value::as_str)
                .is_some_and(|existing| existing == label)
        }) {
            continue;
        }
        variants.push(json!({
            "key": variant_key_from_label(&label, index),
            "label": label,
        }));
        index += 1;
    }
}

fn split_variant_labels(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.is_empty() {
        return Vec::new();
    }
    if value.contains([',', '，', '、', ';', '；', '\n', '|']) {
        return dedupe(
            value
                .split([',', '，', '、', ';', '；', '\n', '|'])
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        );
    }

    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.len() >= 4 && tokens.iter().filter(|token| is_size_marker(token)).count() >= 2 {
        let mut labels = Vec::new();
        let mut current = Vec::new();
        for token in tokens {
            if is_size_marker(token) && !current.is_empty() {
                labels.push(current.join(" "));
                current.clear();
            }
            current.push(token);
        }
        if !current.is_empty() {
            labels.push(current.join(" "));
        }
        return dedupe(labels);
    }

    vec![value.to_owned()]
}

fn is_size_marker(token: &str) -> bool {
    let token = token.trim_matches(|ch: char| {
        ch == ':' || ch == '：' || ch == '-' || ch == '(' || ch == ')' || ch == '（' || ch == '）'
    });
    matches!(
        token.to_ascii_uppercase().as_str(),
        "XXS" | "XS" | "S" | "M" | "L" | "XL" | "XXL" | "XXXL" | "2XL" | "3XL"
    )
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || deduped.iter().any(|existing| existing == value) {
            continue;
        }
        deduped.push(value.to_owned());
    }
    deduped
}

fn variant_key_from_label(label: &str, index: usize) -> String {
    let mut key = String::new();
    for ch in label.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
        } else if !key.ends_with('-') {
            key.push('-');
        }
    }
    let key = key.trim_matches('-');
    if key.is_empty() {
        format!("variant-{index}")
    } else {
        key.chars().take(80).collect()
    }
}

fn json_string(specs: &BTreeMap<String, String>) -> Result<String, DbErr> {
    serde_json::to_string(specs).map_err(|err| DbErr::Custom(err.to_string()))
}
