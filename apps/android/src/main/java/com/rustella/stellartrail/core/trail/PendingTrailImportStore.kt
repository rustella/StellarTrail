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
    fun createFromIntent(intent: Intent): TrailImportIntentResult
    fun get(id: String): PendingTrailImport?
    fun readBytes(id: String): ByteArray?
    fun clear(id: String)
}

sealed class TrailImportIntentResult {
    data class Created(val pending: PendingTrailImport) : TrailImportIntentResult()
    object Ignored : TrailImportIntentResult()
    object Unsupported : TrailImportIntentResult()
}

class AndroidPendingTrailImportStore(context: Context) : PendingTrailImportStore {
    private val appContext = context.applicationContext
    private val importsDir = File(appContext.cacheDir, "trail-imports")
    private val json = Json { ignoreUnknownKeys = true }

    override fun createFromIntent(intent: Intent): TrailImportIntentResult {
        val uris = intent.trailImportUris()
        if (uris.isEmpty()) return TrailImportIntentResult.Ignored
        return uris.firstNotNullOfOrNull { uri -> createFromUri(intent, uri) }
            ?.let(TrailImportIntentResult::Created)
            ?: TrailImportIntentResult.Unsupported
    }

    private fun createFromUri(intent: Intent, uri: Uri): PendingTrailImport? = runCatching {
        val intentContentType = intent.type?.takeIf { it.isNotBlank() }
        val resolverContentType = appContext.contentResolver.getType(uri)
        val rawContentType = if (isGenericTrailContentType(intentContentType)) {
            resolverContentType ?: intentContentType
        } else {
            intentContentType ?: resolverContentType
        }
        val rawFilename = filenameFor(uri)
        val fileType = resolveTrailFileType(rawFilename, rawContentType) ?: return@runCatching null
        val filename = canonicalTrailFilename(rawFilename, fileType)
        val size = sizeFor(uri)
        val id = UUID.randomUUID().toString()
        importsDir.mkdirs()
        val cacheFile = File(importsDir, "$id-${filename.sanitizeFilename()}")
        appContext.contentResolver.openInputStream(uri)?.use { input ->
            cacheFile.outputStream().use { output -> input.copyTo(output) }
        } ?: return@runCatching null
        val import = PendingTrailImport(
            id = id,
            filename = filename,
            contentType = fileType.canonicalContentType,
            sizeBytes = size.takeIf { it > 0 } ?: cacheFile.length(),
            cachePath = cacheFile.absolutePath,
        )
        metadataFile(id).writeText(json.encodeToString(PendingTrailImport.serializer(), import))
        import
    }.getOrNull()

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

    private fun filenameFor(uri: Uri): String {
        val displayName = runCatching {
            appContext.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
            }
        }.getOrNull()?.takeIf { it.isNotBlank() }
        return displayName
            ?: uri.lastPathSegment?.substringAfterLast('/')?.takeIf { it.isNotBlank() }
            ?: "imported-trail"
    }

    private fun sizeFor(uri: Uri): Long = runCatching {
        appContext.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.SIZE)
            if (index >= 0 && cursor.moveToFirst()) cursor.getLong(index) else 0L
        } ?: 0L
    }.getOrDefault(0L)

    private fun metadataFile(id: String): File = File(importsDir, "$id.json")
}

class InMemoryPendingTrailImportStore : PendingTrailImportStore {
    private val imports = mutableMapOf<String, Pair<PendingTrailImport, ByteArray>>()

    override fun createFromIntent(intent: Intent): TrailImportIntentResult = TrailImportIntentResult.Ignored

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

private fun Intent.trailImportUris(): List<Uri> {
    if (action !in setOf(Intent.ACTION_SEND, Intent.ACTION_SEND_MULTIPLE, Intent.ACTION_VIEW)) {
        return emptyList()
    }
    val uris = linkedSetOf<Uri>()
    if (action == Intent.ACTION_SEND || action == Intent.ACTION_SEND_MULTIPLE) {
        streamUris().forEach(uris::add)
    }
    data?.let(uris::add)
    clipData?.let { clip ->
        repeat(clip.itemCount) { index ->
            clip.getItemAt(index).uri?.let(uris::add)
        }
    }
    return uris.toList()
}

private fun Intent.streamUris(): List<Uri> = buildList {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)?.let(::add)
        getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java)?.let(::addAll)
    } else {
        @Suppress("DEPRECATION")
        getParcelableExtra<Uri>(Intent.EXTRA_STREAM)?.let(::add)
        @Suppress("DEPRECATION")
        getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM)?.let(::addAll)
    }
}

private fun String.sanitizeFilename(): String =
    replace(Regex("[^A-Za-z0-9._-]+"), "-").trim('-').ifBlank { "trail" }
