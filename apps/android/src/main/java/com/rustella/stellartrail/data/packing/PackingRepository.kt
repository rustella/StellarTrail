package com.rustella.stellartrail.data.packing

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.packing.AddGearPackingItemsRequest
import com.rustella.stellartrail.domain.packing.CreateGearPackingListRequest
import com.rustella.stellartrail.domain.packing.GearPackingListDetail
import com.rustella.stellartrail.domain.packing.ListGearPackingListsRequest
import com.rustella.stellartrail.domain.packing.ListGearPackingListsResponse
import com.rustella.stellartrail.domain.packing.UpdateGearPackingItemRequest

class PackingApi(private val apiClient: ApiClient) {
    suspend fun list(request: ListGearPackingListsRequest): ListGearPackingListsResponse = apiClient.get(
        "/me/packing-lists",
        query = mapOf(
            "limit" to request.limit.toString(),
            "cursor" to request.cursor,
        ),
    )

    suspend fun create(request: CreateGearPackingListRequest): GearPackingListDetail =
        apiClient.post("/me/packing-lists", request)

    suspend fun get(id: String): GearPackingListDetail = apiClient.get("/me/packing-lists/$id")

    suspend fun update(id: String, request: CreateGearPackingListRequest): GearPackingListDetail =
        apiClient.patch("/me/packing-lists/$id", request)

    suspend fun delete(id: String) {
        apiClient.delete("/me/packing-lists/$id")
    }

    suspend fun addItems(id: String, request: AddGearPackingItemsRequest): GearPackingListDetail =
        apiClient.post("/me/packing-lists/$id/items", request)

    suspend fun updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest): GearPackingListDetail =
        apiClient.patch("/me/packing-lists/$id/items/$itemId", request)

    suspend fun removeItem(id: String, itemId: String): GearPackingListDetail =
        apiClient.deleteReturning("/me/packing-lists/$id/items/$itemId")
}

interface PackingRepositoryContract {
    suspend fun list(request: ListGearPackingListsRequest = ListGearPackingListsRequest()): ListGearPackingListsResponse
    suspend fun create(request: CreateGearPackingListRequest): GearPackingListDetail
    suspend fun get(id: String): GearPackingListDetail
    suspend fun update(id: String, request: CreateGearPackingListRequest): GearPackingListDetail
    suspend fun delete(id: String)
    suspend fun addItems(id: String, request: AddGearPackingItemsRequest): GearPackingListDetail
    suspend fun updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest): GearPackingListDetail
    suspend fun removeItem(id: String, itemId: String): GearPackingListDetail
}

class PackingRepository(private val api: PackingApi) : PackingRepositoryContract {
    override suspend fun list(request: ListGearPackingListsRequest): ListGearPackingListsResponse = api.list(request)
    override suspend fun create(request: CreateGearPackingListRequest): GearPackingListDetail = api.create(request)
    override suspend fun get(id: String): GearPackingListDetail = api.get(id)
    override suspend fun update(id: String, request: CreateGearPackingListRequest): GearPackingListDetail = api.update(id, request)
    override suspend fun delete(id: String) = api.delete(id)
    override suspend fun addItems(id: String, request: AddGearPackingItemsRequest): GearPackingListDetail = api.addItems(id, request)
    override suspend fun updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest): GearPackingListDetail =
        api.updateItem(id, itemId, request)
    override suspend fun removeItem(id: String, itemId: String): GearPackingListDetail = api.removeItem(id, itemId)
}
