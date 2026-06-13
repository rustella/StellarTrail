package com.rustella.stellartrail.core.trail

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class TrailFileTypesTest {
    @Test
    fun knownMimeTypesResolveToCanonicalTrailTypes() {
        assertEquals(TrailFileType.GPX, resolveTrailFileType("track", "application/gpx+xml"))
        assertEquals(TrailFileType.KML, resolveTrailFileType("route", "application/kml"))
        assertEquals(TrailFileType.FIT, resolveTrailFileType("activity", "application/fit"))
    }

    @Test
    fun genericMimeUsesSupportedFilenameExtension() {
        val fileType = resolveTrailFileType("two-step-track.KML", "*/*")

        assertEquals(TrailFileType.KML, fileType)
        assertEquals("two-step-track.KML", canonicalTrailFilename("two-step-track.KML", fileType!!))
        assertEquals("application/vnd.google-earth.kml+xml", fileType.canonicalContentType)
    }

    @Test
    fun canonicalFilenameAddsExtensionWhenOnlyMimeTypeIdentifiesTrail() {
        val fileType = resolveTrailFileType("shared-route", "application/x-gpx+xml")

        assertEquals(TrailFileType.GPX, fileType)
        assertEquals("shared-route.gpx", canonicalTrailFilename("shared-route", fileType!!))
    }

    @Test
    fun kmzAndUnidentifiedGenericFilesAreRejected() {
        assertNull(resolveTrailFileType("route.kmz", "application/vnd.google-earth.kmz"))
        assertNull(resolveTrailFileType("document.pdf", "*/*"))
        assertNull(resolveTrailFileType("shared-route", "*/*"))
    }
}
