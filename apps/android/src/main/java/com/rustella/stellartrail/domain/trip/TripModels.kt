package com.rustella.stellartrail.domain.trip

import com.rustella.stellartrail.domain.gear.GearCategory
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject

typealias FieldVersions = JsonObject

@Serializable
enum class TripType {
    @SerialName("solo") SOLO,
    @SerialName("team") TEAM,
}

@Serializable
enum class TripTimeBucket {
    @SerialName("ongoing") ONGOING,
    @SerialName("upcoming") UPCOMING,
    @SerialName("past") PAST,
    @SerialName("undated") UNDATED,
}

@Serializable
enum class TripSectionKey {
    @SerialName("members") MEMBERS,
    @SerialName("personal_gear") PERSONAL_GEAR,
    @SerialName("itinerary") ITINERARY,
    @SerialName("shared_gear") SHARED_GEAR,
    @SerialName("food_plan") FOOD_PLAN,
    @SerialName("medical_kit") MEDICAL_KIT,
    @SerialName("safety_plan") SAFETY_PLAN,
    @SerialName("rescue_info") RESCUE_INFO,
    @SerialName("budget") BUDGET,
    @SerialName("goals") GOALS,
}

@Serializable
data class TripReadiness(
    @SerialName("missing_count") val missingCount: Int = 0,
    @SerialName("missing_labels") val missingLabels: List<String> = emptyList(),
    @SerialName("completion_percent") val completionPercent: Int = 0,
)

@Serializable
data class TripFieldConflict(
    val field: String,
    @SerialName("client_value") val clientValue: kotlinx.serialization.json.JsonElement? = null,
    @SerialName("server_value") val serverValue: kotlinx.serialization.json.JsonElement? = null,
    @SerialName("server_version") val serverVersion: Int = 0,
)

@Serializable
data class TripConflictResponse(
    val code: String = "edit_conflict",
    val message: String = "",
    val conflicts: List<TripFieldConflict> = emptyList(),
)

@Serializable
data class TripSummary(
    val id: String = "",
    @SerialName("owner_user_id") val ownerUserId: String = "",
    @SerialName("trip_type") val tripType: TripType = TripType.TEAM,
    val title: String = "",
    val name: String? = null,
    val description: String? = null,
    @SerialName("start_date") val startDate: String? = null,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("enabled_sections") val enabledSections: List<TripSectionKey> = emptyList(),
    @SerialName("route_use_slope_adjustment") val routeUseSlopeAdjustment: Boolean = false,
    @SerialName("route_use_high_altitude_adjustment") val routeUseHighAltitudeAdjustment: Boolean = false,
    @SerialName("route_start_altitude_m") val routeStartAltitudeM: Int? = null,
    @SerialName("day_count") val dayCount: Int = 0,
    @SerialName("itinerary_day_count") val itineraryDayCount: Int = dayCount,
    @SerialName("time_bucket") val timeBucket: TripTimeBucket = TripTimeBucket.UNDATED,
    @SerialName("days_until_start") val daysUntilStart: Int? = null,
    @SerialName("days_until_end") val daysUntilEnd: Int? = null,
    @SerialName("member_count") val memberCount: Int = 1,
    val readiness: TripReadiness = TripReadiness(),
    @SerialName("outdoor_experience_id") val outdoorExperienceId: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
) {
    val displayName: String get() = title.ifBlank { name.orEmpty() }
}

@Serializable
data class ListTripsResponse(
    val items: List<TripSummary> = emptyList(),
    @SerialName("next_cursor") val nextCursor: String? = null,
)

data class ListTripsRequest(
    val limit: Int = 20,
    val cursor: String? = null,
    val bucket: TripTimeBucket? = null,
    val tripType: TripType? = null,
)

@Serializable
data class TripHomeHighlightItem(
    val trip: TripSummary = TripSummary(),
    val status: TripHomeHighlightStatus = TripHomeHighlightStatus.UPCOMING,
    @SerialName("days_until_start") val daysUntilStart: Int = 0,
    @SerialName("days_until_end") val daysUntilEnd: Int = 0,
)

@Serializable
enum class TripHomeHighlightStatus {
    @SerialName("ongoing") ONGOING,
    @SerialName("upcoming") UPCOMING,
}

@Serializable
data class TripHomeHighlightResponse(
    val item: TripHomeHighlightItem? = null,
)

@Serializable
data class CreateTripRequest(
    @SerialName("trip_type") val tripType: TripType,
    val title: String,
    val description: String? = null,
    @SerialName("start_date") val startDate: String? = null,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("route_use_slope_adjustment") val routeUseSlopeAdjustment: Boolean? = null,
    @SerialName("route_use_high_altitude_adjustment") val routeUseHighAltitudeAdjustment: Boolean? = null,
    @SerialName("route_start_altitude_m") val routeStartAltitudeM: Int? = null,
)

@Serializable
data class UpdateTripRequest(
    val title: String? = null,
    val description: String? = null,
    @SerialName("start_date") val startDate: String? = null,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("route_use_slope_adjustment") val routeUseSlopeAdjustment: Boolean? = null,
    @SerialName("route_use_high_altitude_adjustment") val routeUseHighAltitudeAdjustment: Boolean? = null,
    @SerialName("route_start_altitude_m") val routeStartAltitudeM: Int? = null,
    @SerialName("base_field_versions") val baseFieldVersions: FieldVersions? = null,
    @SerialName("force_fields") val forceFields: List<String>? = null,
)

@Serializable
data class UpdateTripSectionsRequest(
    @SerialName("enabled_sections") val enabledSections: List<TripSectionKey>,
    @SerialName("base_field_versions") val baseFieldVersions: FieldVersions? = null,
    @SerialName("force_fields") val forceFields: List<String>? = null,
)

@Serializable
data class TripMemberProfile(
    @SerialName("display_name") val displayName: String = "",
    @SerialName("outdoor_id") val outdoorId: String? = null,
    @SerialName("real_name") val realName: String? = null,
    val gender: String? = null,
    val age: Int? = null,
    @SerialName("height_cm") val heightCm: Int? = null,
    val phone: String? = null,
    @SerialName("emergency_contact") val emergencyContact: String? = null,
    @SerialName("emergency_contact_relationship") val emergencyContactRelationship: String? = null,
    @SerialName("emergency_phone") val emergencyPhone: String? = null,
    @SerialName("blood_type") val bloodType: String? = null,
    @SerialName("medical_history") val medicalHistory: String? = null,
    @SerialName("allergy_history") val allergyHistory: String? = null,
    @SerialName("medical_response_note") val medicalResponseNote: String? = null,
    @SerialName("diet_preference") val dietPreference: String? = null,
    @SerialName("insurance_policy_no") val insurancePolicyNo: String? = null,
    @SerialName("insurance_company_phone") val insuranceCompanyPhone: String? = null,
    @SerialName("experience_note") val experienceNote: String? = null,
    @SerialName("role_label") val roleLabel: String? = null,
)

@Serializable
data class TripMember(
    val id: String,
    @SerialName("trip_id") val tripId: String? = null,
    @SerialName("plan_id") val planId: String? = null,
    @SerialName("user_id") val userId: String,
    @SerialName("is_owner") val isOwner: Boolean = false,
    val profile: TripMemberProfile = TripMemberProfile(),
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripGearSnapshotBase(
    val id: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("unit_weight_g") val unitWeightG: Int? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripPersonalGearItem(
    val id: String,
    @SerialName("member_id") val memberId: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("unit_weight_g") val unitWeightG: Int? = null,
    @SerialName("source_packing_list_id") val sourcePackingListId: String? = null,
    @SerialName("source_packing_item_id") val sourcePackingItemId: String? = null,
    @SerialName("source_gear_id") val sourceGearId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripSharedGearDemand(
    val id: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("unit_weight_g") val unitWeightG: Int? = null,
    @SerialName("source_member_id") val sourceMemberId: String? = null,
    @SerialName("source_gear_id") val sourceGearId: String? = null,
    @SerialName("responsible_member_id") val responsibleMemberId: String,
    @SerialName("created_by_user_id") val createdByUserId: String? = null,
    @SerialName("template_key") val templateKey: String? = null,
    @SerialName("demand_name") val demandName: String? = null,
    @SerialName("slot_key") val slotKey: String? = null,
    @SerialName("slot_name") val slotName: String? = null,
    @SerialName("concrete_name") val concreteName: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class SharedGearDemandTemplate(
    @SerialName("template_key") val templateKey: String,
    @SerialName("demand_name") val demandName: String,
    @SerialName("group_label") val groupLabel: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("sort_order") val sortOrder: Int = 0,
    @SerialName("slot_key") val slotKey: String? = null,
    @SerialName("slot_name") val slotName: String? = null,
)

@Serializable
data class TripRouteSegment(
    val id: String,
    val name: String,
    @SerialName("start_point") val startPoint: String? = null,
    @SerialName("end_point") val endPoint: String? = null,
    val checkpoint: String? = null,
    @SerialName("leader_member_id") val leaderMemberId: String? = null,
    @SerialName("bailout_route") val bailoutRoute: String? = null,
    @SerialName("trail_condition") val trailCondition: String? = null,
    @SerialName("distance_km") val distanceKm: Double = 0.0,
    @SerialName("ascent_m") val ascentM: Int = 0,
    @SerialName("descent_m") val descentM: Int = 0,
    @SerialName("descent_profile") val descentProfile: String = "none",
    @SerialName("technical_factor") val technicalFactor: Double = 1.0,
    @SerialName("rest_factor") val restFactor: Double = 1.0,
    @SerialName("pack_factor") val packFactor: Double = 1.0,
    @SerialName("formula_estimate_minutes") val formulaEstimateMinutes: Int = 0,
    @SerialName("final_estimate_minutes") val finalEstimateMinutes: Int = 0,
    @SerialName("manual_estimate_minutes") val manualEstimateMinutes: Int? = null,
    @SerialName("estimated_start_altitude_m") val estimatedStartAltitudeM: Int? = null,
    @SerialName("estimated_end_altitude_m") val estimatedEndAltitudeM: Int? = null,
    @SerialName("estimated_highest_altitude_m") val estimatedHighestAltitudeM: Int? = null,
    @SerialName("high_altitude_factor") val highAltitudeFactor: Double? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripItineraryTimeSlot(
    val id: String,
    @SerialName("day_id") val dayId: String,
    @SerialName("slot_key") val slotKey: String,
    @SerialName("route_segment_id") val routeSegmentId: String? = null,
    @SerialName("route_description") val routeDescription: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripItineraryDay(
    val id: String,
    @SerialName("day_index") val dayIndex: Int,
    @SerialName("date_label") val dateLabel: String? = null,
    val title: String? = null,
    val notes: String? = null,
    val weather: String? = null,
    @SerialName("high_temperature_c") val highTemperatureC: Int? = null,
    @SerialName("low_temperature_c") val lowTemperatureC: Int? = null,
    @SerialName("weather_summary") val weatherSummary: String? = null,
    @SerialName("weather_notes") val weatherNotes: String? = null,
    @SerialName("camp_name") val campName: String? = null,
    @SerialName("camp_altitude_m") val campAltitudeM: Int? = null,
    @SerialName("camp_terrain") val campTerrain: String? = null,
    @SerialName("camp_slope") val campSlope: String? = null,
    @SerialName("camp_area") val campArea: String? = null,
    @SerialName("camp_water_source") val campWaterSource: String? = null,
    @SerialName("camp_notes") val campNotes: String? = null,
    @SerialName("estimate_minutes") val estimateMinutes: Int = 0,
    @SerialName("time_slots") val timeSlots: List<TripItineraryTimeSlot> = emptyList(),
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripFoodItem(
    val id: String,
    @SerialName("food_meal_id") val foodMealId: String,
    val name: String,
    @SerialName("amount_g") val amountG: Int? = null,
    @SerialName("per_person_amount_g") val perPersonAmountG: Int? = null,
    @SerialName("total_price_cents") val totalPriceCents: Long? = null,
    @SerialName("responsible_member_id") val responsibleMemberId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripFoodMeal(
    val id: String,
    @SerialName("itinerary_day_id") val itineraryDayId: String,
    @SerialName("meal_key") val mealKey: String,
    @SerialName("meal_type") val mealType: String? = null,
    val skipped: Boolean = false,
    @SerialName("dish_name") val dishName: String? = null,
    @SerialName("responsible_member_id") val responsibleMemberId: String? = null,
    val notes: String? = null,
    val items: List<TripFoodItem> = emptyList(),
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripFoodSupply(
    val id: String,
    val name: String,
    @SerialName("supply_type") val supplyType: String? = null,
    @SerialName("amount_g") val amountG: Int? = null,
    @SerialName("per_person_amount_g") val perPersonAmountG: Int? = null,
    @SerialName("total_price_cents") val totalPriceCents: Long? = null,
    @SerialName("responsible_member_id") val responsibleMemberId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripMedicalItem(
    val id: String,
    val name: String,
    @SerialName("item_type") val itemType: String? = null,
    val scope: String? = null,
    @SerialName("suggested_quantity") val suggestedQuantity: Int? = null,
    @SerialName("required_quantity") val requiredQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("responsible_member_id") val responsibleMemberId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripSegmentAssignment(
    val id: String,
    @SerialName("route_segment_id") val routeSegmentId: String? = null,
    val checkpoint: String? = null,
    @SerialName("leader_record_member_id") val leaderRecordMemberId: String? = null,
    @SerialName("navigator_safety_member_id") val navigatorSafetyMemberId: String? = null,
    @SerialName("collaborator_member_id") val collaboratorMemberId: String? = null,
    @SerialName("photographer_member_id") val photographerMemberId: String? = null,
    @SerialName("safety_member_id") val safetyMemberId: String? = null,
    @SerialName("environment_member_id") val environmentMemberId: String? = null,
    @SerialName("sweeper_member_id") val sweeperMemberId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripSafetyRisk(
    val id: String,
    @SerialName("risk_type") val riskType: String,
    val prevention: String? = null,
    val response: String? = null,
    @SerialName("responsible_member_id") val responsibleMemberId: String? = null,
    @SerialName("itinerary_day_id") val itineraryDayId: String? = null,
    @SerialName("route_segment_id") val routeSegmentId: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripRescueContact(
    val id: String,
    val organization: String,
    val address: String? = null,
    val phone: String? = null,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripBudgetItem(
    val id: String,
    val category: String? = null,
    val name: String,
    val quantity: Int = 1,
    @SerialName("unit_price_cents") val unitPriceCents: Long? = null,
    @SerialName("total_price_cents") val totalPriceCents: Long? = null,
    @SerialName("split_member_count") val splitMemberCount: Int? = null,
    val notes: String? = null,
    @SerialName("linked_shared_gear_id") val linkedSharedGearId: String? = null,
    @SerialName("linked_shared_gear_deleted") val linkedSharedGearDeleted: Boolean = false,
    @SerialName("linked_shared_gear_name") val linkedSharedGearName: String? = null,
    @SerialName("linked_shared_gear_responsible_member_id") val linkedSharedGearResponsibleMemberId: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripGoalItem(
    val id: String,
    val scope: String,
    @SerialName("member_id") val memberId: String? = null,
    val content: String,
    val notes: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class TripMemberGearWeightSummary(
    @SerialName("member_id") val memberId: String,
    @SerialName("all_weight_g") val allWeightG: Int = 0,
    @SerialName("actual_weight_g") val actualWeightG: Int = 0,
)

@Serializable
data class TripMemberGearViewItem(
    val id: String,
    val source: String,
    val name: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("unit_weight_g") val unitWeightG: Int? = null,
    val labels: List<String> = emptyList(),
    @SerialName("counts_weight") val countsWeight: Boolean = true,
)

@Serializable
data class TripMemberGearView(
    @SerialName("member_id") val memberId: String,
    @SerialName("all_weight_g") val allWeightG: Int = 0,
    @SerialName("actual_weight_g") val actualWeightG: Int = 0,
    val items: List<TripMemberGearViewItem> = emptyList(),
)

@Serializable
data class TripDetail(
    val trip: TripSummary = TripSummary(),
    val sections: List<TripSectionKey> = emptyList(),
    @SerialName("my_member_id") val myMemberId: String = "",
    val members: List<TripMember> = emptyList(),
    @SerialName("personal_gear") val personalGear: List<TripPersonalGearItem> = emptyList(),
    @SerialName("shared_gear_demands") val sharedGearDemands: List<TripSharedGearDemand> = emptyList(),
    @SerialName("shared_gear_demand_templates") val sharedGearDemandTemplates: List<SharedGearDemandTemplate> = emptyList(),
    @SerialName("itinerary_days") val itineraryDays: List<TripItineraryDay> = emptyList(),
    @SerialName("route_segments") val routeSegments: List<TripRouteSegment> = emptyList(),
    @SerialName("food_meals") val foodMeals: List<TripFoodMeal> = emptyList(),
    @SerialName("food_supplies") val foodSupplies: List<TripFoodSupply> = emptyList(),
    @SerialName("medical_items") val medicalItems: List<TripMedicalItem> = emptyList(),
    @SerialName("segment_assignments") val segmentAssignments: List<TripSegmentAssignment> = emptyList(),
    @SerialName("safety_risks") val safetyRisks: List<TripSafetyRisk> = emptyList(),
    @SerialName("rescue_contacts") val rescueContacts: List<TripRescueContact> = emptyList(),
    @SerialName("budget_items") val budgetItems: List<TripBudgetItem> = emptyList(),
    val goals: List<TripGoalItem> = emptyList(),
    @SerialName("weight_summaries") val weightSummaries: List<TripMemberGearWeightSummary> = emptyList(),
    @SerialName("member_gear_views") val memberGearViews: List<TripMemberGearView> = emptyList(),
)

@Serializable
data class TripInvitation(
    val id: String,
    @SerialName("plan_id") val planId: String? = null,
    @SerialName("trip_id") val tripId: String? = null,
    val token: String,
    @SerialName("created_by_user_id") val createdByUserId: String,
    @SerialName("revoked_at") val revokedAt: String? = null,
    @SerialName("created_at") val createdAt: String = "",
)

@Serializable
data class CreateTripInvitationResponse(
    val invitation: TripInvitation,
)

@Serializable
data class OutdoorExperience(
    val id: String,
    @SerialName("user_id") val userId: String,
    @SerialName("source_trip_id") val sourceTripId: String? = null,
    @SerialName("trip_type") val tripType: TripType = TripType.TEAM,
    val title: String,
    @SerialName("start_date") val startDate: String? = null,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("day_count") val dayCount: Int? = null,
    @SerialName("companion_count") val companionCount: Int? = null,
    @SerialName("route_summary") val routeSummary: String? = null,
    @SerialName("gear_summary") val gearSummary: String? = null,
    @SerialName("food_summary") val foodSummary: String? = null,
    @SerialName("budget_summary") val budgetSummary: String? = null,
    val notes: String? = null,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class ImportTripPackingListRequest(
    @SerialName("packing_list_id") val packingListId: String,
)

enum class TripRecordKind(val section: TripSectionKey, val collectionPath: String, val label: String) {
    PersonalGear(TripSectionKey.PERSONAL_GEAR, "personal-gear", "个人装备"),
    SharedGear(TripSectionKey.SHARED_GEAR, "shared-gear-demands", "公共装备"),
    ItineraryDay(TripSectionKey.ITINERARY, "itinerary-days", "行程日"),
    RouteSegment(TripSectionKey.ITINERARY, "route-segments", "路线段"),
    SegmentAssignment(TripSectionKey.SAFETY_PLAN, "segment-assignments", "分段分工"),
    FoodMeal(TripSectionKey.FOOD_PLAN, "food-meals", "餐次"),
    FoodSupply(TripSectionKey.FOOD_PLAN, "food-supplies", "公共食材"),
    MedicalItem(TripSectionKey.MEDICAL_KIT, "medical-items", "医药物资"),
    SafetyRisk(TripSectionKey.SAFETY_PLAN, "safety-risks", "安全风险"),
    RescueContact(TripSectionKey.RESCUE_INFO, "rescue-contacts", "救援信息"),
    BudgetItem(TripSectionKey.BUDGET, "budget-items", "预算条目"),
    Goal(TripSectionKey.GOALS, "goals", "目标"),
}

fun emptyFieldVersions(): FieldVersions = buildJsonObject { }

fun TripType.apiValue(): String = when (this) {
    TripType.SOLO -> "solo"
    TripType.TEAM -> "team"
}

fun TripType.label(): String = when (this) {
    TripType.SOLO -> "单人"
    TripType.TEAM -> "多人"
}

fun TripTimeBucket.apiValue(): String = when (this) {
    TripTimeBucket.ONGOING -> "ongoing"
    TripTimeBucket.UPCOMING -> "upcoming"
    TripTimeBucket.PAST -> "past"
    TripTimeBucket.UNDATED -> "undated"
}

fun TripTimeBucket.label(): String = when (this) {
    TripTimeBucket.ONGOING -> "进行中"
    TripTimeBucket.UPCOMING -> "未来行程"
    TripTimeBucket.PAST -> "历史行程"
    TripTimeBucket.UNDATED -> "未定日期"
}

fun TripSectionKey.apiValue(): String = when (this) {
    TripSectionKey.MEMBERS -> "members"
    TripSectionKey.PERSONAL_GEAR -> "personal_gear"
    TripSectionKey.ITINERARY -> "itinerary"
    TripSectionKey.SHARED_GEAR -> "shared_gear"
    TripSectionKey.FOOD_PLAN -> "food_plan"
    TripSectionKey.MEDICAL_KIT -> "medical_kit"
    TripSectionKey.SAFETY_PLAN -> "safety_plan"
    TripSectionKey.RESCUE_INFO -> "rescue_info"
    TripSectionKey.BUDGET -> "budget"
    TripSectionKey.GOALS -> "goals"
}

fun TripSectionKey.label(): String = when (this) {
    TripSectionKey.MEMBERS -> "成员"
    TripSectionKey.PERSONAL_GEAR -> "个人装备"
    TripSectionKey.ITINERARY -> "行程"
    TripSectionKey.SHARED_GEAR -> "公共装备"
    TripSectionKey.FOOD_PLAN -> "食品"
    TripSectionKey.MEDICAL_KIT -> "医药"
    TripSectionKey.SAFETY_PLAN -> "安全"
    TripSectionKey.RESCUE_INFO -> "救援"
    TripSectionKey.BUDGET -> "预算"
    TripSectionKey.GOALS -> "目标"
}

fun TripSummary.dateText(): String = when {
    startDate != null && endDate != null && startDate != endDate -> "$startDate - $endDate"
    startDate != null -> startDate
    else -> "未设置行程时间"
}

fun TripSummary.durationText(): String =
    if (itineraryDayCount > 0) "${itineraryDayCount}天${(itineraryDayCount - 1).coerceAtLeast(0)}夜" else "待规划"
