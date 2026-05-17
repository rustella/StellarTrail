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
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.feature.home.HomeViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroButton
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.StatCard
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun HomeScreen(
    viewModel: HomeViewModel,
    onOpenGears: () -> Unit,
    onOpenSkills: () -> Unit,
    onOpenGear: (String) -> Unit,
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
                eyebrow = "寻径星野 · 出行准备",
                title = "今天准备去哪？",
                subtitle = "先核对装备库存、路线风险和技能清单，把下一次出行准备得更从容。",
                action = { HeroButton("装备库", onOpenGears) },
            )
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::load) }
        if (state.loading) item { LoadingState() }
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
        item { SectionTitle("最近装备", "延续微信端卡片式信息层级，快速查看近期更新。") }
        if (!state.loading && state.recentGears.isEmpty()) {
            item { EmptyState("暂无近期装备", "进入装备库添加第一件装备，开始建立你的出行清单。") }
        }
        items(state.recentGears, key = { it.id }) { gear ->
            GearPreviewCard(gear = gear, onClick = { onOpenGear(gear.id) })
        }
        item { PrimaryPillButton("管理全部装备", onOpenGears, Modifier.fillMaxWidth()) }
        item { SectionTitle("户外技能", "先掌握常用绳结与营地技能，再出发。") }
        if (!state.loading && state.skills.isEmpty()) {
            item { EmptyState("暂无技能分类", "稍后刷新或检查 API 地址。") }
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
