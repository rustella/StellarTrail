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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.rustella.stellartrail.ui.theme.StellarTrailDesignColors
import com.rustella.stellartrail.ui.theme.StellarTrailPalette

val TrailCardShape = RoundedCornerShape(24.dp)
val TrailHeroShape = RoundedCornerShape(30.dp)
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
        elevation = CardDefaults.cardElevation(defaultElevation = if (palette == StellarTrailDesignColors.Light) 3.dp else 0.dp),
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
    Box(
        modifier = modifier
            .fillMaxWidth()
            .clip(TrailHeroShape)
            .background(Brush.linearGradient(listOf(palette.heroStart, palette.heroEnd)))
            .padding(22.dp),
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(
                eyebrow,
                color = Color.White.copy(alpha = 0.72f),
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
            )
            Text(
                title,
                color = Color.White,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.ExtraBold,
            )
            Text(
                subtitle,
                color = Color.White.copy(alpha = 0.86f),
                style = MaterialTheme.typography.bodyMedium,
                lineHeight = MaterialTheme.typography.bodyMedium.lineHeight,
            )
            if (chips.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    chips.take(2).forEach { chip -> HeroChip(chip) }
                }
            }
            if (actions != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), content = actions)
            }
        }
    }
}

@Composable
private fun HeroChip(text: String) {
    Text(
        text = text,
        modifier = Modifier
            .clip(TrailPillShape)
            .background(Color.White.copy(alpha = 0.16f))
            .border(1.dp, Color.White.copy(alpha = 0.22f), TrailPillShape)
            .padding(horizontal = 10.dp, vertical = 5.dp),
        color = Color.White,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
    )
}

@Composable
fun HeroButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Button(
        onClick = onClick,
        modifier = modifier,
        shape = TrailPillShape,
        colors = ButtonDefaults.buttonColors(containerColor = Color.White, contentColor = StellarTrailDesignColors.Light.brand),
        contentPadding = PaddingValues(horizontal = 18.dp, vertical = 10.dp),
    ) { Text(text, fontWeight = FontWeight.Bold) }
}

@Composable
fun HeroSoftButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    OutlinedButton(
        onClick = onClick,
        modifier = modifier,
        shape = TrailPillShape,
        border = BorderStroke(1.dp, Color.White.copy(alpha = 0.42f)),
        colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White),
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
) {
    val palette = currentTrailPalette()
    SurfaceCard(modifier = modifier.clickable(onClick = onClick), contentPadding = PaddingValues(16.dp)) {
        Box(
            modifier = Modifier
                .size(44.dp)
                .clip(RoundedCornerShape(16.dp))
                .background(palette.brandSoft),
            contentAlignment = Alignment.Center,
        ) {
            Text(icon, style = MaterialTheme.typography.titleLarge)
        }
        Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
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
