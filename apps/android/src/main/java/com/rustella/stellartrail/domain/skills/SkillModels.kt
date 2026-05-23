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
)

@Serializable
data class KnotListResponse(
    val locale: SkillLocale,
    val items: List<KnotSummary>,
    val page: PageInfo,
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
    val href: String,
    val description: String? = null,
    val steps: List<String> = emptyList(),
    val locale: SkillLocale,
)

data class ListKnotsRequest(
    val offset: Int = 0,
    val limit: Int = 20,
    val category: String? = null,
    val query: String? = null,
)

fun resolveMediaUrl(assetsBaseUrl: String, mediaUrl: String): String {
    if (mediaUrl.startsWith("http://") || mediaUrl.startsWith("https://")) return mediaUrl
    return assetsBaseUrl.trimEnd('/') + "/" + mediaUrl.trimStart('/')
}
