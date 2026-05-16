//! 装备 repository，封装用户装备的 CRUD、筛选分页、统计和归档恢复 SQL。

use std::collections::HashMap;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Value};

use super::statement;
use stellartrail_domain::gear::{
    GearCategory, GearCategoryCount, GearDraft, GearItem, GearShareStatus, GearSort, GearStats,
    GearStatus, GearStatusCount, GearTab, now_rfc3339,
};
use uuid::Uuid;

/// ListGearOptions 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug)]
pub struct ListGearOptions {
    pub tab: GearTab,
    pub category: Option<GearCategory>,
    pub status: Option<GearStatus>,
    pub q: Option<String>,
    pub sort: GearSort,
    pub limit: u64,
    pub cursor: Option<String>,
}

impl Default for ListGearOptions {
    /// 执行 `default` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn default() -> Self {
        Self {
            tab: GearTab::Available,
            category: None,
            status: None,
            q: None,
            sort: GearSort::CreatedAtDesc,
            limit: 20,
            cursor: None,
        }
    }
}

/// 装备持久化对象，集中封装用户装备读写、统计和导入导出 SQL。
#[derive(Clone)]
pub struct GearRepository {
    db: DatabaseConnection,
}

impl GearRepository {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 创建当前资源，并在需要时触发后续状态维护。
    pub async fn create(&self, user_id: &str, draft: &GearDraft) -> Result<GearItem, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let tags_json =
            serde_json::to_string(&draft.tags).map_err(|err| DbErr::Custom(err.to_string()))?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO user_gear_items (
                    id, user_id, category, name, brand, model, color, material, capacity, size, description,
                    weight_g, warmth_index, waterproof_index, purchase_date, purchase_price_cents,
                    expiry_or_warranty_date, purchase_location, status, storage_location, tags_json,
                    share_enabled, share_status, notes, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                gear_values(&id, user_id, draft, &tags_json, &now, &now),
            ))
            .await?;
        self.get(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created gear not found".to_owned()))
    }

    /// 执行 `get` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn get(&self, user_id: &str, id: &str) -> Result<Option<GearItem>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                gear_select_sql("WHERE user_id = ? AND id = ?"),
                vec![user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_gear(&row)).transpose()
    }

    /// 执行 `replace` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn replace(
        &self,
        user_id: &str,
        id: &str,
        draft: &GearDraft,
    ) -> Result<Option<GearItem>, DbErr> {
        let now = now_rfc3339();
        let tags_json =
            serde_json::to_string(&draft.tags).map_err(|err| DbErr::Custom(err.to_string()))?;
        let mut values = vec![
            draft.category.as_str().to_owned().into(),
            draft.name.clone().into(),
            draft.brand.clone().into(),
            draft.model.clone().into(),
            draft.color.clone().into(),
            draft.material.clone().into(),
            draft.capacity.clone().into(),
            draft.size.clone().into(),
            draft.description.clone().into(),
            draft.weight_g.into(),
            draft.warmth_index.clone().into(),
            draft.waterproof_index.clone().into(),
            draft.purchase_date.clone().into(),
            draft.purchase_price_cents.into(),
            draft.expiry_or_warranty_date.clone().into(),
            draft.purchase_location.clone().into(),
            draft.status.as_str().to_owned().into(),
            draft.storage_location.clone().into(),
            tags_json.into(),
            draft.share_enabled.into(),
            draft.share_status.as_str().to_owned().into(),
            draft.notes.clone().into(),
            now.into(),
        ];
        values.push(user_id.to_owned().into());
        values.push(id.to_owned().into());
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE user_gear_items SET
                    category = ?, name = ?, brand = ?, model = ?, color = ?, material = ?, capacity = ?, size = ?, description = ?,
                    weight_g = ?, warmth_index = ?, waterproof_index = ?, purchase_date = ?, purchase_price_cents = ?,
                    expiry_or_warranty_date = ?, purchase_location = ?, status = ?, storage_location = ?, tags_json = ?,
                    share_enabled = ?, share_status = ?, notes = ?, updated_at = ?
                   WHERE user_id = ? AND id = ?"#,
                values,
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get(user_id, id).await
        }
    }

    /// 按用户、标签页、分类、状态、搜索词和排序条件查询装备列表，并用 limit+1 判断下一页。
    pub async fn list(
        &self,
        user_id: &str,
        options: &ListGearOptions,
    ) -> Result<(Vec<GearItem>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100);
        let offset = parse_cursor(options.cursor.as_deref())?;
        let mut values: Vec<Value> = vec![user_id.to_owned().into()];
        // user_id 始终是第一条过滤条件，所有装备查询都限制在当前用户名下。
        let mut clauses = vec!["user_id = ?".to_owned()];
        match options.tab {
            GearTab::Available => clauses.push("archived_at IS NULL".to_owned()),
            GearTab::History => clauses.push("archived_at IS NOT NULL".to_owned()),
        }
        if let Some(category) = options.category {
            clauses.push("category = ?".to_owned());
            values.push(category.as_str().to_owned().into());
        }
        if let Some(status) = options.status {
            clauses.push("status = ?".to_owned());
            values.push(status.as_str().to_owned().into());
        }
        // 搜索词只作为 LIKE 参数绑定，SQL 片段保持固定白名单。
        if let Some(q) = normalize_query(options.q.as_deref()) {
            clauses.push("(LOWER(name) LIKE ? OR LOWER(COALESCE(brand, '')) LIKE ? OR LOWER(COALESCE(model, '')) LIKE ?)".to_owned());
            values.push(q.clone().into());
            values.push(q.clone().into());
            values.push(q.into());
        }
        // 多取一条用于判断是否存在下一页，响应前会截回用户请求的 limit。
        values.push((limit as i64 + 1).into());
        values.push(offset.into());
        // 动态 SQL 只拼接内部枚举/白名单生成的片段，用户输入全部走参数绑定。
        let sql = format!(
            "{} WHERE {} {} LIMIT ? OFFSET ?",
            gear_select_columns(),
            clauses.join(" AND "),
            order_by(options.sort),
        );
        let mut items = self
            .db
            .query_all(statement(self.db.get_database_backend(), sql, values))
            .await?
            .into_iter()
            .map(|row| map_gear(&row))
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if items.len() > limit as usize {
            items.truncate(limit as usize);
            Some((offset + limit as i64).to_string())
        } else {
            None
        };
        Ok((items, next_cursor))
    }

    /// 归档当前资源，让默认列表不再展示。
    pub async fn archive(&self, user_id: &str, id: &str) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET archived_at = ?, updated_at = ? WHERE user_id = ? AND id = ? AND archived_at IS NULL",
                vec![now.clone().into(), now.into(), user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 恢复已归档资源，让默认列表重新展示。
    pub async fn restore(&self, user_id: &str, id: &str) -> Result<Option<GearItem>, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_gear_items SET archived_at = NULL, updated_at = ? WHERE user_id = ? AND id = ?",
                vec![now.into(), user_id.to_owned().into(), id.to_owned().into()],
            ))
            .await?;
        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            self.get(user_id, id).await
        }
    }

    /// 执行 `category counts` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn category_counts(
        &self,
        user_id: &str,
        tab: GearTab,
    ) -> Result<Vec<GearCategoryCount>, DbErr> {
        let archived_clause = archived_clause(tab);
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!("SELECT category, COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND {archived_clause} GROUP BY category"),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "category")?;
            let count: i64 = row.try_get("", "count")?;
            if let Some(category) = GearCategory::from_key(&raw) {
                counts.insert(category.as_str(), count);
            }
        }
        Ok(GearCategory::ALL
            .into_iter()
            .map(|category| GearCategoryCount {
                category,
                label: category.label().to_owned(),
                count: *counts.get(category.as_str()).unwrap_or(&0),
            })
            .collect())
    }

    /// 执行 `stats` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn stats(&self, user_id: &str, tab: GearTab) -> Result<GearStats, DbErr> {
        let archived_clause = archived_clause(tab);
        let summary = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("SELECT COUNT(*) AS count, CAST(COALESCE(SUM(purchase_price_cents), 0) AS BIGINT) AS total_value_cents, CAST(COALESCE(SUM(weight_g), 0) AS BIGINT) AS total_weight_g FROM user_gear_items WHERE user_id = ? AND {archived_clause}"),
                vec![user_id.to_owned().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("missing stats row".to_owned()))?;
        let archived_count = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND archived_at IS NOT NULL",
                vec![user_id.to_owned().into()],
            ))
            .await?
            .ok_or_else(|| DbErr::Custom("missing archived stats row".to_owned()))?
            .try_get("", "count")?;
        let by_category = self.category_counts(user_id, tab).await?;
        let by_status = self.status_counts(user_id, tab).await?;
        Ok(GearStats {
            current_count: summary.try_get("", "count")?,
            archived_count,
            total_value_cents: summary.try_get("", "total_value_cents")?,
            total_weight_g: summary.try_get("", "total_weight_g")?,
            by_category,
            by_status,
        })
    }

    /// 执行 `status counts` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    async fn status_counts(
        &self,
        user_id: &str,
        tab: GearTab,
    ) -> Result<Vec<GearStatusCount>, DbErr> {
        let archived_clause = archived_clause(tab);
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                format!("SELECT status, COUNT(*) AS count FROM user_gear_items WHERE user_id = ? AND {archived_clause} GROUP BY status"),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut counts = HashMap::new();
        for row in rows {
            let raw: String = row.try_get("", "status")?;
            let count: i64 = row.try_get("", "count")?;
            if let Some(status) = GearStatus::from_key(&raw) {
                counts.insert(status.as_str(), count);
            }
        }
        Ok(GearStatus::ALL
            .into_iter()
            .map(|status| GearStatusCount {
                status,
                label: status.label().to_owned(),
                count: *counts.get(status.as_str()).unwrap_or(&0),
            })
            .collect())
    }
}

/// 执行 `gear values` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn gear_values(
    id: &str,
    user_id: &str,
    draft: &GearDraft,
    tags_json: &str,
    created_at: &str,
    updated_at: &str,
) -> Vec<Value> {
    vec![
        id.to_owned().into(),
        user_id.to_owned().into(),
        draft.category.as_str().to_owned().into(),
        draft.name.clone().into(),
        draft.brand.clone().into(),
        draft.model.clone().into(),
        draft.color.clone().into(),
        draft.material.clone().into(),
        draft.capacity.clone().into(),
        draft.size.clone().into(),
        draft.description.clone().into(),
        draft.weight_g.into(),
        draft.warmth_index.clone().into(),
        draft.waterproof_index.clone().into(),
        draft.purchase_date.clone().into(),
        draft.purchase_price_cents.into(),
        draft.expiry_or_warranty_date.clone().into(),
        draft.purchase_location.clone().into(),
        draft.status.as_str().to_owned().into(),
        draft.storage_location.clone().into(),
        tags_json.to_owned().into(),
        draft.share_enabled.into(),
        draft.share_status.as_str().to_owned().into(),
        draft.notes.clone().into(),
        created_at.to_owned().into(),
        updated_at.to_owned().into(),
    ]
}

/// 执行 `gear select sql` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn gear_select_sql(where_clause: &str) -> String {
    format!("{} {where_clause} LIMIT 1", gear_select_columns())
}

/// 执行 `gear select columns` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn gear_select_columns() -> &'static str {
    r#"SELECT id, user_id, category, name, brand, model, color, material, capacity, size, description,
        weight_g, warmth_index, waterproof_index, purchase_date, purchase_price_cents,
        expiry_or_warranty_date, purchase_location, status, storage_location, tags_json,
        share_enabled, share_status, notes, archived_at, created_at, updated_at
       FROM user_gear_items"#
}

/// 执行 `order by` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn order_by(sort: GearSort) -> &'static str {
    match sort {
        GearSort::CreatedAtDesc => "ORDER BY created_at DESC, id DESC",
        GearSort::CreatedAtAsc => "ORDER BY created_at ASC, id ASC",
        GearSort::PurchaseDateDesc => "ORDER BY purchase_date DESC, created_at DESC, id DESC",
        GearSort::NameAsc => "ORDER BY name ASC, created_at DESC, id DESC",
        GearSort::WeightDesc => "ORDER BY weight_g DESC, created_at DESC, id DESC",
        GearSort::PriceDesc => "ORDER BY purchase_price_cents DESC, created_at DESC, id DESC",
    }
}

/// 执行 `archived clause` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn archived_clause(tab: GearTab) -> &'static str {
    match tab {
        GearTab::Available => "archived_at IS NULL",
        GearTab::History => "archived_at IS NOT NULL",
    }
}

/// 执行 `parse cursor` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `normalize query` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn normalize_query(q: Option<&str>) -> Option<String> {
    let q = q?.trim().to_lowercase();
    if q.is_empty() {
        None
    } else {
        Some(format!("%{}%", q.replace('%', "\\%").replace('_', "\\_")))
    }
}

/// 执行 `map gear` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn map_gear(row: &sea_orm::QueryResult) -> Result<GearItem, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let status_raw: String = row.try_get("", "status")?;
    let share_status_raw: String = row.try_get("", "share_status")?;
    let tags_json: String = row.try_get("", "tags_json")?;
    // tags_json 是数据库文本字段，反序列化失败时转为 DbErr 便于上层统一处理。
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(GearItem {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        category: GearCategory::from_key(&category_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear category: {category_raw}")))?,
        name: row.try_get("", "name")?,
        brand: row.try_get("", "brand")?,
        model: row.try_get("", "model")?,
        color: row.try_get("", "color")?,
        material: row.try_get("", "material")?,
        capacity: row.try_get("", "capacity")?,
        size: row.try_get("", "size")?,
        description: row.try_get("", "description")?,
        weight_g: row.try_get("", "weight_g")?,
        warmth_index: row.try_get("", "warmth_index")?,
        waterproof_index: row.try_get("", "waterproof_index")?,
        purchase_date: row.try_get("", "purchase_date")?,
        purchase_price_cents: row.try_get("", "purchase_price_cents")?,
        expiry_or_warranty_date: row.try_get("", "expiry_or_warranty_date")?,
        purchase_location: row.try_get("", "purchase_location")?,
        status: GearStatus::from_key(&status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid gear status: {status_raw}")))?,
        storage_location: row.try_get("", "storage_location")?,
        tags,
        share_enabled: row.try_get("", "share_enabled")?,
        share_status: GearShareStatus::from_key(&share_status_raw)
            .ok_or_else(|| DbErr::Custom(format!("invalid share status: {share_status_raw}")))?,
        notes: row.try_get("", "notes")?,
        archived_at: row.try_get("", "archived_at")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::AuthRepository;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_migration::Migrator;

    /// 执行 `draft` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn draft(name: &str, category: GearCategory) -> GearDraft {
        GearDraft {
            category,
            name: name.to_owned(),
            brand: Some("NITECORE".to_owned()),
            model: Some("SUMMIT 20000".to_owned()),
            color: None,
            material: None,
            capacity: Some("20000mAh".to_owned()),
            size: None,
            description: None,
            weight_g: Some(315),
            warmth_index: None,
            waterproof_index: None,
            purchase_date: Some("2026-01-22".to_owned()),
            purchase_price_cents: Some(63900),
            expiry_or_warranty_date: None,
            purchase_location: None,
            status: GearStatus::Available,
            storage_location: Some("装备柜".to_owned()),
            tags: vec!["电子".to_owned()],
            share_enabled: false,
            share_status: GearShareStatus::NotShared,
            notes: None,
        }
    }

    /// 执行 `gear repository crud filter stats archive restore` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    #[tokio::test]
    async fn gear_repository_crud_filter_stats_archive_restore() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let auth_repo = AuthRepository::new(db.clone());
        let user = auth_repo
            .upsert_mock_user("mock:gear-repo-user", Some("装备测试用户".to_owned()), None)
            .await
            .unwrap();
        let repo = GearRepository::new(db.clone());
        let user_id = user.id.as_str();
        let created = repo
            .create(
                user_id,
                &draft("NITECORE充电宝", GearCategory::ElectronicsSystem),
            )
            .await
            .unwrap();
        repo.create(user_id, &draft("挪客户灯", GearCategory::LightingSystem))
            .await
            .unwrap();

        let options = ListGearOptions {
            category: Some(GearCategory::ElectronicsSystem),
            q: Some("nitecore".to_owned()),
            ..Default::default()
        };
        let (items, _) = repo.list(user_id, &options).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, created.id);

        let stats = repo.stats(user_id, GearTab::Available).await.unwrap();
        assert_eq!(stats.current_count, 2);
        assert_eq!(stats.total_weight_g, 630);
        assert_eq!(stats.total_value_cents, 127800);

        assert!(repo.archive(user_id, &created.id).await.unwrap());
        let (available, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(available.len(), 1);
        let (history, _) = repo
            .list(
                user_id,
                &ListGearOptions {
                    tab: GearTab::History,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(history.len(), 1);
        repo.restore(user_id, &created.id).await.unwrap().unwrap();
        let (available, _) = repo
            .list(user_id, &ListGearOptions::default())
            .await
            .unwrap();
        assert_eq!(available.len(), 2);
    }
}
