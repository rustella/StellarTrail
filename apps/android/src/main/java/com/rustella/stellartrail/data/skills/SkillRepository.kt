package com.rustella.stellartrail.data.skills

import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.StateFlow

interface SkillRepositoryContract {
    val knotCacheStatus: StateFlow<KnotCacheStatus>
    suspend fun listSkills(locale: SkillLocale = SkillLocale.ZH_CN): SkillCategoriesResponse
    suspend fun listKnots(locale: SkillLocale = SkillLocale.ZH_CN, request: ListKnotsRequest = ListKnotsRequest()): KnotListResponse
    suspend fun knotDetail(id: String, locale: SkillLocale = SkillLocale.ZH_CN): KnotDetail
    suspend fun listFavoriteSkills(
        locale: SkillLocale = SkillLocale.ZH_CN,
        request: ListFavoriteSkillsRequest = ListFavoriteSkillsRequest(),
    ): ListFavoriteSkillsResponse
    suspend fun cacheAllKnots(locale: SkillLocale = SkillLocale.ZH_CN): KnotCacheStatus
    suspend fun clearKnotCache(): KnotCacheStatus

    fun resolveMediaUrl(pathOrUrl: String): String
}

class SkillRepository(
    private val api: SkillApi,
    private val cacheStore: KnotCacheStore = InMemoryKnotCacheStore(),
) : SkillRepositoryContract {
    override val knotCacheStatus: StateFlow<KnotCacheStatus> = cacheStore.status

    override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = api.listSkills(locale)

    override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse =
        runCatching { api.listKnots(locale, request) }.getOrElse { throwable ->
            cachedKnotList(locale, request).takeIf { it.items.isNotEmpty() } ?: throw throwable
        }

    override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail =
        runCatching { api.knotDetail(id, locale) }.getOrElse { throwable ->
            cacheStore.findDetail(id, locale) ?: throw throwable
        }

    override suspend fun listFavoriteSkills(locale: SkillLocale, request: ListFavoriteSkillsRequest): ListFavoriteSkillsResponse =
        api.listFavoriteSkills(locale, request)

    override suspend fun cacheAllKnots(locale: SkillLocale): KnotCacheStatus {
        val summaries = mutableListOf<KnotSummary>()
        val visitedOffsets = mutableSetOf<Int>()
        var offset: Int? = 0
        while (offset != null && visitedOffsets.add(offset)) {
            val response = api.listKnots(
                locale = locale,
                request = ListKnotsRequest(offset = offset, limit = KNOT_CACHE_PAGE_LIMIT),
            )
            summaries += response.items
            offset = response.page.nextOffset
        }
        val details = summaries
            .distinctBy { it.id }
            .map { summary -> api.knotDetail(summary.id, locale) }
        return cacheStore.save(locale, details)
    }

    override suspend fun clearKnotCache(): KnotCacheStatus = cacheStore.clear()

    override fun resolveMediaUrl(pathOrUrl: String): String = api.resolveMediaUrl(pathOrUrl)

    private suspend fun cachedKnotList(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse {
        val limit = request.limit.coerceAtLeast(1)
        val offset = request.offset.coerceAtLeast(0)
        val filtered = cacheStore.readDetails(locale)
            .asSequence()
            .filter { detail -> detail.matchesCategory(request.category) }
            .filter { detail -> detail.matchesQuery(request.query) }
            .map { detail -> detail.toSummary() }
            .toList()
        val items = filtered.drop(offset).take(limit)
        val nextOffset = (offset + items.size).takeIf { it < filtered.size }
        return KnotListResponse(
            locale = locale,
            items = items,
            page = PageInfo(limit = limit, offset = offset, nextOffset = nextOffset),
        )
    }

    private fun KnotDetail.toSummary(): KnotSummary =
        KnotSummary(
            id = id,
            slug = slug,
            title = title,
            summary = summary,
            categories = categories,
            types = types,
            media = media,
            href = href ?: "$KNOT_DETAIL_PATH_PREFIX$id",
            aliases = aliases,
        )

    private fun KnotDetail.matchesCategory(category: String?): Boolean {
        val needle = category?.trim()?.takeIf { it.isNotEmpty() } ?: return true
        return categories.any { item ->
            item.id == needle || item.slug == needle || item.title == needle
        }
    }

    private fun KnotDetail.matchesQuery(query: String?): Boolean {
        val needle = query?.trim()?.lowercase()?.takeIf { it.isNotEmpty() } ?: return true
        return title.lowercase().contains(needle) ||
            summary.lowercase().contains(needle) ||
            aliases.any { alias -> alias.lowercase().contains(needle) }
    }

    private companion object {
        const val KNOT_CACHE_PAGE_LIMIT = 100
        const val KNOT_DETAIL_PATH_PREFIX = "/api/v1/skills/knots/detail/"
    }
}
