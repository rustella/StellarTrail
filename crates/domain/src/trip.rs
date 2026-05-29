//! Trip domain models for solo and collaborative outdoor preparation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    gear::GearCategory,
    outdoor_profile::{normalize_common_profile_fields, normalize_display_name},
    validation::{
        FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
    },
};

/// Per-field optimistic concurrency versions returned with editable records.
pub type FieldVersions = BTreeMap<String, i64>;

/// Lowest accepted plan start altitude for route estimation.
pub const ROUTE_START_ALTITUDE_MIN_M: i32 = -500;
/// Highest accepted plan start altitude for route estimation.
pub const ROUTE_START_ALTITUDE_MAX_M: i32 = 9_000;

/// Field-level optimistic concurrency conflict returned to API clients.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FieldConflict {
    pub field: String,
    pub client_value: JsonValue,
    pub server_value: JsonValue,
    pub server_version: i64,
}

/// Whether a trip is personal preparation or a collaborative team trip.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TripType {
    Solo,
    #[default]
    Team,
}

/// Date-derived bucket used by the trip home list.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TripTimeBucket {
    Ongoing,
    Upcoming,
    Past,
    Undated,
}

/// Lightweight readiness summary for trip cards.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TripReadiness {
    pub missing_count: i32,
    pub missing_labels: Vec<String>,
    pub completion_percent: i32,
}

/// Client-controlled editable sections in a trip.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TripSectionKey {
    Members,
    PersonalGear,
    Itinerary,
    SharedGear,
    FoodPlan,
    MedicalKit,
    SafetyPlan,
    RescueInfo,
    Budget,
    Goals,
}

impl TripSectionKey {
    pub const DEFAULT: [Self; 2] = [Self::Members, Self::PersonalGear];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Members => "members",
            Self::PersonalGear => "personal_gear",
            Self::Itinerary => "itinerary",
            Self::SharedGear => "shared_gear",
            Self::FoodPlan => "food_plan",
            Self::MedicalKit => "medical_kit",
            Self::SafetyPlan => "safety_plan",
            Self::RescueInfo => "rescue_info",
            Self::Budget => "budget",
            Self::Goals => "goals",
        }
    }
}

/// Root trip metadata used in detail responses.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trip {
    pub id: String,
    pub owner_user_id: String,
    pub trip_type: TripType,
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub enabled_sections: Vec<TripSectionKey>,
    #[serde(default)]
    pub route_use_slope_adjustment: bool,
    #[serde(default)]
    pub route_use_high_altitude_adjustment: bool,
    pub route_start_altitude_m: Option<i32>,
    pub day_count: i64,
    pub field_versions: FieldVersions,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary used by the trip home list.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripSummary {
    pub id: String,
    pub owner_user_id: String,
    pub trip_type: TripType,
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub enabled_sections: Vec<TripSectionKey>,
    pub route_use_slope_adjustment: bool,
    pub route_use_high_altitude_adjustment: bool,
    pub route_start_altitude_m: Option<i32>,
    pub day_count: i64,
    pub time_bucket: TripTimeBucket,
    pub days_until_start: Option<i64>,
    pub days_until_end: Option<i64>,
    pub member_count: i64,
    pub readiness: TripReadiness,
    pub outdoor_experience_id: Option<String>,
    pub field_versions: FieldVersions,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Writable metadata supplied when creating a trip.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripDraft {
    pub trip_type: TripType,
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub route_use_slope_adjustment: bool,
    pub route_use_high_altitude_adjustment: bool,
    pub route_start_altitude_m: Option<i32>,
}

impl TripDraft {
    /// Validates and trims trip metadata before persistence.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.title =
            normalize_required_text(std::mem::take(&mut self.title), 100, "title", &mut errors);
        self.description =
            normalize_optional_text(self.description.take(), 1000, "description", &mut errors);
        self.start_date =
            normalize_optional_text(self.start_date.take(), 30, "start_date", &mut errors);
        self.end_date = normalize_optional_text(self.end_date.take(), 30, "end_date", &mut errors);
        validate_route_estimation_settings(
            self.route_use_high_altitude_adjustment,
            self.route_start_altitude_m,
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Member profile kept inside one trip.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TripMemberProfile {
    pub display_name: String,
    pub outdoor_id: Option<String>,
    pub real_name: Option<String>,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub height_cm: Option<i32>,
    pub phone: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub emergency_phone: Option<String>,
    pub blood_type: Option<String>,
    pub medical_history: Option<String>,
    pub allergy_history: Option<String>,
    pub medical_response_note: Option<String>,
    pub diet_preference: Option<String>,
    pub insurance_policy_no: Option<String>,
    pub insurance_company_phone: Option<String>,
    pub experience_note: Option<String>,
    pub role_label: Option<String>,
}

impl TripMemberProfile {
    /// Normalizes editable member fields before they are persisted in one trip.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.display_name =
            normalize_display_name(std::mem::take(&mut self.display_name), &mut errors);
        let mut ignored_birth_date = None;
        normalize_common_profile_fields(
            &mut self.outdoor_id,
            &mut self.real_name,
            &mut self.gender,
            &mut ignored_birth_date,
            &mut self.height_cm,
            &mut self.phone,
            &mut self.emergency_contact,
            &mut self.emergency_contact_relationship,
            &mut self.emergency_phone,
            &mut self.blood_type,
            &mut self.medical_history,
            &mut self.allergy_history,
            &mut self.medical_response_note,
            &mut self.diet_preference,
            &mut self.insurance_policy_no,
            &mut self.insurance_company_phone,
            &mut self.experience_note,
            &mut errors,
        );
        if self.age.is_some_and(|age| !matches!(age, 0..=120)) {
            errors.push(FieldViolation::new("age", "must be between 0 and 120"));
        }
        self.role_label =
            normalize_optional_text(self.role_label.take(), 80, "role_label", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Persisted member row with account binding and editable profile fields.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripMember {
    pub id: String,
    pub trip_id: String,
    pub user_id: String,
    pub is_owner: bool,
    pub profile: TripMemberProfile,
    pub field_versions: FieldVersions,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Personal gear snapshot imported into the trip for one member.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripPersonalGearItem {
    pub id: String,
    pub member_id: String,
    pub source_packing_list_id: Option<String>,
    pub source_packing_item_id: Option<String>,
    pub source_gear_id: Option<String>,
    pub category: GearCategory,
    pub category_label: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub planned_quantity: i32,
    pub packed_quantity: i32,
    pub unit_weight_g: Option<i32>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Public gear demand assigned to one responsible member.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripSharedGearDemand {
    pub id: String,
    pub source_member_id: Option<String>,
    pub source_gear_id: Option<String>,
    pub responsible_member_id: String,
    pub created_by_user_id: Option<String>,
    pub template_key: Option<String>,
    pub demand_name: Option<String>,
    pub concrete_name: Option<String>,
    pub category: GearCategory,
    pub category_label: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub planned_quantity: i32,
    pub packed_quantity: i32,
    pub unit_weight_g: Option<i32>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Backend-owned common shared-gear demand template.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SharedGearDemandTemplate {
    pub template_key: String,
    pub demand_name: String,
    pub group_label: String,
    pub category: GearCategory,
    pub category_label: String,
    pub planned_quantity: i32,
    pub sort_order: i32,
}

/// Time dimension day in the itinerary.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripItineraryDay {
    pub id: String,
    pub day_index: i32,
    pub date_label: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub weather: Option<String>,
    pub high_temperature_c: Option<i32>,
    pub low_temperature_c: Option<i32>,
    pub weather_summary: Option<String>,
    pub weather_notes: Option<String>,
    pub camp_name: Option<String>,
    pub camp_altitude_m: Option<i32>,
    pub camp_terrain: Option<String>,
    pub camp_slope: Option<String>,
    pub camp_area: Option<String>,
    pub camp_water_source: Option<String>,
    pub camp_notes: Option<String>,
    #[serde(default)]
    pub estimate_minutes: i32,
    #[serde(default)]
    pub time_slots: Vec<TripItineraryTimeSlot>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Optional morning, afternoon, or evening itinerary time slot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripItineraryTimeSlot {
    pub id: String,
    pub day_id: String,
    pub slot_key: String,
    pub route_segment_id: Option<String>,
    pub route_description: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Route segment estimate input and output.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripRouteSegment {
    pub id: String,
    pub name: String,
    pub start_point: Option<String>,
    pub end_point: Option<String>,
    pub checkpoint: Option<String>,
    pub leader_member_id: Option<String>,
    pub bailout_route: Option<String>,
    pub trail_condition: Option<String>,
    pub distance_km: f64,
    pub ascent_m: i32,
    pub descent_m: i32,
    pub descent_profile: String,
    pub technical_factor: f64,
    pub rest_factor: f64,
    pub pack_factor: f64,
    pub formula_estimate_minutes: i32,
    pub final_estimate_minutes: i32,
    pub manual_estimate_minutes: Option<i32>,
    pub estimated_start_altitude_m: Option<i32>,
    pub estimated_end_altitude_m: Option<i32>,
    pub estimated_highest_altitude_m: Option<i32>,
    pub high_altitude_factor: Option<f64>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Food meal generated from an itinerary day.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripFoodMeal {
    pub id: String,
    pub itinerary_day_id: String,
    pub meal_key: String,
    pub meal_type: Option<String>,
    pub skipped: bool,
    pub dish_name: Option<String>,
    pub responsible_member_id: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub items: Vec<TripFoodItem>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Food ingredient row inside one generated meal.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripFoodItem {
    pub id: String,
    pub food_meal_id: String,
    pub name: String,
    pub amount_g: Option<i32>,
    pub per_person_amount_g: Option<i32>,
    pub total_price_cents: Option<i64>,
    pub responsible_member_id: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Public food supply or seasoning shared across meals.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripFoodSupply {
    pub id: String,
    pub name: String,
    pub supply_type: Option<String>,
    pub amount_g: Option<i32>,
    pub per_person_amount_g: Option<i32>,
    pub total_price_cents: Option<i64>,
    pub responsible_member_id: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Medical kit row tracked for the whole team.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripMedicalItem {
    pub id: String,
    pub name: String,
    pub item_type: Option<String>,
    pub scope: Option<String>,
    pub suggested_quantity: Option<i32>,
    pub required_quantity: i32,
    pub packed_quantity: i32,
    pub responsible_member_id: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Role assignment for one route segment or checkpoint.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripSegmentAssignment {
    pub id: String,
    pub route_segment_id: Option<String>,
    pub checkpoint: Option<String>,
    pub leader_record_member_id: Option<String>,
    pub navigator_safety_member_id: Option<String>,
    pub collaborator_member_id: Option<String>,
    pub photographer_member_id: Option<String>,
    pub safety_member_id: Option<String>,
    pub environment_member_id: Option<String>,
    pub sweeper_member_id: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Safety plan row for risks on a day or route segment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripSafetyRisk {
    pub id: String,
    pub risk_type: String,
    pub prevention: Option<String>,
    pub response: Option<String>,
    pub responsible_member_id: Option<String>,
    pub itinerary_day_id: Option<String>,
    pub route_segment_id: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Rescue or emergency contact information around the route.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripRescueContact {
    pub id: String,
    pub organization: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Budget item for optional trip finance tracking.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripBudgetItem {
    pub id: String,
    pub category: Option<String>,
    pub name: String,
    pub quantity: i32,
    pub unit_price_cents: Option<i64>,
    pub total_price_cents: Option<i64>,
    pub split_member_count: Option<i32>,
    pub notes: Option<String>,
    pub linked_shared_gear_id: Option<String>,
    #[serde(default)]
    pub linked_shared_gear_deleted: bool,
    pub linked_shared_gear_name: Option<String>,
    pub linked_shared_gear_responsible_member_id: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Team or personal goal recorded inside one plan.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripGoalItem {
    pub id: String,
    pub scope: String,
    pub member_id: Option<String>,
    pub content: String,
    pub notes: Option<String>,
    pub field_versions: FieldVersions,
    pub created_at: String,
    pub updated_at: String,
}

/// Weight summary for one member's combined personal and assigned shared gear.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TripMemberGearWeightSummary {
    pub member_id: String,
    pub all_weight_g: i64,
    pub actual_weight_g: i64,
}

/// One row in a member-centric gear view.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripMemberGearViewItem {
    pub id: String,
    pub source: String,
    pub name: String,
    pub category: GearCategory,
    pub category_label: String,
    pub planned_quantity: i32,
    pub packed_quantity: i32,
    pub unit_weight_g: Option<i32>,
    pub labels: Vec<String>,
    pub counts_weight: bool,
}

/// Member-centric view that merges personal and shared gear rows.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TripMemberGearView {
    pub member_id: String,
    pub all_weight_g: i64,
    pub actual_weight_g: i64,
    pub items: Vec<TripMemberGearViewItem>,
}

/// Full trip detail returned by API reads.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripDetail {
    pub trip: Trip,
    pub sections: Vec<TripSectionKey>,
    pub my_member_id: String,
    pub members: Vec<TripMember>,
    pub personal_gear: Vec<TripPersonalGearItem>,
    pub shared_gear_demands: Vec<TripSharedGearDemand>,
    pub itinerary_days: Vec<TripItineraryDay>,
    pub route_segments: Vec<TripRouteSegment>,
    pub food_meals: Vec<TripFoodMeal>,
    pub food_supplies: Vec<TripFoodSupply>,
    pub medical_items: Vec<TripMedicalItem>,
    pub segment_assignments: Vec<TripSegmentAssignment>,
    pub safety_risks: Vec<TripSafetyRisk>,
    pub rescue_contacts: Vec<TripRescueContact>,
    pub budget_items: Vec<TripBudgetItem>,
    pub goals: Vec<TripGoalItem>,
    pub weight_summaries: Vec<TripMemberGearWeightSummary>,
    pub member_gear_views: Vec<TripMemberGearView>,
}

/// Structured outdoor experience shown on the profile page.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutdoorExperience {
    pub id: String,
    pub user_id: String,
    pub source_trip_id: Option<String>,
    pub trip_type: TripType,
    pub title: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub day_count: i64,
    pub companion_count: i64,
    pub route_summary: Option<String>,
    pub gear_summary: Option<String>,
    pub food_summary: Option<String>,
    pub budget_summary: Option<String>,
    pub notes: Option<String>,
    pub snapshot: JsonValue,
    pub field_versions: FieldVersions,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Writable fields for manually recorded outdoor experiences.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OutdoorExperienceDraft {
    pub title: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub day_count: Option<i64>,
    pub companion_count: Option<i64>,
    pub route_summary: Option<String>,
    pub gear_summary: Option<String>,
    pub food_summary: Option<String>,
    pub budget_summary: Option<String>,
    pub notes: Option<String>,
}

impl OutdoorExperienceDraft {
    /// Validates and trims profile-facing outdoor experience fields.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.title =
            normalize_required_text(std::mem::take(&mut self.title), 100, "title", &mut errors);
        self.start_date =
            normalize_optional_text(self.start_date.take(), 30, "start_date", &mut errors);
        self.end_date = normalize_optional_text(self.end_date.take(), 30, "end_date", &mut errors);
        self.route_summary =
            normalize_optional_text(self.route_summary.take(), 500, "route_summary", &mut errors);
        self.gear_summary =
            normalize_optional_text(self.gear_summary.take(), 500, "gear_summary", &mut errors);
        self.food_summary =
            normalize_optional_text(self.food_summary.take(), 500, "food_summary", &mut errors);
        self.budget_summary = normalize_optional_text(
            self.budget_summary.take(),
            500,
            "budget_summary",
            &mut errors,
        );
        self.notes = normalize_optional_text(self.notes.take(), 1000, "notes", &mut errors);
        if self
            .day_count
            .is_some_and(|value| !(0..=366).contains(&value))
        {
            errors.push(FieldViolation::new(
                "day_count",
                "must be between 0 and 366",
            ));
        }
        if self
            .companion_count
            .is_some_and(|value| !(0..=999).contains(&value))
        {
            errors.push(FieldViolation::new(
                "companion_count",
                "must be between 0 and 999",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Invitation token returned after an owner creates an invite.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TripInvitation {
    pub id: String,
    pub trip_id: String,
    pub token: String,
    pub created_by_user_id: String,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

/// Derived altitude context for one route estimate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RouteAltitudeEstimate {
    pub start_m: i32,
    pub end_m: i32,
    pub highest_m: i32,
}

impl RouteAltitudeEstimate {
    /// Builds an altitude estimate from a start altitude and route elevation deltas.
    pub fn from_segment(start_m: i32, ascent_m: i32, descent_m: i32) -> Self {
        let end_m = start_m + ascent_m.max(0) - descent_m.max(0);
        let highest_m = start_m.max(start_m + ascent_m.max(0)).max(end_m);
        Self {
            start_m,
            end_m,
            highest_m,
        }
    }
}

/// Computes the high-altitude time multiplier for an estimated route altitude.
pub fn high_altitude_factor(highest_altitude_m: i32) -> f64 {
    if highest_altitude_m >= 4_500 {
        1.35
    } else if highest_altitude_m >= 3_500 {
        1.20
    } else if highest_altitude_m >= 2_500 {
        1.10
    } else {
        1.0
    }
}

/// Validates plan-level route estimation settings.
pub fn validate_route_estimation_settings(
    use_high_altitude_adjustment: bool,
    start_altitude_m: Option<i32>,
    errors: &mut Vec<FieldViolation>,
) {
    if let Some(value) = start_altitude_m
        && !(ROUTE_START_ALTITUDE_MIN_M..=ROUTE_START_ALTITUDE_MAX_M).contains(&value)
    {
        errors.push(FieldViolation::new(
            "route_start_altitude_m",
            "must be between -500 and 9000",
        ));
    }
    if use_high_altitude_adjustment && start_altitude_m.is_none() {
        errors.push(FieldViolation::new(
            "route_start_altitude_m",
            "is required when high-altitude adjustment is enabled",
        ));
    }
}

/// Computes Naismith-based estimate minutes with optional route adjustments.
#[allow(clippy::too_many_arguments)]
pub fn estimate_route_minutes(
    distance_km: f64,
    ascent_m: i32,
    descent_m: i32,
    _descent_profile: &str,
    use_slope_adjustment: bool,
    high_altitude_factor: f64,
    technical_factor: f64,
    rest_factor: f64,
    pack_factor: f64,
    manual_estimate_minutes: Option<i32>,
) -> (i32, i32) {
    let distance_minutes = distance_km.max(0.0) / 5.0 * 60.0;
    let ascent_minutes = ascent_m.max(0) as f64
        / ascent_rate_m_per_hour(distance_km, ascent_m, use_slope_adjustment)
        * 60.0;
    let descent_minutes = if use_slope_adjustment {
        descent_rate_m_per_hour(distance_km, descent_m)
            .map(|rate| descent_m.max(0) as f64 / rate * 60.0)
            .unwrap_or(0.0)
    } else {
        0.0
    };
    let formula = round_to_five_minutes(
        (distance_minutes + ascent_minutes + descent_minutes).max(0.0)
            * high_altitude_factor.max(1.0)
            * technical_factor.max(0.1)
            * rest_factor.max(0.1)
            * pack_factor.max(0.1),
    );
    let final_minutes = manual_estimate_minutes.unwrap_or(formula).max(0);
    (formula, final_minutes)
}

/// Selects the ascent speed for the slope-adjusted Naismith calculation.
fn ascent_rate_m_per_hour(distance_km: f64, ascent_m: i32, use_slope_adjustment: bool) -> f64 {
    if !use_slope_adjustment {
        return 600.0;
    }
    let grade = elevation_grade(distance_km, ascent_m);
    if grade >= 0.25 {
        300.0
    } else if grade >= 0.15 {
        400.0
    } else if grade >= 0.08 {
        500.0
    } else {
        600.0
    }
}

/// Selects the descent speed for the slope-adjusted Naismith calculation.
fn descent_rate_m_per_hour(distance_km: f64, descent_m: i32) -> Option<f64> {
    let grade = elevation_grade(distance_km, descent_m);
    if grade >= 0.25 {
        Some(600.0)
    } else if grade >= 0.15 {
        Some(900.0)
    } else if grade >= 0.08 {
        Some(1_200.0)
    } else {
        None
    }
}

/// Computes average elevation grade, treating zero-distance climbs as the steepest band.
fn elevation_grade(distance_km: f64, elevation_m: i32) -> f64 {
    let elevation_m = elevation_m.max(0) as f64;
    if elevation_m == 0.0 {
        return 0.0;
    }
    let distance_m = distance_km.max(0.0) * 1_000.0;
    if distance_m == 0.0 {
        f64::INFINITY
    } else {
        elevation_m / distance_m
    }
}

fn round_to_five_minutes(value: f64) -> i32 {
    ((value / 5.0).round() * 5.0) as i32
}

/// Normalizes positive quantities used by plan item drafts.
pub fn normalize_quantity(value: i32, field: &str, errors: &mut Vec<FieldViolation>) -> i32 {
    if !(0..=9_999).contains(&value) {
        errors.push(FieldViolation::new(field, "must be between 0 and 9999"));
        value.clamp(0, 9_999)
    } else {
        value
    }
}
