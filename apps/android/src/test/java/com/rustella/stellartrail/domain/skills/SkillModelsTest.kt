package com.rustella.stellartrail.domain.skills

import com.rustella.stellartrail.core.network.ApiClient
import kotlinx.serialization.decodeFromString
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SkillModelsTest {
    @Test
    fun knotDetailDecodesBackendResponseWithoutHref() {
        val detail = ApiClient.defaultJson.decodeFromString<KnotDetail>(
            """
            {
              "id": "adjustable-grip-hitch-knot",
              "slug": "adjustable-grip-hitch-knot",
              "title": "可调节绳结",
              "summary": "调节绳索上的张力。",
              "aliases": [],
              "description": null,
              "steps": ["绕过固定点", "收紧绳圈"],
              "categories": [],
              "types": [],
              "media": [],
              "locale": "zh-CN"
            }
            """.trimIndent(),
        )

        assertEquals("adjustable-grip-hitch-knot", detail.id)
        assertEquals("可调节绳结", detail.title)
        assertEquals(SkillLocale.ZH_CN, detail.locale)
        assertNull(detail.href)
    }

    @Test
    fun favoriteKnotStatusDecodesBackendResponse() {
        val status = ApiClient.defaultJson.decodeFromString<FavoriteKnotStatusResponse>(
            """
            {
              "skill_category": "knots",
              "knot_id": "adjustable-grip-hitch-knot",
              "is_favorited": true,
              "favorited_at": "2026-05-01T00:00:00Z"
            }
            """.trimIndent(),
        )

        assertEquals("knots", status.skillCategory)
        assertEquals("adjustable-grip-hitch-knot", status.knotId)
        assertEquals(true, status.isFavorited)
        assertEquals("2026-05-01T00:00:00Z", status.favoritedAt)
    }
}
