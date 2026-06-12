//! Repository for collaborative team trip plans and typed plan section records.

use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use stellartrail_domain::{
    gear::{GearCategory, now_rfc3339},
    trip::{
        FieldConflict, FieldVersions, OutdoorExperience, OutdoorExperienceDraft,
        RouteAltitudeEstimate, SharedGearDemandTemplate, Trip, TripBudgetItem, TripDetail,
        TripDraft, TripFoodItem, TripFoodMeal, TripFoodSupply, TripGoalItem, TripInvitation,
        TripItineraryDay, TripItineraryTimeSlot, TripMedicalItem, TripMember, TripMemberGearView,
        TripMemberGearViewItem, TripMemberGearWeightSummary, TripMemberProfile,
        TripPersonalGearItem, TripReadiness, TripRescueContact, TripRouteSegment, TripSafetyRisk,
        TripSectionKey, TripSegmentAssignment, TripSharedGearDemand, TripSummary, TripTimeBucket,
        TripType, estimate_route_minutes, high_altitude_factor, validate_route_estimation_settings,
    },
    validation::{FieldViolation, ValidationError},
};
use time::Date;
use uuid::Uuid;

use super::{UserRecord, statement};

pub const KIND_PERSONAL_GEAR: &str = "personal_gear";
pub const KIND_SHARED_GEAR: &str = "shared_gear";
pub const KIND_ITINERARY_DAY: &str = "itinerary_day";
pub const KIND_TIME_SLOT: &str = "itinerary_time_slot";
pub const KIND_ROUTE_SEGMENT: &str = "route_segment";
pub const KIND_FOOD_MEAL: &str = "food_meal";
pub const KIND_FOOD_ITEM: &str = "food_item";
pub const KIND_FOOD_SUPPLY: &str = "food_supply";
pub const KIND_MEDICAL_ITEM: &str = "medical_item";
pub const KIND_SEGMENT_ASSIGNMENT: &str = "segment_assignment";
pub const KIND_SAFETY_RISK: &str = "safety_risk";
pub const KIND_RESCUE_CONTACT: &str = "rescue_contact";
pub const KIND_BUDGET_ITEM: &str = "budget_item";
pub const KIND_GOAL_ITEM: &str = "goal_item";
const MEAL_KEYS: [&str; 3] = ["breakfast", "lunch", "dinner"];

const TRIP_RECORD_TABLES: &[(&str, &str)] = &[
    (KIND_PERSONAL_GEAR, "trip_personal_gear_items"),
    (KIND_SHARED_GEAR, "trip_shared_gear_demands"),
    (KIND_ITINERARY_DAY, "trip_itinerary_days"),
    (KIND_TIME_SLOT, "trip_itinerary_time_slots"),
    (KIND_ROUTE_SEGMENT, "trip_route_segments"),
    (KIND_SEGMENT_ASSIGNMENT, "trip_segment_assignments"),
    (KIND_FOOD_MEAL, "trip_food_meals"),
    (KIND_FOOD_ITEM, "trip_food_items"),
    (KIND_FOOD_SUPPLY, "trip_food_supplies"),
    (KIND_MEDICAL_ITEM, "trip_medical_items"),
    (KIND_SAFETY_RISK, "trip_safety_risks"),
    (KIND_RESCUE_CONTACT, "trip_rescue_contacts"),
    (KIND_BUDGET_ITEM, "trip_budget_items"),
    (KIND_GOAL_ITEM, "trip_goals"),
];

/// Error boundary for repository mutations that can fail on optimistic conflicts.
#[derive(Debug)]
pub enum TripRepositoryError {
    Db(DbErr),
    Validation(ValidationError),
    Conflict(Vec<FieldConflict>),
}

impl From<DbErr> for TripRepositoryError {
    fn from(value: DbErr) -> Self {
        Self::Db(value)
    }
}

impl From<ValidationError> for TripRepositoryError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

/// Cursor options for listing plans visible to one user.
#[derive(Clone, Debug)]
pub struct ListTripsOptions {
    pub limit: u64,
    pub cursor: Option<String>,
    pub bucket: Option<TripTimeBucket>,
    pub trip_type: Option<TripType>,
    pub today: Date,
}

impl Default for ListTripsOptions {
    fn default() -> Self {
        Self {
            limit: 20,
            cursor: None,
            bucket: None,
            trip_type: None,
            today: Date::parse(
                "1970-01-01",
                time::macros::format_description!("[year]-[month]-[day]"),
            )
            .expect("valid default date"),
        }
    }
}

/// Highlight status for the current user's homepage trip reminder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TripHighlightStatus {
    Ongoing,
    Upcoming,
}

/// The single team plan chosen for the homepage reminder.
#[derive(Clone, Debug)]
pub struct TripHomeHighlight {
    pub trip: TripSummary,
    pub status: TripHighlightStatus,
    pub days_until_start: i64,
    pub days_until_end: i64,
}

/// Persistence boundary for trip metadata, members, invitations, and section records.
#[derive(Clone)]
pub struct TripRepository {
    db: DatabaseConnection,
}

impl TripRepository {
    /// Creates a repository bound to a database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a plan and automatically adds the owner as the first member.
    pub async fn create_trip(
        &self,
        user: &UserRecord,
        draft: &TripDraft,
    ) -> Result<TripDetail, TripRepositoryError> {
        let id = Uuid::new_v4().to_string();
        let member_id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let sections = serde_json::to_string(&TripSectionKey::DEFAULT).map_err(json_db_error)?;
        let plan_versions = json_string(&initial_versions([
            "title",
            "enabled_sections",
            "route_use_slope_adjustment",
            "route_use_high_altitude_adjustment",
            "route_start_altitude_m",
        ]))?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO trips \
                 (id, owner_user_id, trip_type, title, description, start_date, end_date, \
                  enabled_sections_json, route_use_slope_adjustment, \
                  route_use_high_altitude_adjustment, route_start_altitude_m, \
                  field_versions_json, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user.id.clone().into(),
                    trip_type_key(draft.trip_type).to_owned().into(),
                    draft.title.clone().into(),
                    draft.description.clone().into(),
                    draft.start_date.clone().into(),
                    draft.end_date.clone().into(),
                    sections.into(),
                    draft.route_use_slope_adjustment.into(),
                    draft.route_use_high_altitude_adjustment.into(),
                    draft.route_start_altitude_m.into(),
                    plan_versions.into(),
                    now.clone().into(),
                    now.clone().into(),
                ],
            ))
            .await?;

        let profile = TripMemberProfile {
            display_name: display_name_for_user(user),
            ..TripMemberProfile::default()
        };
        self.insert_member(&id, &member_id, user, &profile, &now)
            .await?;
        self.instantiate_shared_gear_demands(&user.id, &id, &member_id)
            .await?;
        self.detail_for_user(&user.id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created trip not found".to_owned()).into())
    }

    /// Lists active trips where the user is a member.
    pub async fn list_trips(
        &self,
        user_id: &str,
        options: &ListTripsOptions,
    ) -> Result<(Vec<TripSummary>, Option<String>), DbErr> {
        let limit = options.limit.clamp(1, 100) as usize;
        let offset = parse_cursor(options.cursor.as_deref())? as usize;
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT p.id, p.owner_user_id, p.trip_type, p.title, p.description, p.start_date, \
                        p.end_date, p.enabled_sections_json, \
                        p.route_use_slope_adjustment, p.route_use_high_altitude_adjustment, \
                        p.route_start_altitude_m, \
                        (SELECT COUNT(*) FROM trip_itinerary_days r \
                         WHERE r.trip_id = p.id AND r.is_deleted = FALSE) AS day_count, \
                        (SELECT COUNT(*) FROM trip_members tm \
                         WHERE tm.trip_id = p.id AND tm.is_deleted = FALSE) AS member_count, \
                        (SELECT e.id FROM outdoor_experiences e \
                         WHERE e.source_trip_id = p.id AND e.user_id = ? AND e.is_deleted = FALSE LIMIT 1) \
                            AS outdoor_experience_id, \
                        p.field_versions_json, p.is_deleted, p.created_at, p.updated_at \
                 FROM trips p \
                 JOIN trip_members m ON m.trip_id = p.id AND m.user_id = ? AND m.is_deleted = FALSE \
                 WHERE p.is_deleted = FALSE \
                 ORDER BY p.updated_at DESC, p.id DESC",
                vec![user_id.to_owned().into(), user_id.to_owned().into()],
            ))
            .await?;
        let mut items = rows
            .iter()
            .map(|row| map_trip_summary(row, options.today))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|item| {
                options
                    .bucket
                    .is_none_or(|bucket| item.time_bucket == bucket)
            })
            .filter(|item| {
                options
                    .trip_type
                    .is_none_or(|trip_type| item.trip_type == trip_type)
            })
            .collect::<Vec<_>>();
        let next_cursor = if items.len() > offset + limit {
            Some((offset + limit).to_string())
        } else {
            None
        };
        items = items.into_iter().skip(offset).take(limit).collect();
        Ok((items, next_cursor))
    }

    /// Selects the current or next dated plan for the user's homepage reminder.
    pub async fn home_highlight(
        &self,
        user_id: &str,
        today: Date,
    ) -> Result<Option<TripHomeHighlight>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT p.id, p.owner_user_id, p.trip_type, p.title, p.description, p.start_date, \
                        p.end_date, p.enabled_sections_json, \
                        p.route_use_slope_adjustment, p.route_use_high_altitude_adjustment, \
                        p.route_start_altitude_m, \
                        (SELECT COUNT(*) FROM trip_itinerary_days r \
                         WHERE r.trip_id = p.id AND r.is_deleted = FALSE) AS day_count, \
                        p.field_versions_json, p.is_deleted, p.created_at, p.updated_at \
                 FROM trips p \
                 JOIN trip_members m ON m.trip_id = p.id AND m.user_id = ? AND m.is_deleted = FALSE \
                 WHERE p.is_deleted = FALSE AND p.start_date IS NOT NULL",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let mut ongoing: Option<HomeHighlightCandidate> = None;
        let mut upcoming: Option<HomeHighlightCandidate> = None;
        for row in rows {
            let plan = map_plan(&row)?;
            let Some(candidate) = HomeHighlightCandidate::from_plan(plan, today) else {
                continue;
            };
            if candidate.is_ongoing() {
                if ongoing
                    .as_ref()
                    .is_none_or(|current| candidate.ongoing_rank_key() < current.ongoing_rank_key())
                {
                    ongoing = Some(candidate);
                }
            } else if candidate.is_upcoming()
                && upcoming.as_ref().is_none_or(|current| {
                    candidate.upcoming_rank_key() < current.upcoming_rank_key()
                })
            {
                upcoming = Some(candidate);
            }
        }
        Ok(ongoing
            .map(TripHomeHighlight::from_ongoing)
            .or_else(|| upcoming.map(TripHomeHighlight::from_upcoming)))
    }

    /// Reads one full plan detail if the current user is a member.
    pub async fn detail_for_user(
        &self,
        user_id: &str,
        trip_id: &str,
    ) -> Result<Option<TripDetail>, DbErr> {
        let Some(my_member) = self.member_for_user(user_id, trip_id).await? else {
            return Ok(None);
        };
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(None);
        };
        let members = self.members(trip_id).await?;
        let records = self.records(trip_id).await?;
        Ok(Some(build_detail(plan, my_member.id, members, records)?))
    }

    /// Lists backend-owned shared gear slot templates for trip shared gear demand rows.
    pub async fn shared_gear_demand_templates(
        &self,
    ) -> Result<Vec<SharedGearDemandTemplate>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT template_key, demand_name, group_label, category, category_label, \
                        planned_quantity, sort_order \
                 FROM shared_gear_demand_templates \
                 WHERE status = 'active' \
                 ORDER BY sort_order ASC, template_key ASC",
                vec![],
            ))
            .await?;
        rows.iter().map(map_shared_gear_template).collect()
    }

    async fn instantiate_shared_gear_demands(
        &self,
        user_id: &str,
        trip_id: &str,
        responsible_member_id: &str,
    ) -> Result<(), DbErr> {
        for template in self.shared_gear_demand_templates().await? {
            self.insert_record(
                user_id,
                trip_id,
                &Uuid::new_v4().to_string(),
                KIND_SHARED_GEAR,
                None,
                template.sort_order,
                json!({
                    "template_key": template.template_key,
                    "demand_name": template.demand_name,
                    "concrete_name": null,
                    "source_member_id": null,
                    "source_gear_id": null,
                    "responsible_member_id": responsible_member_id,
                    "category": template.category,
                    "category_label": template.category_label,
                    "name": template.demand_name,
                    "brand": null,
                    "model": null,
                    "planned_quantity": template.planned_quantity,
                    "packed_quantity": 0,
                    "unit_weight_g": null,
                    "notes": null,
                }),
            )
            .await?;
        }
        Ok(())
    }

    /// Updates root trip metadata with field-level optimistic concurrency.
    pub async fn update_plan_fields(
        &self,
        user_id: &str,
        trip_id: &str,
        changes: BTreeMap<String, JsonValue>,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        self.require_member(user_id, trip_id).await?;
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(None);
        };
        if plan.owner_user_id != user_id {
            return Err(ValidationError::single(
                "trip_id",
                "only the owner can edit plan settings",
            )
            .into());
        }
        let object = json!({
            "title": plan.title,
            "description": plan.description,
            "start_date": plan.start_date,
            "end_date": plan.end_date,
            "enabled_sections": plan.enabled_sections,
            "route_use_slope_adjustment": plan.route_use_slope_adjustment,
            "route_use_high_altitude_adjustment": plan.route_use_high_altitude_adjustment,
            "route_start_altitude_m": plan.route_start_altitude_m,
        });
        let route_settings_may_change = [
            "route_use_slope_adjustment",
            "route_use_high_altitude_adjustment",
            "route_start_altitude_m",
        ]
        .iter()
        .any(|field| changes.contains_key(*field));
        let (patched, versions) = apply_field_patch(
            object,
            plan.field_versions,
            changes,
            &base_versions,
            &force_fields,
        )?;
        validate_plan_route_settings(&patched)?;
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trips SET title = ?, description = ?, \
                 start_date = ?, end_date = ?, enabled_sections_json = ?, \
                 route_use_slope_adjustment = ?, route_use_high_altitude_adjustment = ?, \
                 route_start_altitude_m = ?, field_versions_json = ?, \
                 updated_at = ? WHERE id = ? AND is_deleted = FALSE",
                vec![
                    required_string(&patched, "title")?.into(),
                    optional_string(&patched, "description").into(),
                    optional_string(&patched, "start_date").into(),
                    optional_string(&patched, "end_date").into(),
                    json_string(&patched["enabled_sections"])?.into(),
                    required_bool(&patched, "route_use_slope_adjustment")?.into(),
                    required_bool(&patched, "route_use_high_altitude_adjustment")?.into(),
                    optional_i32(&patched, "route_start_altitude_m").into(),
                    json_string(&versions)?.into(),
                    now.into(),
                    trip_id.to_owned().into(),
                ],
            ))
            .await?;
        if route_settings_may_change {
            self.recalculate_route_records(trip_id).await?;
        }
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Updates enabled sections after validating food-plan dependencies.
    pub async fn update_sections(
        &self,
        user_id: &str,
        trip_id: &str,
        sections: Vec<TripSectionKey>,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        if sections.contains(&TripSectionKey::FoodPlan)
            && !sections.contains(&TripSectionKey::Itinerary)
        {
            return Err(ValidationError::single(
                "enabled_sections",
                "food_plan requires itinerary",
            )
            .into());
        }
        let mut changes = BTreeMap::new();
        changes.insert("enabled_sections".to_owned(), json!(sections));
        self.update_plan_fields(user_id, trip_id, changes, base_versions, force_fields)
            .await
    }

    /// Soft-deletes a plan. Only the owner can delete the plan.
    pub async fn soft_delete_plan(&self, user_id: &str, trip_id: &str) -> Result<bool, DbErr> {
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(false);
        };
        if plan.owner_user_id != user_id {
            return Ok(false);
        }
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trips SET is_deleted = TRUE, updated_at = ? \
                 WHERE id = ? AND is_deleted = FALSE",
                vec![now.into(), trip_id.to_owned().into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Creates an owner-managed invitation token for the plan.
    pub async fn create_invitation(
        &self,
        user_id: &str,
        trip_id: &str,
    ) -> Result<Option<TripInvitation>, TripRepositoryError> {
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(None);
        };
        if plan.trip_type == TripType::Solo {
            return Err(ValidationError::single(
                "trip_type",
                "solo trips do not support invitations",
            )
            .into());
        }
        if plan.owner_user_id != user_id {
            return Ok(None);
        }
        let record = TripInvitation {
            id: Uuid::new_v4().to_string(),
            trip_id: trip_id.to_owned(),
            token: Uuid::new_v4().to_string(),
            created_by_user_id: user_id.to_owned(),
            revoked_at: None,
            created_at: now_rfc3339(),
        };
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO trip_invitations \
                 (id, trip_id, token, created_by_user_id, revoked_at, created_at) \
                 VALUES (?, ?, ?, ?, NULL, ?)",
                vec![
                    record.id.clone().into(),
                    record.trip_id.clone().into(),
                    record.token.clone().into(),
                    record.created_by_user_id.clone().into(),
                    record.created_at.clone().into(),
                ],
            ))
            .await?;
        Ok(Some(record))
    }

    /// Accepts an invitation token and adds the current user as a member.
    pub async fn accept_invitation(
        &self,
        user: &UserRecord,
        token: &str,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        let Some(invite) = self.invitation_by_token(token).await? else {
            return Ok(None);
        };
        if invite.revoked_at.is_some() {
            return Ok(None);
        }
        let Some(plan) = self.plan(&invite.trip_id).await? else {
            return Ok(None);
        };
        if plan.is_deleted {
            return Ok(None);
        }
        if plan.trip_type == TripType::Solo {
            return Err(ValidationError::single(
                "trip_type",
                "solo trips do not support invitations",
            )
            .into());
        }
        if self
            .member_for_user(&user.id, &invite.trip_id)
            .await?
            .is_none()
        {
            let profile = TripMemberProfile {
                display_name: display_name_for_user(user),
                ..TripMemberProfile::default()
            };
            self.insert_member(
                &invite.trip_id,
                &Uuid::new_v4().to_string(),
                user,
                &profile,
                &now_rfc3339(),
            )
            .await?;
        }
        self.detail_for_user(&user.id, &invite.trip_id)
            .await
            .map_err(Into::into)
    }

    /// Updates one member profile. Owners can update any member; members can update themselves.
    pub async fn update_member(
        &self,
        user_id: &str,
        trip_id: &str,
        member_id: &str,
        changes: BTreeMap<String, JsonValue>,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        let Some(actor) = self.member_for_user(user_id, trip_id).await? else {
            return Ok(None);
        };
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(None);
        };
        if actor.id != member_id && plan.owner_user_id != user_id {
            return Err(ValidationError::single("member_id", "cannot edit this member").into());
        }
        let Some(member) = self.member_by_id(trip_id, member_id).await? else {
            return Ok(None);
        };
        let payload = serde_json::to_value(&member.profile).map_err(json_db_error)?;
        let (patched, versions) = apply_field_patch(
            payload,
            member.field_versions,
            changes,
            &base_versions,
            &force_fields,
        )?;
        let mut profile: TripMemberProfile =
            serde_json::from_value(patched).map_err(json_db_error)?;
        profile.validate_and_normalize()?;
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trip_members SET profile_json = ?, field_versions_json = ?, updated_at = ? \
                 WHERE trip_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    json_string(&profile)?.into(),
                    json_string(&versions)?.into(),
                    now.into(),
                    trip_id.to_owned().into(),
                    member_id.to_owned().into(),
                ],
            ))
            .await?;
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Soft-removes one member. Only the owner can remove non-owner members.
    pub async fn remove_member(
        &self,
        user_id: &str,
        trip_id: &str,
        member_id: &str,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(None);
        };
        if plan.trip_type == TripType::Solo {
            return Err(ValidationError::single(
                "trip_type",
                "solo trips do not support member removal",
            )
            .into());
        }
        if plan.owner_user_id != user_id {
            return Err(
                ValidationError::single("member_id", "only the owner can remove members").into(),
            );
        }
        let Some(member) = self.member_by_id(trip_id, member_id).await? else {
            return Ok(None);
        };
        if member.user_id == plan.owner_user_id {
            return Err(ValidationError::single("member_id", "cannot remove the owner").into());
        }
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trip_members SET is_deleted = TRUE, updated_at = ? \
                 WHERE trip_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    trip_id.to_owned().into(),
                    member_id.to_owned().into(),
                ],
            ))
            .await?;
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Creates a typed record in one plan section.
    pub async fn create_record(
        &self,
        user_id: &str,
        trip_id: &str,
        kind: &str,
        parent_id: Option<&str>,
        sort_order: i32,
        payload: JsonValue,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        self.require_member(user_id, trip_id).await?;
        self.validate_record_payload(trip_id, kind, &payload)
            .await?;
        let id = Uuid::new_v4().to_string();
        self.insert_record(user_id, trip_id, &id, kind, parent_id, sort_order, payload)
            .await?;
        if kind == KIND_ITINERARY_DAY {
            self.ensure_food_meals_for_day(user_id, trip_id, &id)
                .await?;
        }
        if route_order_may_change(kind) {
            self.recalculate_route_records(trip_id).await?;
        }
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Updates a typed record payload with field-level optimistic concurrency.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_record(
        &self,
        user_id: &str,
        trip_id: &str,
        kind: &str,
        record_id: &str,
        changes: BTreeMap<String, JsonValue>,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        self.require_member(user_id, trip_id).await?;
        let Some(record) = self.record(trip_id, kind, record_id).await? else {
            return Ok(None);
        };
        let (payload, versions) = apply_field_patch(
            record.payload,
            record.field_versions,
            changes,
            &base_versions,
            &force_fields,
        )?;
        self.validate_record_payload(trip_id, kind, &payload)
            .await?;
        let now = now_rfc3339();
        let table = record_table(kind)?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                format!(
                    "UPDATE {table} SET payload_json = ?, field_versions_json = ?, updated_at = ? \
                     WHERE trip_id = ? AND id = ? AND is_deleted = FALSE"
                ),
                vec![
                    json_string(&payload)?.into(),
                    json_string(&versions)?.into(),
                    now.into(),
                    trip_id.to_owned().into(),
                    record_id.to_owned().into(),
                ],
            ))
            .await?;
        if route_order_may_change(kind) {
            self.recalculate_route_records(trip_id).await?;
        }
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Soft-deletes a typed record.
    /// Soft-deletes a typed record.
    pub async fn delete_record(
        &self,
        user_id: &str,
        trip_id: &str,
        kind: &str,
        record_id: &str,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        self.require_member(user_id, trip_id).await?;
        let now = now_rfc3339();
        let table = record_table(kind)?;
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                format!(
                    "UPDATE {table} SET is_deleted = TRUE, updated_at = ? \
                     WHERE trip_id = ? AND id = ? AND is_deleted = FALSE"
                ),
                vec![
                    now.into(),
                    trip_id.to_owned().into(),
                    record_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        if route_order_may_change(kind) {
            self.recalculate_route_records(trip_id).await?;
        }
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Imports the current member's existing packing-list items as independent snapshots.
    pub async fn import_packing_list(
        &self,
        user_id: &str,
        trip_id: &str,
        packing_list_id: &str,
    ) -> Result<Option<TripDetail>, TripRepositoryError> {
        let Some(member) = self.member_for_user(user_id, trip_id).await? else {
            return Ok(None);
        };
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT i.id AS packing_item_id, i.gear_id, i.planned_quantity, i.packed_quantity, \
                        g.category, g.name, g.brand, g.model, g.weight_g \
                 FROM gear_packing_lists l \
                 JOIN gear_packing_list_items i ON i.packing_list_id = l.id AND i.user_id = l.user_id \
                 JOIN user_gear_items g ON g.id = i.gear_id AND g.user_id = i.user_id \
                 WHERE l.user_id = ? AND l.id = ? AND l.is_deleted = FALSE \
                   AND g.is_deleted = FALSE \
                 ORDER BY i.created_at ASC, i.id ASC",
                vec![user_id.to_owned().into(), packing_list_id.to_owned().into()],
            ))
            .await?;
        if rows.is_empty() {
            return Err(ValidationError::single(
                "packing_list_id",
                "packing list has no importable items",
            )
            .into());
        }
        for (index, row) in rows.iter().enumerate() {
            let category_raw: String = row.try_get("", "category")?;
            let category = GearCategory::from_key(&category_raw)
                .ok_or_else(|| DbErr::Custom(format!("invalid gear category: {category_raw}")))?;
            let packing_item_id: String = row.try_get("", "packing_item_id")?;
            let gear_id: String = row.try_get("", "gear_id")?;
            let name: String = row.try_get("", "name")?;
            let brand: Option<String> = row.try_get("", "brand")?;
            let model: Option<String> = row.try_get("", "model")?;
            let planned_quantity: i32 = row.try_get("", "planned_quantity")?;
            let packed_quantity: i32 = row.try_get("", "packed_quantity")?;
            let unit_weight_g: Option<i32> = row.try_get("", "weight_g")?;
            let payload = json!({
                "member_id": member.id,
                "source_packing_list_id": packing_list_id,
                "source_packing_item_id": packing_item_id,
                "source_gear_id": gear_id,
                "category": category,
                "category_label": category.label(),
                "name": name,
                "brand": brand,
                "model": model,
                "planned_quantity": planned_quantity,
                "packed_quantity": packed_quantity,
                "unit_weight_g": unit_weight_g,
                "notes": null,
            });
            self.insert_record(
                user_id,
                trip_id,
                &Uuid::new_v4().to_string(),
                KIND_PERSONAL_GEAR,
                Some(&member.id),
                index as i32,
                payload,
            )
            .await?;
        }
        self.detail_for_user(user_id, trip_id)
            .await
            .map_err(Into::into)
    }

    /// Lists structured outdoor experiences owned by one user.
    pub async fn list_outdoor_experiences(
        &self,
        user_id: &str,
    ) -> Result<Vec<OutdoorExperience>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT id, user_id, source_trip_id, trip_type, title, start_date, end_date, \
                        day_count, companion_count, route_summary, gear_summary, food_summary, \
                        budget_summary, notes, snapshot_json, field_versions_json, is_deleted, \
                        created_at, updated_at \
                 FROM outdoor_experiences \
                 WHERE user_id = ? AND is_deleted = FALSE \
                 ORDER BY COALESCE(end_date, start_date, updated_at) DESC, updated_at DESC",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_outdoor_experience).collect()
    }

    /// Reads one outdoor experience owned by the current user.
    pub async fn outdoor_experience(
        &self,
        user_id: &str,
        experience_id: &str,
    ) -> Result<Option<OutdoorExperience>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, user_id, source_trip_id, trip_type, title, start_date, end_date, \
                        day_count, companion_count, route_summary, gear_summary, food_summary, \
                        budget_summary, notes, snapshot_json, field_versions_json, is_deleted, \
                        created_at, updated_at \
                 FROM outdoor_experiences \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE LIMIT 1",
                vec![user_id.to_owned().into(), experience_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_outdoor_experience).transpose()
    }

    /// Creates one manual outdoor experience row.
    pub async fn create_outdoor_experience(
        &self,
        user_id: &str,
        mut draft: OutdoorExperienceDraft,
    ) -> Result<OutdoorExperience, TripRepositoryError> {
        draft.validate_and_normalize()?;
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let day_count = draft.day_count.unwrap_or_else(|| {
            inclusive_trip_days(draft.start_date.as_deref(), draft.end_date.as_deref()).unwrap_or(0)
        });
        let companion_count = draft.companion_count.unwrap_or(0);
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO outdoor_experiences \
                 (id, user_id, source_trip_id, trip_type, title, start_date, end_date, day_count, \
                  companion_count, route_summary, gear_summary, food_summary, budget_summary, notes, \
                  snapshot_json, field_versions_json, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, NULL, 'solo', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.title.into(),
                    draft.start_date.into(),
                    draft.end_date.into(),
                    day_count.into(),
                    companion_count.into(),
                    draft.route_summary.into(),
                    draft.gear_summary.into(),
                    draft.food_summary.into(),
                    draft.budget_summary.into(),
                    draft.notes.into(),
                    json_string(&json!({ "source": "manual" }))?.into(),
                    json_string(&initial_versions([
                        "title",
                        "start_date",
                        "end_date",
                        "day_count",
                        "companion_count",
                        "route_summary",
                        "gear_summary",
                        "food_summary",
                        "budget_summary",
                        "notes",
                    ]))?.into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.outdoor_experience(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created outdoor experience not found".to_owned()).into())
    }

    /// Updates a structured outdoor experience row.
    pub async fn update_outdoor_experience(
        &self,
        user_id: &str,
        experience_id: &str,
        mut draft: OutdoorExperienceDraft,
    ) -> Result<Option<OutdoorExperience>, TripRepositoryError> {
        if self
            .outdoor_experience(user_id, experience_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        draft.validate_and_normalize()?;
        let day_count = draft.day_count.unwrap_or_else(|| {
            inclusive_trip_days(draft.start_date.as_deref(), draft.end_date.as_deref()).unwrap_or(0)
        });
        let companion_count = draft.companion_count.unwrap_or(0);
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE outdoor_experiences SET title = ?, start_date = ?, end_date = ?, \
                 day_count = ?, companion_count = ?, route_summary = ?, gear_summary = ?, \
                 food_summary = ?, budget_summary = ?, notes = ?, updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    draft.title.into(),
                    draft.start_date.into(),
                    draft.end_date.into(),
                    day_count.into(),
                    companion_count.into(),
                    draft.route_summary.into(),
                    draft.gear_summary.into(),
                    draft.food_summary.into(),
                    draft.budget_summary.into(),
                    draft.notes.into(),
                    now.into(),
                    user_id.to_owned().into(),
                    experience_id.to_owned().into(),
                ],
            ))
            .await?;
        self.outdoor_experience(user_id, experience_id)
            .await
            .map_err(Into::into)
    }

    /// Soft deletes one structured outdoor experience row.
    pub async fn delete_outdoor_experience(
        &self,
        user_id: &str,
        experience_id: &str,
    ) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE outdoor_experiences SET is_deleted = TRUE, updated_at = ? \
                 WHERE user_id = ? AND id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    user_id.to_owned().into(),
                    experience_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Converts a past trip into the user's structured outdoor experience list.
    pub async fn convert_trip_to_outdoor_experience(
        &self,
        user_id: &str,
        trip_id: &str,
        today: Date,
    ) -> Result<Option<OutdoorExperience>, TripRepositoryError> {
        let Some(detail) = self.detail_for_user(user_id, trip_id).await? else {
            return Ok(None);
        };
        if trip_time_context(&detail.trip, today).0 != TripTimeBucket::Past {
            return Err(ValidationError::single(
                "trip_id",
                "only past trips can be converted to outdoor experiences",
            )
            .into());
        }
        if self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id FROM outdoor_experiences \
                 WHERE user_id = ? AND source_trip_id = ? AND is_deleted = FALSE LIMIT 1",
                vec![user_id.to_owned().into(), trip_id.to_owned().into()],
            ))
            .await?
            .is_some()
        {
            return Err(ValidationError::single(
                "trip_id",
                "trip has already been converted to an outdoor experience",
            )
            .into());
        }
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let companion_count = detail.members.len().saturating_sub(1) as i64;
        let day_count = detail.trip.day_count.max(
            inclusive_trip_days(
                detail.trip.start_date.as_deref(),
                detail.trip.end_date.as_deref(),
            )
            .unwrap_or(0),
        );
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO outdoor_experiences \
                 (id, user_id, source_trip_id, trip_type, title, start_date, end_date, day_count, \
                  companion_count, route_summary, gear_summary, food_summary, budget_summary, notes, \
                  snapshot_json, field_versions_json, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, NULL, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    trip_id.to_owned().into(),
                    trip_type_key(detail.trip.trip_type).to_owned().into(),
                    detail.trip.title.clone().into(),
                    detail.trip.start_date.clone().into(),
                    detail.trip.end_date.clone().into(),
                    day_count.into(),
                    companion_count.into(),
                    json_string(&detail)?.into(),
                    json_string(&initial_versions([
                        "title",
                        "start_date",
                        "end_date",
                        "day_count",
                        "companion_count",
                        "route_summary",
                        "gear_summary",
                        "food_summary",
                        "budget_summary",
                        "notes",
                    ]))?.into(),
                    now.clone().into(),
                    now.clone().into(),
                ],
            ))
            .await?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO outdoor_experience_trails \
                 (outdoor_experience_id, trail_id, linked_by_user_id, role, sort_order, notes, \
                  is_deleted, created_at, updated_at) \
                 SELECT ?, tt.trail_id, ?, tt.role, tt.sort_order, tt.notes, FALSE, ?, ? \
                 FROM trip_trails tt \
                 JOIN trails t ON t.id = tt.trail_id \
                 WHERE tt.trip_id = ? AND tt.is_deleted = FALSE \
                    AND t.owner_user_id = ? AND t.is_deleted = FALSE",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    now.clone().into(),
                    now.clone().into(),
                    trip_id.to_owned().into(),
                    user_id.to_owned().into(),
                ],
            ))
            .await?;
        self.outdoor_experience(user_id, &id)
            .await
            .map_err(Into::into)
    }

    async fn insert_member(
        &self,
        trip_id: &str,
        member_id: &str,
        user: &UserRecord,
        profile: &TripMemberProfile,
        now: &str,
    ) -> Result<(), DbErr> {
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO trip_members \
                 (id, trip_id, user_id, profile_json, field_versions_json, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, FALSE, ?, ?) \
                 ON CONFLICT(trip_id, user_id) DO UPDATE SET is_deleted = FALSE, updated_at = excluded.updated_at",
                vec![
                    member_id.to_owned().into(),
                    trip_id.to_owned().into(),
                    user.id.clone().into(),
                    json_string(profile)?.into(),
                    json_string(&initial_versions(["display_name"]))?.into(),
                    now.to_owned().into(),
                    now.to_owned().into(),
                ],
            ))
            .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_record(
        &self,
        user_id: &str,
        trip_id: &str,
        id: &str,
        kind: &str,
        parent_id: Option<&str>,
        sort_order: i32,
        payload: JsonValue,
    ) -> Result<(), DbErr> {
        let now = now_rfc3339();
        let table = record_table(kind)?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                format!(
                    "INSERT INTO {table} \
                     (id, trip_id, parent_id, sort_order, payload_json, field_versions_json, \
                      created_by_user_id, is_deleted, created_at, updated_at) \
                     VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?)"
                ),
                vec![
                    id.to_owned().into(),
                    trip_id.to_owned().into(),
                    parent_id.map(str::to_owned).into(),
                    sort_order.into(),
                    json_string(&payload)?.into(),
                    json_string(&versions_for_payload(&payload))?.into(),
                    user_id.to_owned().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(())
    }

    async fn ensure_food_meals_for_day(
        &self,
        user_id: &str,
        trip_id: &str,
        day_id: &str,
    ) -> Result<(), DbErr> {
        for (index, meal_key) in MEAL_KEYS.iter().enumerate() {
            let exists = self
                .db
                .query_one(statement(
                    self.db.get_database_backend(),
                    "SELECT id FROM trip_food_meals \
                     WHERE trip_id = ? AND parent_id = ? AND is_deleted = FALSE \
                       AND payload_json LIKE ? LIMIT 1",
                    vec![
                        trip_id.to_owned().into(),
                        day_id.to_owned().into(),
                        format!("%\\\"meal_key\\\":\\\"{meal_key}\\\"%").into(),
                    ],
                ))
                .await?;
            if exists.is_some() {
                continue;
            }
            self.insert_record(
                user_id,
                trip_id,
                &Uuid::new_v4().to_string(),
                KIND_FOOD_MEAL,
                Some(day_id),
                index as i32,
                json!({
                    "itinerary_day_id": day_id,
                    "meal_key": meal_key,
                    "meal_type": null,
                    "skipped": false,
                    "dish_name": null,
                    "responsible_member_id": null,
                    "notes": null,
                }),
            )
            .await?;
        }
        Ok(())
    }

    async fn require_member(&self, user_id: &str, trip_id: &str) -> Result<TripMember, DbErr> {
        self.member_for_user(user_id, trip_id)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("trip not found".to_owned()))
    }

    async fn member_for_user(
        &self,
        user_id: &str,
        trip_id: &str,
    ) -> Result<Option<TripMember>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT m.id, m.trip_id, m.user_id, m.profile_json, m.field_versions_json, \
                        m.is_deleted, m.created_at, m.updated_at, p.owner_user_id \
                 FROM trip_members m \
                 JOIN trips p ON p.id = m.trip_id AND p.is_deleted = FALSE \
                 WHERE m.user_id = ? AND m.trip_id = ? AND m.is_deleted = FALSE LIMIT 1",
                vec![user_id.to_owned().into(), trip_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_member).transpose()
    }

    async fn member_by_id(
        &self,
        trip_id: &str,
        member_id: &str,
    ) -> Result<Option<TripMember>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT m.id, m.trip_id, m.user_id, m.profile_json, m.field_versions_json, \
                        m.is_deleted, m.created_at, m.updated_at, p.owner_user_id \
                 FROM trip_members m \
                 JOIN trips p ON p.id = m.trip_id \
                 WHERE m.trip_id = ? AND m.id = ? AND m.is_deleted = FALSE LIMIT 1",
                vec![trip_id.to_owned().into(), member_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_member).transpose()
    }

    async fn members(&self, trip_id: &str) -> Result<Vec<TripMember>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT m.id, m.trip_id, m.user_id, m.profile_json, m.field_versions_json, \
                        m.is_deleted, m.created_at, m.updated_at, p.owner_user_id \
                 FROM trip_members m \
                 JOIN trips p ON p.id = m.trip_id \
                 WHERE m.trip_id = ? AND m.is_deleted = FALSE ORDER BY m.created_at ASC, m.id ASC",
                vec![trip_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_member).collect()
    }

    async fn plan(&self, trip_id: &str) -> Result<Option<Trip>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, owner_user_id, trip_type, title, description, start_date, end_date, \
                        enabled_sections_json, \
                        route_use_slope_adjustment, route_use_high_altitude_adjustment, \
                        route_start_altitude_m, \
                        (SELECT COUNT(*) FROM trip_itinerary_days r \
                         WHERE r.trip_id = trips.id AND r.is_deleted = FALSE) AS day_count, \
                        field_versions_json, is_deleted, created_at, updated_at \
                 FROM trips WHERE id = ? AND is_deleted = FALSE LIMIT 1",
                vec![trip_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_plan).transpose()
    }

    async fn invitation_by_token(&self, token: &str) -> Result<Option<TripInvitation>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id, trip_id, token, created_by_user_id, revoked_at, created_at \
                 FROM trip_invitations WHERE token = ? LIMIT 1",
                vec![token.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_invitation).transpose()
    }

    async fn records(&self, trip_id: &str) -> Result<Vec<TripRecordRow>, DbErr> {
        let mut records = Vec::new();
        for (kind, table) in TRIP_RECORD_TABLES {
            let rows = self
                .db
                .query_all(statement(
                    self.db.get_database_backend(),
                    format!(
                        "SELECT id, trip_id, parent_id, sort_order, payload_json, field_versions_json, \
                         created_by_user_id, is_deleted, created_at, updated_at \
                         FROM {table} WHERE trip_id = ? AND is_deleted = FALSE"
                    ),
                    vec![trip_id.to_owned().into()],
                ))
                .await?;
            for row in &rows {
                records.push(map_record_row(row, kind)?);
            }
        }
        records.sort_by(|left, right| {
            (
                left.kind.as_str(),
                left.sort_order,
                left.created_at.as_str(),
                left.id.as_str(),
            )
                .cmp(&(
                    right.kind.as_str(),
                    right.sort_order,
                    right.created_at.as_str(),
                    right.id.as_str(),
                ))
        });
        Ok(records)
    }

    async fn record(
        &self,
        trip_id: &str,
        kind: &str,
        record_id: &str,
    ) -> Result<Option<TripRecordRow>, DbErr> {
        let table = record_table(kind)?;
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT id, trip_id, parent_id, sort_order, payload_json, field_versions_json, \
                     created_by_user_id, is_deleted, created_at, updated_at \
                     FROM {table} WHERE trip_id = ? AND id = ? AND is_deleted = FALSE LIMIT 1"
                ),
                vec![trip_id.to_owned().into(), record_id.to_owned().into()],
            ))
            .await?;
        row.as_ref()
            .map(|row| map_record_row(row, kind))
            .transpose()
    }

    async fn validate_record_payload(
        &self,
        trip_id: &str,
        kind: &str,
        payload: &JsonValue,
    ) -> Result<(), TripRepositoryError> {
        match kind {
            KIND_SHARED_GEAR => {
                self.ensure_section_enabled(trip_id, TripSectionKey::SharedGear, "shared_gear")
                    .await?;
                let responsible_member_id = payload
                    .get("responsible_member_id")
                    .and_then(JsonValue::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        ValidationError::single(
                            "responsible_member_id",
                            "shared gear demand requires a responsible member",
                        )
                    })?;
                if self
                    .member_by_id(trip_id, responsible_member_id)
                    .await?
                    .is_none()
                {
                    return Err(ValidationError::single(
                        "responsible_member_id",
                        "responsible member does not belong to this trip",
                    )
                    .into());
                }
                if let Some(source_member_id) = payload
                    .get("source_member_id")
                    .and_then(JsonValue::as_str)
                    .filter(|value| !value.trim().is_empty())
                    && self
                        .member_by_id(trip_id, source_member_id)
                        .await?
                        .is_none()
                {
                    return Err(ValidationError::single(
                        "source_member_id",
                        "source member does not belong to this trip",
                    )
                    .into());
                }
            }
            KIND_ITINERARY_DAY | KIND_TIME_SLOT | KIND_ROUTE_SEGMENT | KIND_SEGMENT_ASSIGNMENT => {
                self.ensure_section_enabled(trip_id, TripSectionKey::Itinerary, "itinerary")
                    .await?;
            }
            KIND_FOOD_MEAL | KIND_FOOD_ITEM | KIND_FOOD_SUPPLY => {
                self.ensure_section_enabled(trip_id, TripSectionKey::Itinerary, "itinerary")
                    .await?;
                self.ensure_section_enabled(trip_id, TripSectionKey::FoodPlan, "food_plan")
                    .await?;
                let has_day = self
                    .db
                    .query_one(statement(
                        self.db.get_database_backend(),
                        "SELECT id FROM trip_itinerary_days \
                         WHERE trip_id = ? AND is_deleted = FALSE LIMIT 1",
                        vec![trip_id.to_owned().into()],
                    ))
                    .await?
                    .is_some();
                if !has_day {
                    return Err(ValidationError::single(
                        "food_plan",
                        "add an itinerary day before editing food plan",
                    )
                    .into());
                }
            }
            KIND_MEDICAL_ITEM => {
                self.ensure_section_enabled(trip_id, TripSectionKey::MedicalKit, "medical_kit")
                    .await?;
            }
            KIND_SAFETY_RISK => {
                self.ensure_section_enabled(trip_id, TripSectionKey::SafetyPlan, "safety_plan")
                    .await?;
            }
            KIND_RESCUE_CONTACT => {
                self.ensure_section_enabled(trip_id, TripSectionKey::RescueInfo, "rescue_info")
                    .await?;
            }
            KIND_BUDGET_ITEM => {
                self.ensure_section_enabled(trip_id, TripSectionKey::Budget, "budget")
                    .await?;
            }
            KIND_GOAL_ITEM => {
                self.ensure_section_enabled(trip_id, TripSectionKey::Goals, "goals")
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn ensure_section_enabled(
        &self,
        trip_id: &str,
        section: TripSectionKey,
        field: &str,
    ) -> Result<(), TripRepositoryError> {
        let plan = self
            .plan(trip_id)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("trip not found".to_owned()))?;
        if plan.enabled_sections.contains(&section) {
            Ok(())
        } else {
            Err(ValidationError::single(field, "section is not enabled").into())
        }
    }

    async fn recalculate_route_records(&self, trip_id: &str) -> Result<(), DbErr> {
        let Some(plan) = self.plan(trip_id).await? else {
            return Ok(());
        };
        let records = self.records(trip_id).await?;
        let route_order = ordered_assigned_route_ids(&records);
        let mut route_records = records
            .iter()
            .filter(|record| record.kind == KIND_ROUTE_SEGMENT)
            .map(|record| (record.id.clone(), record.payload.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut processed = BTreeSet::new();
        let mut current_altitude_m = plan.route_start_altitude_m.unwrap_or(0);
        let mut updates = BTreeMap::new();

        for route_id in route_order {
            let Some(payload) = route_records.get_mut(&route_id) else {
                continue;
            };
            let altitude = if plan.route_use_high_altitude_adjustment {
                let estimate = RouteAltitudeEstimate::from_segment(
                    current_altitude_m,
                    json_i32(payload, "ascent_m"),
                    json_i32(payload, "descent_m"),
                );
                current_altitude_m = estimate.end_m;
                Some(estimate)
            } else {
                None
            };
            recalculate_route_payload(payload, &plan, altitude)?;
            processed.insert(route_id.clone());
            updates.insert(route_id, payload.clone());
        }

        for record in records
            .iter()
            .filter(|record| record.kind == KIND_ROUTE_SEGMENT && !processed.contains(&record.id))
        {
            let mut payload = record.payload.clone();
            let altitude = plan.route_use_high_altitude_adjustment.then(|| {
                RouteAltitudeEstimate::from_segment(
                    plan.route_start_altitude_m.unwrap_or(0),
                    json_i32(&payload, "ascent_m"),
                    json_i32(&payload, "descent_m"),
                )
            });
            recalculate_route_payload(&mut payload, &plan, altitude)?;
            updates.insert(record.id.clone(), payload);
        }

        let now = now_rfc3339();
        for record in records
            .iter()
            .filter(|record| record.kind == KIND_ROUTE_SEGMENT)
        {
            let Some(payload) = updates.get(&record.id) else {
                continue;
            };
            if *payload == record.payload {
                continue;
            }
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "UPDATE trip_route_segments SET payload_json = ?, updated_at = ? \
                     WHERE trip_id = ? AND id = ? AND is_deleted = FALSE",
                    vec![
                        json_string(payload)?.into(),
                        now.clone().into(),
                        trip_id.to_owned().into(),
                        record.id.clone().into(),
                    ],
                ))
                .await?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct TripRecordRow {
    id: String,
    kind: String,
    sort_order: i32,
    payload: JsonValue,
    field_versions: FieldVersions,
    created_by_user_id: Option<String>,
    created_at: String,
    updated_at: String,
}

fn build_detail(
    plan: Trip,
    my_member_id: String,
    members: Vec<TripMember>,
    records: Vec<TripRecordRow>,
) -> Result<TripDetail, DbErr> {
    let mut personal_gear: Vec<TripPersonalGearItem> = Vec::new();
    let mut shared_gear_demands: Vec<TripSharedGearDemand> = Vec::new();
    let mut itinerary_days: Vec<TripItineraryDay> = Vec::new();
    let mut time_slots: Vec<TripItineraryTimeSlot> = Vec::new();
    let mut route_segments: Vec<TripRouteSegment> = Vec::new();
    let mut food_meals: Vec<TripFoodMeal> = Vec::new();
    let mut food_items: Vec<TripFoodItem> = Vec::new();
    let mut food_supplies: Vec<TripFoodSupply> = Vec::new();
    let mut medical_items: Vec<TripMedicalItem> = Vec::new();
    let mut segment_assignments: Vec<TripSegmentAssignment> = Vec::new();
    let mut safety_risks: Vec<TripSafetyRisk> = Vec::new();
    let mut rescue_contacts: Vec<TripRescueContact> = Vec::new();
    let mut budget_items: Vec<TripBudgetItem> = Vec::new();
    let mut goals: Vec<TripGoalItem> = Vec::new();

    for record in records {
        match record.kind.as_str() {
            KIND_PERSONAL_GEAR => personal_gear.push(record_typed(record)?),
            KIND_SHARED_GEAR if !is_food_generated_shared_gear_payload(&record.payload) => {
                shared_gear_demands.push(record_typed(record)?);
            }
            KIND_SHARED_GEAR => {}
            KIND_ITINERARY_DAY => itinerary_days.push(record_typed(record)?),
            KIND_TIME_SLOT => time_slots.push(record_typed(record)?),
            KIND_ROUTE_SEGMENT => route_segments.push(record_typed(record)?),
            KIND_FOOD_MEAL => food_meals.push(record_typed(record)?),
            KIND_FOOD_ITEM => food_items.push(record_typed(record)?),
            KIND_FOOD_SUPPLY => food_supplies.push(record_typed(record)?),
            KIND_MEDICAL_ITEM => medical_items.push(record_typed(record)?),
            KIND_SEGMENT_ASSIGNMENT => segment_assignments.push(record_typed(record)?),
            KIND_SAFETY_RISK => safety_risks.push(record_typed(record)?),
            KIND_RESCUE_CONTACT => rescue_contacts.push(record_typed(record)?),
            KIND_BUDGET_ITEM => budget_items.push(record_typed(record)?),
            KIND_GOAL_ITEM => goals.push(record_typed(record)?),
            _ => {}
        }
    }

    let route_estimates = route_segments
        .iter()
        .map(|segment| (segment.id.clone(), segment.final_estimate_minutes))
        .collect::<BTreeMap<_, _>>();
    for day in &mut itinerary_days {
        day.time_slots = time_slots
            .iter()
            .filter(|slot| slot.day_id == day.id)
            .cloned()
            .collect();
        day.estimate_minutes = day
            .time_slots
            .iter()
            .filter_map(|slot| slot.route_segment_id.as_ref())
            .filter_map(|id| route_estimates.get(id))
            .sum();
    }
    for meal in &mut food_meals {
        meal.items = food_items
            .iter()
            .filter(|item| item.food_meal_id == meal.id)
            .cloned()
            .collect();
    }
    let shared_gear_by_id = shared_gear_demands
        .iter()
        .map(|item| (item.id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    for item in &mut budget_items {
        let Some(shared_id) = item.linked_shared_gear_id.as_deref() else {
            continue;
        };
        if let Some(shared) = shared_gear_by_id.get(shared_id) {
            item.linked_shared_gear_deleted = false;
            item.linked_shared_gear_name = Some(
                shared
                    .concrete_name
                    .as_ref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(&shared.name)
                    .clone(),
            );
            item.linked_shared_gear_responsible_member_id =
                Some(shared.responsible_member_id.clone());
        } else {
            item.linked_shared_gear_deleted = true;
        }
    }
    let (weight_summaries, member_gear_views) =
        build_member_gear_views(&members, &personal_gear, &shared_gear_demands);
    let sections = plan.enabled_sections.clone();
    Ok(TripDetail {
        trip: plan,
        sections,
        my_member_id,
        members,
        personal_gear,
        shared_gear_demands,
        itinerary_days,
        route_segments,
        food_meals,
        food_supplies,
        medical_items,
        segment_assignments,
        safety_risks,
        rescue_contacts,
        budget_items,
        goals,
        weight_summaries,
        member_gear_views,
    })
}

fn build_member_gear_views(
    members: &[TripMember],
    personal: &[TripPersonalGearItem],
    shared: &[TripSharedGearDemand],
) -> (Vec<TripMemberGearWeightSummary>, Vec<TripMemberGearView>) {
    let mut summaries = Vec::new();
    let mut views = Vec::new();
    for member in members {
        let mut all_weight_g = 0_i64;
        let mut actual_weight_g = 0_i64;
        let mut items = Vec::new();
        for item in personal.iter().filter(|item| item.member_id == member.id) {
            let planned =
                item.planned_quantity.max(0) as i64 * item.unit_weight_g.unwrap_or(0) as i64;
            let actual =
                item.packed_quantity.max(0) as i64 * item.unit_weight_g.unwrap_or(0) as i64;
            all_weight_g += planned;
            actual_weight_g += actual;
            items.push(TripMemberGearViewItem {
                id: item.id.clone(),
                source: "personal".to_owned(),
                name: item.name.clone(),
                category: item.category,
                category_label: item.category_label.clone(),
                planned_quantity: item.planned_quantity,
                packed_quantity: item.packed_quantity,
                unit_weight_g: item.unit_weight_g,
                labels: vec!["个人装备".to_owned()],
                counts_weight: true,
            });
        }
        for item in shared {
            if !is_bound_shared_gear(item) {
                continue;
            }
            let mut labels = vec!["公共装备".to_owned()];
            if item
                .source_member_id
                .as_deref()
                .is_some_and(|id| id != member.id)
            {
                labels.push("非本人装备".to_owned());
            }
            let counts_weight = item.responsible_member_id == member.id;
            labels.push(if counts_weight {
                "我负责".to_owned()
            } else {
                "他人负责".to_owned()
            });
            if counts_weight {
                all_weight_g +=
                    item.planned_quantity.max(0) as i64 * item.unit_weight_g.unwrap_or(0) as i64;
                actual_weight_g +=
                    item.packed_quantity.max(0) as i64 * item.unit_weight_g.unwrap_or(0) as i64;
            }
            items.push(TripMemberGearViewItem {
                id: item.id.clone(),
                source: "shared".to_owned(),
                name: item
                    .concrete_name
                    .as_ref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(&item.name)
                    .clone(),
                category: item.category,
                category_label: item.category_label.clone(),
                planned_quantity: item.planned_quantity,
                packed_quantity: item.packed_quantity,
                unit_weight_g: item.unit_weight_g,
                labels,
                counts_weight,
            });
        }
        summaries.push(TripMemberGearWeightSummary {
            member_id: member.id.clone(),
            all_weight_g,
            actual_weight_g,
        });
        views.push(TripMemberGearView {
            member_id: member.id.clone(),
            all_weight_g,
            actual_weight_g,
            items,
        });
    }
    (summaries, views)
}

fn is_bound_shared_gear(item: &TripSharedGearDemand) -> bool {
    item.concrete_name
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || item.source_gear_id.is_some()
        || (item.template_key.is_none() && item.demand_name.is_none())
}

fn record_typed<T: serde::de::DeserializeOwned>(record: TripRecordRow) -> Result<T, DbErr> {
    let mut payload = match record.payload {
        JsonValue::Object(map) => map,
        _ => JsonMap::new(),
    };
    payload.insert("id".to_owned(), JsonValue::String(record.id));
    payload.insert(
        "field_versions".to_owned(),
        serde_json::to_value(record.field_versions).map_err(json_db_error)?,
    );
    payload.insert(
        "created_by_user_id".to_owned(),
        record
            .created_by_user_id
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    );
    payload.insert(
        "created_at".to_owned(),
        JsonValue::String(record.created_at),
    );
    payload.insert(
        "updated_at".to_owned(),
        JsonValue::String(record.updated_at),
    );
    serde_json::from_value(JsonValue::Object(payload)).map_err(json_db_error)
}

fn apply_field_patch(
    mut object: JsonValue,
    mut versions: FieldVersions,
    changes: BTreeMap<String, JsonValue>,
    base_versions: &FieldVersions,
    force_fields: &BTreeSet<String>,
) -> Result<(JsonValue, FieldVersions), TripRepositoryError> {
    let Some(map) = object.as_object_mut() else {
        return Err(DbErr::Custom("editable payload must be an object".to_owned()).into());
    };
    let mut conflicts = Vec::new();
    for (field, client_value) in &changes {
        let current_version = *versions.get(field).unwrap_or(&0);
        let base_version = base_versions.get(field).copied().unwrap_or(0);
        if base_version != current_version && !force_fields.contains(field) {
            conflicts.push(FieldConflict {
                field: field.clone(),
                client_value: client_value.clone(),
                server_value: map.get(field).cloned().unwrap_or(JsonValue::Null),
                server_version: current_version,
            });
        }
    }
    if !conflicts.is_empty() {
        return Err(TripRepositoryError::Conflict(conflicts));
    }
    for (field, value) in changes {
        map.insert(field.clone(), value);
        let next = versions.get(&field).copied().unwrap_or(0) + 1;
        versions.insert(field, next);
    }
    Ok((object, versions))
}

fn validate_plan_route_settings(value: &JsonValue) -> Result<(), TripRepositoryError> {
    let mut errors: Vec<FieldViolation> = Vec::new();
    validate_route_estimation_settings(
        required_bool(value, "route_use_high_altitude_adjustment")?,
        optional_i32(value, "route_start_altitude_m"),
        &mut errors,
    );
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::new(errors).into())
    }
}

fn route_order_may_change(kind: &str) -> bool {
    matches!(
        kind,
        KIND_ITINERARY_DAY | KIND_TIME_SLOT | KIND_ROUTE_SEGMENT
    )
}

fn ordered_assigned_route_ids(records: &[TripRecordRow]) -> Vec<String> {
    let mut days = records
        .iter()
        .filter(|record| record.kind == KIND_ITINERARY_DAY)
        .collect::<Vec<_>>();
    days.sort_by_key(|record| {
        (
            json_i32(&record.payload, "day_index"),
            record.sort_order,
            record.created_at.as_str(),
            record.id.as_str(),
        )
    });

    let mut route_ids = Vec::new();
    for day in days {
        let mut slots = records
            .iter()
            .filter(|record| {
                record.kind == KIND_TIME_SLOT
                    && record
                        .payload
                        .get("day_id")
                        .and_then(JsonValue::as_str)
                        .is_some_and(|id| id == day.id)
            })
            .collect::<Vec<_>>();
        slots.sort_by_key(|record| {
            (
                record.sort_order,
                slot_key_order(
                    record
                        .payload
                        .get("slot_key")
                        .and_then(JsonValue::as_str)
                        .unwrap_or_default(),
                ),
                record.created_at.as_str(),
                record.id.as_str(),
            )
        });
        route_ids.extend(slots.into_iter().filter_map(|slot| {
            slot.payload
                .get("route_segment_id")
                .and_then(JsonValue::as_str)
                .map(str::to_owned)
        }));
    }
    route_ids
}

fn slot_key_order(value: &str) -> i32 {
    match value {
        "morning" => 0,
        "afternoon" => 1,
        "evening" => 2,
        "route" => 3,
        _ => 10,
    }
}

fn recalculate_route_payload(
    payload: &mut JsonValue,
    plan: &Trip,
    altitude: Option<RouteAltitudeEstimate>,
) -> Result<(), DbErr> {
    let distance_km = payload
        .get("distance_km")
        .and_then(JsonValue::as_f64)
        .unwrap_or(0.0);
    let ascent_m = json_i32(payload, "ascent_m");
    let descent_m = json_i32(payload, "descent_m");
    let descent_profile = payload
        .get("descent_profile")
        .and_then(JsonValue::as_str)
        .unwrap_or("none");
    let technical_factor = payload
        .get("technical_factor")
        .and_then(JsonValue::as_f64)
        .unwrap_or(1.0);
    let rest_factor = payload
        .get("rest_factor")
        .and_then(JsonValue::as_f64)
        .unwrap_or(1.0);
    let pack_factor = payload
        .get("pack_factor")
        .and_then(JsonValue::as_f64)
        .unwrap_or(1.0);
    let manual = payload
        .get("manual_estimate_minutes")
        .and_then(JsonValue::as_i64)
        .map(|value| value as i32);
    let altitude_factor = altitude
        .map(|estimate| high_altitude_factor(estimate.highest_m))
        .filter(|_| plan.route_use_high_altitude_adjustment)
        .unwrap_or(1.0);
    let (formula, final_minutes) = estimate_route_minutes(
        distance_km,
        ascent_m,
        descent_m,
        descent_profile,
        plan.route_use_slope_adjustment,
        altitude_factor,
        technical_factor,
        rest_factor,
        pack_factor,
        manual,
    );
    let Some(map) = payload.as_object_mut() else {
        return Err(DbErr::Custom("route payload must be an object".to_owned()));
    };
    map.insert("formula_estimate_minutes".to_owned(), json!(formula));
    map.insert("final_estimate_minutes".to_owned(), json!(final_minutes));
    if let Some(altitude) = altitude.filter(|_| plan.route_use_high_altitude_adjustment) {
        map.insert(
            "estimated_start_altitude_m".to_owned(),
            json!(altitude.start_m),
        );
        map.insert("estimated_end_altitude_m".to_owned(), json!(altitude.end_m));
        map.insert(
            "estimated_highest_altitude_m".to_owned(),
            json!(altitude.highest_m),
        );
        map.insert("high_altitude_factor".to_owned(), json!(altitude_factor));
    } else {
        map.insert("estimated_start_altitude_m".to_owned(), JsonValue::Null);
        map.insert("estimated_end_altitude_m".to_owned(), JsonValue::Null);
        map.insert("estimated_highest_altitude_m".to_owned(), JsonValue::Null);
        map.insert("high_altitude_factor".to_owned(), json!(1.0));
    }
    Ok(())
}

fn record_table(kind: &str) -> Result<&'static str, DbErr> {
    TRIP_RECORD_TABLES
        .iter()
        .find_map(|(candidate, table)| (*candidate == kind).then_some(*table))
        .ok_or_else(|| DbErr::Custom(format!("unknown trip record kind: {kind}")))
}

fn trip_type_key(trip_type: TripType) -> &'static str {
    match trip_type {
        TripType::Solo => "solo",
        TripType::Team => "team",
    }
}

fn parse_trip_type(value: &str) -> Result<TripType, DbErr> {
    match value {
        "solo" => Ok(TripType::Solo),
        "team" => Ok(TripType::Team),
        other => Err(DbErr::Custom(format!("invalid trip_type: {other}"))),
    }
}

fn json_i32(value: &JsonValue, field: &str) -> i32 {
    value
        .get(field)
        .and_then(JsonValue::as_i64)
        .unwrap_or_default() as i32
}

#[derive(Clone, Debug)]
struct HomeHighlightCandidate {
    plan: Trip,
    today: Date,
    start_date: Date,
    end_date: Date,
}

impl HomeHighlightCandidate {
    fn from_plan(plan: Trip, today: Date) -> Option<Self> {
        let start_date = parse_trip_date(plan.start_date.as_deref())?;
        let end_date = match plan.end_date.as_deref() {
            Some(value) => parse_trip_date(Some(value))?,
            None => start_date,
        };
        if end_date < start_date {
            return None;
        }
        Some(Self {
            plan,
            today,
            start_date,
            end_date,
        })
    }

    fn is_ongoing(&self) -> bool {
        self.start_date <= self.today && self.today <= self.end_date
    }

    fn is_upcoming(&self) -> bool {
        self.start_date > self.today
    }

    fn ongoing_rank_key(&self) -> (Date, Date, &str) {
        (self.end_date, self.start_date, self.plan.id.as_str())
    }

    fn upcoming_rank_key(&self) -> (Date, &str) {
        (self.start_date, self.plan.id.as_str())
    }
}

impl TripHomeHighlight {
    fn from_ongoing(candidate: HomeHighlightCandidate) -> Self {
        Self {
            days_until_start: (candidate.start_date - candidate.today).whole_days(),
            days_until_end: (candidate.end_date - candidate.today).whole_days(),
            trip: trip_summary_from_trip(candidate.plan, candidate.today, 0, None),
            status: TripHighlightStatus::Ongoing,
        }
    }

    fn from_upcoming(candidate: HomeHighlightCandidate) -> Self {
        Self {
            days_until_start: (candidate.start_date - candidate.today).whole_days(),
            days_until_end: (candidate.end_date - candidate.today).whole_days(),
            trip: trip_summary_from_trip(candidate.plan, candidate.today, 0, None),
            status: TripHighlightStatus::Upcoming,
        }
    }
}

fn parse_trip_date(value: Option<&str>) -> Option<Date> {
    Date::parse(
        value?,
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .ok()
}

fn map_plan(row: &QueryResult) -> Result<Trip, DbErr> {
    let sections_json: String = row.try_get("", "enabled_sections_json")?;
    let versions_json: String = row.try_get("", "field_versions_json")?;
    let trip_type_raw: String = row.try_get("", "trip_type")?;
    Ok(Trip {
        id: row.try_get("", "id")?,
        owner_user_id: row.try_get("", "owner_user_id")?,
        trip_type: parse_trip_type(&trip_type_raw)?,
        title: row.try_get("", "title")?,
        description: row.try_get("", "description")?,
        start_date: row.try_get("", "start_date")?,
        end_date: row.try_get("", "end_date")?,
        enabled_sections: serde_json::from_str(&sections_json)
            .unwrap_or_else(|_| TripSectionKey::DEFAULT.into_iter().collect::<Vec<_>>()),
        route_use_slope_adjustment: row.try_get("", "route_use_slope_adjustment")?,
        route_use_high_altitude_adjustment: row
            .try_get("", "route_use_high_altitude_adjustment")?,
        route_start_altitude_m: row.try_get("", "route_start_altitude_m")?,
        day_count: row.try_get("", "day_count")?,
        field_versions: serde_json::from_str(&versions_json).unwrap_or_default(),
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_trip_summary(row: &QueryResult, today: Date) -> Result<TripSummary, DbErr> {
    let trip = map_plan(row)?;
    let member_count = row.try_get("", "member_count").unwrap_or(0);
    let outdoor_experience_id = row.try_get("", "outdoor_experience_id").ok();
    Ok(trip_summary_from_trip(
        trip,
        today,
        member_count,
        outdoor_experience_id,
    ))
}

fn trip_summary_from_trip(
    trip: Trip,
    today: Date,
    member_count: i64,
    outdoor_experience_id: Option<String>,
) -> TripSummary {
    let (time_bucket, days_until_start, days_until_end) = trip_time_context(&trip, today);
    let readiness = trip_readiness(&trip);
    TripSummary {
        id: trip.id,
        owner_user_id: trip.owner_user_id,
        trip_type: trip.trip_type,
        title: trip.title,
        description: trip.description,
        start_date: trip.start_date,
        end_date: trip.end_date,
        enabled_sections: trip.enabled_sections,
        route_use_slope_adjustment: trip.route_use_slope_adjustment,
        route_use_high_altitude_adjustment: trip.route_use_high_altitude_adjustment,
        route_start_altitude_m: trip.route_start_altitude_m,
        day_count: trip.day_count,
        time_bucket,
        days_until_start,
        days_until_end,
        member_count,
        readiness,
        outdoor_experience_id,
        field_versions: trip.field_versions,
        is_deleted: trip.is_deleted,
        created_at: trip.created_at,
        updated_at: trip.updated_at,
    }
}

fn trip_time_context(trip: &Trip, today: Date) -> (TripTimeBucket, Option<i64>, Option<i64>) {
    let Some(start_date) = parse_trip_date(trip.start_date.as_deref()) else {
        return (TripTimeBucket::Undated, None, None);
    };
    let end_date = trip
        .end_date
        .as_deref()
        .and_then(|value| parse_trip_date(Some(value)))
        .unwrap_or(start_date);
    if today < start_date {
        (
            TripTimeBucket::Upcoming,
            Some((start_date - today).whole_days()),
            Some((end_date - today).whole_days()),
        )
    } else if today <= end_date {
        (
            TripTimeBucket::Ongoing,
            Some((start_date - today).whole_days()),
            Some((end_date - today).whole_days()),
        )
    } else {
        (
            TripTimeBucket::Past,
            Some((start_date - today).whole_days()),
            Some((end_date - today).whole_days()),
        )
    }
}

fn trip_readiness(trip: &Trip) -> TripReadiness {
    let mut missing_labels = Vec::new();
    if trip.start_date.is_none() || trip.end_date.is_none() {
        missing_labels.push("日期".to_owned());
    }
    if trip.day_count == 0 {
        missing_labels.push("行程安排".to_owned());
    }
    let total = 2_i32;
    let missing_count = missing_labels.len() as i32;
    TripReadiness {
        missing_count,
        missing_labels,
        completion_percent: ((total - missing_count).max(0) * 100) / total,
    }
}

fn map_member(row: &QueryResult) -> Result<TripMember, DbErr> {
    let profile_json: String = row.try_get("", "profile_json")?;
    let versions_json: String = row.try_get("", "field_versions_json")?;
    let owner_user_id: String = row.try_get("", "owner_user_id")?;
    let user_id: String = row.try_get("", "user_id")?;
    Ok(TripMember {
        id: row.try_get("", "id")?,
        trip_id: row.try_get("", "trip_id")?,
        user_id: user_id.clone(),
        is_owner: user_id == owner_user_id,
        profile: serde_json::from_str(&profile_json).map_err(json_db_error)?,
        field_versions: serde_json::from_str(&versions_json).unwrap_or_default(),
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_invitation(row: &QueryResult) -> Result<TripInvitation, DbErr> {
    Ok(TripInvitation {
        id: row.try_get("", "id")?,
        trip_id: row.try_get("", "trip_id")?,
        token: row.try_get("", "token")?,
        created_by_user_id: row.try_get("", "created_by_user_id")?,
        revoked_at: row.try_get("", "revoked_at")?,
        created_at: row.try_get("", "created_at")?,
    })
}

fn map_record_row(row: &QueryResult, kind: &str) -> Result<TripRecordRow, DbErr> {
    let payload_json: String = row.try_get("", "payload_json")?;
    let versions_json: String = row.try_get("", "field_versions_json")?;
    Ok(TripRecordRow {
        id: row.try_get("", "id")?,
        kind: kind.to_owned(),
        sort_order: row.try_get("", "sort_order")?,
        payload: serde_json::from_str(&payload_json).map_err(json_db_error)?,
        field_versions: serde_json::from_str(&versions_json).unwrap_or_default(),
        created_by_user_id: row.try_get("", "created_by_user_id")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_shared_gear_template(row: &QueryResult) -> Result<SharedGearDemandTemplate, DbErr> {
    let category_raw: String = row.try_get("", "category")?;
    let category = GearCategory::from_key(&category_raw).ok_or_else(|| {
        DbErr::Custom(format!(
            "invalid shared gear template category: {category_raw}"
        ))
    })?;
    let category_label: String = row.try_get("", "category_label")?;
    Ok(SharedGearDemandTemplate {
        template_key: row.try_get("", "template_key")?,
        demand_name: row.try_get("", "demand_name")?,
        group_label: row.try_get("", "group_label")?,
        category,
        category_label: if category_label.trim().is_empty() {
            category.label().to_owned()
        } else {
            category_label
        },
        planned_quantity: row.try_get("", "planned_quantity")?,
        sort_order: row.try_get("", "sort_order")?,
    })
}

fn map_outdoor_experience(row: &QueryResult) -> Result<OutdoorExperience, DbErr> {
    let trip_type_raw: String = row.try_get("", "trip_type")?;
    let snapshot_json: String = row.try_get("", "snapshot_json")?;
    let versions_json: String = row.try_get("", "field_versions_json")?;
    Ok(OutdoorExperience {
        id: row.try_get("", "id")?,
        user_id: row.try_get("", "user_id")?,
        source_trip_id: row.try_get("", "source_trip_id")?,
        trip_type: parse_trip_type(&trip_type_raw)?,
        title: row.try_get("", "title")?,
        start_date: row.try_get("", "start_date")?,
        end_date: row.try_get("", "end_date")?,
        day_count: row.try_get("", "day_count")?,
        companion_count: row.try_get("", "companion_count")?,
        route_summary: row.try_get("", "route_summary")?,
        gear_summary: row.try_get("", "gear_summary")?,
        food_summary: row.try_get("", "food_summary")?,
        budget_summary: row.try_get("", "budget_summary")?,
        notes: row.try_get("", "notes")?,
        snapshot: serde_json::from_str(&snapshot_json).map_err(json_db_error)?,
        field_versions: serde_json::from_str(&versions_json).unwrap_or_default(),
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn inclusive_trip_days(start_date: Option<&str>, end_date: Option<&str>) -> Option<i64> {
    let start = parse_trip_date(start_date)?;
    let end = end_date
        .and_then(|value| parse_trip_date(Some(value)))
        .unwrap_or(start);
    if end < start {
        None
    } else {
        Some((end - start).whole_days() + 1)
    }
}

fn display_name_for_user(user: &UserRecord) -> String {
    user.nickname
        .as_deref()
        .or(user.username.as_deref())
        .or(user.email.as_deref())
        .unwrap_or("队员")
        .to_owned()
}

fn initial_versions<const N: usize>(fields: [&str; N]) -> FieldVersions {
    fields
        .into_iter()
        .map(|field| (field.to_owned(), 1))
        .collect()
}

fn versions_for_payload(payload: &JsonValue) -> FieldVersions {
    payload
        .as_object()
        .map(|map| map.keys().map(|key| (key.clone(), 1)).collect())
        .unwrap_or_default()
}

fn required_string(value: &JsonValue, field: &str) -> Result<String, DbErr> {
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DbErr::Custom(format!("{field} is required")))
}

fn required_bool(value: &JsonValue, field: &str) -> Result<bool, DbErr> {
    value
        .get(field)
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| DbErr::Custom(format!("{field} is required")))
}

fn optional_string(value: &JsonValue, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .map(str::to_owned)
}

fn optional_i32(value: &JsonValue, field: &str) -> Option<i32> {
    value
        .get(field)
        .and_then(JsonValue::as_i64)
        .map(|value| value as i32)
}

fn is_food_generated_shared_gear_payload(payload: &JsonValue) -> bool {
    ["source_food_item_id", "source_food_supply_id"]
        .iter()
        .any(|field| {
            payload
                .get(field)
                .and_then(JsonValue::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        })
}

fn json_string(value: &impl serde::Serialize) -> Result<String, DbErr> {
    serde_json::to_string(value).map_err(json_db_error)
}

fn json_db_error(error: impl ToString) -> DbErr {
    DbErr::Custom(error.to_string())
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
