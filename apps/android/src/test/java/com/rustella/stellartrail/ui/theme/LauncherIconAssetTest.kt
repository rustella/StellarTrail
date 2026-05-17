package com.rustella.stellartrail.ui.theme

import java.io.File
import java.security.MessageDigest
import javax.imageio.ImageIO
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LauncherIconAssetTest {
    private val androidProjectDir: File = locateAndroidProjectDir()

    @Test
    fun manifestUsesMipmapLauncherIcons() {
        val manifest = File(androidProjectDir, "src/main/AndroidManifest.xml").readText()

        assertTrue(manifest.contains("android:icon=\"@mipmap/ic_launcher\""))
        assertTrue(manifest.contains("android:roundIcon=\"@mipmap/ic_launcher_round\""))
    }

    @Test
    fun xxhdpiLauncherIconMatchesProvidedOutdoorArtwork() {
        val icon = File(androidProjectDir, "src/main/res/mipmap-xxhdpi/ic_launcher.png")

        assertTrue("Expected xxhdpi launcher icon PNG to exist.", icon.isFile)
        assertEquals(PROVIDED_ICON_SHA256, sha256(icon))
        ImageIO.read(icon).also { image ->
            assertNotNull(image)
            assertEquals(144, image.width)
            assertEquals(144, image.height)
        }
    }

    @Test
    fun launcherIconsCoverRequiredDensities() {
        val densities = mapOf(
            "mipmap-mdpi" to 48,
            "mipmap-hdpi" to 72,
            "mipmap-xhdpi" to 96,
            "mipmap-xxhdpi" to 144,
            "mipmap-xxxhdpi" to 192,
        )

        densities.forEach { (density, size) ->
            listOf("ic_launcher.png", "ic_launcher_round.png").forEach { fileName ->
                val icon = File(androidProjectDir, "src/main/res/$density/$fileName")
                assertTrue("Missing $density/$fileName", icon.isFile)
                ImageIO.read(icon).also { image ->
                    assertNotNull(image)
                    assertEquals("$density/$fileName width", size, image.width)
                    assertEquals("$density/$fileName height", size, image.height)
                }
            }
        }
    }

    @Test
    fun adaptiveLauncherIconsUseGeneratedForegroundAndBackground() {
        val foreground = File(androidProjectDir, "src/main/res/drawable-nodpi/ic_launcher_foreground.png")
        val background = File(androidProjectDir, "src/main/res/values/ic_launcher_colors.xml")
        val adaptiveIcon = File(androidProjectDir, "src/main/res/mipmap-anydpi-v26/ic_launcher.xml")
        val adaptiveRoundIcon = File(androidProjectDir, "src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml")

        assertTrue("Missing adaptive icon foreground.", foreground.isFile)
        ImageIO.read(foreground).also { image ->
            assertNotNull(image)
            assertEquals(432, image.width)
            assertEquals(432, image.height)
        }
        assertTrue("Missing launcher icon background color resource.", background.isFile)
        listOf(adaptiveIcon, adaptiveRoundIcon).forEach { xml ->
            assertTrue("Missing adaptive icon XML ${xml.name}", xml.isFile)
            val content = xml.readText()
            assertTrue(content.contains("android:drawable=\"@color/ic_launcher_background\""))
            assertTrue(content.contains("android:drawable=\"@drawable/ic_launcher_foreground\""))
        }
    }

    private fun locateAndroidProjectDir(): File {
        val workingDir = File(System.getProperty("user.dir") ?: error("Missing user.dir"))
        return listOf(workingDir, File(workingDir, "apps/android"))
            .first { File(it, "src/main/AndroidManifest.xml").isFile }
    }

    private fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(file.readBytes())
        return digest.joinToString("") { byte -> "%02x".format(byte) }
    }

    private companion object {
        const val PROVIDED_ICON_SHA256 = "a2442106ee75036b7042d1ab9150b8b7a92a00db8174a3197a95d2b5fa584a38"
    }
}
