package com.rustella.stellartrail.domain.atlas

import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.apiValue
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class GearAtlasStatus {
    @SerialName("pending") PENDING,
    @SerialName("approved") APPROVED,
    @SerialName("rejected") REJECTED,
}

@Serializable
enum class GearAtlasSourceType {
    @SerialName("manual") MANUAL,
    @SerialName("user_gear") USER_GEAR,
    @SerialName("external_import") EXTERNAL_IMPORT,
}

@Serializable
enum class GearAtlasSort {
    @SerialName("approved_at_desc") APPROVED_AT_DESC,
    @SerialName("name_asc") NAME_ASC,
    @SerialName("weight_desc") WEIGHT_DESC,
    @SerialName("official_price_desc") OFFICIAL_PRICE_DESC,
}

@Serializable
data class GearAtlasPublicItem(
    val id: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String? = null,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("official_price_cents") val officialPriceCents: Long? = null,
    @SerialName("official_price_currency") val officialPriceCurrency: String? = null,
    val specs: Map<String, String>? = null,
    @SerialName("approved_at") val approvedAt: String? = null,
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class GearAtlasSubmission(
    val id: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String? = null,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("official_price_cents") val officialPriceCents: Long? = null,
    @SerialName("official_price_currency") val officialPriceCurrency: String? = null,
    val specs: Map<String, String>? = null,
    @SerialName("approved_at") val approvedAt: String? = null,
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    @SerialName("source_type") val sourceType: GearAtlasSourceType,
    @SerialName("source_user_gear_id") val sourceUserGearId: String? = null,
    val status: GearAtlasStatus,
    @SerialName("rejection_reason") val rejectionReason: String? = null,
    @SerialName("reviewed_at") val reviewedAt: String? = null,
)

@Serializable
data class CreateGearAtlasSubmissionRequest(
    val category: GearCategory,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("official_price_cents") val officialPriceCents: Long? = null,
    @SerialName("official_price_currency") val officialPriceCurrency: String? = null,
    val specs: Map<String, String>? = null,
)

data class ListGearAtlasRequest(
    val category: GearCategory? = null,
    val query: String? = null,
    val sort: GearAtlasSort = GearAtlasSort.APPROVED_AT_DESC,
    val limit: Int = 20,
    val cursor: String? = null,
)

@Serializable
data class ListGearAtlasResponse(
    val items: List<GearAtlasPublicItem>,
    @SerialName("next_cursor") val nextCursor: String? = null,
)

data class ListGearAtlasSubmissionsRequest(
    val limit: Int = 20,
    val cursor: String? = null,
)

@Serializable
data class ListGearAtlasSubmissionsResponse(
    val items: List<GearAtlasSubmission>,
    @SerialName("next_cursor") val nextCursor: String? = null,
)

fun GearAtlasSort.apiValue(): String = when (this) {
    GearAtlasSort.APPROVED_AT_DESC -> "approved_at_desc"
    GearAtlasSort.NAME_ASC -> "name_asc"
    GearAtlasSort.WEIGHT_DESC -> "weight_desc"
    GearAtlasSort.OFFICIAL_PRICE_DESC -> "official_price_desc"
}

fun ListGearAtlasRequest.toQueryMap(): Map<String, String?> = mapOf(
    "category" to category?.apiValue(),
    "q" to query,
    "sort" to sort.apiValue(),
    "limit" to limit.toString(),
    "cursor" to cursor,
)
