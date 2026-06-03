package com.rustella.stellartrail.domain.profile

import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ProfileUserResponse(
    val user: LoginUser,
)

@Serializable
data class OutdoorProfile(
    @SerialName("user_id") val userId: String = "",
    @SerialName("outdoor_id") val outdoorId: String? = null,
    @SerialName("real_name") val realName: String? = null,
    val gender: String? = null,
    @SerialName("birth_date") val birthDate: String? = null,
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
    @SerialName("created_at") val createdAt: String? = null,
    @SerialName("updated_at") val updatedAt: String? = null,
)

@Serializable
data class OutdoorProfileResponse(
    val profile: OutdoorProfile,
)

@Serializable
data class OutdoorExperienceRequest(
    val title: String,
    @SerialName("start_date") val startDate: String? = null,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("day_count") val dayCount: Long? = null,
    @SerialName("companion_count") val companionCount: Long? = null,
    @SerialName("route_summary") val routeSummary: String? = null,
    @SerialName("gear_summary") val gearSummary: String? = null,
    @SerialName("food_summary") val foodSummary: String? = null,
    @SerialName("budget_summary") val budgetSummary: String? = null,
    val notes: String? = null,
)

@Serializable
data class ListOutdoorExperiencesResponse(
    val items: List<OutdoorExperience> = emptyList(),
)

@Serializable
data class RoadmapItem(
    val id: String,
    @SerialName("client_key") val clientKey: String = "android",
    val title: String,
    val summary: String,
    val details: String? = null,
    val category: String = "community",
    val status: String = "planned",
    val priority: Int = 0,
    @SerialName("sort_order") val sortOrder: Int = 0,
    @SerialName("is_published") val isPublished: Boolean = true,
    @SerialName("vote_count") val voteCount: Int = 0,
    @SerialName("subscription_count") val subscriptionCount: Int = 0,
    @SerialName("is_voted") val isVoted: Boolean = false,
    @SerialName("is_subscribed") val isSubscribed: Boolean = false,
    @SerialName("published_at") val publishedAt: String? = null,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)

@Serializable
data class ListRoadmapResponse(
    val items: List<RoadmapItem> = emptyList(),
    @SerialName("next_cursor") val nextCursor: String? = null,
)

enum class RoadmapStatusFilter(val apiValue: String?, val label: String) {
    All(null, "全部状态"),
    Planned("planned", "已规划"),
    Designing("designing", "设计中"),
    Building("building", "推进中"),
    Preview("preview", "预览中"),
    Shipped("shipped", "已上线"),
}

fun RoadmapItem.categoryLabel(): String = when (category) {
    "gear" -> "装备"
    "skills" -> "技能"
    "routes" -> "路线"
    "offline" -> "离线"
    "safety" -> "安全"
    "community" -> "社区"
    else -> "规划"
}

fun RoadmapItem.statusLabel(): String = when (status) {
    "planned" -> "已规划"
    "designing" -> "设计中"
    "building" -> "推进中"
    "preview" -> "预览中"
    "shipped" -> "已上线"
    else -> "规划中"
}
