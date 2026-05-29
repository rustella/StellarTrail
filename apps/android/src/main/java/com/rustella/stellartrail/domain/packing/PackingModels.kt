package com.rustella.stellartrail.domain.packing

import com.rustella.stellartrail.domain.gear.GearCategory
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class GearPackingListStats(
    @SerialName("total_items") val totalItems: Int = 0,
    @SerialName("packed_items") val packedItems: Int = 0,
    @SerialName("total_weight_g") val totalWeightG: Int = 0,
    @SerialName("packed_weight_g") val packedWeightG: Int = 0,
)

@Serializable
data class GearPackingListSummary(
    val id: String,
    val title: String,
    val description: String? = null,
    @SerialName("target_date") val targetDate: String? = null,
    @SerialName("source_trip_id") val sourceTripId: String? = null,
    @SerialName("trip_type") val tripType: String? = null,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
    @SerialName("total_items") val totalItems: Int = 0,
    @SerialName("packed_items") val packedItems: Int = 0,
    @SerialName("total_weight_g") val totalWeightG: Int = 0,
    @SerialName("packed_weight_g") val packedWeightG: Int = 0,
)

@Serializable
data class ListGearPackingListsResponse(
    val items: List<GearPackingListSummary> = emptyList(),
    @SerialName("next_cursor") val nextCursor: String? = null,
)

data class ListGearPackingListsRequest(
    val limit: Int = 20,
    val cursor: String? = null,
)

@Serializable
data class CreateGearPackingListRequest(
    val title: String,
    val description: String? = null,
    @SerialName("target_date") val targetDate: String? = null,
)

@Serializable
data class AddGearPackingItemsRequest(
    @SerialName("gear_ids") val gearIds: List<String>,
)

@Serializable
data class UpdateGearPackingItemRequest(
    @SerialName("packed_quantity") val packedQuantity: Int,
)

@Serializable
data class GearPackingListItem(
    val id: String,
    @SerialName("gear_id") val gearId: String,
    val category: GearCategory,
    @SerialName("category_label") val categoryLabel: String,
    val name: String,
    @SerialName("planned_quantity") val plannedQuantity: Int = 1,
    @SerialName("packed_quantity") val packedQuantity: Int = 0,
    @SerialName("unit_weight_g") val unitWeightG: Int? = null,
    @SerialName("unavailable_reason") val unavailableReason: String? = null,
)

@Serializable
data class GearPackingListDetail(
    val id: String,
    val title: String,
    val description: String? = null,
    @SerialName("target_date") val targetDate: String? = null,
    val stats: GearPackingListStats = GearPackingListStats(),
    val items: List<GearPackingListItem> = emptyList(),
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)
