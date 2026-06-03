package com.rustella.stellartrail.domain.skills

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class SkillLocale(val headerValue: String) {
    @SerialName("zh-CN") ZH_CN("zh-CN"),
    @SerialName("en") EN("en"),
}

@Serializable
data class SkillCategorySummary(
    val id: String,
    val slug: String,
    val title: String,
    val summary: String,
    @SerialName("item_count") val itemCount: Int,
    val href: String,
)

@Serializable
data class SkillCategoriesResponse(
    val items: List<SkillCategorySummary>,
)

@Serializable
data class PageInfo(
    val limit: Int,
    val offset: Int,
    @SerialName("next_offset") val nextOffset: Int? = null,
)

@Serializable
data class KnotTaxonomyItem(
    val id: String,
    val slug: String,
    val title: String,
)

@Serializable
data class KnotMediaAsset(
    val id: String,
    @SerialName("media_type") val mediaType: String,
    val url: String,
    @SerialName("mime_type") val mimeType: String,
    val width: Int? = null,
    val height: Int? = null,
    val attribution: String? = null,
    @SerialName("license_note") val licenseNote: String? = null,
)

@Serializable
data class KnotSummary(
    val id: String,
    val slug: String,
    val title: String,
    val summary: String,
    val categories: List<KnotTaxonomyItem> = emptyList(),
    val types: List<KnotTaxonomyItem> = emptyList(),
    val media: List<KnotMediaAsset> = emptyList(),
    val href: String,
    val aliases: List<String> = emptyList(),
)

@Serializable
data class KnotListResponse(
    val locale: SkillLocale,
    val items: List<KnotSummary>,
    val page: PageInfo,
)

@Serializable
enum class FavoriteSkillCategory(val queryValue: String) {
    @SerialName("all") ALL("all"),
    @SerialName("knots") KNOTS("knots"),
}

@Serializable
data class FavoriteSkillFilterOption(
    val id: FavoriteSkillCategory,
    val title: String,
    val count: Int,
)

@Serializable
data class FavoriteKnotItem(
    @SerialName("skill_category") val skillCategory: String,
    @SerialName("favorited_at") val favoritedAt: String,
    val knot: KnotSummary,
)

@Serializable
data class ListFavoriteSkillsResponse(
    val locale: SkillLocale,
    val filters: List<FavoriteSkillFilterOption> = emptyList(),
    val items: List<FavoriteKnotItem>,
    val page: PageInfo,
)

@Serializable
data class FavoriteKnotStatusResponse(
    @SerialName("skill_category") val skillCategory: String,
    @SerialName("knot_id") val knotId: String,
    @SerialName("is_favorited") val isFavorited: Boolean,
    @SerialName("favorited_at") val favoritedAt: String? = null,
)

data class ListFavoriteSkillsRequest(
    val skillCategory: FavoriteSkillCategory = FavoriteSkillCategory.ALL,
    val offset: Int = 0,
    val limit: Int = 20,
)

@Serializable
data class KnotDetail(
    val id: String,
    val slug: String,
    val title: String,
    val summary: String,
    val categories: List<KnotTaxonomyItem> = emptyList(),
    val types: List<KnotTaxonomyItem> = emptyList(),
    val media: List<KnotMediaAsset> = emptyList(),
    val href: String? = null,
    val description: String? = null,
    val steps: List<String> = emptyList(),
    val locale: SkillLocale,
    val aliases: List<String> = emptyList(),
)

data class ListKnotsRequest(
    val offset: Int = 0,
    val limit: Int = 20,
    val category: String? = null,
    val query: String? = null,
)

fun resolveMediaUrl(assetsBaseUrl: String, mediaUrl: String): String {
    if (mediaUrl.startsWith("http://") || mediaUrl.startsWith("https://")) return mediaUrl
    if (
        mediaUrl.startsWith("android.resource://") ||
        mediaUrl.startsWith("content://") ||
        mediaUrl.startsWith("file://")
    ) {
        return mediaUrl
    }
    return assetsBaseUrl.trimEnd('/') + "/" + mediaUrl.trimStart('/')
}

fun List<KnotMediaAsset>.preferredThumbnailUrl(): String? =
    firstOrNull { it.mediaType == "thumbnail" }?.url
        ?: firstOrNull { it.mediaType == "preview" }?.url
        ?: firstOrNull { it.mimeType.startsWith("image/") }?.url

fun List<KnotMediaAsset>.preferredPreviewUrl(): String? =
    firstOrNull { it.mediaType == "preview" }?.url
        ?: firstOrNull { it.mediaType == "thumbnail" }?.url
        ?: firstOrNull { it.mimeType.startsWith("image/") }?.url
