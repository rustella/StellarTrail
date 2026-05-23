//! Gear packing-list repository wrapping SQL for user-owned route preparation lists.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult};
use stellartrail_domain::{
    gear::{GearItem, now_rfc3339},
    gear_packing::{
        GearPackingList, GearPackingListDetail, GearPackingListDraft, GearPackingListItem,
        GearPackingListStats, GearPackingListSummary,
    },
};
use uuid::Uuid;

use super::{GearRepository, statement};

/// Pagination options for packing-list index reads.
#[derive(Clone, Debug)]
pub struct ListGearPackingListsOptions {
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearPackingListsOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            cursor: None,
        }
    }
}

/// Result returned by a bulk item-add request.
#[derive(Clone, Debug)]
pub struct AddGearPackingItemsResult {
    pub detail: Option<GearPackingListDetail>,
    pub invalid_gear_ids: Vec<String>,
}

/// Persistence object for packing-list metadata and item state.
#[derive(Clone)]
pub struct GearPackingRepository {
    db: DatabaseConnection,
}

impl GearPackingRepository {
    /// Creates a repository bound to the supplied database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates one packing list for the current user.
    pub async fn create(
        &self,
        user_id: &str,
        draft: &GearPackingListDraft,
    ) -> Result<GearPackingListDetail, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO gear_packing_lists \
                 (id, user_id, name, route_name, duration_label, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.name.clone().into(),
                    draft.route_name.clone().into(),
                    draft.duration_label.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.detail(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created packing list not found".to_owned()))
    }

    /// Lists active packing lists with aggregate item counters.
    pub async fn list(
        &self,
        user_id: &str,
        options: &ListGearPackingListsOptions,
    ) -> Result<(Vec<GearPackingListSummary>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE l.user_id = ? AND l.is_deleted = FALSE \
                     GROUP BY l.id, l.user_id, l.name, l.route_name, l.duration_label, \
                              l.is_deleted, l.created_at, l.updated_at \
                     ORDER BY l.updated_at DESC, l.id DESC LIMIT ? OFFSET ?",
                    packing_list_summary_select_sql(),
                ),
                vec![
                    user_id.to_owned().into(),
                    (limit as i64 + 1).into(),
                    offset.into(),
                ],
            ))
            .await?;
        let mut items = rows
            .iter()
            .map(map_packing_list_summary)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if items.len() > limit as usize {
            items.truncate(limit as usize);
            Some((offset + limit as i64).to_string())
        } else {
            None
        };
        Ok((items, next_cursor))
    }

    /// Reads one active packing list plus its item rows.
    pub async fn detail(
        &self,
        user_id: &str,
        list_id: &str,
    ) -> Result<Option<GearPackingListDetail>, DbErr> {
        let Some(summary) = self.summary(user_id, list_id).await? else {
            return Ok(None);
        };
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT id, packing_list_id, user_id, gear_id, planned_quantity, packed_quantity, packed, created_at, updated_at \
                 FROM gear_packing_list_items \
                 WHERE user_id = ? AND packing_list_id = ? \
                 ORDER BY created_at ASC, id ASC",
                vec![user_id.to_owned().into(), list_id.to_owned().into()],
            ))
            .await?;
        let items = rows
            .iter()
            .map(map_packing_list_item)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(GearPackingListDetail {
            list: summary.list,
            stats: summary.stats,
            items,
        }))
    }

    /// Updates packing-list metadata.
    pub async fn update(
        &self,
        user_id: &str,
        list_id: &str,
        draft: &GearPackingListDraft,
    ) -> Result<Option<GearPackingListDetail>, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_packing_lists \
                 SET name = ?, route_name = ?, duration_label = ?, updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    draft.name.clone().into(),
                    draft.route_name.clone().into(),
                    draft.duration_label.clone().into(),
                    now.into(),
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.detail(user_id, list_id).await
        }
    }

    /// Soft-deletes a packing list while preserving item rows for audit and potential future recovery.
    pub async fn soft_delete(&self, user_id: &str, list_id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_packing_lists SET is_deleted = TRUE, updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Adds valid available gear rows to a packing list; duplicate gear ids are ignored.
    pub async fn add_items(
        &self,
        user_id: &str,
        list_id: &str,
        gear_ids: &[String],
    ) -> Result<AddGearPackingItemsResult, DbErr> {
        if self.summary(user_id, list_id).await?.is_none() {
            return Ok(AddGearPackingItemsResult {
                detail: None,
                invalid_gear_ids: Vec::new(),
            });
        }
        let invalid_gear_ids = self.invalid_addable_gear_ids(user_id, gear_ids).await?;
        if !invalid_gear_ids.is_empty() {
            return Ok(AddGearPackingItemsResult {
                detail: self.detail(user_id, list_id).await?,
                invalid_gear_ids,
            });
        }
        for gear_id in gear_ids {
            if self.item_exists(user_id, list_id, gear_id).await? {
                continue;
            }
            let id = Uuid::new_v4().to_string();
            let now = now_rfc3339();
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "INSERT INTO gear_packing_list_items \
                     (id, packing_list_id, user_id, gear_id, planned_quantity, packed_quantity, packed, created_at, updated_at) \
                     VALUES (?, ?, ?, ?, 1, 0, FALSE, ?, ?)",
                    vec![
                        id.into(),
                        list_id.to_owned().into(),
                        user_id.to_owned().into(),
                        gear_id.to_owned().into(),
                        now.clone().into(),
                        now.into(),
                    ],
                ))
                .await?;
        }
        let detail = self.detail(user_id, list_id).await?;
        Ok(AddGearPackingItemsResult {
            detail,
            invalid_gear_ids: Vec::new(),
        })
    }

    /// Updates planned and packed quantities for one packing-list item.
    pub async fn update_item_quantities(
        &self,
        user_id: &str,
        list_id: &str,
        item_id: &str,
        planned_quantity: Option<i32>,
        packed_quantity: Option<i32>,
        packed: Option<bool>,
    ) -> Result<Option<GearPackingListDetail>, DbErr> {
        if self.summary(user_id, list_id).await?.is_none() {
            return Ok(None);
        }
        let Some(current) = self.item(user_id, list_id, item_id).await? else {
            return Ok(None);
        };
        let stock_quantity = self
            .gear_for_item(user_id, &current.gear_id)
            .await?
            .map(|gear| gear.quantity.max(1))
            .unwrap_or(1);
        let planned_quantity = planned_quantity
            .unwrap_or(current.planned_quantity)
            .clamp(1, stock_quantity);
        let packed_quantity = if let Some(value) = packed_quantity {
            value.clamp(0, planned_quantity)
        } else if let Some(packed) = packed {
            if packed { planned_quantity } else { 0 }
        } else {
            current.packed_quantity.clamp(0, planned_quantity)
        };
        let packed = packed_quantity >= planned_quantity;
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_packing_list_items \
                 SET planned_quantity = ?, packed_quantity = ?, packed = ?, updated_at = ? \
                 WHERE user_id = ? AND packing_list_id = ? AND id = ?",
                vec![
                    planned_quantity.into(),
                    packed_quantity.into(),
                    packed.into(),
                    now.into(),
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                    item_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.touch_list(user_id, list_id).await?;
            self.detail(user_id, list_id).await
        }
    }

    /// Removes one item from a packing list.
    pub async fn remove_item(
        &self,
        user_id: &str,
        list_id: &str,
        item_id: &str,
    ) -> Result<Option<GearPackingListDetail>, DbErr> {
        if self.summary(user_id, list_id).await?.is_none() {
            return Ok(None);
        }
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "DELETE FROM gear_packing_list_items \
                 WHERE user_id = ? AND packing_list_id = ? AND id = ?",
                vec![
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                    item_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.touch_list(user_id, list_id).await?;
            self.detail(user_id, list_id).await
        }
    }

    /// Reads one gear row for detail rendering, including archived or soft-deleted records.
    pub async fn gear_for_item(
        &self,
        user_id: &str,
        gear_id: &str,
    ) -> Result<Option<GearItem>, DbErr> {
        GearRepository::new(self.db.clone())
            .get_any_for_user(user_id, gear_id)
            .await
    }

    async fn summary(
        &self,
        user_id: &str,
        list_id: &str,
    ) -> Result<Option<GearPackingListSummary>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "{} WHERE l.user_id = ? AND l.id = ? AND l.is_deleted = FALSE \
                     GROUP BY l.id, l.user_id, l.name, l.route_name, l.duration_label, \
                              l.is_deleted, l.created_at, l.updated_at",
                    packing_list_summary_select_sql(),
                ),
                vec![user_id.to_owned().into(), list_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_packing_list_summary).transpose()
    }

    async fn invalid_addable_gear_ids(
        &self,
        user_id: &str,
        gear_ids: &[String],
    ) -> Result<Vec<String>, DbErr> {
        let gear_repo = GearRepository::new(self.db.clone());
        let mut invalid = Vec::new();
        for gear_id in gear_ids {
            let gear = gear_repo.get_any_for_user(user_id, gear_id).await?;
            let addable = gear
                .as_ref()
                .is_some_and(|item| !item.is_deleted && item.archived_at.is_none());
            if !addable {
                invalid.push(gear_id.clone());
            }
        }
        Ok(invalid)
    }

    async fn item_exists(
        &self,
        user_id: &str,
        list_id: &str,
        gear_id: &str,
    ) -> Result<bool, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id FROM gear_packing_list_items \
                 WHERE user_id = ? AND packing_list_id = ? AND gear_id = ? LIMIT 1",
                vec![
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                    gear_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(row.is_some())
    }

    async fn item(
        &self,
        user_id: &str,
        list_id: &str,
        item_id: &str,
    ) -> Result<Option<GearPackingListItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, packing_list_id, user_id, gear_id, planned_quantity, packed_quantity, packed, created_at, updated_at \
                 FROM gear_packing_list_items \
                 WHERE user_id = ? AND packing_list_id = ? AND id = ? LIMIT 1",
                vec![
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                    item_id.to_owned().into(),
                ],
            ))
            .await?;
        row.as_ref().map(map_packing_list_item).transpose()
    }

    async fn touch_list(&self, user_id: &str, list_id: &str) -> Result<(), DbErr> {
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE gear_packing_lists SET updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    user_id.to_owned().into(),
                    list_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(())
    }
}

fn packing_list_summary_select_sql() -> &'static str {
    "SELECT l.id, l.user_id, l.name, l.route_name, l.duration_label, l.is_deleted, \
            l.created_at, l.updated_at, \
            CAST(COALESCE(SUM(CASE WHEN i.id IS NULL THEN 0 ELSE i.planned_quantity END), 0) AS BIGINT) AS item_count, \
            CAST(COALESCE(SUM(COALESCE(i.packed_quantity, 0)), 0) AS BIGINT) AS packed_count, \
            CAST(COALESCE(SUM(COALESCE(g.weight_g, 0) * COALESCE(i.planned_quantity, 0)), 0) AS BIGINT) AS total_weight_g \
     FROM gear_packing_lists l \
     LEFT JOIN gear_packing_list_items i ON i.packing_list_id = l.id AND i.user_id = l.user_id \
     LEFT JOIN user_gear_items g ON g.id = i.gear_id AND g.user_id = i.user_id"
}

fn map_packing_list_summary(row: &QueryResult) -> Result<GearPackingListSummary, DbErr> {
    Ok(GearPackingListSummary {
        list: GearPackingList {
            id: row.try_get("", "id")?,
            user_id: row.try_get("", "user_id")?,
            name: row.try_get("", "name")?,
            route_name: row.try_get("", "route_name")?,
            duration_label: row.try_get("", "duration_label")?,
            is_deleted: row.try_get("", "is_deleted")?,
            created_at: row.try_get("", "created_at")?,
            updated_at: row.try_get("", "updated_at")?,
        },
        stats: GearPackingListStats {
            item_count: row.try_get("", "item_count")?,
            packed_count: row.try_get("", "packed_count")?,
            total_weight_g: row.try_get("", "total_weight_g")?,
        },
    })
}

fn map_packing_list_item(row: &QueryResult) -> Result<GearPackingListItem, DbErr> {
    Ok(GearPackingListItem {
        id: row.try_get("", "id")?,
        packing_list_id: row.try_get("", "packing_list_id")?,
        user_id: row.try_get("", "user_id")?,
        gear_id: row.try_get("", "gear_id")?,
        planned_quantity: row.try_get("", "planned_quantity")?,
        packed_quantity: row.try_get("", "packed_quantity")?,
        packed: row.try_get("", "packed")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::AuthRepository;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_domain::gear::{
        GearCategory, GearDraft, GearShareStatus, GearSpecs, GearStatus,
    };
    use stellartrail_migration::Migrator;

    fn gear_draft(name: &str, weight_g: i32) -> GearDraft {
        GearDraft {
            category: GearCategory::BackpackSystem,
            name: name.to_owned(),
            brand: None,
            model: None,
            description: None,
            weight_g: Some(weight_g),
            official_price_cents: None,
            official_price_currency: None,
            purchase_date: None,
            purchase_price_cents: None,
            purchase_price_currency: None,
            purchase_location: None,
            status: GearStatus::Available,
            storage_location: None,
            atlas_item_id: None,
            selected_variant_key: None,
            selected_variant_label: None,
            quantity: 1,
            specs: GearSpecs::new(),
            tags: Vec::new(),
            share_enabled: false,
            share_status: GearShareStatus::NotShared,
            notes: None,
        }
    }

    #[tokio::test]
    async fn packing_repository_crud_user_scope_idempotent_add_and_stats() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let auth_repo = AuthRepository::new(db.clone());
        let user = auth_repo
            .upsert_mock_user("mock:packing-user", Some("打包用户".to_owned()), None)
            .await
            .unwrap();
        let other = auth_repo
            .upsert_mock_user("mock:packing-other", Some("其它用户".to_owned()), None)
            .await
            .unwrap();
        let gear_repo = GearRepository::new(db.clone());
        let mut backpack_draft = gear_draft("轻量背包", 800);
        backpack_draft.quantity = 2;
        let backpack = gear_repo.create(&user.id, &backpack_draft).await.unwrap();
        let headlamp = gear_repo
            .create(&user.id, &gear_draft("头灯", 90))
            .await
            .unwrap();
        let other_gear = gear_repo
            .create(&other.id, &gear_draft("别人的装备", 100))
            .await
            .unwrap();

        let repo = GearPackingRepository::new(db.clone());
        let mut draft = GearPackingListDraft {
            name: " 武功山一日 ".to_owned(),
            route_name: Some(" 武功山 ".to_owned()),
            duration_label: Some(" 一日 ".to_owned()),
        };
        draft.validate_and_normalize().unwrap();
        let created = repo.create(&user.id, &draft).await.unwrap();
        assert_eq!(created.list.name, "武功山一日");
        assert_eq!(created.stats.item_count, 0);

        let add = repo
            .add_items(
                &user.id,
                &created.list.id,
                &[
                    backpack.id.clone(),
                    headlamp.id.clone(),
                    backpack.id.clone(),
                ],
            )
            .await
            .unwrap();
        assert!(add.invalid_gear_ids.is_empty());
        let detail = add.detail.unwrap();
        assert_eq!(detail.items.len(), 2);
        assert_eq!(detail.stats.item_count, 2);
        assert_eq!(detail.stats.total_weight_g, 890);

        let invalid = repo
            .add_items(&user.id, &created.list.id, &[other_gear.id.clone()])
            .await
            .unwrap();
        assert_eq!(invalid.invalid_gear_ids, vec![other_gear.id]);

        let first_item_id = detail.items[0].id.clone();
        let checked = repo
            .update_item_quantities(
                &user.id,
                &created.list.id,
                &first_item_id,
                Some(2),
                Some(1),
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(checked.stats.packed_count, 1);
        assert_eq!(checked.stats.item_count, 3);
        assert_eq!(checked.stats.total_weight_g, 1690);

        let (lists, next_cursor) = repo
            .list(&user.id, &ListGearPackingListsOptions::default())
            .await
            .unwrap();
        assert_eq!(lists.len(), 1);
        assert!(next_cursor.is_none());
        assert_eq!(lists[0].stats.packed_count, 1);

        let other_detail = repo.detail(&other.id, &created.list.id).await.unwrap();
        assert!(other_detail.is_none());

        let removed = repo
            .remove_item(&user.id, &created.list.id, &first_item_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(removed.stats.item_count, 1);

        assert!(repo.soft_delete(&user.id, &created.list.id).await.unwrap());
        assert!(
            repo.detail(&user.id, &created.list.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn packing_repository_rejects_archived_or_deleted_adds_but_keeps_existing_items() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let user = AuthRepository::new(db.clone())
            .upsert_mock_user("mock:packing-archive", Some("打包用户".to_owned()), None)
            .await
            .unwrap();
        let gear_repo = GearRepository::new(db.clone());
        let archived = gear_repo
            .create(&user.id, &gear_draft("会归档的装备", 500))
            .await
            .unwrap();
        let deleted = gear_repo
            .create(&user.id, &gear_draft("会删除的装备", 300))
            .await
            .unwrap();
        let repo = GearPackingRepository::new(db.clone());
        let mut draft = GearPackingListDraft {
            name: "周末路线".to_owned(),
            route_name: None,
            duration_label: None,
        };
        draft.validate_and_normalize().unwrap();
        let list = repo.create(&user.id, &draft).await.unwrap();
        repo.add_items(&user.id, &list.list.id, std::slice::from_ref(&archived.id))
            .await
            .unwrap();

        gear_repo.archive(&user.id, &archived.id).await.unwrap();
        gear_repo.soft_delete(&user.id, &deleted.id).await.unwrap();
        let rejected = repo
            .add_items(
                &user.id,
                &list.list.id,
                &[archived.id.clone(), deleted.id.clone()],
            )
            .await
            .unwrap();
        assert_eq!(
            rejected.invalid_gear_ids,
            vec![archived.id.clone(), deleted.id]
        );

        let detail = repo.detail(&user.id, &list.list.id).await.unwrap().unwrap();
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].gear_id, archived.id);
    }
}
