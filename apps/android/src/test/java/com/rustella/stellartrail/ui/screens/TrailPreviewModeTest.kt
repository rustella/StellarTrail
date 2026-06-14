package com.rustella.stellartrail.ui.screens

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class TrailPreviewModeTest {
    @Test
    fun previewDefaultsToFlatMapWithTerrainAsNext3dContent() {
        val state = defaultTrailMapPreviewState()

        assertEquals(TrailMapPreviewMode.FlatMap, state.mode)
        assertEquals(TrailMap3dContent.Terrain, state.content3d)
    }

    @Test
    fun entering3dAlwaysStartsOnTerrainMap() {
        val state = selectTrailMap3dContent(defaultTrailMapPreviewState(), TrailMap3dContent.Track)

        val next = enterTrailMapPreview3d(state)

        assertEquals(TrailMapPreviewMode.Map3d, next.mode)
        assertEquals(TrailMap3dContent.Terrain, next.content3d)
    }

    @Test
    fun selectingTrackKeepsPreviewIn3dMode() {
        val next = selectTrailMap3dContent(defaultTrailMapPreviewState(), TrailMap3dContent.Track)

        assertEquals(TrailMapPreviewMode.Map3d, next.mode)
        assertEquals(TrailMap3dContent.Track, next.content3d)
    }

    @Test
    fun exiting3dReturnsToFlatMap() {
        val state = selectTrailMap3dContent(defaultTrailMapPreviewState(), TrailMap3dContent.Track)

        val next = exitTrailMapPreview3d(state)

        assertEquals(TrailMapPreviewMode.FlatMap, next.mode)
        assertEquals(TrailMap3dContent.Track, next.content3d)
    }

    @Test
    fun previewHeaderSummaryDoesNotIncludePointCount() {
        val summary = trailPreviewHeaderSummary(distanceM = 12_360.0)

        assertEquals("12.4 km", summary)
        assertFalse(summary.contains("点"))
    }
}
