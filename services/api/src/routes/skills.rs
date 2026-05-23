//! Public outdoor skill routes for DB-backed skill categories and knots.

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use stellartrail_db::repositories::DisclaimerAcceptanceDraft;
use stellartrail_domain::skill::SkillCategoriesResponse;

use crate::{
    dto::disclaimer::{AcceptKnotDisclaimerRequest, KnotDisclaimerResponse},
    error::ApiError,
    extractors::AuthenticatedUser,
    state::AppState,
};

use super::localization::{
    cached_localized_json_with, parse_u32_query, reject_all_query_parameters, reject_query_locale,
    resolve_locale,
};

const KNOT_DISCLAIMER_KEY: &str = "knot_tutorial_disclaimer";
const KNOT_DISCLAIMER_VERSION: &str = "v1";
const KNOT_DISCLAIMER_TITLE: &str = "绳结教程免责声明";
const KNOT_DISCLAIMER_CONTENT: &str = "本程序提供的绳结教程、图文和媒体内容由个人基于兴趣免费整理，仅用于一般绳结知识学习和非承重练习，不构成安全、救援、攀登、航海、工业吊装、高空作业或其他专业活动建议，也不提供专业培训、资质认证或现场安全评估。绳结的适用性和安全性会受绳材、载荷、环境、操作熟练度、检查维护和现场条件等因素影响。除非已经接受相应专业训练，并由具备资质或充分经验的人员结合现场条件进行指导、检查和复核，用户不得将本程序内容用于承载人体、攀登、救援、吊装、高空作业、航海安全作业或其他可能造成人身、财产损害的活动。发现教程可能存在错误、不完整或与现场情况不符时，应立即停止参考，并以权威规范、设备说明或专业人员意见为准。用户自行参考、学习或使用本程序内容的，应自行判断并承担相应风险；法律另有规定或因本程序、开发者依法应承担责任的除外。";

/// Builds all DB-backed outdoor skill routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/skills", get(skill_categories))
        .route("/skills/knots/list", get(knot_list))
        .route("/skills/knots/filters", get(knot_filters))
        .route("/skills/knots/offline-manifest", get(knot_offline_manifest))
        .route("/skills/knots/detail/:id", get(knot_detail))
        .route("/me/skills/knots/disclaimer", get(knot_disclaimer))
        .route(
            "/me/skills/knots/disclaimer/acceptance",
            post(accept_knot_disclaimer),
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
        "v3|{}|offset={offset}|limit={limit}|category={}|q={}",
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
        &format!("v3|{}", locale.as_str()),
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
        &format!("v3|{}", locale.as_str()),
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
    let normalized_input = format!("v3|{}|id={id}", locale.as_str());
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

async fn knot_disclaimer(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<KnotDisclaimerResponse>, ApiError> {
    reject_all_query_parameters(&query)?;
    let acceptance = state
        .disclaimer_acceptance_repository()
        .get(&user.id, KNOT_DISCLAIMER_KEY, KNOT_DISCLAIMER_VERSION)
        .await?;
    Ok(Json(knot_disclaimer_response(
        acceptance.map(|record| record.accepted_at),
    )))
}

async fn accept_knot_disclaimer(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<HashMap<String, String>>,
    Json(payload): Json<AcceptKnotDisclaimerRequest>,
) -> Result<Json<KnotDisclaimerResponse>, ApiError> {
    reject_all_query_parameters(&query)?;
    let draft = DisclaimerAcceptanceDraft {
        disclaimer_key: KNOT_DISCLAIMER_KEY.to_owned(),
        version: KNOT_DISCLAIMER_VERSION.to_owned(),
        title: KNOT_DISCLAIMER_TITLE.to_owned(),
        content: KNOT_DISCLAIMER_CONTENT.to_owned(),
        client_platform: normalize_optional_client_text(payload.client_platform),
        client_version: normalize_optional_client_text(payload.client_version),
        device_model: normalize_optional_client_text(payload.device_model),
    };
    let acceptance = state
        .disclaimer_acceptance_repository()
        .accept(&user.id, &draft)
        .await?;
    Ok(Json(knot_disclaimer_response(Some(acceptance.accepted_at))))
}

fn knot_disclaimer_response(accepted_at: Option<String>) -> KnotDisclaimerResponse {
    KnotDisclaimerResponse {
        key: KNOT_DISCLAIMER_KEY,
        version: KNOT_DISCLAIMER_VERSION,
        title: KNOT_DISCLAIMER_TITLE,
        content: KNOT_DISCLAIMER_CONTENT,
        accepted: accepted_at.is_some(),
        accepted_at,
    }
}

fn normalize_optional_client_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().chars().take(128).collect::<String>())
        .filter(|value| !value.is_empty())
}
