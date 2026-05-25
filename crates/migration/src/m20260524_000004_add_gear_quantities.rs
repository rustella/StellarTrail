//! Adds quantity-aware personal gear inventory and packing-list quantities.
//!
//! Personal gear rows now represent a stock keeping unit owned by the user:
//! per-item weight and price stay on the row, while `quantity` counts how many
//! physical units that row represents. Packing-list items keep separate planned
//! and packed quantities so a user can own two units and carry only one.

use std::collections::BTreeMap;

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, QueryResult, Statement},
};

/// Adds gear quantity fields and folds historical duplicate rows into one row.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates quantity columns, migrates legacy specs, and merges duplicate personal gear rows.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute_unprepared(
            "ALTER TABLE user_gear_items ADD COLUMN quantity INTEGER NOT NULL DEFAULT 1",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE gear_packing_list_items ADD COLUMN planned_quantity INTEGER NOT NULL DEFAULT 1",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE gear_packing_list_items ADD COLUMN packed_quantity INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        db.execute_unprepared(
            "UPDATE gear_packing_list_items \
             SET packed_quantity = CASE WHEN packed THEN planned_quantity ELSE 0 END",
        )
        .await?;

        migrate_legacy_spec_quantity(db, backend).await?;
        merge_duplicate_gear_rows(db, backend).await?;
        Ok(())
    }

    /// This migration intentionally keeps merged data because duplicate rows cannot be reconstructed.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct GearRow {
    id: String,
    user_id: String,
    category: String,
    name: String,
    brand: Option<String>,
    model: Option<String>,
    purchase_date: Option<String>,
    purchase_price_cents: Option<i64>,
    purchase_location: Option<String>,
    status: String,
    storage_location: Option<String>,
    atlas_item_id: Option<String>,
    selected_variant_key: Option<String>,
    selected_variant_label: Option<String>,
    specs: BTreeMap<String, String>,
    tags: Vec<String>,
    notes: Option<String>,
    archived_at: Option<String>,
    is_deleted: bool,
    updated_at: String,
    quantity: i32,
    changed: bool,
}

#[derive(Clone, Debug)]
struct MergePair {
    duplicate_id: String,
    canonical_id: String,
}

#[derive(Clone, Debug)]
struct PackingItemRow {
    id: String,
    packing_list_id: String,
    user_id: String,
    planned_quantity: i32,
    packed_quantity: i32,
}

async fn migrate_legacy_spec_quantity(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            backend,
            "SELECT id, specs_json, quantity FROM user_gear_items".to_owned(),
        ))
        .await?;
    let update_sql = sql(
        backend,
        "UPDATE user_gear_items SET specs_json = ?, quantity = ? WHERE id = ?",
    );

    for row in rows {
        let id: String = row.try_get("", "id")?;
        let current_quantity: i32 = row.try_get("", "quantity")?;
        let specs_json: String = row.try_get("", "specs_json")?;
        let mut specs = parse_specs(&specs_json);
        let Some(raw_quantity) = specs.remove("quantity") else {
            continue;
        };
        let quantity = parse_quantity(&raw_quantity).unwrap_or(current_quantity);
        db.execute(Statement::from_sql_and_values(
            backend,
            update_sql.clone(),
            vec![json_string(&specs)?.into(), quantity.into(), id.into()],
        ))
        .await?;
    }
    Ok(())
}

async fn merge_duplicate_gear_rows(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let rows = db
        .query_all(Statement::from_string(
            backend,
            "SELECT id, user_id, category, name, brand, model, purchase_date, \
                    purchase_price_cents, purchase_location, status, storage_location, \
                    atlas_item_id, selected_variant_key, selected_variant_label, specs_json, \
                    tags_json, notes, archived_at, is_deleted, created_at, updated_at, quantity \
             FROM user_gear_items \
             ORDER BY user_id ASC, is_deleted ASC, archived_at ASC, created_at ASC, id ASC"
                .to_owned(),
        ))
        .await?;
    let mut canonical_rows: Vec<GearRow> = Vec::new();
    let mut merge_pairs = Vec::new();

    for row in rows {
        let row = map_gear_row(&row)?;
        if let Some(canonical) = canonical_rows.iter_mut().find(|candidate| {
            same_merge_bucket(candidate, &row) && same_gear_identity(candidate, &row)
        }) {
            merge_pairs.push(MergePair {
                duplicate_id: row.id.clone(),
                canonical_id: canonical.id.clone(),
            });
            merge_row(canonical, &row);
        } else {
            canonical_rows.push(row);
        }
    }

    for pair in &merge_pairs {
        merge_packing_items(db, backend, &pair.duplicate_id, &pair.canonical_id).await?;
        db.execute(Statement::from_sql_and_values(
            backend,
            sql(
                backend,
                "UPDATE gear_atlas_items SET source_user_gear_id = ? WHERE source_user_gear_id = ?",
            ),
            vec![
                pair.canonical_id.clone().into(),
                pair.duplicate_id.clone().into(),
            ],
        ))
        .await?;
        db.execute(Statement::from_sql_and_values(
            backend,
            sql(backend, "DELETE FROM user_gear_items WHERE id = ?"),
            vec![pair.duplicate_id.clone().into()],
        ))
        .await?;
    }

    let update_sql = sql(
        backend,
        "UPDATE user_gear_items \
         SET quantity = ?, specs_json = ?, tags_json = ?, notes = ?, updated_at = ? \
         WHERE id = ?",
    );
    for row in canonical_rows.into_iter().filter(|row| row.changed) {
        db.execute(Statement::from_sql_and_values(
            backend,
            update_sql.clone(),
            vec![
                row.quantity.into(),
                json_string(&row.specs)?.into(),
                serde_json::to_string(&row.tags)
                    .map_err(|err| DbErr::Custom(err.to_string()))?
                    .into(),
                row.notes.into(),
                row.updated_at.into(),
                row.id.into(),
            ],
        ))
        .await?;
    }
    Ok(())
}

async fn merge_packing_items(
    db: &impl ConnectionTrait,
    backend: DatabaseBackend,
    duplicate_gear_id: &str,
    canonical_gear_id: &str,
) -> Result<(), DbErr> {
    let duplicate_items = db
        .query_all(Statement::from_sql_and_values(
            backend,
            sql(
                backend,
                "SELECT id, packing_list_id, user_id, planned_quantity, packed_quantity \
                 FROM gear_packing_list_items WHERE gear_id = ?",
            ),
            vec![duplicate_gear_id.to_owned().into()],
        ))
        .await?;
    for row in duplicate_items {
        let duplicate = map_packing_item_row(&row)?;
        let canonical = db
            .query_one(Statement::from_sql_and_values(
                backend,
                sql(
                    backend,
                    "SELECT id, packing_list_id, user_id, planned_quantity, packed_quantity \
                     FROM gear_packing_list_items \
                     WHERE user_id = ? AND packing_list_id = ? AND gear_id = ? LIMIT 1",
                ),
                vec![
                    duplicate.user_id.clone().into(),
                    duplicate.packing_list_id.clone().into(),
                    canonical_gear_id.to_owned().into(),
                ],
            ))
            .await?;
        if let Some(canonical) = canonical {
            let canonical = map_packing_item_row(&canonical)?;
            let planned_quantity = canonical.planned_quantity + duplicate.planned_quantity;
            let packed_quantity =
                (canonical.packed_quantity + duplicate.packed_quantity).min(planned_quantity);
            let packed = planned_quantity > 0 && packed_quantity >= planned_quantity;
            db.execute(Statement::from_sql_and_values(
                backend,
                sql(
                    backend,
                    "UPDATE gear_packing_list_items \
                     SET planned_quantity = ?, packed_quantity = ?, packed = ? WHERE id = ?",
                ),
                vec![
                    planned_quantity.into(),
                    packed_quantity.into(),
                    packed.into(),
                    canonical.id.into(),
                ],
            ))
            .await?;
            db.execute(Statement::from_sql_and_values(
                backend,
                sql(backend, "DELETE FROM gear_packing_list_items WHERE id = ?"),
                vec![duplicate.id.into()],
            ))
            .await?;
        } else {
            db.execute(Statement::from_sql_and_values(
                backend,
                sql(
                    backend,
                    "UPDATE gear_packing_list_items SET gear_id = ? WHERE id = ?",
                ),
                vec![canonical_gear_id.to_owned().into(), duplicate.id.into()],
            ))
            .await?;
        }
    }
    Ok(())
}

fn map_gear_row(row: &QueryResult) -> Result<GearRow, DbErr> {
    let specs_json: String = row.try_get("", "specs_json")?;
    let tags_json: String = row.try_get("", "tags_json")?;
    Ok(GearRow {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        category: row.try_get("", "category")?,
        name: row.try_get("", "name")?,
        brand: row.try_get("", "brand")?,
        model: row.try_get("", "model")?,
        purchase_date: row.try_get("", "purchase_date")?,
        purchase_price_cents: row.try_get("", "purchase_price_cents")?,
        purchase_location: row.try_get("", "purchase_location")?,
        status: row.try_get("", "status")?,
        storage_location: row.try_get("", "storage_location")?,
        atlas_item_id: row.try_get("", "atlas_item_id")?,
        selected_variant_key: row.try_get("", "selected_variant_key")?,
        selected_variant_label: row.try_get("", "selected_variant_label")?,
        specs: parse_specs(&specs_json),
        tags: serde_json::from_str(&tags_json).unwrap_or_default(),
        notes: row.try_get("", "notes")?,
        archived_at: row.try_get("", "archived_at")?,
        is_deleted: row.try_get("", "is_deleted")?,
        updated_at: row.try_get("", "updated_at")?,
        quantity: row.try_get("", "quantity")?,
        changed: false,
    })
}

fn map_packing_item_row(row: &QueryResult) -> Result<PackingItemRow, DbErr> {
    Ok(PackingItemRow {
        id: row.try_get("", "id")?,
        packing_list_id: row.try_get("", "packing_list_id")?,
        user_id: row.try_get("", "user_id")?,
        planned_quantity: row.try_get("", "planned_quantity")?,
        packed_quantity: row.try_get("", "packed_quantity")?,
    })
}

fn merge_row(target: &mut GearRow, duplicate: &GearRow) {
    target.quantity = (target.quantity + duplicate.quantity).min(9_999);
    for (key, value) in &duplicate.specs {
        target
            .specs
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }
    for tag in &duplicate.tags {
        if target.tags.len() >= 20 {
            break;
        }
        if !target.tags.iter().any(|existing| existing == tag) {
            target.tags.push(tag.clone());
        }
    }
    target.notes = merged_notes(target, duplicate);
    if duplicate.updated_at > target.updated_at {
        target.updated_at = duplicate.updated_at.clone();
    }
    target.changed = true;
}

fn merged_notes(target: &GearRow, duplicate: &GearRow) -> Option<String> {
    let mut extra = Vec::new();
    push_note_diff(
        &mut extra,
        "购买日期",
        duplicate.purchase_date.as_deref(),
        target.purchase_date.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "购买渠道",
        duplicate.purchase_location.as_deref(),
        target.purchase_location.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "存放位置",
        duplicate.storage_location.as_deref(),
        target.storage_location.as_deref(),
    );
    push_note_diff(
        &mut extra,
        "购入价",
        duplicate
            .purchase_price_cents
            .map(|value| value.to_string())
            .as_deref(),
        target
            .purchase_price_cents
            .map(|value| value.to_string())
            .as_deref(),
    );
    if duplicate.status != target.status {
        extra.push(format!("状态：{}", duplicate.status));
    }
    push_note_diff(
        &mut extra,
        "备注",
        duplicate.notes.as_deref(),
        target.notes.as_deref(),
    );
    if extra.is_empty() {
        return target.notes.clone();
    }
    let addition = format!("合并历史记录信息：{}", extra.join("；"));
    let merged = match target
        .notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(notes) => format!("{notes}\n{addition}"),
        None => addition,
    };
    Some(truncate_chars(&merged, 1000))
}

fn same_merge_bucket(left: &GearRow, right: &GearRow) -> bool {
    left.user_id == right.user_id
        && left.category == right.category
        && left.is_deleted == right.is_deleted
        && left.archived_at.is_some() == right.archived_at.is_some()
}

fn same_gear_identity(left: &GearRow, right: &GearRow) -> bool {
    if same_non_empty(
        left.atlas_item_id.as_deref(),
        right.atlas_item_id.as_deref(),
    ) {
        return !variant_conflicts(
            left.selected_variant_key.as_deref(),
            left.selected_variant_label.as_deref(),
            right.selected_variant_key.as_deref(),
            right.selected_variant_label.as_deref(),
        );
    }
    if normalize_identity_text(Some(&left.name)) != normalize_identity_text(Some(&right.name)) {
        return false;
    }
    if text_conflicts(left.brand.as_deref(), right.brand.as_deref())
        || text_conflicts(left.model.as_deref(), right.model.as_deref())
        || variant_conflicts(
            left.selected_variant_key.as_deref(),
            left.selected_variant_label.as_deref(),
            right.selected_variant_key.as_deref(),
            right.selected_variant_label.as_deref(),
        )
        || specs_conflict(&left.specs, &right.specs)
    {
        return false;
    }
    same_non_empty(left.model.as_deref(), right.model.as_deref())
        || same_non_empty(left.brand.as_deref(), right.brand.as_deref())
        || variants_match(
            left.selected_variant_key.as_deref(),
            left.selected_variant_label.as_deref(),
            right.selected_variant_key.as_deref(),
            right.selected_variant_label.as_deref(),
        )
        || specs_overlap(&left.specs, &right.specs)
        || (normalize_identity_text(left.brand.as_deref()).is_empty()
            && normalize_identity_text(right.brand.as_deref()).is_empty()
            && normalize_identity_text(left.model.as_deref()).is_empty()
            && normalize_identity_text(right.model.as_deref()).is_empty())
}

const MERGE_SPEC_KEYS: &[&str] = &[
    "capacity",
    "net_content",
    "battery_capacity",
    "rated_energy",
    "output_power",
    "ports",
    "packed_size",
    "specification",
    "length",
    "people_count",
    "type",
];

fn parse_specs(value: &str) -> BTreeMap<String, String> {
    serde_json::from_str(value).unwrap_or_default()
}

fn parse_quantity(value: &str) -> Option<i32> {
    let token = value
        .trim()
        .split(|ch: char| !ch.is_ascii_digit())
        .find(|item| !item.is_empty())?;
    let quantity = token.parse::<i32>().ok()?;
    (1..=9_999).contains(&quantity).then_some(quantity)
}

fn push_note_diff(
    output: &mut Vec<String>,
    label: &str,
    incoming: Option<&str>,
    existing: Option<&str>,
) {
    let Some(value) = incoming.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    if normalize_identity_text(Some(value)) != normalize_identity_text(existing) {
        output.push(format!("{label}：{value}"));
    }
}

fn same_non_empty(left: Option<&str>, right: Option<&str>) -> bool {
    let left = normalize_identity_text(left);
    let right = normalize_identity_text(right);
    !left.is_empty() && left == right
}

fn text_conflicts(left: Option<&str>, right: Option<&str>) -> bool {
    let left = normalize_identity_text(left);
    let right = normalize_identity_text(right);
    !left.is_empty() && !right.is_empty() && left != right
}

fn variant_conflicts(
    left_key: Option<&str>,
    left_label: Option<&str>,
    right_key: Option<&str>,
    right_label: Option<&str>,
) -> bool {
    let left = normalized_variant_identity(left_key, left_label);
    let right = normalized_variant_identity(right_key, right_label);
    !left.is_empty() && !right.is_empty() && left != right
}

fn variants_match(
    left_key: Option<&str>,
    left_label: Option<&str>,
    right_key: Option<&str>,
    right_label: Option<&str>,
) -> bool {
    let left = normalized_variant_identity(left_key, left_label);
    let right = normalized_variant_identity(right_key, right_label);
    !left.is_empty() && left == right
}

fn normalized_variant_identity(key: Option<&str>, label: Option<&str>) -> String {
    let key = normalize_identity_text(key);
    if key.is_empty() {
        normalize_identity_text(label)
    } else {
        key
    }
}

fn specs_conflict(left: &BTreeMap<String, String>, right: &BTreeMap<String, String>) -> bool {
    MERGE_SPEC_KEYS.iter().any(|key| {
        let left = normalize_identity_text(left.get(*key).map(String::as_str));
        let right = normalize_identity_text(right.get(*key).map(String::as_str));
        !left.is_empty() && !right.is_empty() && left != right
    })
}

fn specs_overlap(left: &BTreeMap<String, String>, right: &BTreeMap<String, String>) -> bool {
    MERGE_SPEC_KEYS.iter().any(|key| {
        let left = normalize_identity_text(left.get(*key).map(String::as_str));
        let right = normalize_identity_text(right.get(*key).map(String::as_str));
        !left.is_empty() && left == right
    })
}

fn normalize_identity_text(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn json_string(specs: &BTreeMap<String, String>) -> Result<String, DbErr> {
    serde_json::to_string(specs).map_err(|err| DbErr::Custom(err.to_string()))
}

fn sql(backend: DatabaseBackend, query: &str) -> String {
    if !matches!(backend, DatabaseBackend::Postgres) {
        return query.to_owned();
    }
    let mut converted = String::with_capacity(query.len());
    let mut index = 1;
    for ch in query.chars() {
        if ch == '?' {
            converted.push('$');
            converted.push_str(&index.to_string());
            index += 1;
        } else {
            converted.push(ch);
        }
    }
    converted
}
