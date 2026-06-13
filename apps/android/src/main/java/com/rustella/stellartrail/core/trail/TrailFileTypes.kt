package com.rustella.stellartrail.core.trail

internal enum class TrailFileType(
    val extension: String,
    val canonicalContentType: String,
    private val acceptedContentTypes: Set<String>,
) {
    GPX(
        extension = "gpx",
        canonicalContentType = "application/gpx+xml",
        acceptedContentTypes = setOf(
            "application/gpx+xml",
            "application/gpx",
            "application/x-gpx+xml",
            "text/gpx+xml",
        ),
    ),
    KML(
        extension = "kml",
        canonicalContentType = "application/vnd.google-earth.kml+xml",
        acceptedContentTypes = setOf(
            "application/vnd.google-earth.kml+xml",
            "application/kml",
            "application/x-kml",
        ),
    ),
    FIT(
        extension = "fit",
        canonicalContentType = "application/vnd.ant.fit",
        acceptedContentTypes = setOf(
            "application/vnd.ant.fit",
            "application/fit",
            "application/x-fit",
        ),
    );

    fun acceptsContentType(contentType: String): Boolean = acceptedContentTypes.contains(contentType)

    companion object {
        fun fromExtension(extension: String): TrailFileType? = when (extension.lowercase()) {
            GPX.extension -> GPX
            KML.extension -> KML
            FIT.extension -> FIT
            else -> null
        }

        fun fromContentType(contentType: String): TrailFileType? =
            entries.firstOrNull { it.acceptsContentType(contentType.normalizedMimeType()) }
    }
}

internal fun resolveTrailFileType(filename: String?, contentType: String?): TrailFileType? {
    val extension = filename?.trailExtension()
    if (extension == "kmz" || contentType.normalizedMimeType() == "application/vnd.google-earth.kmz") {
        return null
    }
    return extension?.let(TrailFileType::fromExtension)
        ?: contentType.normalizedMimeType().takeIf { it.isNotBlank() }?.let(TrailFileType::fromContentType)
}

internal fun isGenericTrailContentType(contentType: String?): Boolean =
    contentType.normalizedMimeType() in setOf("*/*", "application/octet-stream")

internal fun canonicalTrailFilename(filename: String, fileType: TrailFileType): String =
    if (TrailFileType.fromExtension(filename.trailExtension().orEmpty()) == fileType) {
        filename
    } else {
        "$filename.${fileType.extension}"
    }

private fun String?.normalizedMimeType(): String =
    this?.substringBefore(';')?.trim()?.lowercase().orEmpty()

private fun String.trailExtension(): String? =
    substringAfterLast('/', this)
        .substringBefore('?')
        .substringBefore('#')
        .substringAfterLast('.', missingDelimiterValue = "")
        .lowercase()
        .takeIf { it.isNotBlank() }
