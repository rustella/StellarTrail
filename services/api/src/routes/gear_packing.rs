//! Gear packing-list routes for authenticated users preparing route-specific loadouts.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use stellartrail_db::repositories::{GearPackingRepository, ListGearPackingListsOptions};
use stellartrail_domain::{
    gear::GearItem,
    validation::{FieldViolation, ValidationError},
};

use crate::{
    dto::gear_packing::{
        AddGearPackingItemsRequest, CreateGearPackingListRequest, GearPackingListDetailResponse,
        GearPackingListItemResponse, ListGearPackingListsQuery, ListGearPackingListsResponse,
        UpdateGearPackingItemRequest, UpdateGearPackingListRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    state::AppState,
};

/// Builds authenticated packing-list routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/packing-lists", get(list).post(create))
        .route(
            "/me/packing-lists/:id",
            get(get_one).patch(update).delete(soft_delete),
        )
        .route(
            "/me/packing-lists/:id/items",
            axum::routing::post(add_items),
        )
        .route(
            "/me/packing-lists/:id/items/:item_id",
            axum::routing::patch(update_item).delete(remove_item),
        )
}

/// Returns paginated packing-list summaries for the current user.
async fn list(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListGearPackingListsQuery>,
) -> Result<Json<ListGearPackingListsResponse>, ApiError> {
    let (items, next_cursor) = GearPackingRepository::new(state.db().clone())
        .list(
            &user.id,
            &ListGearPackingListsOptions {
                limit: query.limit.unwrap_or(20),
                cursor: query.cursor,
            },
        )
        .await?;
    Ok(Json(ListGearPackingListsResponse {
        items: items.into_iter().map(Into::into).collect(),
        next_cursor,
    }))
}

/// Creates a packing list for the current user.
async fn create(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateGearPackingListRequest>,
) -> Result<(StatusCode, Json<GearPackingListDetailResponse>), ApiError> {
    let mut draft = payload.into_draft();
    draft.validate_and_normalize()?;
    let repo = GearPackingRepository::new(state.db().clone());
    let detail = repo.create(&user.id, &draft).await?;
    let response = build_detail_response(&state, &repo, &user.id, detail).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Reads one packing-list detail.
async fn get_one(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearPackingListDetailResponse>, ApiError> {
    let repo = GearPackingRepository::new(state.db().clone());
    let detail = repo
        .detail(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        build_detail_response(&state, &repo, &user.id, detail).await?,
    ))
}

/// Updates list metadata.
async fn update(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateGearPackingListRequest>,
) -> Result<Json<GearPackingListDetailResponse>, ApiError> {
    let mut draft = payload.into_draft();
    draft.validate_and_normalize()?;
    let repo = GearPackingRepository::new(state.db().clone());
    let detail = repo
        .update(&user.id, &id, &draft)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        build_detail_response(&state, &repo, &user.id, detail).await?,
    ))
}

/// Soft-deletes one packing list.
async fn soft_delete(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let deleted = GearPackingRepository::new(state.db().clone())
        .soft_delete(&user.id, &id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Bulk-adds user gear to a packing list.
async fn add_items(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<AddGearPackingItemsRequest>,
) -> Result<Json<GearPackingListDetailResponse>, ApiError> {
    let gear_ids = payload.normalized_gear_ids()?;
    let repo = GearPackingRepository::new(state.db().clone());
    let result = repo.add_items(&user.id, &id, &gear_ids).await?;
    if !result.invalid_gear_ids.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "gear_ids",
            "contains unavailable or unknown gear ids",
        )]));
    }
    let detail = result.detail.ok_or(ApiError::NotFound)?;
    Ok(Json(
        build_detail_response(&state, &repo, &user.id, detail).await?,
    ))
}

/// Toggles whether a packing-list item has been packed.
async fn update_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<UpdateGearPackingItemRequest>,
) -> Result<Json<GearPackingListDetailResponse>, ApiError> {
    let repo = GearPackingRepository::new(state.db().clone());
    let detail = repo
        .update_item_packed(&user.id, &id, &item_id, payload.packed)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        build_detail_response(&state, &repo, &user.id, detail).await?,
    ))
}

/// Removes one item from a packing list.
async fn remove_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<GearPackingListDetailResponse>, ApiError> {
    let repo = GearPackingRepository::new(state.db().clone());
    let detail = repo
        .remove_item(&user.id, &id, &item_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(
        build_detail_response(&state, &repo, &user.id, detail).await?,
    ))
}

async fn build_detail_response(
    state: &AppState,
    repo: &GearPackingRepository,
    user_id: &str,
    detail: stellartrail_domain::gear_packing::GearPackingListDetail,
) -> Result<GearPackingListDetailResponse, ApiError> {
    let mut pairs = Vec::new();
    let mut tags = Vec::new();
    for item in &detail.items {
        let gear = repo
            .gear_for_item(user_id, &item.gear_id)
            .await?
            .ok_or_else(|| ApiError::internal(missing_gear_error(&item.gear_id)))?;
        collect_tags(&gear, &mut tags);
        pairs.push((item.clone(), gear));
    }
    let tag_colors = state.cache().gear_tag_colors(user_id, &tags).await;
    let items = pairs
        .into_iter()
        .map(|(item, gear)| {
            GearPackingListItemResponse::from_item_and_gear(item, &gear, &tag_colors)
        })
        .collect();
    Ok(GearPackingListDetailResponse::from_detail(detail, items))
}

fn collect_tags(item: &GearItem, tags: &mut Vec<String>) {
    for tag in &item.tags {
        if !tags.iter().any(|existing| existing == tag) {
            tags.push(tag.clone());
        }
    }
}

fn missing_gear_error(gear_id: &str) -> ValidationError {
    ValidationError::single(
        "gear_id",
        format!("packing list item points to missing gear `{gear_id}`"),
    )
}
