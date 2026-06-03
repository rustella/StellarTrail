package com.rustella.stellartrail.core.config

import com.rustella.stellartrail.BuildConfig
import org.junit.Assert.assertTrue
import org.junit.Test

class AndroidClientConfigTest {
    @Test
    fun smsCodeCooldownConfigIsPositive() {
        assertTrue(BuildConfig.SMS_CODE_COOLDOWN_SECONDS > 0)
    }
}
