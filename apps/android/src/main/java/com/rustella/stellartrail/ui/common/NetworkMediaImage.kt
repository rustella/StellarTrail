package com.rustella.stellartrail.ui.common

import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import coil.ImageLoader
import coil.compose.AsyncImagePainter
import coil.compose.SubcomposeAsyncImage
import coil.compose.SubcomposeAsyncImageContent
import coil.decode.GifDecoder
import coil.decode.ImageDecoderDecoder
import coil.request.ImageRequest

@Composable
fun NetworkMediaImage(
    imageUrl: String?,
    contentDescription: String?,
    fallbackLabel: String,
    modifier: Modifier = Modifier,
    shape: Shape = TrailInnerCardShape,
    contentScale: ContentScale = ContentScale.Crop,
    loadingLabel: String = fallbackLabel,
    errorLabel: String = fallbackLabel,
) {
    val palette = currentTrailPalette()
    val context = LocalContext.current
    val imageLoader = remember(context) {
        ImageLoader.Builder(context)
            .components {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    add(ImageDecoderDecoder.Factory())
                } else {
                    add(GifDecoder.Factory())
                }
            }
            .build()
    }
    Box(
        modifier = modifier
            .clip(shape)
            .background(palette.controlBackground),
        contentAlignment = Alignment.Center,
    ) {
        if (!imageUrl.isNullOrBlank()) {
            val request = remember(context, imageUrl) {
                ImageRequest.Builder(context)
                    .data(imageUrl)
                    .crossfade(false)
                    .build()
            }
            SubcomposeAsyncImage(
                model = request,
                contentDescription = contentDescription,
                imageLoader = imageLoader,
                contentScale = contentScale,
                modifier = Modifier.fillMaxSize(),
            ) {
                when (painter.state) {
                    is AsyncImagePainter.State.Success -> SubcomposeAsyncImageContent()
                    is AsyncImagePainter.State.Error -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        MediaFallbackText(errorLabel)
                    }
                    else -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        MediaFallbackText(loadingLabel)
                    }
                }
            }
        } else {
            MediaFallbackText(fallbackLabel)
        }
    }
}

@Composable
private fun MediaFallbackText(label: String) {
    val palette = currentTrailPalette()
    Text(
        label,
        color = palette.textMuted,
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.ExtraBold,
    )
}
