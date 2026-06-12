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
    val contentType = resolver.getType(uri)
    val filename = resolver.query(uri, null, null, null, null)?.use { cursor ->
        val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
    }?.takeIf { it.isNotBlank() }
        ?: uri.lastPathSegment?.substringAfterLast('/')?.takeIf { it.isNotBlank() }
        ?: "trail"
    TrailUploadFile(filename = filename, contentType = contentType, bytes = bytes)
}.getOrNull()

val trailDocumentMimeTypes: Array<String> = arrayOf(
    "application/gpx+xml",
    "application/vnd.google-earth.kml+xml",
    "application/vnd.ant.fit",
    "application/octet-stream",
    "application/xml",
    "text/xml",
)
