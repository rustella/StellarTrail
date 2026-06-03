package com.rustella.stellartrail.ui.screens

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotMediaAsset
import com.rustella.stellartrail.feature.skills.detail.SkillDetailUiState
import com.rustella.stellartrail.feature.skills.detail.SkillDetailViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.NetworkMediaImage
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailCardShape
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.TrailPillShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun SkillDetailScreen(
    viewModel: SkillDetailViewModel,
    isLoggedIn: Boolean,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            SkillDetailTopBar(title = state.detail?.title ?: "技能详情", onBack = onBack)
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (state.error != null) {
                ErrorState(state.error!!, onRetry = { viewModel.load(isLoggedIn) })
            }
            if (state.loading && state.detail == null) LoadingState()
            state.detail?.let {
                SkillDetailContent(
                    state = state,
                    isLoggedIn = isLoggedIn,
                    onToggleFavorite = viewModel::toggleFavorite,
                    onLogin = onLogin,
                    resolveMediaUrl = viewModel::resolveMediaUrl,
                )
            }
            if (!state.loading && state.detail == null && state.error == null) {
                EmptyState("暂无绳结详情", "返回列表后重新打开。")
            }
        }
    }
}

@Composable
private fun SkillDetailTopBar(title: String, onBack: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.background),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp)
                .background(KnotTopBarBackground),
        ) {
            IconButton(
                onClick = onBack,
                modifier = Modifier.align(Alignment.CenterStart),
            ) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回", tint = Color.White)
            }
            Text(
                text = title,
                modifier = Modifier
                    .align(Alignment.Center)
                    .fillMaxWidth()
                    .padding(horizontal = 70.dp),
                color = Color.White,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.ExtraBold,
                textAlign = TextAlign.Center,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun SkillDetailContent(
    state: SkillDetailUiState,
    isLoggedIn: Boolean,
    onToggleFavorite: () -> Unit,
    onLogin: () -> Unit,
    resolveMediaUrl: (String) -> String,
) {
    val detail = state.detail ?: return
    val media = remember(detail.id, detail.media) { detail.media.filter(::isDetailMediaAsset) }
    var activeMediaIndex by remember(detail.id, media.size) { mutableStateOf(preferredMediaIndex(media)) }
    val activeMedia = media.getOrNull(activeMediaIndex) ?: media.firstOrNull()
    val activeMeta = activeMedia?.let { mediaMeta(it.mediaType) }

    SurfaceCard(contentPadding = PaddingValues(0.dp)) {
        Column(Modifier.fillMaxWidth()) {
            MediaStage(
                detail = detail,
                media = media,
                activeMedia = activeMedia,
                activeMediaIndex = activeMediaIndex,
                activeMeta = activeMeta,
                onSelectMedia = { activeMediaIndex = it },
                resolveMediaUrl = resolveMediaUrl,
            )
            SummaryPanel(
                detail = detail,
                mediaCredit = mediaCredit(activeMedia),
                state = state,
                isLoggedIn = isLoggedIn,
                onToggleFavorite = onToggleFavorite,
                onLogin = onLogin,
            )
        }
    }
    SafetyNoticeCard()
    if (!detail.description.isNullOrBlank()) {
        InfoCard(title = "资料说明", body = detail.description)
    }
}

@Composable
private fun MediaStage(
    detail: KnotDetail,
    media: List<KnotMediaAsset>,
    activeMedia: KnotMediaAsset?,
    activeMediaIndex: Int,
    activeMeta: MediaMeta?,
    onSelectMedia: (Int) -> Unit,
    resolveMediaUrl: (String) -> String,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(KnotStageBackground)
            .padding(top = 0.dp, bottom = 14.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(208.dp)
                .padding(horizontal = 36.dp)
                .background(Color(0xFFF8FAFC)),
            contentAlignment = Alignment.Center,
        ) {
            NetworkMediaImage(
                imageUrl = activeMedia?.url?.let(resolveMediaUrl),
                contentDescription = detail.title,
                fallbackLabel = if (detail.media.isEmpty()) "暂无演示图" else "${detail.media.size} 个素材",
                modifier = Modifier.fillMaxSize(),
                shape = RoundedCornerShape(0.dp),
                contentScale = ContentScale.Fit,
                fallbackContent = {
                    KnotMediaFallback(
                        mediaType = activeMedia?.mediaType,
                        label = activeMeta?.label ?: if (detail.media.isEmpty()) "暂无演示图" else "${detail.media.size} 个素材",
                    )
                },
            )
        }
        if (media.isNotEmpty()) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(18.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                media.forEachIndexed { index, item ->
                    MediaControl(
                        meta = mediaMeta(item.mediaType),
                        selected = index == activeMediaIndex,
                        onClick = { onSelectMedia(index) },
                    )
                }
            }
        }
        if (activeMeta != null) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    activeMeta.label,
                    color = Color.White,
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.ExtraBold,
                )
                Spacer(Modifier.width(8.dp))
                Text(
                    activeMeta.helpText,
                    color = Color.White.copy(alpha = 0.72f),
                    style = MaterialTheme.typography.labelMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun MediaControl(meta: MediaMeta, selected: Boolean, onClick: () -> Unit) {
    Column(
        modifier = Modifier
            .size(width = 62.dp, height = 54.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(if (selected) Color.White.copy(alpha = 0.22f) else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(vertical = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(meta.icon, color = Color.White, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
        Text(
            meta.label,
            color = Color.White.copy(alpha = 0.82f),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun SummaryPanel(
    detail: KnotDetail,
    mediaCredit: String,
    state: SkillDetailUiState,
    isLoggedIn: Boolean,
    onToggleFavorite: () -> Unit,
    onLogin: () -> Unit,
) {
    val palette = currentTrailPalette()
    Column(Modifier.fillMaxWidth().padding(17.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(detail.title, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
                if (detail.aliases.isNotEmpty()) {
                    Text(
                        detail.aliases.joinToString(" / "),
                        color = palette.textMuted,
                        style = MaterialTheme.typography.bodySmall,
                        fontWeight = FontWeight.Bold,
                    )
                }
                Text(detail.summary, color = palette.textMuted, style = MaterialTheme.typography.bodyMedium)
            }
            Spacer(Modifier.width(12.dp))
            FavoriteButton(
                isFavorited = state.isFavorited,
                loading = state.favoriteLoading,
                onClick = {
                    if (isLoggedIn) {
                        onToggleFavorite()
                    } else {
                        onLogin()
                    }
                },
            )
        }
        val tags = knotTags(detail)
        if (tags.isNotEmpty()) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                tags.take(3).forEach { tag -> Badge(tag, tone = BadgeTone.Info) }
            }
        }
        Text(
            mediaCredit,
            color = palette.textMuted,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.Bold,
        )
        if (state.actionError != null) {
            Text(
                state.actionError,
                color = palette.dangerText,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
            )
        }
    }
}

@Composable
private fun KnotMediaFallback(mediaType: String?, label: String) {
    val animated = mediaType == "draw_gif" || mediaType == "turntable_gif"
    val turntable = mediaType == "turntable_gif"
    val transition = rememberInfiniteTransition(label = "knot-media-fallback")
    val progress by transition.animateFloat(
        initialValue = if (animated) 0f else 1f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 1400, easing = LinearEasing),
            repeatMode = if (animated && !turntable) RepeatMode.Reverse else RepeatMode.Restart,
        ),
        label = "knot-media-progress",
    )
    val rotation by transition.animateFloat(
        initialValue = 0f,
        targetValue = if (turntable) 360f else 0f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 1800, easing = LinearEasing),
            repeatMode = RepeatMode.Restart,
        ),
        label = "knot-media-rotation",
    )
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFFF8FAFC))
            .padding(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Canvas(
            modifier = Modifier
                .size(142.dp)
                .graphicsLayer {
                    rotationZ = if (turntable) rotation else 0f
                },
        ) {
            val stroke = size.minDimension * 0.085f
            val loopCenter = androidx.compose.ui.geometry.Offset(
                x = size.width * 0.5f,
                y = size.height * 0.54f,
            )
            val sway = if (animated && !turntable) (progress - 0.5f) * size.minDimension * 0.16f else 0f
            val rope = Color(0xFFD97706)
            val highlight = Color(0xFFFBBF24)
            val shadow = Color(0xFF92400E)
            fun line(color: Color, startX: Float, startY: Float, endX: Float, endY: Float, width: Float) {
                drawLine(
                    color = color,
                    start = androidx.compose.ui.geometry.Offset(startX, startY),
                    end = androidx.compose.ui.geometry.Offset(endX, endY),
                    strokeWidth = width,
                    cap = StrokeCap.Round,
                )
            }
            line(shadow, size.width * 0.16f, size.height * 0.72f, loopCenter.x - size.width * 0.18f, loopCenter.y + sway, stroke * 1.2f)
            line(rope, size.width * 0.16f, size.height * 0.72f, loopCenter.x - size.width * 0.18f, loopCenter.y + sway, stroke)
            drawCircle(
                color = shadow,
                radius = size.minDimension * 0.2f,
                center = loopCenter,
                style = Stroke(width = stroke * 1.15f, cap = StrokeCap.Round),
            )
            drawCircle(
                color = highlight,
                radius = size.minDimension * 0.2f,
                center = loopCenter,
                style = Stroke(width = stroke * 0.78f, cap = StrokeCap.Round),
            )
            line(shadow, loopCenter.x + size.width * 0.16f, loopCenter.y + sway * 0.4f, size.width * 0.84f, size.height * 0.28f, stroke * 1.2f)
            line(highlight, loopCenter.x + size.width * 0.16f, loopCenter.y + sway * 0.4f, size.width * 0.84f, size.height * 0.28f, stroke)
            line(shadow, size.width * 0.5f + sway * 0.35f, size.height * 0.08f, loopCenter.x + sway * 0.2f, loopCenter.y - size.height * 0.24f, stroke * 1.2f)
            line(rope, size.width * 0.5f + sway * 0.35f, size.height * 0.08f, loopCenter.x + sway * 0.2f, loopCenter.y - size.height * 0.24f, stroke)
        }
        Spacer(Modifier.height(8.dp))
        Text(
            label,
            color = Color(0xFF64748B),
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.ExtraBold,
        )
    }
}

@Composable
private fun FavoriteButton(isFavorited: Boolean, loading: Boolean, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    val background = if (isFavorited) palette.warningBackground else palette.softControlBackground
    val foreground = if (isFavorited) palette.warningText else palette.softControlText
    Text(
        text = when {
            loading -> "处理中"
            isFavorited -> "★ 已收藏"
            else -> "☆ 收藏"
        },
        modifier = Modifier
            .clip(TrailPillShape)
            .background(background)
            .clickable(enabled = !loading, onClick = onClick)
            .padding(horizontal = 13.dp, vertical = 8.dp),
        color = foreground,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.ExtraBold,
        maxLines = 1,
    )
}

@Composable
private fun SafetyNoticeCard() {
    val palette = currentTrailPalette()
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = TrailCardShape,
        colors = CardDefaults.cardColors(containerColor = palette.warningBackground),
        border = BorderStroke(1.dp, palette.warningText),
        elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
    ) {
        Column(Modifier.fillMaxWidth().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("安全提示", color = palette.warningText, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            Text(
                "本内容仅供绳结知识学习和非承重练习，不得直接用于承载人体、攀登、救援、吊装、高空作业、航海安全等场景。实际使用前应接受专业训练，并由具备资质或充分经验的人员结合现场条件检查复核。",
                color = palette.warningText,
                style = MaterialTheme.typography.bodyMedium,
                lineHeight = MaterialTheme.typography.bodyMedium.lineHeight,
            )
        }
    }
}

@Composable
private fun InfoCard(title: String, body: String) {
    val palette = currentTrailPalette()
    SurfaceCard {
        Text(title, color = palette.textPrimary, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(body, color = palette.textMuted, style = MaterialTheme.typography.bodyMedium)
    }
}

private data class MediaMeta(
    val label: String,
    val icon: String,
    val helpText: String,
)

private fun isDetailMediaAsset(item: KnotMediaAsset): Boolean =
    item.mediaType == "preview" || item.mediaType == "draw_gif" || item.mediaType == "turntable_gif"

private fun preferredMediaIndex(media: List<KnotMediaAsset>): Int =
    media.indexOfFirst { it.mediaType == "draw_gif" }.takeIf { it >= 0 } ?: 0

private fun mediaMeta(mediaType: String): MediaMeta = when (mediaType) {
    "preview" -> MediaMeta("静态图", "◉", "查看绳结的清晰定格图。")
    "draw_gif" -> MediaMeta("系法动图", "▷", "自动循环演示打结步骤。")
    "turntable_gif" -> MediaMeta("旋转动图", "◎", "自动循环展示绳结结构的旋转动图。")
    else -> MediaMeta("动图", "•", "查看绳结动图。")
}

private fun mediaCredit(media: KnotMediaAsset?): String {
    if (media == null) return "演示素材 · Knots 3D"
    return "${mediaMeta(media.mediaType).label} · ${media.attribution ?: "Knots 3D"}"
}

private fun knotTags(detail: KnotDetail): List<String> =
    (detail.categories.map { it.title } + detail.types.map { it.title })
        .filter { it.isNotBlank() }
        .distinct()

private val KnotTopBarBackground = Color(0xFF0F172A)
private val KnotStageBackground = Color(0xFF020617)
