package com.rustella.stellartrail.domain.gear

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class GearCategory {
    @SerialName("backpack_system") BACKPACK_SYSTEM,
    @SerialName("sleep_system") SLEEP_SYSTEM,
    @SerialName("kitchen_system") KITCHEN_SYSTEM,
    @SerialName("walking_system") WALKING_SYSTEM,
    @SerialName("clothing_system") CLOTHING_SYSTEM,
    @SerialName("lighting_system") LIGHTING_SYSTEM,
    @SerialName("first_aid_system") FIRST_AID_SYSTEM,
    @SerialName("electronics_system") ELECTRONICS_SYSTEM,
    @SerialName("technical_gear") TECHNICAL_GEAR,
    @SerialName("other_gear") OTHER_GEAR,
    @SerialName("consumable") CONSUMABLE,
}

@Serializable
enum class GearStatus {
    @SerialName("available") AVAILABLE,
    @SerialName("in_use") IN_USE,
    @SerialName("maintenance") MAINTENANCE,
    @SerialName("damaged") DAMAGED,
    @SerialName("lost") LOST,
    @SerialName("retired") RETIRED,
    @SerialName("sold") SOLD,
    @SerialName("idle") IDLE,
}

@Serializable
enum class GearShareStatus {
    @SerialName("not_shared") NOT_SHARED,
    @SerialName("pending") PENDING,
    @SerialName("approved") APPROVED,
    @SerialName("rejected") REJECTED,
    @SerialName("withdrawn") WITHDRAWN,
}

@Serializable
enum class GearTab {
    @SerialName("available") AVAILABLE,
    @SerialName("history") HISTORY,
}

@Serializable
enum class GearSort {
    @SerialName("created_at_desc") CREATED_AT_DESC,
    @SerialName("created_at_asc") CREATED_AT_ASC,
    @SerialName("purchase_date_desc") PURCHASE_DATE_DESC,
    @SerialName("name_asc") NAME_ASC,
    @SerialName("weight_desc") WEIGHT_DESC,
    @SerialName("price_desc") PRICE_DESC,
}

@Serializable
data class GearItem(
    val id: String,
    @SerialName("user_id") val userId: String,
    val category: GearCategory,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val color: String? = null,
    val material: String? = null,
    val capacity: String? = null,
    val size: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("warmth_index") val warmthIndex: String? = null,
    @SerialName("waterproof_index") val waterproofIndex: String? = null,
    @SerialName("purchase_date") val purchaseDate: String? = null,
    @SerialName("purchase_price_cents") val purchasePriceCents: Long? = null,
    @SerialName("expiry_or_warranty_date") val expiryOrWarrantyDate: String? = null,
    @SerialName("purchase_location") val purchaseLocation: String? = null,
    val status: GearStatus,
    @SerialName("storage_location") val storageLocation: String? = null,
    val tags: List<String> = emptyList(),
    @SerialName("share_enabled") val shareEnabled: Boolean,
    @SerialName("share_status") val shareStatus: GearShareStatus,
    val notes: String? = null,
    @SerialName("archived_at") val archivedAt: String? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class GearSummary(
    val id: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val status: GearStatus,
    @SerialName("status_label") val statusLabel: String,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("purchase_price_cents") val purchasePriceCents: Long? = null,
    @SerialName("purchase_date") val purchaseDate: String? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class GearTemplateCategory(
    val id: String,
    val name: String,
    val items: List<String>,
)

@Serializable
data class GearTemplate(
    val id: String,
    val title: String,
    val categories: List<GearTemplateCategory>,
)

@Serializable
data class ListGearTemplatesResponse(
    val items: List<GearTemplate>,
)

@Serializable
data class CreateGearRequest(
    val category: GearCategory,
    val name: String,
    val brand: String? = null,
    val model: String? = null,
    val color: String? = null,
    val material: String? = null,
    val capacity: String? = null,
    val size: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("warmth_index") val warmthIndex: String? = null,
    @SerialName("waterproof_index") val waterproofIndex: String? = null,
    @SerialName("purchase_date") val purchaseDate: String? = null,
    @SerialName("purchase_price_cents") val purchasePriceCents: Long? = null,
    @SerialName("expiry_or_warranty_date") val expiryOrWarrantyDate: String? = null,
    @SerialName("purchase_location") val purchaseLocation: String? = null,
    val status: GearStatus? = null,
    @SerialName("storage_location") val storageLocation: String? = null,
    val tags: List<String>? = null,
    @SerialName("share_enabled") val shareEnabled: Boolean? = null,
    val notes: String? = null,
)

@Serializable
data class UpdateGearRequest(
    val category: GearCategory? = null,
    val name: String? = null,
    val brand: String? = null,
    val model: String? = null,
    val color: String? = null,
    val material: String? = null,
    val capacity: String? = null,
    val size: String? = null,
    val description: String? = null,
    @SerialName("weight_g") val weightG: Int? = null,
    @SerialName("warmth_index") val warmthIndex: String? = null,
    @SerialName("waterproof_index") val waterproofIndex: String? = null,
    @SerialName("purchase_date") val purchaseDate: String? = null,
    @SerialName("purchase_price_cents") val purchasePriceCents: Long? = null,
    @SerialName("expiry_or_warranty_date") val expiryOrWarrantyDate: String? = null,
    @SerialName("purchase_location") val purchaseLocation: String? = null,
    val status: GearStatus? = null,
    @SerialName("storage_location") val storageLocation: String? = null,
    val tags: List<String>? = null,
    @SerialName("share_enabled") val shareEnabled: Boolean? = null,
    val notes: String? = null,
)

@Serializable
data class GearCategoryCount(
    val category: GearCategory,
    val label: String,
    val count: Int,
)

@Serializable
data class GearStatusCount(
    val status: GearStatus,
    val label: String,
    val count: Int,
)

@Serializable
data class GearStatsResponse(
    @SerialName("current_count") val currentCount: Int,
    @SerialName("archived_count") val archivedCount: Int,
    @SerialName("total_value_cents") val totalValueCents: Long,
    @SerialName("total_weight_g") val totalWeightG: Int,
    @SerialName("by_category") val byCategory: List<GearCategoryCount> = emptyList(),
    @SerialName("by_status") val byStatus: List<GearStatusCount> = emptyList(),
)

@Serializable
data class GearCategoryFilter(
    val id: String,
    val label: String,
    val count: Int,
)

@Serializable
data class GearCategoriesResponse(
    val items: List<GearCategoryFilter>,
)

@Serializable
data class ListGearsResponse(
    val items: List<GearSummary>,
    @SerialName("next_cursor") val nextCursor: String? = null,
)

data class ListGearsRequest(
    val tab: GearTab = GearTab.AVAILABLE,
    val category: GearCategory? = null,
    val status: GearStatus? = null,
    val query: String? = null,
    val sort: GearSort = GearSort.CREATED_AT_DESC,
    val limit: Int = 20,
    val cursor: String? = null,
)

fun GearTab.apiValue(): String = when (this) {
    GearTab.AVAILABLE -> "available"
    GearTab.HISTORY -> "history"
}

fun GearSort.apiValue(): String = when (this) {
    GearSort.CREATED_AT_DESC -> "created_at_desc"
    GearSort.CREATED_AT_ASC -> "created_at_asc"
    GearSort.PURCHASE_DATE_DESC -> "purchase_date_desc"
    GearSort.NAME_ASC -> "name_asc"
    GearSort.WEIGHT_DESC -> "weight_desc"
    GearSort.PRICE_DESC -> "price_desc"
}

fun GearCategory.apiValue(): String = when (this) {
    GearCategory.BACKPACK_SYSTEM -> "backpack_system"
    GearCategory.SLEEP_SYSTEM -> "sleep_system"
    GearCategory.KITCHEN_SYSTEM -> "kitchen_system"
    GearCategory.WALKING_SYSTEM -> "walking_system"
    GearCategory.CLOTHING_SYSTEM -> "clothing_system"
    GearCategory.LIGHTING_SYSTEM -> "lighting_system"
    GearCategory.FIRST_AID_SYSTEM -> "first_aid_system"
    GearCategory.ELECTRONICS_SYSTEM -> "electronics_system"
    GearCategory.TECHNICAL_GEAR -> "technical_gear"
    GearCategory.OTHER_GEAR -> "other_gear"
    GearCategory.CONSUMABLE -> "consumable"
}

fun GearStatus.apiValue(): String = when (this) {
    GearStatus.AVAILABLE -> "available"
    GearStatus.IN_USE -> "in_use"
    GearStatus.MAINTENANCE -> "maintenance"
    GearStatus.DAMAGED -> "damaged"
    GearStatus.LOST -> "lost"
    GearStatus.RETIRED -> "retired"
    GearStatus.SOLD -> "sold"
    GearStatus.IDLE -> "idle"
}
