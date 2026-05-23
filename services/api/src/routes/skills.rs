//! Public outdoor skill routes for DB-backed skill categories and knots.

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::get,
};
use stellartrail_db::repositories::{KnotFavoriteStatus, SkillFavoriteCounts};
use stellartrail_domain::skill::{KnotSummary, PageInfo, SkillCategoriesResponse};

use crate::{
    dto::skill_favorites::{
        FavoriteKnotItemResponse, FavoriteKnotStatusResponse, FavoriteSkillFilterResponse,
        ListFavoriteSkillsResponse,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    state::AppState,
};

use super::localization::{
    cached_localized_json_with, parse_u32_query, reject_all_query_parameters, reject_query_locale,
    resolve_locale,
};

/// Builds all DB-backed outdoor skill routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/skills", get(skill_categories))
        .route("/skills/knots/list", get(knot_list))
        .route("/skills/knots/filters", get(knot_filters))
        .route("/skills/knots/offline-manifest", get(knot_offline_manifest))
        .route("/skills/knots/detail/:id", get(knot_detail))
        .route("/me/skills/favorites", get(list_favorite_skills))
        .route(
            "/me/skills/favorites/knots/:id",
            get(knot_favorite_status)
                .put(favorite_knot)
                .delete(unfavorite_knot),
        )
}

async fn skill_categories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    if query.contains_key("category") {
        return Err(ApiError::NotFound);
    }
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-categories",
        &format!("v1|{}", locale.as_str()),
        || async {
            let items = state
                .knot_repository()
                .list_skill_categories(locale)
                .await?;
            Ok(SkillCategoriesResponse { items })
        },
    )
    .await
}

async fn knot_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    if query.contains_key("cursor") {
        return Err(ApiError::unsupported_query_parameter("cursor"));
    }
    if query.contains_key("next_cursor") {
        return Err(ApiError::unsupported_query_parameter("next_cursor"));
    }
    if query.contains_key("difficulty") {
        return Err(ApiError::unsupported_query_parameter("difficulty"));
    }
    let locale = resolve_locale(&headers)?;
    let offset = parse_u32_query(&query, "offset", 0)?;
    let limit = parse_u32_query(&query, "limit", 20)?.clamp(1, 100);
    let category = query.get("category").map(String::as_str);
    let q = query.get("q").map(String::as_str);
    let normalized_input = format!(
        "v2|{}|offset={offset}|limit={limit}|category={}|q={}",
        locale.as_str(),
        category.unwrap_or_default(),
        q.unwrap_or_default()
    );
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-list",
        &normalized_input,
        || async {
            state
                .knot_repository()
                .list_knots(locale, offset, limit, category, q)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_filters(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-filters",
        &format!("v2|{}", locale.as_str()),
        || async {
            state
                .knot_repository()
                .list_knot_filters(locale)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_offline_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    reject_all_query_parameters(&query)?;
    let locale = resolve_locale(&headers)?;
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-offline-manifest",
        &format!("v2|{}", locale.as_str()),
        || async {
            state
                .knot_repository()
                .offline_manifest(locale)
                .await
                .map_err(ApiError::from)
        },
    )
    .await
}

async fn knot_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let normalized_input = format!("v2|{}|id={id}", locale.as_str());
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "skills-knots-detail",
        &normalized_input,
        || async {
            state
                .knot_repository()
                .get_knot_detail(&id, locale)
                .await?
                .ok_or(ApiError::NotFound)
        },
    )
    .await
}

async fn list_favorite_skills(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<ListFavoriteSkillsResponse>, ApiError> {
    reject_query_locale(&query)?;
    reject_unknown_favorite_query_parameters(&query)?;
    let locale = resolve_locale(&headers)?;
    let skill_category = query
        .get("skill_category")
        .map(String::as_str)
        .unwrap_or("all");
    if !matches!(skill_category, "all" | "knots") {
        return Err(ApiError::invalid_query_parameter(
            "skill_category",
            "skill_category must be one of: all, knots".to_owned(),
        ));
    }
    let offset = parse_u32_query(&query, "offset", 0)?;
    let limit = parse_u32_query(&query, "limit", 20)?.clamp(1, 100);
    let counts = state.skill_favorite_repository().counts(&user.id).await?;
    let (entries, _total_count, next_offset) = state
        .skill_favorite_repository()
        .list_knot_favorites(&user.id, offset, limit)
        .await?;
    let mut items = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(detail) = state
            .knot_repository()
            .get_knot_detail(&entry.knot_id, locale)
            .await?
        else {
            continue;
        };
        items.push(FavoriteKnotItemResponse {
            skill_category: "knots",
            favorited_at: entry.favorited_at,
            knot: KnotSummary {
                id: detail.id.clone(),
                slug: detail.slug.clone(),
                title: detail.title.clone(),
                summary: detail.summary.clone(),
                categories: detail.categories.clone(),
                types: detail.types.clone(),
                media: detail.media.clone(),
                href: format!("/api/v1/skills/knots/detail/{}", detail.id),
            },
        });
    }
    Ok(Json(ListFavoriteSkillsResponse {
        locale,
        filters: favorite_skill_filters(counts),
        items,
        page: PageInfo {
            limit,
            offset,
            next_offset: if skill_category == "knots" || skill_category == "all" {
                next_offset
            } else {
                None
            },
        },
    }))
}

async fn knot_favorite_status(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<FavoriteKnotStatusResponse>, ApiError> {
    reject_all_query_parameters(&query)?;
    ensure_knot_exists(&state, &id).await?;
    let status = state
        .skill_favorite_repository()
        .knot_status(&user.id, &id)
        .await?;
    Ok(Json(favorite_status_response(status)))
}

async fn favorite_knot(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<FavoriteKnotStatusResponse>, ApiError> {
    reject_all_query_parameters(&query)?;
    ensure_knot_exists(&state, &id).await?;
    let status = state
        .skill_favorite_repository()
        .favorite_knot(&user.id, &id)
        .await?;
    Ok(Json(favorite_status_response(status)))
}

async fn unfavorite_knot(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<FavoriteKnotStatusResponse>, ApiError> {
    reject_all_query_parameters(&query)?;
    ensure_knot_exists(&state, &id).await?;
    let status = state
        .skill_favorite_repository()
        .unfavorite_knot(&user.id, &id)
        .await?;
    Ok(Json(favorite_status_response(status)))
}

fn reject_unknown_favorite_query_parameters(
    query: &HashMap<String, String>,
) -> Result<(), ApiError> {
    for key in query.keys() {
        if !matches!(key.as_str(), "skill_category" | "offset" | "limit") {
            return Err(ApiError::unsupported_query_parameter(key.clone()));
        }
    }
    Ok(())
}

fn favorite_skill_filters(counts: SkillFavoriteCounts) -> Vec<FavoriteSkillFilterResponse> {
    vec![
        FavoriteSkillFilterResponse {
            id: "all".to_owned(),
            title: "全部收藏".to_owned(),
            count: counts.total_count,
        },
        FavoriteSkillFilterResponse {
            id: "knots".to_owned(),
            title: "绳结".to_owned(),
            count: counts.knot_count,
        },
    ]
}

async fn ensure_knot_exists(state: &AppState, knot_id: &str) -> Result<(), ApiError> {
    if state
        .skill_favorite_repository()
        .knot_exists(knot_id)
        .await?
    {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

fn favorite_status_response(status: KnotFavoriteStatus) -> FavoriteKnotStatusResponse {
    FavoriteKnotStatusResponse {
        skill_category: "knots",
        knot_id: status.knot_id,
        is_favorited: status.is_favorited,
        favorited_at: status.favorited_at,
    }
}
