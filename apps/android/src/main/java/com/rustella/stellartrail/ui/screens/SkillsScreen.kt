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
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AssistChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.feature.skills.SkillsViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun SkillsScreen(viewModel: SkillsViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "寻径星野技能库",
                title = "户外技能",
                subtitle = "绳结、营地、急救和打包等出行前技能，按微信端信息卡片节奏组织。",
            )
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::load) }
        if (state.loading) item { LoadingState() }
        item { SectionTitle("技能分类", "先从主题分类进入，再查看具体绳结步骤。") }
        if (!state.loading && state.categories.isEmpty()) item { EmptyState("暂无分类", "请稍后刷新或检查 API 地址。") }
        items(state.categories, key = { it.id }) { category -> SkillCategoryCard(category) }
        item { SectionTitle("绳结库", "点击条目查看步骤。") }
        items(state.knots, key = { it.id }) { knot -> KnotCard(knot, onClick = { viewModel.openKnot(knot.id) }) }
        if (state.nextOffset != null) {
            item {
                PrimaryPillButton(
                    text = if (state.loadingMore) "加载中..." else "加载更多绳结",
                    onClick = viewModel::loadMoreKnots,
                    enabled = !state.loadingMore,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
    state.selectedKnot?.let { KnotDetailDialog(it, onDismiss = viewModel::closeKnot) }
}

@Composable
private fun SkillCategoryCard(category: SkillCategorySummary) {
    SurfaceCard(Modifier.fillMaxWidth()) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge("技能分类")
            Badge("${category.itemCount} 项", tone = BadgeTone.Info)
        }
        Text(category.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(category.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun KnotCard(knot: KnotSummary, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(knot.difficulty ?: "绳结")
            Badge("查看步骤", tone = BadgeTone.Brand)
        }
        Text(knot.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(knot.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (knot.categories.isNotEmpty() || knot.types.isNotEmpty()) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                (knot.categories + knot.types).take(3).forEach { item ->
                    AssistChip(onClick = {}, label = { Text(item.title) })
                }
            }
        }
    }
}

@Composable
private fun KnotDetailDialog(knot: KnotDetail, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = { TextButton(onClick = onDismiss) { Text("关闭") } },
        title = { Text(knot.title) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(knot.description ?: knot.summary)
                if (knot.steps.isNotEmpty()) {
                    Text("步骤", fontWeight = FontWeight.Bold)
                    knot.steps.forEachIndexed { index, step -> Text("${index + 1}. $step") }
                }
                if (knot.media.isNotEmpty()) {
                    Text("媒体素材：${knot.media.size} 个（当前版本暂不内嵌展示）", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        },
    )
}
