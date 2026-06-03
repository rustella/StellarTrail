package com.rustella.stellartrail.core.config

import com.rustella.stellartrail.BuildConfig
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AndroidClientConfigTest {
    @Test
    fun smsCodeCooldownConfigIsPositive() {
        assertTrue(BuildConfig.SMS_CODE_COOLDOWN_SECONDS > 0)
    }

    @Test
    fun requestSignatureConfigUsesAndroidClientFieldNamesAndIgnoresPlaceholders() {
        assertTrue(BuildConfig.DEFAULT_REQUEST_SIGNATURE_APP_ID.isNotBlank())
        assertTrue(BuildConfig.DEFAULT_REQUEST_SIGNATURE_APP_SECRET.isNotBlank())
        assertNull(AppConfig().requestSignature)

        val credentials = requestSignatureCredentials(
            appId = "android-client",
            appSecret = "android-secret",
        )

        assertNotNull(credentials)
        assertEquals("android-client", credentials?.appId)
        assertEquals("android-secret", credentials?.appSecret)
    }
}
