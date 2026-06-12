package com.rustella.stellartrail.domain.trip

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
enum class TrailSourceFormat {
    @SerialName("gpx") GPX,
    @SerialName("kml") KML,
    @SerialName("fit") FIT,
}

@Serializable
data class TrailPoint(
    val lng: Double,
    val lat: Double,
    @SerialName("elevation_m") val elevationM: Double? = null,
    val time: String? = null,
)

@Serializable
data class TrailBounds(
    @SerialName("min_lng") val minLng: Double,
    @SerialName("min_lat") val minLat: Double,
    @SerialName("max_lng") val maxLng: Double,
    @SerialName("max_lat") val maxLat: Double,
)

@Serializable
data class TrailSummary(
    val id: String,
    @SerialName("owner_user_id") val ownerUserId: String,
    @SerialName("display_name") val displayName: String,
    val description: String? = null,
    @SerialName("source_format") val sourceFormat: TrailSourceFormat,
    @SerialName("original_filename") val originalFilename: String,
    @SerialName("content_type") val contentType: String,
    @SerialName("size_bytes") val sizeBytes: Long,
    @SerialName("sha256_hex") val sha256Hex: String,
    val bounds: TrailBounds? = null,
    @SerialName("distance_m") val distanceM: Double,
    @SerialName("ascent_m") val ascentM: Double,
    @SerialName("descent_m") val descentM: Double,
    @SerialName("min_elevation_m") val minElevationM: Double? = null,
    @SerialName("max_elevation_m") val maxElevationM: Double? = null,
    @SerialName("start_elevation_m") val startElevationM: Double? = null,
    @SerialName("end_elevation_m") val endElevationM: Double? = null,
    @SerialName("start_time") val startTime: String? = null,
    @SerialName("end_time") val endTime: String? = null,
    @SerialName("point_count") val pointCount: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class Trail(
    val id: String,
    @SerialName("owner_user_id") val ownerUserId: String,
    @SerialName("display_name") val displayName: String,
    val description: String? = null,
    @SerialName("source_format") val sourceFormat: TrailSourceFormat,
    @SerialName("original_filename") val originalFilename: String,
    @SerialName("content_type") val contentType: String,
    @SerialName("size_bytes") val sizeBytes: Long,
    @SerialName("sha256_hex") val sha256Hex: String,
    val bucket: String,
    @SerialName("object_key") val objectKey: String,
    @SerialName("normalized_points") val normalizedPoints: List<TrailPoint> = emptyList(),
    @SerialName("simplified_geojson") val simplifiedGeojson: JsonElement,
    val bounds: TrailBounds? = null,
    @SerialName("distance_m") val distanceM: Double,
    @SerialName("ascent_m") val ascentM: Double,
    @SerialName("descent_m") val descentM: Double,
    @SerialName("min_elevation_m") val minElevationM: Double? = null,
    @SerialName("max_elevation_m") val maxElevationM: Double? = null,
    @SerialName("start_elevation_m") val startElevationM: Double? = null,
    @SerialName("end_elevation_m") val endElevationM: Double? = null,
    @SerialName("start_time") val startTime: String? = null,
    @SerialName("end_time") val endTime: String? = null,
    @SerialName("point_count") val pointCount: Long,
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class ListTrailsResponse(
    val items: List<TrailSummary> = emptyList(),
)

@Serializable
data class MapTrailLink(
    @SerialName("trail_id") val trailId: String,
    @SerialName("linked_by_user_id") val linkedByUserId: String,
    val role: String,
    @SerialName("sort_order") val sortOrder: Int,
    val notes: String? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    val trail: TrailSummary,
    @SerialName("simplified_geojson") val simplifiedGeojson: JsonElement,
)

@Serializable
data class TripOverviewMapTrail(
    @SerialName("trip_id") val tripId: String,
    @SerialName("trip_title") val tripTitle: String,
    @SerialName("trip_start_date") val tripStartDate: String? = null,
    @SerialName("trip_end_date") val tripEndDate: String? = null,
    @SerialName("trail_id") val trailId: String,
    @SerialName("linked_by_user_id") val linkedByUserId: String,
    val role: String,
    @SerialName("sort_order") val sortOrder: Int,
    val notes: String? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    val trail: TrailSummary,
    @SerialName("simplified_geojson") val simplifiedGeojson: JsonElement,
)

@Serializable
data class MapAnnotation(
    val id: String,
    @SerialName("owner_user_id") val ownerUserId: String,
    @SerialName("trail_id") val trailId: String? = null,
    val lng: Double,
    val lat: Double,
    @SerialName("elevation_m") val elevationM: Double? = null,
    @SerialName("trail_point_index") val trailPointIndex: Long? = null,
    @SerialName("annotation_type") val annotationType: String,
    val title: String? = null,
    val note: String? = null,
    @SerialName("field_versions") val fieldVersions: FieldVersions = emptyFieldVersions(),
    @SerialName("is_deleted") val isDeleted: Boolean = false,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class MapAnnotationRequest(
    @SerialName("trail_id") val trailId: String? = null,
    val lng: Double,
    val lat: Double,
    @SerialName("elevation_m") val elevationM: Double? = null,
    @SerialName("trail_point_index") val trailPointIndex: Long? = null,
    @SerialName("annotation_type") val annotationType: String = "note",
    val title: String? = null,
    val note: String? = null,
)

@Serializable
data class MapStyleOption(
    val id: String,
    val label: String,
    @SerialName("style_url") val styleUrl: String,
)

@Serializable
data class MapConfigResponse(
    val provider: String,
    @SerialName("style_url") val styleUrl: String,
    @SerialName("public_key") val publicKey: String? = null,
    @SerialName("coordinate_system") val coordinateSystem: String = "WGS84",
    val enabled: Boolean = false,
    val styles: List<MapStyleOption> = emptyList(),
    @SerialName("default_style_id") val defaultStyleId: String = "outdoor",
)

@Serializable
data class TripMapStateResponse(
    val map: MapConfigResponse,
    val trails: List<MapTrailLink> = emptyList(),
    val annotations: List<MapAnnotation> = emptyList(),
)

@Serializable
data class TripsMapOverviewStats(
    @SerialName("trip_count") val tripCount: Int = 0,
    @SerialName("trail_count") val trailCount: Int = 0,
    @SerialName("rendered_point_count") val renderedPointCount: Int = 0,
    @SerialName("total_distance_m") val totalDistanceM: Double = 0.0,
    @SerialName("total_ascent_m") val totalAscentM: Double = 0.0,
    @SerialName("total_descent_m") val totalDescentM: Double = 0.0,
)

@Serializable
data class TripsMapOverviewResponse(
    val map: MapConfigResponse,
    val trails: List<TripOverviewMapTrail> = emptyList(),
    val bounds: TrailBounds? = null,
    val stats: TripsMapOverviewStats = TripsMapOverviewStats(),
    val truncated: Boolean = false,
)
