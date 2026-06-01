package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.dateText
import com.rustella.stellartrail.domain.trip.durationText
import com.rustella.stellartrail.feature.home.HomeViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroButton
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.HeroSoftButton
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TagList
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun HomeScreen(
    viewModel: HomeViewModel,
    onOpenGears: () -> Unit,
    onCreateGear: () -> Unit,
    onOpenSkills: () -> Unit,
    onOpenTrips: () -> Unit,
    onOpenTrip: (String) -> Unit,
    onOpenProfile: () -> Unit,
    onOpenGear: (String) -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(start = 16.dp, top = 12.dp, end = 16.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(HomeHeroVisualContract.followingSectionGapDp.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "寻径星野",
                title = "今天准备好出发了吗？",
                subtitle = "出发前准备充分，留意天气变化，注意户外安全。",
                chips = listOf(homeHeroStatusText(state.stats, state.isLoggedIn)),
                eyebrowPill = true,
                actions = {
                    HeroButton("检查装备", onOpenGears, Modifier.weight(1f))
                    HeroSoftButton("学习技能", onOpenSkills, Modifier.weight(1f))
                },
            )
        }
        state.tripHighlight?.let { highlight ->
            item { HomeTripHighlightCard(highlight = highlight, onClick = { onOpenTrip(highlight.trip.id) }) }
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.load(state.isLoggedIn) }) }
        if (state.loading) item { LoadingState() }
        item {
            GearOverviewCard(
                overview = HomeGearOverview.from(state.stats, state.isLoggedIn),
                onOpenGears = onOpenGears,
                onLogin = onLogin,
            )
        }
        item { FeaturedSkillsSection(state.featuredKnots, onOpenSkills) }
    }
}

private fun homeHeroStatusText(stats: com.rustella.stellartrail.domain.gear.GearStatsResponse, isLoggedIn: Boolean): String {
    if (!isLoggedIn) return "未登录也可先浏览"
    return if (stats.currentCount > 0) "装备 ${stats.currentCount} 件" else "还没有装备记录"
}

@Composable
private fun HomeTripHighlightCard(highlight: TripHomeHighlightItem, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.weight(1f)) {
                Badge("近期行程", tone = BadgeTone.Info)
                Text(highlight.trip.displayName, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            }
            SoftPillButton("查看行程", onClick)
        }
        Text(highlight.trip.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            MetricTile("准备度", "${highlight.trip.readiness.completionPercent}%", Modifier.weight(1f))
            MetricTile("成员", "${highlight.trip.memberCount}", Modifier.weight(1f))
            MetricTile("行程", highlight.trip.durationText(), Modifier.weight(1f))
        }
        Text("出发前检查：装备、技能、天气和安全预案。", color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun GearOverviewCard(overview: HomeGearOverview, onOpenGears: () -> Unit, onLogin: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(contentPadding = PaddingValues(16.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    overview.eyebrow,
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.ExtraBold,
                    color = palette.brandSoftText,
                )
                Text(overview.title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            }
            SectionLinkPill("查看装备", onOpenGears)
        }
        if (overview.promptTitle != null && overview.promptBody != null) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(16.dp))
                    .background(palette.warningBackground)
                    .padding(12.dp),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(overview.promptTitle, color = palette.warningText, fontWeight = FontWeight.ExtraBold)
                        Text(overview.promptBody, color = palette.warningText, style = MaterialTheme.typography.bodySmall)
                    }
                    PrimaryPillButton("去登录", onLogin)
                }
            }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            overview.stats.forEach { stat ->
                OverviewStatTile(stat = stat, modifier = Modifier.weight(1f))
            }
        }
    }
}

@Composable
private fun OverviewStatTile(stat: HomeOverviewStat, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(16.dp))
            .background(palette.controlBackground)
            .padding(horizontal = 8.dp, vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(
            stat.value,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(stat.label, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(
            stat.hint,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun FeaturedSkillsSection(knots: List<KnotSummary>, onOpenSkills: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(contentPadding = PaddingValues(16.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    "技能复习",
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.ExtraBold,
                    color = palette.brandSoftText,
                )
                Text("户外技能精选", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            }
            SectionLinkPill("全部技能", onOpenSkills)
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(16.dp))
                .background(palette.controlBackground)
                .padding(12.dp),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("绳结", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                        Text("常用连接、固定与收纳绳结", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    PrimaryPillButton("查看全部", onOpenSkills)
                }
                if (knots.isEmpty()) {
                    Text("技能内容准备中。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                } else {
                    knots.take(3).forEach { knot -> KnotFeatureRow(knot, onOpenSkills) }
                }
            }
        }
    }
}

@Composable
private fun KnotFeatureRow(knot: KnotSummary, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(MaterialTheme.colorScheme.background.copy(alpha = 0.58f))
            .clickable(onClick = onClick)
            .padding(12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .weight(0.32f)
                .clip(RoundedCornerShape(12.dp))
                .background(palette.softControlBackground)
                .padding(vertical = 18.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text("绳结", color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.ExtraBold)
        }
        Column(Modifier.weight(0.68f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                knot.categories.take(1).forEach { category ->
                    Badge(category.title, tone = BadgeTone.Info)
                }
            }
            Text(knot.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            if (knot.aliases.isNotEmpty()) {
                Text(knot.aliases.joinToString(" / "), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
            Text(knot.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun SectionLinkPill(text: String, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    Text(
        text = text,
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(palette.brandSoft)
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp),
        color = palette.brandSoftText,
        style = MaterialTheme.typography.labelLarge,
        fontWeight = FontWeight.ExtraBold,
    )
}

@Composable
private fun GearPreviewCard(gear: GearSummary, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(gear.categoryLabel)
            Badge(gear.statusLabel, tone = BadgeTone.Success)
        }
        Text(gear.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(gear.brand, gear.model), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(gear.weightG), Modifier.weight(1f))
            MetricTile("价格", formatPrice(gear.purchasePriceCents), Modifier.weight(1f))
            MetricTile("购买", gear.purchaseDate ?: "未记录", Modifier.weight(1f))
        }
    }
}

@Composable
private fun TemplateMiniCard(template: GearTemplate) {
    val previewItems = template.categories.flatMap { it.items }.take(4)
    SurfaceCard(Modifier.fillMaxWidth()) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge("参考清单")
            Badge("${template.categories.size} 类", tone = BadgeTone.Info)
        }
        Text(template.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(template.categories.joinToString(" · ") { it.name }, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (previewItems.isNotEmpty()) {
            TagList(previewItems)
        }
    }
}

@Composable
private fun SkillPreviewCard(skill: SkillCategorySummary, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge("技能")
            Badge("${skill.itemCount} 项", tone = BadgeTone.Info)
        }
        Text(skill.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(skill.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
