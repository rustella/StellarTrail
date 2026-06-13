package com.rustella.stellartrail.core.trail

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns

data class TrailUploadFile(
    val filename: String,
    val contentType: String?,
    val bytes: ByteArray,
)

fun readTrailUpload(context: Context, uri: Uri): TrailUploadFile? = runCatching {
    val resolver = context.contentResolver
    val bytes = resolver.openInputStream(uri)?.use { it.readBytes() } ?: return@runCatching null
    val rawContentType = resolver.getType(uri)
    val rawFilename = resolver.query(uri, null, null, null, null)?.use { cursor ->
        val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
    }?.takeIf { it.isNotBlank() }
        ?: uri.lastPathSegment?.substringAfterLast('/')?.takeIf { it.isNotBlank() }
        ?: "trail"
    val fileType = resolveTrailFileType(rawFilename, rawContentType) ?: return@runCatching null
    TrailUploadFile(
        filename = canonicalTrailFilename(rawFilename, fileType),
        contentType = fileType.canonicalContentType,
        bytes = bytes,
    )
}.getOrNull()

val trailDocumentMimeTypes: Array<String> = arrayOf(
    "application/gpx+xml",
    "application/gpx",
    "application/x-gpx+xml",
    "text/gpx+xml",
    "application/vnd.google-earth.kml+xml",
    "application/kml",
    "application/x-kml",
    "application/vnd.ant.fit",
    "application/fit",
    "application/x-fit",
    "application/octet-stream",
    "application/xml",
    "text/xml",
)
