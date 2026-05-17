package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
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
import com.rustella.stellartrail.ui.common.NoticeCard
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.StatCard
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TagList

@Composable
fun HomeScreen(
    viewModel: HomeViewModel,
    onOpenGears: () -> Unit,
    onOpenSkills: () -> Unit,
    onOpenGear: (String) -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "寻径星野 · 出发前检查",
                title = "今天准备好出发了吗？",
                subtitle = "先核对出行清单，再补一项户外技能。这里沿用微信真机的轻卡片、圆角和柔和品牌色。",
                chips = listOf(if (state.isLoggedIn) "我的装备已保存" else "可先浏览清单", "绳结教学可直接看"),
                actions = {
                    HeroButton("检查装备", onOpenGears)
                    HeroSoftButton("学习技能", onOpenSkills)
                },
            )
        }
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                FeatureTile("🎒", "装备清单", "按周末轻徒步、露营等场景检查。", onOpenGears, Modifier.weight(1f))
                FeatureTile("🪢", "绳结技能", "直接查看步骤，出发前复习。", onOpenSkills, Modifier.weight(1f))
            }
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.load(state.isLoggedIn) }) }
        if (state.loading) item { LoadingState() }
        item { SectionTitle("装备概览", "登录后同步自己的库存、重量与估值；登录前先看参考清单。") }
        if (!state.isLoggedIn) {
            item {
                NoticeCard(
                    title = "可以先查看出行清单",
                    body = "登录后再管理自己的装备、重量和估值。",
                    action = { PrimaryPillButton("去登录", onLogin) },
                )
            }
        }
        if (state.isLoggedIn) {
            item {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    StatCard("可用装备", state.stats.currentCount.toString(), Modifier.weight(1f), hint = "当前库存")
                    StatCard("历史装备", state.stats.archivedCount.toString(), Modifier.weight(1f), hint = "归档记录")
                }
            }
            item {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    StatCard("总重量", formatWeight(state.stats.totalWeightG), Modifier.weight(1f), hint = "轻量优先")
                    StatCard("总价值", formatPrice(state.stats.totalValueCents), Modifier.weight(1f), hint = "预算参考")
                }
            }
            if (state.recentGears.isNotEmpty()) {
                item { SectionTitle("最近装备", "快速查看近期更新。") }
                items(state.recentGears, key = { it.id }) { gear ->
                    GearPreviewCard(gear = gear, onClick = { onOpenGear(gear.id) })
                }
            }
        }
        item { SectionTitle("出行装备参考", "来自微信端同款公开清单，先按场景准备，再登录保存。") }
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
