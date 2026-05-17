package com.rustella.stellartrail.data.gear

import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest

interface GearRepositoryContract {
    suspend fun listCategories(tab: GearTab): GearCategoriesResponse
    suspend fun stats(tab: GearTab): GearStatsResponse
    suspend fun list(request: ListGearsRequest): ListGearsResponse
    suspend fun get(id: String): GearItem
    suspend fun create(request: CreateGearRequest): GearItem
    suspend fun update(id: String, request: UpdateGearRequest): GearItem
    suspend fun archive(id: String)
    suspend fun restore(id: String): GearItem
}

class GearRepository(private val api: GearApi) : GearRepositoryContract {
    override suspend fun listCategories(tab: GearTab): GearCategoriesResponse = api.listCategories(tab)
    override suspend fun stats(tab: GearTab): GearStatsResponse = api.stats(tab)
    override suspend fun list(request: ListGearsRequest): ListGearsResponse = api.list(request)
    override suspend fun get(id: String): GearItem = api.get(id)
    override suspend fun create(request: CreateGearRequest): GearItem = api.create(request)
    override suspend fun update(id: String, request: UpdateGearRequest): GearItem = api.update(id, request)
    override suspend fun archive(id: String) = api.archive(id)
    override suspend fun restore(id: String): GearItem = api.restore(id)
}
