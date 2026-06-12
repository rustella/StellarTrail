package com.rustella.stellartrail.core.location

import android.Manifest
import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import android.location.Location
import android.os.Looper
import androidx.core.content.ContextCompat
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority

data class ForegroundLocation(
    val longitude: Double,
    val latitude: Double,
    val accuracyMeters: Float?,
)

enum class ForegroundLocationPermission {
    None,
    Coarse,
    Fine,
}

enum class ForegroundLocationTrackingStatus {
    Idle,
    Starting,
    Following,
    PermissionDenied,
    Unavailable,
}

data class ForegroundLocationTrackingState(
    val status: ForegroundLocationTrackingStatus,
    val message: String? = null,
) {
    val isActive: Boolean
        get() = status == ForegroundLocationTrackingStatus.Starting ||
            status == ForegroundLocationTrackingStatus.Following

    companion object {
        val Idle = ForegroundLocationTrackingState(ForegroundLocationTrackingStatus.Idle)
        val Starting = ForegroundLocationTrackingState(ForegroundLocationTrackingStatus.Starting)
        val Following = ForegroundLocationTrackingState(ForegroundLocationTrackingStatus.Following)
        val PermissionDenied = ForegroundLocationTrackingState(
            status = ForegroundLocationTrackingStatus.PermissionDenied,
            message = "需要位置权限才能定位当前位置。",
        )

        fun unavailable(message: String = "暂时无法获取当前位置。") = ForegroundLocationTrackingState(
            status = ForegroundLocationTrackingStatus.Unavailable,
            message = message,
        )
    }
}

interface ForegroundLocationProvider {
    fun startUpdates(
        permission: ForegroundLocationPermission,
        onLocation: (ForegroundLocation) -> Unit,
        onError: (Throwable) -> Unit,
    )

    fun stopUpdates()
}

class AndroidForegroundLocationProvider(
    context: Context,
    private val updateIntervalMillis: Long = 4_000L,
    private val minimumUpdateIntervalMillis: Long = 2_000L,
) : ForegroundLocationProvider {
    private val client: FusedLocationProviderClient =
        LocationServices.getFusedLocationProviderClient(context.applicationContext)
    private var callback: LocationCallback? = null

    @SuppressLint("MissingPermission")
    override fun startUpdates(
        permission: ForegroundLocationPermission,
        onLocation: (ForegroundLocation) -> Unit,
        onError: (Throwable) -> Unit,
    ) {
        stopUpdates()
        if (permission == ForegroundLocationPermission.None) {
            onError(SecurityException("Foreground location permission is not granted."))
            return
        }

        val request = LocationRequest.Builder(permission.locationPriority(), updateIntervalMillis)
            .setMinUpdateIntervalMillis(minimumUpdateIntervalMillis)
            .setMaxUpdateDelayMillis(updateIntervalMillis + minimumUpdateIntervalMillis)
            .setWaitForAccurateLocation(permission == ForegroundLocationPermission.Fine)
            .build()
        val nextCallback = object : LocationCallback() {
            override fun onLocationResult(result: LocationResult) {
                result.lastLocation?.let { onLocation(it.toForegroundLocation()) }
            }
        }
        callback = nextCallback
        client.lastLocation
            .addOnSuccessListener { location -> location?.let { onLocation(it.toForegroundLocation()) } }
        client.requestLocationUpdates(request, nextCallback, Looper.getMainLooper())
            .addOnFailureListener { error ->
                if (callback === nextCallback) {
                    stopUpdates()
                    onError(error)
                }
            }
    }

    override fun stopUpdates() {
        callback?.let { client.removeLocationUpdates(it) }
        callback = null
    }
}

fun Context.foregroundLocationPermission(): ForegroundLocationPermission =
    resolveForegroundLocationPermission(
        fineGranted = ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED,
        coarseGranted = ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_COARSE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED,
    )

fun resolveForegroundLocationPermission(
    fineGranted: Boolean,
    coarseGranted: Boolean,
): ForegroundLocationPermission = when {
    fineGranted -> ForegroundLocationPermission.Fine
    coarseGranted -> ForegroundLocationPermission.Coarse
    else -> ForegroundLocationPermission.None
}

fun ForegroundLocationPermission.locationPriority(): Int = when (this) {
    ForegroundLocationPermission.Fine -> Priority.PRIORITY_HIGH_ACCURACY
    ForegroundLocationPermission.Coarse -> Priority.PRIORITY_BALANCED_POWER_ACCURACY
    ForegroundLocationPermission.None -> Priority.PRIORITY_PASSIVE
}

private fun Location.toForegroundLocation() = ForegroundLocation(
    longitude = longitude,
    latitude = latitude,
    accuracyMeters = if (hasAccuracy()) accuracy else null,
)
