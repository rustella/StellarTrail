package com.rustella.stellartrail.data.gear

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.ListGearTemplatesResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest
import com.rustella.stellartrail.domain.gear.apiValue

class GearApi(private val apiClient: ApiClient) {
    suspend fun listTemplates(): ListGearTemplatesResponse = apiClient.get("/gear-templates")

    suspend fun listCategories(tab: GearTab): GearCategoriesResponse =
        apiClient.get("/me/gears/categories", query = mapOf("tab" to tab.apiValue()))

    suspend fun stats(tab: GearTab): GearStatsResponse =
        apiClient.get("/me/gears/stats", query = mapOf("tab" to tab.apiValue()))

    suspend fun list(request: ListGearsRequest): ListGearsResponse = apiClient.get(
        "/me/gears",
        query = mapOf(
            "tab" to request.tab.apiValue(),
            "category" to request.category?.apiValue(),
            "status" to request.status?.apiValue(),
            "deleted" to request.deleted.apiValue(),
            "q" to request.query,
            "sort" to request.sort.apiValue(),
            "limit" to request.limit.toString(),
            "cursor" to request.cursor,
        ),
    )

    suspend fun get(id: String): GearItem = apiClient.get("/me/gears/$id")

    suspend fun create(request: CreateGearRequest): GearItem = apiClient.post("/me/gears", request)

    suspend fun update(id: String, request: UpdateGearRequest): GearItem = apiClient.patch("/me/gears/$id", request)

    suspend fun archive(id: String) {
        apiClient.delete("/me/gears/$id")
    }

    suspend fun delete(id: String) {
        apiClient.post("/me/gears/$id/delete", EmptyRequest)
    }

    suspend fun undelete(id: String): GearItem = apiClient.post("/me/gears/$id/undelete", EmptyRequest)

    suspend fun restore(id: String): GearItem = apiClient.post("/me/gears/$id/restore", EmptyRequest)

    @kotlinx.serialization.Serializable
    private object EmptyRequest
}
