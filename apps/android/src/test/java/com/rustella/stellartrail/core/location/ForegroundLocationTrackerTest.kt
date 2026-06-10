package com.rustella.stellartrail.core.location

import com.google.android.gms.location.Priority
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ForegroundLocationTrackerTest {
    @Test
    fun permissionResolutionPrefersFineOverCoarse() {
        assertEquals(
            ForegroundLocationPermission.Fine,
            resolveForegroundLocationPermission(fineGranted = true, coarseGranted = true),
        )
        assertEquals(
            ForegroundLocationPermission.Coarse,
            resolveForegroundLocationPermission(fineGranted = false, coarseGranted = true),
        )
        assertEquals(
            ForegroundLocationPermission.None,
            resolveForegroundLocationPermission(fineGranted = false, coarseGranted = false),
        )
    }

    @Test
    fun trackingStateMarksOnlyActiveStatesAsActive() {
        assertFalse(ForegroundLocationTrackingState.Idle.isActive)
        assertFalse(ForegroundLocationTrackingState.PermissionDenied.isActive)
        assertFalse(ForegroundLocationTrackingState.unavailable().isActive)
        assertTrue(ForegroundLocationTrackingState.Starting.isActive)
        assertTrue(ForegroundLocationTrackingState.Following.isActive)
    }

    @Test
    fun locationPriorityUsesAccuracyFromGrantedPermission() {
        assertEquals(Priority.PRIORITY_HIGH_ACCURACY, ForegroundLocationPermission.Fine.locationPriority())
        assertEquals(Priority.PRIORITY_BALANCED_POWER_ACCURACY, ForegroundLocationPermission.Coarse.locationPriority())
        assertEquals(Priority.PRIORITY_PASSIVE, ForegroundLocationPermission.None.locationPriority())
    }
}
