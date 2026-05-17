package com.rustella.stellartrail.domain.gear

import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Test

class GearTemplateModelsTest {
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false }

    @Test
    fun parsesPublicGearTemplateResponseFromWechatContract() {
        val response = json.decodeFromString<ListGearTemplatesResponse>(
            """
            {
              "items": [
                {
                  "id": "weekend-hike",
                  "title": "周末轻徒步清单",
                  "categories": [
                    {"id": "carry", "name": "背负与收纳", "items": ["20L 背包", "收纳袋"]},
                    {"id": "weather", "name": "天气应对", "items": ["雨衣", "头灯"]}
                  ]
                }
              ]
            }
            """.trimIndent(),
        )

        assertEquals("weekend-hike", response.items.single().id)
        assertEquals("周末轻徒步清单", response.items.single().title)
        assertEquals("背负与收纳", response.items.single().categories.first().name)
        assertEquals(listOf("20L 背包", "收纳袋"), response.items.single().categories.first().items)
    }
}
