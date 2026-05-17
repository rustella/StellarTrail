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
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.feature.home.HomeViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.FeatureTile
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
    onOpenProfile: () -> Unit,
    onOpenGear: (String) -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val quickActions = HomeQuickAction.defaults(state.isLoggedIn)
    val openAction: (HomeActionTarget) -> Unit = { target ->
        when (target) {
            HomeActionTarget.Gears -> onOpenGears()
            HomeActionTarget.NewGear -> onCreateGear()
            HomeActionTarget.Skills -> onOpenSkills()
            HomeActionTarget.Profile -> onOpenProfile()
            HomeActionTarget.Login -> onLogin()
        }
    }
    LazyColumn(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(start = 16.dp, top = 12.dp, end = 16.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "寻径星野 · 出发前检查",
                title = "今天准备好出发了吗？",
                subtitle = "跟着清单确认背包、技能和个人设置，轻松开始下一段路线。",
                chips = listOf(if (state.isLoggedIn) "我的装备已保存" else "可先浏览清单", "绳结教学可直接看"),
                actions = {
                    HeroButton("查看装备", onOpenGears)
                    HeroSoftButton("学习技能", onOpenSkills)
                },
            )
        }
        item { QuickActionGrid(actions = quickActions, onAction = openAction) }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.load(state.isLoggedIn) }) }
        if (state.loading) item { LoadingState() }
        item {
            GearOverviewCard(
                overview = HomeGearOverview.from(state.stats, state.isLoggedIn),
                onOpenGears = onOpenGears,
                onLogin = onLogin,
            )
        }
        if (state.isLoggedIn && state.recentGears.isNotEmpty()) {
            item { SectionTitle("最近装备", "快速查看近期更新。") }
            items(state.recentGears, key = { it.id }) { gear ->
                GearPreviewCard(gear = gear, onClick = { onOpenGear(gear.id) })
            }
        }
        item { SectionTitle("出行装备参考", "按场景准备背包，登录后保存自己的清单。") }
        if (!state.loading && state.templates.isEmpty()) {
            item { EmptyState("暂无装备参考", "稍后刷新或检查网络。") }
        }
        items(state.templates, key = { it.id }) { template ->
            TemplateMiniCard(template = template)
        }
        item { PrimaryPillButton("查看全部装备", onOpenGears, Modifier.fillMaxWidth()) }
        item { SectionTitle("户外技能", "出发前先掌握常用绳结与营地技能。") }
        if (!state.loading && state.skills.isEmpty()) {
            item { EmptyState("暂无技能分类", "稍后刷新或检查网络。") }
        }
        items(state.skills, key = { it.id }) { skill ->
            SkillPreviewCard(skill = skill, onClick = onOpenSkills)
        }
        item { PrimaryPillButton("学习技能", onOpenSkills, Modifier.fillMaxWidth()) }
    }
}

@Composable
private fun QuickActionGrid(actions: List<HomeQuickAction>, onAction: (HomeActionTarget) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        actions.chunked(2).forEach { rowActions ->
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                rowActions.forEach { action ->
                    FeatureTile(
                        icon = action.icon,
                        title = action.title,
                        body = action.body,
                        onClick = { onAction(action.target) },
                        modifier = Modifier.weight(1f),
                        compact = true,
                    )
                }
            }
        }
    }
}

@Composable
private fun GearOverviewCard(overview: HomeGearOverview, onOpenGears: () -> Unit, onLogin: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(contentPadding = PaddingValues(16.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Badge(overview.eyebrow, tone = BadgeTone.Info)
                Text(overview.title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            }
            SoftPillButton("查看装备", onOpenGears)
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
        Text(stat.label, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
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
