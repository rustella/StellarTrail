package com.rustella.stellartrail.data.skills

import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillLocale

class SkillApi(private val apiClient: ApiClient) {
    suspend fun listSkills(locale: SkillLocale = SkillLocale.ZH_CN): SkillCategoriesResponse =
        apiClient.get("/skills", locale = locale)

    suspend fun listKnots(locale: SkillLocale = SkillLocale.ZH_CN, request: ListKnotsRequest = ListKnotsRequest()): KnotListResponse =
        apiClient.get(
            "/skills/knots/list",
            query = mapOf(
                "offset" to request.offset.toString(),
                "limit" to request.limit.toString(),
                "category" to request.category,
                "q" to request.query,
            ),
            locale = locale,
        )

    suspend fun knotDetail(id: String, locale: SkillLocale = SkillLocale.ZH_CN): KnotDetail =
        apiClient.get("/skills/knots/detail/$id", locale = locale)

    fun resolveMediaUrl(pathOrUrl: String): String = apiClient.resolveAssetUrl(pathOrUrl)
}
