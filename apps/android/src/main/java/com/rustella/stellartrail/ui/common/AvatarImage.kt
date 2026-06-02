package com.rustella.stellartrail.ui.common

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL

@Composable
fun AvatarImage(
    avatarUrl: String?,
    fallbackText: String,
    modifier: Modifier = Modifier,
    backgroundColor: Color = currentTrailPalette().brand,
    contentColor: Color = currentTrailPalette().brandText,
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
) {
    val image by rememberAvatarBitmap(avatarUrl)
    Box(
        modifier = modifier
            .clip(CircleShape)
            .background(backgroundColor),
        contentAlignment = Alignment.Center,
    ) {
        if (image != null) {
            Image(
                bitmap = image!!,
                contentDescription = "头像",
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
            )
        } else {
            Text(
                text = fallbackText,
                color = contentColor,
                style = textStyle,
                fontWeight = FontWeight.ExtraBold,
            )
        }
    }
}

@Composable
private fun rememberAvatarBitmap(avatarUrl: String?) = produceState<ImageBitmap?>(initialValue = null, avatarUrl) {
    value = null
    val url = avatarUrl?.trim()?.takeIf { it.startsWith("https://") || it.startsWith("http://") } ?: return@produceState
    value = withContext(Dispatchers.IO) {
        runCatching {
            val connection = (URL(url).openConnection() as HttpURLConnection).apply {
                connectTimeout = 5_000
                readTimeout = 5_000
                instanceFollowRedirects = true
            }
            try {
                connection.inputStream.use { stream ->
                    BitmapFactory.decodeStream(stream)?.asImageBitmap()
                }
            } finally {
                connection.disconnect()
            }
        }.getOrNull()
    }
}
