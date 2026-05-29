package com.rustella.stellartrail.feature.trips

import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.apiValue
import com.rustella.stellartrail.domain.trip.FieldVersions
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.emptyFieldVersions
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

object TripPayloads {
    fun defaultCreate(kind: TripRecordKind, detail: TripDetail): JsonObject = buildJsonObject {
        when (kind) {
            TripRecordKind.PersonalGear -> {
                put("member_id", detail.myMemberId)
                put("name", "待确认个人装备")
                put("category", GearCategory.OTHER_GEAR.apiValue())
                put("planned_quantity", 1)
                put("packed_quantity", 0)
            }
            TripRecordKind.SharedGear -> {
                put("responsible_member_id", detail.myMemberId)
                put("name", "公共装备需求")
                put("category", GearCategory.OTHER_GEAR.apiValue())
                put("planned_quantity", 1)
            }
            TripRecordKind.ItineraryDay -> {
                put("day_index", detail.itineraryDays.size + 1)
                put("title", "第 ${detail.itineraryDays.size + 1} 天")
            }
            TripRecordKind.RouteSegment -> {
                put("name", "新路线段")
                put("distance_km", 0.0)
                put("ascent_m", 0)
                put("descent_m", 0)
            }
            TripRecordKind.SegmentAssignment -> {
                put("checkpoint", "待分工检查点")
                put("leader_record_member_id", detail.myMemberId)
            }
            TripRecordKind.FoodMeal -> {
                val day = detail.itineraryDays.firstOrNull()
                if (day != null) put("itinerary_day_id", day.id)
                put("meal_key", "breakfast")
                put("dish_name", "待定餐食")
            }
            TripRecordKind.FoodSupply -> {
                put("name", "公共食材")
                put("amount_g", 0)
            }
            TripRecordKind.MedicalItem -> {
                put("name", "常备药品")
                put("required_quantity", 1)
                put("packed_quantity", 0)
            }
            TripRecordKind.SafetyRisk -> {
                put("risk_type", "天气变化")
                put("prevention", "出发前检查天气并准备备选方案")
            }
            TripRecordKind.RescueContact -> {
                put("organization", "当地救援电话")
                put("phone", "")
            }
            TripRecordKind.BudgetItem -> {
                put("name", "待分摊费用")
                put("quantity", 1)
            }
            TripRecordKind.Goal -> {
                put("scope", "team")
                put("content", "完成本次行程准备")
            }
        }
    }

    fun defaultPatch(kind: TripRecordKind, versions: FieldVersions = emptyFieldVersions()): JsonObject = buildJsonObject {
        when (kind) {
            TripRecordKind.PersonalGear,
            TripRecordKind.SharedGear,
            TripRecordKind.MedicalItem -> put("packed_quantity", 1)
            TripRecordKind.ItineraryDay -> put("notes", "Android 端已更新行程日信息")
            TripRecordKind.RouteSegment -> put("notes", "Android 端已复核路线段")
            TripRecordKind.SegmentAssignment -> put("notes", "Android 端已复核分工")
            TripRecordKind.FoodMeal -> put("notes", "Android 端已复核餐次")
            TripRecordKind.FoodSupply -> put("notes", "Android 端已复核公共食材")
            TripRecordKind.SafetyRisk -> put("notes", "Android 端已复核风险预案")
            TripRecordKind.RescueContact -> put("notes", "Android 端已复核救援信息")
            TripRecordKind.BudgetItem -> put("notes", "Android 端已复核预算")
            TripRecordKind.Goal -> put("notes", "Android 端已复核目标")
        }
        put("base_field_versions", versions)
    }

    fun memberPatch(displayName: String, versions: FieldVersions): JsonObject = buildJsonObject {
        put("display_name", displayName)
        put("role_label", "已确认")
        put("base_field_versions", versions)
    }

    fun forcePatch(conflictFields: List<String>, original: JsonObject): JsonObject = buildJsonObject {
        original.forEach { (key, value) -> put(key, value) }
        put("force_fields", kotlinx.serialization.json.JsonArray(conflictFields.map { JsonPrimitive(it) }))
    }

    val defaultSections: List<TripSectionKey> = listOf(
        TripSectionKey.MEMBERS,
        TripSectionKey.PERSONAL_GEAR,
        TripSectionKey.ITINERARY,
        TripSectionKey.SHARED_GEAR,
        TripSectionKey.FOOD_PLAN,
        TripSectionKey.MEDICAL_KIT,
        TripSectionKey.SAFETY_PLAN,
        TripSectionKey.RESCUE_INFO,
        TripSectionKey.BUDGET,
        TripSectionKey.GOALS,
    )
}
