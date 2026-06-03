package com.rustella.stellartrail.ui.screens

import org.junit.Assert.assertEquals
import org.junit.Test

class SkillsVisualContractTest {
    @Test
    fun skillsCatalogMatchesMiniProgramEntryCards() {
        assertEquals("寻径星野技能库", SkillsVisualContract.heroEyebrow)
        assertEquals("户外技能", SkillsVisualContract.heroTitle)
        assertEquals("收藏清单", SkillsVisualContract.favoriteTitle)
        assertEquals("快速找到已经收藏的户外技能", SkillsVisualContract.favoriteDescription)
        assertEquals("查看 >", SkillsVisualContract.favoriteAction)
    }

    @Test
    fun catalogKeepsSingleKnotCategoryEntry() {
        val categories = SkillsVisualContract.catalogCategories

        assertEquals(1, categories.size)
        assertEquals("knots", categories.single().id)
        assertEquals("Knots", categories.single().subtitle)
        assertEquals("绳结", categories.single().title)
        assertEquals("查看绳结列表", categories.single().actionText)
        assertEquals("knot", categories.single().icon)
        assertEquals(44, SkillsVisualContract.categoryIconBoxDp)
        assertEquals(30, SkillsVisualContract.knotIconGraphicDp)
    }
}
