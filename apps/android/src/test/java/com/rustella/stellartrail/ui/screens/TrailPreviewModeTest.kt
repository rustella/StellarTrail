package com.rustella.stellartrail.ui.screens

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TrailPreviewModeTest {
    @Test
    fun previewDefaultsToFlatMap() {
        val state = defaultTrailMapPreviewState()

        assertEquals(TrailMapPreviewMode.FlatMap, state.mode)
        assertFalse(trailPreviewTerrainEnabled(state))
    }

    @Test
    fun entering3dEnablesTerrainMap() {
        val next = enterTrailMapPreview3d(defaultTrailMapPreviewState())

        assertEquals(TrailMapPreviewMode.Map3d, next.mode)
        assertTrue(trailPreviewTerrainEnabled(next))
    }

    @Test
    fun exiting3dReturnsToFlatMap() {
        val state = enterTrailMapPreview3d(defaultTrailMapPreviewState())

        val next = exitTrailMapPreview3d(state)

        assertEquals(TrailMapPreviewMode.FlatMap, next.mode)
        assertFalse(trailPreviewTerrainEnabled(next))
    }

    @Test
    fun previewHeaderSummaryDoesNotIncludePointCount() {
        val summary = trailPreviewHeaderSummary(distanceM = 12_360.0)

        assertEquals("12.4 km", summary)
        assertFalse(summary.contains("点"))
    }
}
