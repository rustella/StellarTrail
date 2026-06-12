package com.rustella.stellartrail.core.trail

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.io.File
import java.util.UUID

@Serializable
data class PendingTrailImport(
    val id: String,
    val filename: String,
    val contentType: String? = null,
    val sizeBytes: Long = 0L,
    val cachePath: String,
)

interface PendingTrailImportStore {
    fun createFromIntent(intent: Intent): PendingTrailImport?
    fun get(id: String): PendingTrailImport?
    fun readBytes(id: String): ByteArray?
    fun clear(id: String)
}

class AndroidPendingTrailImportStore(context: Context) : PendingTrailImportStore {
    private val appContext = context.applicationContext
    private val importsDir = File(appContext.cacheDir, "trail-imports")
    private val json = Json { ignoreUnknownKeys = true }

    override fun createFromIntent(intent: Intent): PendingTrailImport? {
        val uri = intent.trailImportUri() ?: return null
        val contentType = intent.type?.takeIf { it.isNotBlank() } ?: appContext.contentResolver.getType(uri)
        val filename = filenameFor(uri, contentType)
        val size = sizeFor(uri)
        val id = UUID.randomUUID().toString()
        importsDir.mkdirs()
        val cacheFile = File(importsDir, "$id-${filename.sanitizeFilename()}")
        appContext.contentResolver.openInputStream(uri)?.use { input ->
            cacheFile.outputStream().use { output -> input.copyTo(output) }
        } ?: return null
        val import = PendingTrailImport(
            id = id,
            filename = filename,
            contentType = contentType,
            sizeBytes = size.takeIf { it > 0 } ?: cacheFile.length(),
            cachePath = cacheFile.absolutePath,
        )
        metadataFile(id).writeText(json.encodeToString(PendingTrailImport.serializer(), import))
        return import
    }

    override fun get(id: String): PendingTrailImport? = runCatching {
        val file = metadataFile(id)
        if (!file.exists()) return null
        json.decodeFromString(PendingTrailImport.serializer(), file.readText())
            .takeIf { File(it.cachePath).exists() }
    }.getOrNull()

    override fun readBytes(id: String): ByteArray? = get(id)?.let { File(it.cachePath).takeIf(File::exists)?.readBytes() }

    override fun clear(id: String) {
        get(id)?.let { runCatching { File(it.cachePath).delete() } }
        runCatching { metadataFile(id).delete() }
    }

    private fun filenameFor(uri: Uri, contentType: String?): String {
        val displayName = appContext.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
        }?.takeIf { it.isNotBlank() }
        val rawName = displayName
            ?: uri.lastPathSegment?.substringAfterLast('/')?.takeIf { it.isNotBlank() }
            ?: "imported-trail"
        return rawName.ensureTrailExtension(contentType)
    }

    private fun sizeFor(uri: Uri): Long = appContext.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
        val index = cursor.getColumnIndex(OpenableColumns.SIZE)
        if (index >= 0 && cursor.moveToFirst()) cursor.getLong(index) else 0L
    } ?: 0L

    private fun metadataFile(id: String): File = File(importsDir, "$id.json")
}

class InMemoryPendingTrailImportStore : PendingTrailImportStore {
    private val imports = mutableMapOf<String, Pair<PendingTrailImport, ByteArray>>()

    override fun createFromIntent(intent: Intent): PendingTrailImport? = null

    fun put(filename: String, contentType: String?, bytes: ByteArray): PendingTrailImport {
        val id = UUID.randomUUID().toString()
        val import = PendingTrailImport(id, filename, contentType, bytes.size.toLong(), "memory:$id")
        imports[id] = import to bytes
        return import
    }

    override fun get(id: String): PendingTrailImport? = imports[id]?.first
    override fun readBytes(id: String): ByteArray? = imports[id]?.second
    override fun clear(id: String) {
        imports.remove(id)
    }
}

private fun Intent.trailImportUri(): Uri? = when (action) {
    Intent.ACTION_SEND -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
    } else {
        @Suppress("DEPRECATION")
        getParcelableExtra(Intent.EXTRA_STREAM)
    }
    Intent.ACTION_VIEW -> data
    else -> null
}

private fun String.ensureTrailExtension(contentType: String?): String {
    val lower = lowercase()
    if (lower.endsWith(".gpx") || lower.endsWith(".kml") || lower.endsWith(".fit")) return this
    val extension = when (contentType?.lowercase()) {
        "application/gpx+xml" -> "gpx"
        "application/vnd.google-earth.kml+xml" -> "kml"
        "application/vnd.ant.fit" -> "fit"
        else -> null
    }
    return if (extension == null) this else "$this.$extension"
}

private fun String.sanitizeFilename(): String =
    replace(Regex("[^A-Za-z0-9._-]+"), "-").trim('-').ifBlank { "trail" }
