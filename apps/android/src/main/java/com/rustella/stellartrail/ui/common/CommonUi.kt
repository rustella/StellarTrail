package com.rustella.stellartrail.ui.common

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.rustella.stellartrail.ui.theme.StellarTrailDesignColors
import com.rustella.stellartrail.ui.theme.StellarTrailPalette

val TrailCardShape = RoundedCornerShape(20.dp)
val TrailHeroShape = RoundedCornerShape(22.dp)
val TrailPillShape = RoundedCornerShape(999.dp)

enum class BadgeTone { Brand, Neutral, Success, Warning, Danger, Info }

@Composable
fun currentTrailPalette(): StellarTrailPalette =
    if (MaterialTheme.colorScheme.background == StellarTrailDesignColors.Dark.pageBackground) {
        StellarTrailDesignColors.Dark
    } else {
        StellarTrailDesignColors.Light
    }

@Composable
fun LoadingState(modifier: Modifier = Modifier) {
    SurfaceCard(modifier = modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        CircularProgressIndicator()
        Text("正在加载...", color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
fun ErrorState(message: String, onRetry: (() -> Unit)? = null, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    SurfaceCard(
        modifier = modifier.fillMaxWidth(),
        containerColor = palette.dangerBackground,
        contentPadding = PaddingValues(18.dp),
    ) {
        Text(text = message, color = palette.dangerText)
        if (onRetry != null) {
            PrimaryPillButton(text = "重试", onClick = onRetry)
        }
    }
}

@Composable
fun SurfaceCard(
    modifier: Modifier = Modifier,
    containerColor: Color? = null,
    contentPadding: PaddingValues = PaddingValues(18.dp),
    horizontalAlignment: Alignment.Horizontal = Alignment.Start,
    content: @Composable ColumnScope.() -> Unit,
) {
    val palette = currentTrailPalette()
    Card(
        modifier = modifier,
        shape = TrailCardShape,
        colors = CardDefaults.cardColors(containerColor = containerColor ?: palette.surface),
        border = BorderStroke(1.dp, palette.softBorder),
        elevation = CardDefaults.cardElevation(defaultElevation = if (palette == StellarTrailDesignColors.Light) 1.dp else 0.dp),
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(contentPadding),
            verticalArrangement = Arrangement.spacedBy(10.dp),
            horizontalAlignment = horizontalAlignment,
            content = content,
        )
    }
}

@Composable
fun HeroCard(
    eyebrow: String,
    title: String,
    subtitle: String,
    modifier: Modifier = Modifier,
    chips: List<String> = emptyList(),
    actions: (@Composable RowScope.() -> Unit)? = null,
) {
    val palette = currentTrailPalette()
    val lightHero = palette == StellarTrailDesignColors.Light
    val headlineColor = if (lightHero) palette.textPrimary else Color.White
    val bodyColor = if (lightHero) palette.textMuted else Color.White.copy(alpha = 0.86f)
    val eyebrowColor = if (lightHero) palette.brandSoftText else Color.White.copy(alpha = 0.72f)
    val heroModifier = modifier
        .fillMaxWidth()
        .clip(TrailHeroShape)
        .background(Brush.linearGradient(listOf(palette.heroStart, palette.heroMid, palette.heroEnd)))
        .then(if (lightHero) Modifier.drawBehind { drawDayHeroDecoration(palette) } else Modifier)
        .padding(18.dp)
    Box(modifier = heroModifier) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(end = if (lightHero) 28.dp else 0.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text(
                eyebrow,
                color = eyebrowColor,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
            )
            Text(
                title,
                color = headlineColor,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.ExtraBold,
            )
            Text(
                subtitle,
                color = bodyColor,
                style = MaterialTheme.typography.bodyMedium,
                lineHeight = MaterialTheme.typography.bodyMedium.lineHeight,
            )
            if (chips.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    chips.take(2).forEach { chip -> HeroChip(chip, lightHero) }
                }
            }
            if (actions != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), content = actions)
            }
        }
    }
}

private fun DrawScope.drawDayHeroDecoration(palette: StellarTrailPalette) {
    val width = size.width
    val height = size.height
    val sunRadius = 30.dp.toPx()
    drawCircle(
        color = palette.heroSun.copy(alpha = 0.22f),
        radius = sunRadius * 1.9f,
        center = androidx.compose.ui.geometry.Offset(width - 46.dp.toPx(), 42.dp.toPx()),
    )
    drawCircle(
        color = palette.heroSun.copy(alpha = 0.86f),
        radius = sunRadius,
        center = androidx.compose.ui.geometry.Offset(width - 46.dp.toPx(), 42.dp.toPx()),
    )
    val backHill = Path().apply {
        moveTo(width * 0.36f, height)
        quadraticTo(width * 0.68f, height * 0.36f, width, height * 0.58f)
        lineTo(width, height)
        close()
    }
    drawPath(backHill, palette.heroHill.copy(alpha = 0.58f))
    val frontHill = Path().apply {
        moveTo(width * 0.18f, height)
        quadraticTo(width * 0.56f, height * 0.50f, width, height * 0.72f)
        lineTo(width, height)
        close()
    }
    drawPath(frontHill, palette.brandSoft.copy(alpha = 0.54f))
    drawCircle(
        color = palette.brand.copy(alpha = 0.10f),
        radius = 7.dp.toPx(),
        center = androidx.compose.ui.geometry.Offset(width * 0.88f, height * 0.72f),
    )
    drawCircle(
        color = palette.heroSun.copy(alpha = 0.22f),
        radius = 4.dp.toPx(),
        center = androidx.compose.ui.geometry.Offset(width * 0.77f, height * 0.24f),
    )
}

@Composable
private fun HeroChip(text: String, lightHero: Boolean) {
    val palette = currentTrailPalette()
    Text(
        text = text,
        modifier = Modifier
            .clip(TrailPillShape)
            .background(if (lightHero) Color.White.copy(alpha = 0.74f) else Color.White.copy(alpha = 0.16f))
            .border(
                1.dp,
                if (lightHero) palette.softBorder else Color.White.copy(alpha = 0.22f),
                TrailPillShape,
            )
            .padding(horizontal = 10.dp, vertical = 5.dp),
        color = if (lightHero) palette.brandSoftText else Color.White,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
    )
}

@Composable
fun HeroButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    val lightHero = palette == StellarTrailDesignColors.Light
    Button(
        onClick = onClick,
        modifier = modifier,
        shape = TrailPillShape,
        colors = ButtonDefaults.buttonColors(
            containerColor = if (lightHero) palette.brand else Color.White,
            contentColor = if (lightHero) palette.brandText else StellarTrailDesignColors.Light.brand,
        ),
        contentPadding = PaddingValues(horizontal = 18.dp, vertical = 10.dp),
    ) { Text(text, fontWeight = FontWeight.Bold) }
}

@Composable
fun HeroSoftButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    val lightHero = palette == StellarTrailDesignColors.Light
    OutlinedButton(
        onClick = onClick,
        modifier = modifier,
        shape = TrailPillShape,
        border = BorderStroke(1.dp, if (lightHero) palette.brandSoft else Color.White.copy(alpha = 0.42f)),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = if (lightHero) Color.White.copy(alpha = 0.74f) else Color.Transparent,
            contentColor = if (lightHero) palette.brandSoftText else Color.White,
        ),
        contentPadding = PaddingValues(horizontal = 18.dp, vertical = 10.dp),
    ) { Text(text, fontWeight = FontWeight.Bold) }
}

@Composable
fun PrimaryPillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = modifier,
        shape = TrailPillShape,
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 12.dp),
    ) { Text(text, fontWeight = FontWeight.Bold) }
}

@Composable
fun SoftPillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val palette = currentTrailPalette()
    OutlinedButton(
        onClick = onClick,
        enabled = enabled,
        modifier = modifier,
        shape = TrailPillShape,
        border = BorderStroke(1.dp, palette.border),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = palette.softControlBackground,
            contentColor = palette.softControlText,
        ),
        contentPadding = PaddingValues(horizontal = 18.dp, vertical = 10.dp),
    ) { Text(text, fontWeight = FontWeight.Bold) }
}

@Composable
fun StatCard(label: String, value: String, modifier: Modifier = Modifier, hint: String? = null) {
    SurfaceCard(modifier = modifier, contentPadding = PaddingValues(16.dp)) {
        Text(value, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
        Text(label, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
        if (hint != null) {
            Text(hint, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun MetricTile(label: String, value: String, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(18.dp))
            .background(palette.controlBackground)
            .border(1.dp, palette.softBorder, RoundedCornerShape(18.dp))
            .padding(horizontal = 10.dp, vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(
            value,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(label, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
fun EmptyState(title: String, body: String, modifier: Modifier = Modifier) {
    SurfaceCard(modifier = modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold, textAlign = TextAlign.Center)
        Text(body, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    }
}

@Composable
fun MetadataRow(label: String, value: String, modifier: Modifier = Modifier) {
    Row(
        modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.Top,
    ) {
        Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(0.42f))
        Spacer(Modifier.width(16.dp))
        Text(
            value,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.End,
            modifier = Modifier.weight(0.58f),
        )
    }
}

@Composable
fun Badge(text: String, tone: BadgeTone = BadgeTone.Brand, modifier: Modifier = Modifier) {
    val colors = badgeColors(tone)
    Text(
        text = text,
        modifier = modifier
            .clip(TrailPillShape)
            .background(colors.first)
            .padding(horizontal = 10.dp, vertical = 5.dp),
        color = colors.second,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
    )
}

@Composable
fun TagList(tags: List<String>, modifier: Modifier = Modifier) {
    Row(modifier = modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        tags.take(4).forEach { tag ->
            val palette = currentTrailPalette()
            AssistChip(
                onClick = {},
                label = { Text(tag) },
                colors = AssistChipDefaults.assistChipColors(
                    containerColor = palette.chipBackground,
                    labelColor = palette.brandSoftText,
                ),
                border = BorderStroke(0.dp, Color.Transparent),
            )
        }
    }
}

@Composable
fun SectionTitle(title: String, subtitle: String? = null, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
        if (subtitle != null) {
            Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun FeatureTile(
    icon: String,
    title: String,
    body: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    compact: Boolean = false,
) {
    val palette = currentTrailPalette()
    val iconSize = if (compact) 36.dp else 44.dp
    SurfaceCard(
        modifier = modifier
            .heightIn(min = if (compact) 108.dp else 0.dp)
            .clickable(onClick = onClick),
        contentPadding = PaddingValues(if (compact) 14.dp else 16.dp),
    ) {
        Box(
            modifier = Modifier
                .size(iconSize)
                .clip(RoundedCornerShape(if (compact) 13.dp else 16.dp))
                .background(palette.brandSoft),
            contentAlignment = Alignment.Center,
        ) {
            Text(icon, style = if (compact) MaterialTheme.typography.titleMedium else MaterialTheme.typography.titleLarge)
        }
        Text(
            title,
            style = if (compact) MaterialTheme.typography.titleSmall else MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            body,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
fun NoticeCard(title: String, body: String, modifier: Modifier = Modifier, action: (@Composable () -> Unit)? = null) {
    val palette = currentTrailPalette()
    SurfaceCard(
        modifier = modifier.fillMaxWidth(),
        containerColor = palette.warningBackground,
        contentPadding = PaddingValues(16.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(title, color = palette.warningText, fontWeight = FontWeight.ExtraBold)
                Text(body, color = palette.warningText, style = MaterialTheme.typography.bodySmall)
            }
            if (action != null) {
                Box(Modifier.padding(start = 12.dp)) { action() }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LoginPromptSheet(
    visible: Boolean,
    message: String,
    onDismiss: () -> Unit,
    onLogin: () -> Unit,
) {
    if (!visible) return
    val palette = currentTrailPalette()
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        contentColor = palette.textPrimary,
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Badge("登录后继续", tone = BadgeTone.Warning)
            Text("登录后继续", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
            Text(message, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                SoftPillButton("暂不登录", onDismiss, Modifier.weight(1f))
                PrimaryPillButton("去登录", onLogin, Modifier.weight(1f))
            }
            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
fun StepItem(index: Int, text: String, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Top) {
        Box(
            modifier = Modifier
                .size(28.dp)
                .clip(TrailPillShape)
                .background(palette.brand),
            contentAlignment = Alignment.Center,
        ) {
            Text((index + 1).toString(), color = palette.brandText, fontWeight = FontWeight.Bold)
        }
        Text(text, modifier = Modifier.weight(1f), color = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
private fun badgeColors(tone: BadgeTone): Pair<Color, Color> {
    val palette = currentTrailPalette()
    return when (tone) {
        BadgeTone.Brand -> palette.brandSoft to palette.brandSoftText
        BadgeTone.Neutral -> palette.softControlBackground to palette.softControlText
        BadgeTone.Success -> palette.successBackground to palette.successText
        BadgeTone.Warning -> palette.warningBackground to palette.warningText
        BadgeTone.Danger -> palette.dangerBackground to palette.dangerText
        BadgeTone.Info -> palette.infoBackground to palette.infoText
    }
}
