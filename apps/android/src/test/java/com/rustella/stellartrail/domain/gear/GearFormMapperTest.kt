package com.rustella.stellartrail.domain.gear

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class GearFormMapperTest {
    @Test
    fun toCreateRequestTrimsAndParsesOptionalFields() {
        val request = GearFormState(
            category = GearCategory.SLEEP_SYSTEM,
            name = "  羽绒睡袋  ",
            brand = "  山野  ",
            model = "  X1  ",
            weightG = "950",
            purchasePrice = "1299.90",
            tags = "冬季，保暖 backpack 冬季",
            shareEnabled = true,
        ).toCreateRequest()

        assertEquals(GearCategory.SLEEP_SYSTEM, request.category)
        assertEquals("羽绒睡袋", request.name)
        assertEquals("山野", request.brand)
        assertEquals("X1", request.model)
        assertEquals(950, request.weightG)
        assertEquals(129990L, request.purchasePriceCents)
        assertEquals(listOf("冬季", "保暖", "backpack"), request.tags)
        assertEquals(true, request.shareEnabled)
    }

    @Test
    fun toCreateRequestConvertsBlankOptionalFieldsToNull() {
        val request = GearFormState(name = "头灯", brand = " ", model = "", purchasePrice = "").toCreateRequest()

        assertNull(request.brand)
        assertNull(request.model)
        assertNull(request.purchasePriceCents)
    }

    @Test(expected = IllegalArgumentException::class)
    fun toCreateRequestRejectsBlankName() {
        GearFormState(name = " ").toCreateRequest()
    }

    @Test(expected = IllegalArgumentException::class)
    fun toCreateRequestRejectsInvalidWeight() {
        GearFormState(name = "炉头", weightG = "-1").toCreateRequest()
    }

    @Test(expected = IllegalArgumentException::class)
    fun toCreateRequestRejectsInvalidPrice() {
        GearFormState(name = "炉头", purchasePrice = "not-a-price").toCreateRequest()
    }
}
