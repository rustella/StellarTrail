package com.rustella.stellartrail.data.skills

import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillLocale

interface SkillRepositoryContract {
    suspend fun listSkills(locale: SkillLocale = SkillLocale.ZH_CN): SkillCategoriesResponse
    suspend fun listKnots(locale: SkillLocale = SkillLocale.ZH_CN, request: ListKnotsRequest = ListKnotsRequest()): KnotListResponse
    suspend fun knotDetail(id: String, locale: SkillLocale = SkillLocale.ZH_CN): KnotDetail
    fun resolveMediaUrl(pathOrUrl: String): String
}

class SkillRepository(private val api: SkillApi) : SkillRepositoryContract {
    override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = api.listSkills(locale)
    override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse = api.listKnots(locale, request)
    override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = api.knotDetail(id, locale)
    override fun resolveMediaUrl(pathOrUrl: String): String = api.resolveMediaUrl(pathOrUrl)
}
