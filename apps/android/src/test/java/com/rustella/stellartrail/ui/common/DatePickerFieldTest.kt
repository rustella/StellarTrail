package com.rustella.stellartrail.ui.common

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DatePickerFieldTest {
    @Test
    fun datePickerMillisRoundTripsIsoDateText() {
        val millis = dateStringToPickerMillis("2026-06-02")

        assertEquals("2026-06-02", pickerMillisToDateString(millis!!))
    }

    @Test
    fun invalidDateTextHasNoInitialSelection() {
        assertNull(dateStringToPickerMillis(""))
        assertNull(dateStringToPickerMillis("2026/06/02"))
    }
}
