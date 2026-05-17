package com.rustella.stellartrail.data.gear

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest
import com.rustella.stellartrail.domain.gear.apiValue

class GearApi(private val apiClient: ApiClient) {
    suspend fun listCategories(tab: GearTab): GearCategoriesResponse =
        apiClient.get("/api/me/gears/categories", query = mapOf("tab" to tab.apiValue()))

    suspend fun stats(tab: GearTab): GearStatsResponse =
        apiClient.get("/api/me/gears/stats", query = mapOf("tab" to tab.apiValue()))

    suspend fun list(request: ListGearsRequest): ListGearsResponse = apiClient.get(
        "/api/me/gears",
        query = mapOf(
            "tab" to request.tab.apiValue(),
            "category" to request.category?.apiValue(),
            "status" to request.status?.apiValue(),
            "q" to request.query,
            "sort" to request.sort.apiValue(),
            "limit" to request.limit.toString(),
            "cursor" to request.cursor,
        ),
    )

    suspend fun get(id: String): GearItem = apiClient.get("/api/me/gears/$id")

    suspend fun create(request: CreateGearRequest): GearItem = apiClient.post("/api/me/gears", request)

    suspend fun update(id: String, request: UpdateGearRequest): GearItem = apiClient.patch("/api/me/gears/$id", request)

    suspend fun archive(id: String) {
        apiClient.delete("/api/me/gears/$id")
    }

    suspend fun restore(id: String): GearItem = apiClient.post("/api/me/gears/$id/restore", EmptyRequest)

    @kotlinx.serialization.Serializable
    private object EmptyRequest
}
