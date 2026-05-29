//! Authenticated routes for solo and collaborative trips.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use stellartrail_db::repositories::{
    KIND_BUDGET_ITEM, KIND_FOOD_ITEM, KIND_FOOD_MEAL, KIND_FOOD_SUPPLY, KIND_GOAL_ITEM,
    KIND_ITINERARY_DAY, KIND_MEDICAL_ITEM, KIND_PERSONAL_GEAR, KIND_RESCUE_CONTACT,
    KIND_ROUTE_SEGMENT, KIND_SAFETY_RISK, KIND_SEGMENT_ASSIGNMENT, KIND_SHARED_GEAR,
    KIND_TIME_SLOT, ListTripsOptions, TripHighlightStatus, TripHomeHighlight, TripRepository,
};
use stellartrail_domain::{
    gear::GearCategory,
    trip::{
        OutdoorExperience, TripBudgetItem, TripDetail, TripFoodMeal, TripGoalItem,
        TripItineraryDay, TripMedicalItem, TripMember, TripPersonalGearItem, TripRouteSegment,
        TripSectionKey, TripSharedGearDemand,
    },
    validation::ValidationError,
};

use crate::{
    dto::trip::{
        CreateTripInvitationResponse, CreateTripRecordRequest, CreateTripRequest,
        ImportPackingListRequest, ListOutdoorExperiencesResponse, ListTripsQuery,
        ListTripsResponse, OutdoorExperienceRequest, PatchTripFieldsRequest, TripHomeHighlightItem,
        TripHomeHighlightQuery, TripHomeHighlightResponse, TripHomeHighlightStatus,
        UpdateTripRequest, UpdateTripSectionsRequest,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    state::AppState,
};
use time::{Date, OffsetDateTime};

/// Builds authenticated trip-center routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me/trips", get(list).post(create))
        .route(
            "/me/outdoor-experiences",
            get(list_outdoor_experiences).post(create_outdoor_experience),
        )
        .route(
            "/me/outdoor-experiences/:experience_id",
            get(get_outdoor_experience)
                .patch(update_outdoor_experience)
                .delete(delete_outdoor_experience),
        )
        .route("/me/trips/home-highlight", get(home_highlight))
        .route(
            "/me/trips/:id/convert-to-outdoor-experience",
            axum::routing::post(convert_trip_to_outdoor_experience),
        )
        .route(
            "/me/trips/:id",
            get(get_one).patch(update).delete(delete_plan),
        )
        .route(
            "/me/trips/:id/sections",
            axum::routing::patch(update_sections),
        )
        .route(
            "/me/trips/:id/invitations",
            axum::routing::post(create_invitation),
        )
        .route(
            "/me/trip-invitations/:token/accept",
            axum::routing::post(accept_invitation),
        )
        .route("/me/trips/:id/members", get(list_members))
        .route(
            "/me/trips/:id/members/:member_id",
            axum::routing::patch(update_member).delete(remove_member),
        )
        .route(
            "/me/trips/:id/personal-gear/import-packing-list",
            axum::routing::post(import_packing_list),
        )
        .route(
            "/me/trips/:id/personal-gear",
            get(list_personal_gear).post(create_personal_gear),
        )
        .route(
            "/me/trips/:id/personal-gear/:item_id",
            axum::routing::patch(update_personal_gear).delete(delete_personal_gear),
        )
        .route(
            "/me/trips/:id/shared-gear-demands",
            get(list_shared_gear_demands).post(create_shared_gear_demand),
        )
        .route(
            "/me/trips/:id/shared-gear-demands/:item_id/bind-my-gear",
            axum::routing::post(bind_shared_gear_demand_my_gear),
        )
        .route(
            "/me/trips/:id/shared-gear-demands/:item_id/fill-concrete-gear",
            axum::routing::post(fill_shared_gear_demand_concrete_gear),
        )
        .route(
            "/me/trips/:id/shared-gear-demands/:item_id",
            axum::routing::patch(update_shared_gear_demand).delete(delete_shared_gear_demand),
        )
        .route(
            "/me/trips/:id/itinerary-days",
            get(list_itinerary_days).post(create_itinerary_day),
        )
        .route(
            "/me/trips/:id/itinerary-days/:day_id",
            axum::routing::patch(update_itinerary_day).delete(delete_itinerary_day),
        )
        .route(
            "/me/trips/:id/itinerary-days/:day_id/time-slots",
            axum::routing::post(create_time_slot),
        )
        .route(
            "/me/trips/:id/itinerary-days/:day_id/time-slots/:slot_id",
            axum::routing::patch(update_time_slot).delete(delete_time_slot),
        )
        .route(
            "/me/trips/:id/route-segments",
            get(list_route_segments).post(create_route_segment),
        )
        .route(
            "/me/trips/:id/route-segments/:segment_id",
            axum::routing::patch(update_route_segment).delete(delete_route_segment),
        )
        .route(
            "/me/trips/:id/segment-assignments",
            axum::routing::post(create_segment_assignment),
        )
        .route(
            "/me/trips/:id/segment-assignments/:assignment_id",
            axum::routing::patch(update_segment_assignment).delete(delete_segment_assignment),
        )
        .route(
            "/me/trips/:id/food-meals",
            get(list_food_meals).post(create_food_meal),
        )
        .route(
            "/me/trips/:id/food-meals/:meal_id",
            axum::routing::patch(update_food_meal).delete(delete_food_meal),
        )
        .route(
            "/me/trips/:id/food-meals/:meal_id/items",
            axum::routing::post(create_food_item),
        )
        .route(
            "/me/trips/:id/food-meals/:meal_id/items/:item_id",
            axum::routing::patch(update_food_item).delete(delete_food_item),
        )
        .route(
            "/me/trips/:id/food-supplies",
            axum::routing::post(create_food_supply),
        )
        .route(
            "/me/trips/:id/food-supplies/:supply_id",
            axum::routing::patch(update_food_supply).delete(delete_food_supply),
        )
        .route(
            "/me/trips/:id/medical-items",
            get(list_medical_items).post(create_medical_item),
        )
        .route(
            "/me/trips/:id/medical-items/:item_id",
            axum::routing::patch(update_medical_item).delete(delete_medical_item),
        )
        .route(
            "/me/trips/:id/safety-risks",
            axum::routing::post(create_safety_risk),
        )
        .route(
            "/me/trips/:id/safety-risks/:risk_id",
            axum::routing::patch(update_safety_risk).delete(delete_safety_risk),
        )
        .route(
            "/me/trips/:id/rescue-contacts",
            axum::routing::post(create_rescue_contact),
        )
        .route(
            "/me/trips/:id/rescue-contacts/:contact_id",
            axum::routing::patch(update_rescue_contact).delete(delete_rescue_contact),
        )
        .route(
            "/me/trips/:id/budget-items",
            get(list_budget_items).post(create_budget_item),
        )
        .route(
            "/me/trips/:id/budget-items/:item_id",
            axum::routing::patch(update_budget_item).delete(delete_budget_item),
        )
        .route(
            "/me/trips/:id/goals",
            get(list_goals).post(create_goal_item),
        )
        .route(
            "/me/trips/:id/goals/:goal_id",
            axum::routing::patch(update_goal_item).delete(delete_goal_item),
        )
}

async fn list_outdoor_experiences(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<ListOutdoorExperiencesResponse>, ApiError> {
    let items = TripRepository::new(state.db().clone())
        .list_outdoor_experiences(&user.id)
        .await?;
    Ok(Json(ListOutdoorExperiencesResponse { items }))
}

async fn create_outdoor_experience(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<OutdoorExperienceRequest>,
) -> Result<(StatusCode, Json<OutdoorExperience>), ApiError> {
    let item = TripRepository::new(state.db().clone())
        .create_outdoor_experience(&user.id, payload.into_draft())
        .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn get_outdoor_experience(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(experience_id): Path<String>,
) -> Result<Json<OutdoorExperience>, ApiError> {
    let item = TripRepository::new(state.db().clone())
        .outdoor_experience(&user.id, &experience_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(item))
}

async fn update_outdoor_experience(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(experience_id): Path<String>,
    Json(payload): Json<OutdoorExperienceRequest>,
) -> Result<Json<OutdoorExperience>, ApiError> {
    let item = TripRepository::new(state.db().clone())
        .update_outdoor_experience(&user.id, &experience_id, payload.into_draft())
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(item))
}

async fn delete_outdoor_experience(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(experience_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let deleted = TripRepository::new(state.db().clone())
        .delete_outdoor_experience(&user.id, &experience_id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn convert_trip_to_outdoor_experience(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<OutdoorExperience>), ApiError> {
    let today = OffsetDateTime::now_utc().date();
    let item = TripRepository::new(state.db().clone())
        .convert_trip_to_outdoor_experience(&user.id, &id, today)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Lists trips visible to the current user.
async fn list(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListTripsQuery>,
) -> Result<Json<ListTripsResponse>, ApiError> {
    let (items, next_cursor) = TripRepository::new(state.db().clone())
        .list_trips(
            &user.id,
            &ListTripsOptions {
                limit: query.limit.unwrap_or(20),
                cursor: query.cursor,
                bucket: query.bucket.and_then(|bucket| bucket.into_filter()),
                trip_type: query
                    .trip_type
                    .and_then(|trip_type| trip_type.into_filter()),
                today: parse_home_highlight_today(query.today.as_deref())?,
            },
        )
        .await?;
    Ok(Json(ListTripsResponse { items, next_cursor }))
}

/// Reads the current or next dated trip for the homepage reminder.
async fn home_highlight(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<TripHomeHighlightQuery>,
) -> Result<Json<TripHomeHighlightResponse>, ApiError> {
    let today = parse_home_highlight_today(query.today.as_deref())?;
    let item = TripRepository::new(state.db().clone())
        .home_highlight(&user.id, today)
        .await?
        .map(TripHomeHighlightItem::from);
    Ok(Json(TripHomeHighlightResponse { item }))
}

/// Creates a trip and adds the creator as owner member.
async fn create(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateTripRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    let mut draft = payload.into_draft();
    draft.validate_and_normalize()?;
    let detail = TripRepository::new(state.db().clone())
        .create_trip(&user, &draft)
        .await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

/// Reads one trip detail for a member.
async fn get_one(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<TripDetail>, ApiError> {
    let detail = load_trip_detail(&state, &user.id, &id).await?;
    Ok(Json(detail))
}

async fn load_trip_detail(
    state: &AppState,
    user_id: &str,
    trip_id: &str,
) -> Result<TripDetail, ApiError> {
    TripRepository::new(state.db().clone())
        .detail_for_user(user_id, trip_id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Updates trip metadata using field-level optimistic concurrency.
async fn update(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTripRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    let (changes, meta) = payload.into_changes()?;
    let detail = TripRepository::new(state.db().clone())
        .update_plan_fields(
            &user.id,
            &id,
            changes,
            meta.base_field_versions,
            meta.force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

/// Soft-deletes a trip when the current user is the owner.
async fn delete_plan(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let deleted = TripRepository::new(state.db().clone())
        .soft_delete_plan(&user.id, &id)
        .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Updates visible optional sections. Members and personal gear remain default sections.
async fn update_sections(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTripSectionsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    let mut sections = payload.enabled_sections;
    ensure_default_sections(&mut sections);
    let detail = TripRepository::new(state.db().clone())
        .update_sections(
            &user.id,
            &id,
            sections,
            payload.meta.base_field_versions,
            payload.meta.force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

/// Creates an invitation token owned by the trip creator.
async fn create_invitation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<CreateTripInvitationResponse>), ApiError> {
    let invitation = TripRepository::new(state.db().clone())
        .create_invitation(&user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateTripInvitationResponse { invitation }),
    ))
}

/// Accepts a team trip invitation token as the current user.
async fn accept_invitation(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(token): Path<String>,
) -> Result<Json<TripDetail>, ApiError> {
    let detail = TripRepository::new(state.db().clone())
        .accept_invitation(&user, &token)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

/// Updates one member profile. Owners may edit all profiles; members may edit themselves.
async fn update_member(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, member_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    let (changes, meta) = payload.into_parts()?;
    let detail = TripRepository::new(state.db().clone())
        .update_member(
            &user.id,
            &id,
            &member_id,
            changes,
            meta.base_field_versions,
            meta.force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

/// Removes one non-owner member from a trip.
async fn remove_member(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, member_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    let detail = TripRepository::new(state.db().clone())
        .remove_member(&user.id, &id, &member_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

async fn list_members(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripMember>>, ApiError> {
    Ok(Json(load_trip_detail(&state, &user.id, &id).await?.members))
}

/// Imports a personal packing list as immutable trip gear snapshots.
async fn import_packing_list(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<ImportPackingListRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    let detail = TripRepository::new(state.db().clone())
        .import_packing_list(&user.id, &id, &payload.packing_list_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

async fn list_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripPersonalGearItem>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id).await?.personal_gear,
    ))
}

async fn create_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_PERSONAL_GEAR, None, payload).await
}

async fn update_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_PERSONAL_GEAR, &item_id, payload).await
}

async fn delete_personal_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_PERSONAL_GEAR, &item_id).await
}

async fn list_shared_gear_demands(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripSharedGearDemand>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id)
            .await?
            .shared_gear_demands,
    ))
}

async fn create_shared_gear_demand(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_SHARED_GEAR, None, payload).await
}

async fn update_shared_gear_demand(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_SHARED_GEAR, &item_id, payload).await
}

async fn bind_shared_gear_demand_my_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_SHARED_GEAR, &item_id, payload).await
}

async fn fill_shared_gear_demand_concrete_gear(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_SHARED_GEAR, &item_id, payload).await
}

async fn delete_shared_gear_demand(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_SHARED_GEAR, &item_id).await
}

async fn list_itinerary_days(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripItineraryDay>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id)
            .await?
            .itinerary_days,
    ))
}

async fn create_itinerary_day(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_ITINERARY_DAY, None, payload).await
}

async fn update_itinerary_day(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, day_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_ITINERARY_DAY, &day_id, payload).await
}

async fn delete_itinerary_day(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, day_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_ITINERARY_DAY, &day_id).await
}

async fn create_time_slot(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, day_id)): Path<(String, String)>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(
        &state,
        &user.id,
        &id,
        KIND_TIME_SLOT,
        Some(day_id.as_str()),
        payload,
    )
    .await
}

async fn update_time_slot(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, _day_id, slot_id)): Path<(String, String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_TIME_SLOT, &slot_id, payload).await
}

async fn delete_time_slot(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, _day_id, slot_id)): Path<(String, String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_TIME_SLOT, &slot_id).await
}

async fn create_route_segment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_ROUTE_SEGMENT, None, payload).await
}

async fn update_route_segment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, segment_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(
        &state,
        &user.id,
        &id,
        KIND_ROUTE_SEGMENT,
        &segment_id,
        payload,
    )
    .await
}

async fn delete_route_segment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, segment_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_ROUTE_SEGMENT, &segment_id).await
}

async fn list_route_segments(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripRouteSegment>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id)
            .await?
            .route_segments,
    ))
}

// The following sections share the same record CRUD pipeline as route segments.
async fn create_segment_assignment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(
        &state,
        &user.id,
        &id,
        KIND_SEGMENT_ASSIGNMENT,
        None,
        payload,
    )
    .await
}

async fn update_segment_assignment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, assignment_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(
        &state,
        &user.id,
        &id,
        KIND_SEGMENT_ASSIGNMENT,
        &assignment_id,
        payload,
    )
    .await
}

async fn delete_segment_assignment(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, assignment_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(
        &state,
        &user.id,
        &id,
        KIND_SEGMENT_ASSIGNMENT,
        &assignment_id,
    )
    .await
}

async fn list_food_meals(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripFoodMeal>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id).await?.food_meals,
    ))
}

async fn create_food_meal(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    let parent_id = payload.parent_id.clone().or_else(|| {
        payload
            .payload
            .get("itinerary_day_id")
            .and_then(JsonValue::as_str)
            .map(str::to_owned)
    });
    create_record_response(
        &state,
        &user.id,
        &id,
        KIND_FOOD_MEAL,
        parent_id.as_deref(),
        payload,
    )
    .await
}

async fn update_food_meal(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, meal_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_FOOD_MEAL, &meal_id, payload).await
}

async fn delete_food_meal(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, meal_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_FOOD_MEAL, &meal_id).await
}

async fn create_food_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, meal_id)): Path<(String, String)>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(
        &state,
        &user.id,
        &id,
        KIND_FOOD_ITEM,
        Some(meal_id.as_str()),
        payload,
    )
    .await
}

async fn update_food_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, _meal_id, item_id)): Path<(String, String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_FOOD_ITEM, &item_id, payload).await
}

async fn delete_food_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, _meal_id, item_id)): Path<(String, String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_FOOD_ITEM, &item_id).await
}

async fn create_food_supply(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_FOOD_SUPPLY, None, payload).await
}

async fn update_food_supply(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, supply_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_FOOD_SUPPLY, &supply_id, payload).await
}

async fn delete_food_supply(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, supply_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_FOOD_SUPPLY, &supply_id).await
}

async fn list_medical_items(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripMedicalItem>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id).await?.medical_items,
    ))
}

async fn create_medical_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_MEDICAL_ITEM, None, payload).await
}

async fn update_medical_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_MEDICAL_ITEM, &item_id, payload).await
}

async fn delete_medical_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_MEDICAL_ITEM, &item_id).await
}

async fn create_safety_risk(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_SAFETY_RISK, None, payload).await
}

async fn update_safety_risk(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, risk_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_SAFETY_RISK, &risk_id, payload).await
}

async fn delete_safety_risk(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, risk_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_SAFETY_RISK, &risk_id).await
}

async fn create_rescue_contact(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_RESCUE_CONTACT, None, payload).await
}

async fn update_rescue_contact(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, contact_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(
        &state,
        &user.id,
        &id,
        KIND_RESCUE_CONTACT,
        &contact_id,
        payload,
    )
    .await
}

async fn delete_rescue_contact(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, contact_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_RESCUE_CONTACT, &contact_id).await
}

async fn list_budget_items(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripBudgetItem>>, ApiError> {
    Ok(Json(
        load_trip_detail(&state, &user.id, &id).await?.budget_items,
    ))
}

async fn create_budget_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_BUDGET_ITEM, None, payload).await
}

async fn update_budget_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_BUDGET_ITEM, &item_id, payload).await
}

async fn delete_budget_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_BUDGET_ITEM, &item_id).await
}

async fn list_goals(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<TripGoalItem>>, ApiError> {
    Ok(Json(load_trip_detail(&state, &user.id, &id).await?.goals))
}

async fn create_goal_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTripRecordRequest>,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    create_record_response(&state, &user.id, &id, KIND_GOAL_ITEM, None, payload).await
}

async fn update_goal_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, goal_id)): Path<(String, String)>,
    Json(payload): Json<PatchTripFieldsRequest>,
) -> Result<Json<TripDetail>, ApiError> {
    update_record_response(&state, &user.id, &id, KIND_GOAL_ITEM, &goal_id, payload).await
}

async fn delete_goal_item(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((id, goal_id)): Path<(String, String)>,
) -> Result<Json<TripDetail>, ApiError> {
    delete_record_response(&state, &user.id, &id, KIND_GOAL_ITEM, &goal_id).await
}

async fn create_record_response(
    state: &AppState,
    user_id: &str,
    trip_id: &str,
    kind: &str,
    parent_id: Option<&str>,
    payload: CreateTripRecordRequest,
) -> Result<(StatusCode, Json<TripDetail>), ApiError> {
    let repo = TripRepository::new(state.db().clone());
    let CreateTripRecordRequest {
        parent_id: request_parent,
        sort_order,
        payload,
    } = payload;
    let parent = parent_id.map(str::to_owned).or(request_parent);
    let payload = normalize_create_payload(
        &repo,
        user_id,
        trip_id,
        kind,
        parent.as_deref(),
        sort_order,
        JsonValue::Object(payload),
    )
    .await?;
    let detail = repo
        .create_record(
            user_id,
            trip_id,
            kind,
            parent.as_deref(),
            payload.sort_order,
            payload.payload,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn update_record_response(
    state: &AppState,
    user_id: &str,
    trip_id: &str,
    kind: &str,
    record_id: &str,
    payload: PatchTripFieldsRequest,
) -> Result<Json<TripDetail>, ApiError> {
    let repo = TripRepository::new(state.db().clone());
    let (mut changes, meta) = payload.into_parts()?;
    normalize_patch_fields(kind, &mut changes)?;
    let detail = repo
        .update_record(
            user_id,
            trip_id,
            kind,
            record_id,
            changes,
            meta.base_field_versions,
            meta.force_fields,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

async fn delete_record_response(
    state: &AppState,
    user_id: &str,
    trip_id: &str,
    kind: &str,
    record_id: &str,
) -> Result<Json<TripDetail>, ApiError> {
    let detail = TripRepository::new(state.db().clone())
        .delete_record(user_id, trip_id, kind, record_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(detail))
}

struct NormalizedCreatePayload {
    sort_order: i32,
    payload: JsonValue,
}

async fn normalize_create_payload(
    repo: &TripRepository,
    user_id: &str,
    trip_id: &str,
    kind: &str,
    parent_id: Option<&str>,
    sort_order: i32,
    payload: JsonValue,
) -> Result<NormalizedCreatePayload, ApiError> {
    let mut map = into_object(payload)?;
    map.remove("sort_order");

    match kind {
        KIND_PERSONAL_GEAR => {
            if !map.contains_key("member_id") {
                let detail = repo
                    .detail_for_user(user_id, trip_id)
                    .await?
                    .ok_or(ApiError::NotFound)?;
                map.insert(
                    "member_id".to_owned(),
                    JsonValue::String(detail.my_member_id),
                );
            }
            normalize_gear_payload(&mut map)?;
        }
        KIND_SHARED_GEAR => normalize_gear_payload(&mut map)?,
        KIND_ITINERARY_DAY => {
            default_i32(&mut map, "day_index", 1);
            default_null(&mut map, "date_label");
            default_null(&mut map, "title");
            default_null(&mut map, "notes");
            default_null(&mut map, "weather");
            default_null(&mut map, "high_temperature_c");
            default_null(&mut map, "low_temperature_c");
            default_null(&mut map, "weather_summary");
            default_null(&mut map, "weather_notes");
            default_null(&mut map, "camp_name");
            default_null(&mut map, "camp_altitude_m");
            default_null(&mut map, "camp_terrain");
            default_null(&mut map, "camp_slope");
            default_null(&mut map, "camp_area");
            default_null(&mut map, "camp_water_source");
            default_null(&mut map, "camp_notes");
            map.insert("estimate_minutes".to_owned(), json!(0));
        }
        KIND_TIME_SLOT => {
            let day_id = parent_id.ok_or_else(|| {
                ValidationError::single("day_id", "time slot requires an itinerary day")
            })?;
            map.insert("day_id".to_owned(), JsonValue::String(day_id.to_owned()));
            default_string(&mut map, "slot_key", "morning");
            default_null(&mut map, "route_segment_id");
            default_null(&mut map, "route_description");
            default_null(&mut map, "notes");
        }
        KIND_ROUTE_SEGMENT => {
            require_text(&map, "name")?;
            default_null(&mut map, "start_point");
            default_null(&mut map, "end_point");
            default_null(&mut map, "checkpoint");
            default_null(&mut map, "leader_member_id");
            default_null(&mut map, "bailout_route");
            default_null(&mut map, "trail_condition");
            default_f64(&mut map, "distance_km", 0.0);
            default_i32(&mut map, "ascent_m", 0);
            default_i32(&mut map, "descent_m", 0);
            default_string(&mut map, "descent_profile", "none");
            default_f64(&mut map, "technical_factor", 1.0);
            default_f64(&mut map, "rest_factor", 1.0);
            default_f64(&mut map, "pack_factor", 1.0);
            default_null(&mut map, "manual_estimate_minutes");
            map.insert("formula_estimate_minutes".to_owned(), json!(0));
            map.insert("final_estimate_minutes".to_owned(), json!(0));
            default_null(&mut map, "estimated_start_altitude_m");
            default_null(&mut map, "estimated_end_altitude_m");
            default_null(&mut map, "estimated_highest_altitude_m");
            default_f64(&mut map, "high_altitude_factor", 1.0);
            default_null(&mut map, "notes");
        }
        KIND_SEGMENT_ASSIGNMENT => {
            default_null(&mut map, "route_segment_id");
            default_null(&mut map, "checkpoint");
            default_null(&mut map, "leader_record_member_id");
            default_null(&mut map, "navigator_safety_member_id");
            default_null(&mut map, "collaborator_member_id");
            default_null(&mut map, "photographer_member_id");
            default_null(&mut map, "safety_member_id");
            default_null(&mut map, "environment_member_id");
            default_null(&mut map, "sweeper_member_id");
            default_null(&mut map, "notes");
        }
        KIND_FOOD_MEAL => {
            let day_id = parent_id
                .or_else(|| map.get("itinerary_day_id").and_then(JsonValue::as_str))
                .ok_or_else(|| {
                    ValidationError::single(
                        "itinerary_day_id",
                        "food meal requires an itinerary day",
                    )
                })?;
            map.insert(
                "itinerary_day_id".to_owned(),
                JsonValue::String(day_id.to_owned()),
            );
            default_string(&mut map, "meal_key", "breakfast");
            default_null(&mut map, "meal_type");
            default_bool(&mut map, "skipped", false);
            default_null(&mut map, "dish_name");
            default_null(&mut map, "responsible_member_id");
            default_null(&mut map, "notes");
        }
        KIND_FOOD_ITEM => {
            let meal_id = parent_id.ok_or_else(|| {
                ValidationError::single("food_meal_id", "food item requires a meal")
            })?;
            map.insert(
                "food_meal_id".to_owned(),
                JsonValue::String(meal_id.to_owned()),
            );
            require_text(&map, "name")?;
            default_null(&mut map, "amount_g");
            default_null(&mut map, "per_person_amount_g");
            default_null(&mut map, "total_price_cents");
            default_null(&mut map, "responsible_member_id");
            default_null(&mut map, "notes");
        }
        KIND_FOOD_SUPPLY => {
            require_text(&map, "name")?;
            default_null(&mut map, "supply_type");
            default_null(&mut map, "amount_g");
            default_null(&mut map, "per_person_amount_g");
            default_null(&mut map, "total_price_cents");
            default_null(&mut map, "responsible_member_id");
            default_null(&mut map, "notes");
        }
        KIND_MEDICAL_ITEM => {
            require_text(&map, "name")?;
            default_null(&mut map, "item_type");
            default_null(&mut map, "scope");
            default_null(&mut map, "suggested_quantity");
            default_i32(&mut map, "required_quantity", 1);
            default_i32(&mut map, "packed_quantity", 0);
            default_null(&mut map, "responsible_member_id");
            default_null(&mut map, "notes");
        }
        KIND_SAFETY_RISK => {
            require_text(&map, "risk_type")?;
            default_null(&mut map, "prevention");
            default_null(&mut map, "response");
            default_null(&mut map, "responsible_member_id");
            default_null(&mut map, "itinerary_day_id");
            default_null(&mut map, "route_segment_id");
            default_null(&mut map, "notes");
        }
        KIND_RESCUE_CONTACT => {
            require_text(&map, "organization")?;
            default_null(&mut map, "address");
            default_null(&mut map, "phone");
            default_null(&mut map, "notes");
        }
        KIND_BUDGET_ITEM => {
            require_text(&map, "name")?;
            default_null(&mut map, "category");
            default_i32(&mut map, "quantity", 1);
            default_null(&mut map, "unit_price_cents");
            default_null(&mut map, "total_price_cents");
            default_null(&mut map, "split_member_count");
            default_null(&mut map, "notes");
            default_null(&mut map, "linked_shared_gear_id");
            default_bool(&mut map, "linked_shared_gear_deleted", false);
            default_null(&mut map, "linked_shared_gear_name");
            default_null(&mut map, "linked_shared_gear_responsible_member_id");
        }
        KIND_GOAL_ITEM => {
            require_text(&map, "content")?;
            default_string(&mut map, "scope", "team");
            default_null(&mut map, "member_id");
            default_null(&mut map, "notes");
        }
        _ => {
            return Err(ApiError::BadRequest("unknown trip record kind".to_owned()));
        }
    }

    Ok(NormalizedCreatePayload {
        sort_order,
        payload: JsonValue::Object(map),
    })
}

fn normalize_patch_fields(
    kind: &str,
    changes: &mut std::collections::BTreeMap<String, JsonValue>,
) -> Result<(), ApiError> {
    if matches!(kind, KIND_PERSONAL_GEAR | KIND_SHARED_GEAR)
        && let Some(category) = changes.get("category").and_then(JsonValue::as_str)
    {
        let category = GearCategory::from_key(category)
            .ok_or_else(|| ValidationError::single("category", "unknown gear category"))?;
        changes
            .entry("category_label".to_owned())
            .or_insert_with(|| JsonValue::String(category.label().to_owned()));
    }
    Ok(())
}

impl From<TripHomeHighlight> for TripHomeHighlightItem {
    fn from(value: TripHomeHighlight) -> Self {
        Self {
            trip: value.trip,
            status: match value.status {
                TripHighlightStatus::Ongoing => TripHomeHighlightStatus::Ongoing,
                TripHighlightStatus::Upcoming => TripHomeHighlightStatus::Upcoming,
            },
            days_until_start: value.days_until_start,
            days_until_end: value.days_until_end,
        }
    }
}

fn parse_home_highlight_today(value: Option<&str>) -> Result<Date, ApiError> {
    let Some(value) = value else {
        return Ok(OffsetDateTime::now_utc().date());
    };
    Date::parse(
        value,
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .map_err(|_| {
        ApiError::invalid_query_parameter("today", "must use YYYY-MM-DD format".to_owned())
    })
}

fn normalize_gear_payload(map: &mut JsonMap<String, JsonValue>) -> Result<(), ApiError> {
    require_text(map, "name")?;
    let category = map
        .get("category")
        .and_then(JsonValue::as_str)
        .map(|value| {
            GearCategory::from_key(value)
                .ok_or_else(|| ValidationError::single("category", "unknown gear category"))
        })
        .transpose()?
        .unwrap_or(GearCategory::OtherGear);
    map.insert(
        "category".to_owned(),
        JsonValue::String(category.as_str().to_owned()),
    );
    map.insert(
        "category_label".to_owned(),
        JsonValue::String(category.label().to_owned()),
    );
    default_null(map, "source_packing_list_id");
    default_null(map, "source_packing_item_id");
    default_null(map, "source_gear_id");
    default_null(map, "source_member_id");
    map.remove("source_food_item_id");
    map.remove("source_food_supply_id");
    default_null(map, "template_key");
    default_null(map, "demand_name");
    default_null(map, "concrete_name");
    default_null(map, "brand");
    default_null(map, "model");
    default_i32(map, "planned_quantity", 1);
    default_i32(map, "packed_quantity", 0);
    default_null(map, "unit_weight_g");
    default_null(map, "notes");
    Ok(())
}

fn ensure_default_sections(sections: &mut Vec<TripSectionKey>) {
    let mut ordered = Vec::new();
    for section in TripSectionKey::DEFAULT {
        if !ordered.contains(&section) {
            ordered.push(section);
        }
    }
    for section in sections.iter().copied() {
        if !ordered.contains(&section) {
            ordered.push(section);
        }
    }
    *sections = ordered;
}

fn into_object(value: JsonValue) -> Result<JsonMap<String, JsonValue>, ApiError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(ApiError::BadRequest(
            "request body must be a JSON object".to_owned(),
        )),
    }
}

fn require_text(map: &JsonMap<String, JsonValue>, field: &str) -> Result<(), ValidationError> {
    if map
        .get(field)
        .and_then(JsonValue::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        Ok(())
    } else {
        Err(ValidationError::single(field, "is required"))
    }
}

fn default_string(map: &mut JsonMap<String, JsonValue>, field: &str, value: &str) {
    map.entry(field.to_owned())
        .or_insert_with(|| JsonValue::String(value.to_owned()));
}

fn default_i32(map: &mut JsonMap<String, JsonValue>, field: &str, value: i32) {
    map.entry(field.to_owned()).or_insert_with(|| json!(value));
}

fn default_f64(map: &mut JsonMap<String, JsonValue>, field: &str, value: f64) {
    map.entry(field.to_owned()).or_insert_with(|| json!(value));
}

fn default_bool(map: &mut JsonMap<String, JsonValue>, field: &str, value: bool) {
    map.entry(field.to_owned()).or_insert_with(|| json!(value));
}

fn default_null(map: &mut JsonMap<String, JsonValue>, field: &str) {
    map.entry(field.to_owned()).or_insert(JsonValue::Null);
}
