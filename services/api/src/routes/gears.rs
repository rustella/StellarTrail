//! 装备库路由模块，处理当前用户装备的列表、详情、导入导出、归档恢复和统计查询。

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use stellartrail_db::repositories::{GearRepository, ListGearOptions};
use stellartrail_domain::gear::GearItem;

use crate::{
    dto::gear::{
        CreateGearRequest, GearCategoriesResponse, GearCategoryFilterResponse, GearExportQuery,
        GearStatsQuery, ImportGearError, ImportGearsRequest, ImportGearsResponse, ListGearQuery,
        ListGearResponse, UpdateGearRequest,
    },
    error::ApiError,
    services::{auth_service, gear_service},
    state::AppState,
};

/// 执行 `routes` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/me/gears/categories", get(categories))
        .route("/api/me/gears/stats", get(stats))
        .route("/api/me/gears/export", get(export_csv))
        .route("/api/me/gears/import", post(import_json))
        .route("/api/me/gears", get(list).post(create))
        .route(
            "/api/me/gears/:id",
            get(get_one).patch(update).delete(archive),
        )
        .route("/api/me/gears/:id/restore", post(restore))
}

/// 执行 `categories` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn categories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GearStatsQuery>,
) -> Result<Json<GearCategoriesResponse>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let cache_payload = json!({ "tab": query.tab }).to_string();
    // 高频读接口先尝试 read-through cache，缓存不可用时自然跳过。
    let cache_key = state
        .cache()
        .gear_response_key(&user.id, "categories", &cache_payload)
        .await;
    if let Some(key) = cache_key.as_deref() {
        if let Some(cached) = state.cache().get_json::<GearCategoriesResponse>(key).await {
            return Ok(Json(cached));
        }
    }

    let counts = GearRepository::new(state.db().clone())
        .category_counts(&user.id, query.tab)
        .await?;
    let total = counts.iter().map(|item| item.count).sum();
    let mut items = vec![GearCategoryFilterResponse {
        id: "all".to_owned(),
        label: "全部装备".to_owned(),
        count: total,
    }];
    items.extend(counts.into_iter().map(|item| GearCategoryFilterResponse {
        id: item.category.as_str().to_owned(),
        label: item.label,
        count: item.count,
    }));
    let response = GearCategoriesResponse { items };
    if let Some(key) = cache_key.as_deref() {
        state.cache().set_json(key, &response).await;
    }
    Ok(Json(response))
}

/// 执行 `stats` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GearStatsQuery>,
) -> Result<Json<stellartrail_domain::gear::GearStats>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let cache_payload = json!({ "tab": query.tab }).to_string();
    let cache_key = state
        .cache()
        .gear_response_key(&user.id, "stats", &cache_payload)
        .await;
    if let Some(key) = cache_key.as_deref() {
        if let Some(cached) = state
            .cache()
            .get_json::<stellartrail_domain::gear::GearStats>(key)
            .await
        {
            return Ok(Json(cached));
        }
    }

    let stats = GearRepository::new(state.db().clone())
        .stats(&user.id, query.tab)
        .await?;
    if let Some(key) = cache_key.as_deref() {
        state.cache().set_json(key, &stats).await;
    }
    Ok(Json(stats))
}

/// 处理装备分页列表接口：解析查询参数、认证用户、生成缓存 key，并在未命中时读取数据库。
async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListGearQuery>,
) -> Result<Json<ListGearResponse>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let limit = query.limit.unwrap_or(20);
    let cache_payload = json!({
        "tab": query.tab,
        "category": query.category,
        "status": query.status,
        "q": query.q.as_deref(),
        "sort": query.sort,
        "limit": limit,
        "cursor": query.cursor.as_deref(),
    })
    .to_string();
    let cache_key = state
        .cache()
        .gear_response_key(&user.id, "list", &cache_payload)
        .await;
    if let Some(key) = cache_key.as_deref() {
        if let Some(cached) = state.cache().get_json::<ListGearResponse>(key).await {
            return Ok(Json(cached));
        }
    }

    let (items, next_cursor) = GearRepository::new(state.db().clone())
        .list(
            &user.id,
            &ListGearOptions {
                tab: query.tab,
                category: query.category,
                status: query.status,
                q: query.q,
                sort: query.sort,
                limit,
                cursor: query.cursor,
            },
        )
        .await?;
    let response = ListGearResponse {
        items: items.iter().map(Into::into).collect(),
        next_cursor,
    };
    if let Some(key) = cache_key.as_deref() {
        state.cache().set_json(key, &response).await;
    }
    Ok(Json(response))
}

/// 创建当前资源，并在需要时触发后续状态维护。
async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGearRequest>,
) -> Result<(StatusCode, Json<GearItem>), ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let item = gear_service::create_gear(&state, &user.id, payload.into_draft()).await?;
    // 写入成功后递增用户级版本号，避免后续读接口命中旧装备数据。
    state.cache().invalidate_user_gear(&user.id).await;
    Ok((StatusCode::CREATED, Json(item)))
}

/// 执行 `get one` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn get_one(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GearItem>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let cache_payload = json!({ "id": id }).to_string();
    let cache_key = state
        .cache()
        .gear_response_key(&user.id, "item", &cache_payload)
        .await;
    if let Some(key) = cache_key.as_deref() {
        if let Some(cached) = state.cache().get_json::<GearItem>(key).await {
            return Ok(Json(cached));
        }
    }

    let item = GearRepository::new(state.db().clone())
        .get(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if let Some(key) = cache_key.as_deref() {
        state.cache().set_json(key, &item).await;
    }
    Ok(Json(item))
}

/// 更新当前资源，并在写入成功后维护相关派生状态。
async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateGearRequest>,
) -> Result<Json<GearItem>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let item = gear_service::update_gear(&state, &user.id, &id, payload).await?;
    state.cache().invalidate_user_gear(&user.id).await;
    Ok(Json(item))
}

/// 归档当前资源，让默认列表不再展示。
async fn archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let archived = GearRepository::new(state.db().clone())
        .archive(&user.id, &id)
        .await?;
    if archived {
        state.cache().invalidate_user_gear(&user.id).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// 恢复已归档资源，让默认列表重新展示。
async fn restore(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GearItem>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let item = GearRepository::new(state.db().clone())
        .restore(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    state.cache().invalidate_user_gear(&user.id).await;
    Ok(Json(item))
}

/// 按当前筛选条件导出装备 CSV。
async fn export_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GearExportQuery>,
) -> Result<Response, ApiError> {
    if query.format != "csv" {
        return Err(ApiError::BadRequest(
            "only csv export is supported".to_owned(),
        ));
    }
    let user = auth_service::authenticate(&headers, &state).await?;
    let items = gear_service::list_for_export(&state, &user.id, query.tab).await?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record([
            "category",
            "name",
            "brand",
            "model",
            "color",
            "material",
            "capacity",
            "size",
            "description",
            "weight_g",
            "warmth_index",
            "waterproof_index",
            "purchase_date",
            "purchase_price_cents",
            "expiry_or_warranty_date",
            "purchase_location",
            "status",
            "storage_location",
            "tags",
            "share_enabled",
            "notes",
        ])
        .map_err(ApiError::internal)?;
    for item in items {
        writer
            .write_record([
                item.category.as_str().to_owned(),
                item.name,
                item.brand.unwrap_or_default(),
                item.model.unwrap_or_default(),
                item.color.unwrap_or_default(),
                item.material.unwrap_or_default(),
                item.capacity.unwrap_or_default(),
                item.size.unwrap_or_default(),
                item.description.unwrap_or_default(),
                item.weight_g.map(|v| v.to_string()).unwrap_or_default(),
                item.warmth_index.unwrap_or_default(),
                item.waterproof_index.unwrap_or_default(),
                item.purchase_date.unwrap_or_default(),
                item.purchase_price_cents
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                item.expiry_or_warranty_date.unwrap_or_default(),
                item.purchase_location.unwrap_or_default(),
                item.status.as_str().to_owned(),
                item.storage_location.unwrap_or_default(),
                item.tags.join(";"),
                item.share_enabled.to_string(),
                item.notes.unwrap_or_default(),
            ])
            .map_err(ApiError::internal)?;
    }
    let body = writer
        .into_inner()
        .map_err(|err| ApiError::internal(err.into_error()))?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"stellartrail-gears.csv\"",
            ),
        ],
        body,
    )
        .into_response())
}

/// 批量导入装备 JSON；dry-run 只校验不写库，真实导入后失效缓存。
async fn import_json(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ImportGearsRequest>,
) -> Result<Json<ImportGearsResponse>, ApiError> {
    let user = auth_service::authenticate(&headers, &state).await?;
    let mut created_count = 0;
    let mut errors = Vec::new();
    let mut drafts = Vec::new();
    for (index, item) in payload.items.into_iter().enumerate() {
        let mut draft = item.into_draft();
        match draft.validate_and_normalize() {
            Ok(()) => drafts.push(draft),
            Err(error) => {
                for field in error.fields {
                    errors.push(ImportGearError {
                        row: index + 1,
                        field: field.field,
                        message: field.message,
                    });
                }
            }
        }
    }
    if !payload.dry_run {
        let repo = GearRepository::new(state.db().clone());
        for draft in drafts {
            repo.create(&user.id, &draft).await?;
            created_count += 1;
        }
        if created_count > 0 {
            state.cache().invalidate_user_gear(&user.id).await;
        }
    }
    Ok(Json(ImportGearsResponse {
        created_count,
        updated_count: 0,
        failed_count: errors.len(),
        errors,
    }))
}
