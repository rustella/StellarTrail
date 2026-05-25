package com.rustella.stellartrail.domain.common

import kotlinx.serialization.Serializable

@Serializable
data class HealthResponse(
    val status: String,
)

@Serializable
data class MetaResponse(
    val name: String,
    val env: String,
    val database_kind: String,
)

@Serializable
data class ContentListResponse<T>(
    val items: List<T>,
)
