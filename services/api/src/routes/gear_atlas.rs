//! Public gear atlas, user submission, and administrator review routes.
//!
//! User-gear submissions are always materialized server-side from the personal
//! gear row so clients cannot accidentally upload private purchase, storage, or
//! note fields into the public atlas table.

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post, put},
};
use serde_json::json;
use stellartrail_db::repositories::{
    GearAtlasRepository, GearRepository, ListGearAtlasAdminOptions, ListGearAtlasOptions,
};
use stellartrail_domain::{
    gear_atlas::{
        GEAR_ATLAS_LOCALIZATION_STATUS_DRAFT, GEAR_ATLAS_LOCALIZATION_STATUS_NEEDS_REVIEW,
        GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED, GearAtlasItem, GearAtlasLocalizationDraft,
        GearAtlasLocalizationReviewState, GearAtlasLocalizationReviewStatus,
        GearAtlasLocalizationTranslator, draft_from_personal_gear, now_atlas_rfc3339,
    },
    locale::Locale,
    validation::FieldViolation,
};

use crate::{
    dto::gear_atlas::{
        CreateGearAtlasSubmissionRequest, GearAtlasAdminSubmissionResponse,
        GearAtlasLocalizationResponse, GearAtlasPublicItemResponse, GearAtlasSubmissionResponse,
        GenerateGearAtlasLocalizationDraftRequest, ListAdminGearAtlasSubmissionsQuery,
        ListAdminGearAtlasSubmissionsResponse, ListGearAtlasQuery, ListGearAtlasResponse,
        ListGearAtlasSubmissionsResponse, ListMyGearAtlasSubmissionsQuery,
        RejectGearAtlasSubmissionRequest, UpdateGearAtlasLocalizationRequest,
        UpdateGearAtlasSubmissionRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::admin_service,
    state::AppState,
};

use super::localization::{cached_localized_json_with, reject_query_locale, resolve_locale};
use crate::services::public_response_cache::invalidate_gear_atlas_public_responses;

/// Builds all gear atlas routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/gear-atlas", get(list_public))
        .route("/gear-atlas/:id", get(get_public))
        .route(
            "/me/gear-atlas-submissions",
            get(list_my_submissions).post(create_manual_submission),
        )
        .route(
            "/me/gears/:id/atlas-submission",
            post(create_submission_from_personal_gear),
        )
        .route("/admin/gear-atlas-submissions", get(list_admin_submissions))
        .route(
            "/admin/gear-atlas-submissions/:id",
            get(get_admin_submission)
                .patch(update_admin_submission)
                .delete(delete_admin_submission),
        )
        .route(
            "/admin/gear-atlas-submissions/:id/restore",
            post(restore_admin_submission),
        )
        .route(
            "/admin/gear-atlas-submissions/:id/localizations/:locale",
            put(update_admin_localization),
        )
        .route(
            "/admin/gear-atlas-submissions/:id/localizations/:locale/generate-draft",
            post(generate_admin_localization_draft),
        )
        .route(
            "/admin/gear-atlas-submissions/:id/approve",
            post(approve_submission),
        )
        .route(
            "/admin/gear-atlas-submissions/:id/reject",
            post(reject_submission),
        )
}

async fn list_public(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListGearAtlasQuery>,
) -> Result<Response, ApiError> {
    if query.locale.is_some() {
        return Err(ApiError::unsupported_query_parameter("locale"));
    }
    let locale = resolve_locale(&headers)?;
    let cache_version = state.cache().public_gear_atlas_version().await;
    let normalized_input = json!({
        "v": 3,
        "cache_version": cache_version,
        "locale": locale.as_str(),
        "category": query.category.map(|category| category.as_str()),
        "q": query.q.as_deref(),
        "sort": query.sort,
        "limit": query.limit.unwrap_or(20),
        "cursor": query.cursor.as_deref(),
    })
    .to_string();
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "gear-atlas-list",
        &normalized_input,
        || async {
            let repo = GearAtlasRepository::new(state.db().clone());
            let (items, next_cursor) = repo
                .list_public(
                    &ListGearAtlasOptions {
                        category: query.category,
                        q: query.q,
                        sort: query.sort,
                        limit: query.limit.unwrap_or(20),
                        cursor: query.cursor,
                    },
                    locale,
                )
                .await?;
            let mut response_items = Vec::with_capacity(items.len());
            for item in &items {
                let category_label = repo.category_label(item.category, locale).await?;
                response_items.push(GearAtlasPublicItemResponse::from_item_and_category_label(
                    item,
                    category_label,
                ));
            }
            Ok(ListGearAtlasResponse {
                items: response_items,
                next_cursor,
            })
        },
    )
    .await
}

async fn get_public(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    reject_query_locale(&query)?;
    let locale = resolve_locale(&headers)?;
    let cache_version = state.cache().public_gear_atlas_version().await;
    let normalized_input = format!(
        "v2|atlas-cache-v{cache_version}|{}|id={id}",
        locale.as_str()
    );
    cached_localized_json_with(
        &state,
        &headers,
        locale,
        "gear-atlas-detail",
        &normalized_input,
        || async {
            let repo = GearAtlasRepository::new(state.db().clone());
            let item = repo
                .get_public(&id, locale)
                .await?
                .ok_or(ApiError::NotFound)?;
            let category_label = repo.category_label(item.category, locale).await?;
            Ok(GearAtlasPublicItemResponse::from_item_and_category_label(
                &item,
                category_label,
            ))
        },
    )
    .await
}

async fn create_manual_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateGearAtlasSubmissionRequest>,
) -> Result<(StatusCode, Json<GearAtlasSubmissionResponse>), ApiError> {
    let mut draft = payload.into_draft(&user.id);
    draft
        .validate_and_normalize()
        .map_err(|error| ApiError::Validation(error.fields))?;
    let item = GearAtlasRepository::new(state.db().clone())
        .create_submission(&draft)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(GearAtlasSubmissionResponse::from(&item)),
    ))
}

async fn create_submission_from_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<GearAtlasSubmissionResponse>), ApiError> {
    let gear = GearRepository::new(state.db().clone())
        .get(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let atlas_repo = GearAtlasRepository::new(state.db().clone());
    if let Some(existing) = atlas_repo
        .active_source_submission(&user.id, &gear.id)
        .await?
    {
        return Ok((
            StatusCode::OK,
            Json(GearAtlasSubmissionResponse::from(&existing)),
        ));
    }
    let mut draft = draft_from_personal_gear(&user.id, &gear);
    draft
        .validate_and_normalize()
        .map_err(|error| ApiError::Validation(error.fields))?;
    let item = atlas_repo.create_submission(&draft).await?;
    Ok((
        StatusCode::CREATED,
        Json(GearAtlasSubmissionResponse::from(&item)),
    ))
}

async fn list_my_submissions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListMyGearAtlasSubmissionsQuery>,
) -> Result<Json<ListGearAtlasSubmissionsResponse>, ApiError> {
    let (items, next_cursor) = GearAtlasRepository::new(state.db().clone())
        .list_user_submissions(&user.id, query.limit.unwrap_or(20), query.cursor.as_deref())
        .await?;
    Ok(Json(ListGearAtlasSubmissionsResponse {
        items: items
            .iter()
            .map(GearAtlasSubmissionResponse::from)
            .collect(),
        next_cursor,
    }))
}

async fn list_admin_submissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListAdminGearAtlasSubmissionsQuery>,
) -> Result<Json<ListAdminGearAtlasSubmissionsResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let (items, next_cursor) = repo
        .list_admin(&ListGearAtlasAdminOptions {
            status: query.status,
            category: query.category,
            deleted: query.deleted,
            q: query.q,
            limit: query.limit.unwrap_or(20),
            cursor: query.cursor,
        })
        .await?;
    let mut response_items = Vec::with_capacity(items.len());
    for item in &items {
        response_items.push(admin_submission_response(&repo, item, locale, false).await?);
    }
    Ok(Json(ListAdminGearAtlasSubmissionsResponse {
        items: response_items,
        next_cursor,
    }))
}

async fn delete_admin_submission(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let deleted = GearAtlasRepository::new(state.db().clone())
        .soft_delete(&id)
        .await?;
    if deleted {
        invalidate_gear_atlas_public_responses(&state).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn restore_admin_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let item = repo.restore_deleted(&id).await?.ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, locale, true).await?,
    ))
}

async fn get_admin_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let item = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(
        admin_submission_response(&repo, &item, locale, true).await?,
    ))
}

async fn update_admin_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateGearAtlasSubmissionRequest>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let existing = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    let mut draft = payload.into_draft(&existing.submitted_by_user_id);
    draft.source_type = existing.source_type;
    draft.source_user_gear_id = existing.source_user_gear_id;
    draft
        .validate_and_normalize()
        .map_err(|error| ApiError::Validation(error.fields))?;
    let item = repo
        .update_submission(&id, &draft)
        .await?
        .ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, locale, true).await?,
    ))
}

async fn update_admin_localization(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, locale_value)): Path<(String, String)>,
    Json(payload): Json<UpdateGearAtlasLocalizationRequest>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let review_locale = resolve_locale(&headers)?;
    let locale = parse_admin_locale_path(&locale_value)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let item = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    let provider = normalize_translation_provider(payload.translation_provider)?;
    let mut localization = GearAtlasLocalizationDraft {
        locale,
        name: payload.name,
        description: payload.description,
        variants: payload.variants,
        specs: payload.specs,
        translation_status: Some(
            if payload.mark_reviewed {
                GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED
            } else {
                GEAR_ATLAS_LOCALIZATION_STATUS_DRAFT
            }
            .to_owned(),
        ),
        translation_provider: Some(provider),
        translated_at: Some(now_atlas_rfc3339()),
    };
    localization
        .validate_and_normalize_for_category(item.category)
        .map_err(|error| ApiError::Validation(error.fields))?;
    repo.upsert_item_localization(&id, &localization).await?;
    let item = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, review_locale, true).await?,
    ))
}

async fn generate_admin_localization_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, locale_value)): Path<(String, String)>,
    Json(payload): Json<GenerateGearAtlasLocalizationDraftRequest>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let review_locale = resolve_locale(&headers)?;
    let locale = parse_admin_locale_path(&locale_value)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let item = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    if let Some(existing) = repo.get_item_localization(&id, locale).await?
        && existing.translation_status.as_deref() == Some(GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED)
        && !payload.overwrite_reviewed
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "overwrite_reviewed",
            "is required to refresh reviewed localized content",
        )]));
    }

    let source_locale = opposite_locale(locale);
    let source = repo.get_item_localization(&id, source_locale).await?;
    let source_name = source
        .as_ref()
        .map(|localization| localization.name.as_str())
        .unwrap_or(item.name.as_str());
    let source_variants = source
        .as_ref()
        .map(|localization| &localization.variants)
        .unwrap_or(&item.variants);
    let source_specs = source
        .as_ref()
        .map(|localization| &localization.specs)
        .unwrap_or(&item.specs);
    let provider = normalize_translation_provider(
        payload
            .translation_provider
            .or_else(|| Some("admin-deterministic".to_owned())),
    )?;
    let translator = GearAtlasLocalizationTranslator::new(provider).ok_or_else(|| {
        ApiError::Validation(vec![FieldViolation::new(
            "translation_provider",
            "is required",
        )])
    })?;
    let mut localization =
        translator.translate_localization(source_name, source_variants, source_specs, locale);
    localization.translation_status = Some(GEAR_ATLAS_LOCALIZATION_STATUS_NEEDS_REVIEW.to_owned());
    localization
        .validate_and_normalize_for_category(item.category)
        .map_err(|error| ApiError::Validation(error.fields))?;
    repo.upsert_item_localization(&id, &localization).await?;
    let item = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, review_locale, true).await?,
    ))
}

async fn approve_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let existing = repo.get_any(&id).await?.ok_or(ApiError::NotFound)?;
    let localization_statuses = repo.localization_review_statuses(&existing).await?;
    let violations = localization_approval_violations(&localization_statuses);
    if !violations.is_empty() {
        return Err(ApiError::Validation(violations));
    }
    let item = repo
        .approve(&id, &user.id)
        .await?
        .ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, locale, true).await?,
    ))
}

async fn reject_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<RejectGearAtlasSubmissionRequest>,
) -> Result<Json<GearAtlasAdminSubmissionResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let locale = resolve_locale(&headers)?;
    let reason = normalize_rejection_reason(payload.reason)?;
    let repo = GearAtlasRepository::new(state.db().clone());
    let item = repo
        .reject(&id, &user.id, reason)
        .await?
        .ok_or(ApiError::NotFound)?;
    invalidate_gear_atlas_public_responses(&state).await;
    Ok(Json(
        admin_submission_response(&repo, &item, locale, true).await?,
    ))
}

async fn admin_submission_response(
    repo: &GearAtlasRepository,
    item: &GearAtlasItem,
    locale: Locale,
    include_localizations: bool,
) -> Result<GearAtlasAdminSubmissionResponse, ApiError> {
    let display_item = repo.localized_display_item(item, locale).await?;
    let display_category_label = repo.category_label(item.category, locale).await?;
    let localization_statuses = repo.localization_review_statuses(item).await?;
    let localizations = if include_localizations {
        repo.list_item_localizations(&item.id)
            .await?
            .iter()
            .map(GearAtlasLocalizationResponse::from)
            .collect()
    } else {
        Vec::new()
    };
    Ok(GearAtlasAdminSubmissionResponse::from_item_and_display(
        item,
        &display_item,
        display_category_label,
        locale,
        localization_statuses,
        localizations,
    ))
}

fn normalize_rejection_reason(value: Option<String>) -> Result<String, ApiError> {
    let Some(value) = value else {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "reason",
            "is required",
        )]));
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "reason",
            "is required",
        )]));
    }
    if trimmed.chars().count() > 200 {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "reason",
            "must be at most 200 characters",
        )]));
    }
    Ok(trimmed.to_owned())
}

fn parse_admin_locale_path(value: &str) -> Result<Locale, ApiError> {
    Locale::parse(value).ok_or_else(|| {
        ApiError::invalid_query_parameter("locale", "must be one of zh-CN or en".to_owned())
    })
}

fn normalize_translation_provider(value: Option<String>) -> Result<String, ApiError> {
    let provider = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("admin-manual");
    if provider.chars().count() > 80 {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "translation_provider",
            "must be at most 80 characters",
        )]));
    }
    Ok(provider.to_owned())
}

fn opposite_locale(locale: Locale) -> Locale {
    match locale {
        Locale::ZhCn => Locale::En,
        Locale::En => Locale::ZhCn,
    }
}

fn localization_approval_violations(
    statuses: &[GearAtlasLocalizationReviewStatus],
) -> Vec<FieldViolation> {
    let mut violations = Vec::new();
    for status in statuses {
        let locale = status.locale.as_str();
        for field in &status.missing_fields {
            violations.push(FieldViolation::new(
                format!("localizations.{locale}.{field}"),
                "is required before approval",
            ));
        }
        if status.state != GearAtlasLocalizationReviewState::Reviewed {
            violations.push(FieldViolation::new(
                format!("localizations.{locale}.translation_status"),
                "must be reviewed before approval",
            ));
        }
    }
    violations
}
