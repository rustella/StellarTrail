package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AssistChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.feature.skills.SkillsViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.IntroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun SkillsScreen(
    viewModel: SkillsViewModel,
    onOpenKnot: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var showKnots by remember { mutableStateOf(false) }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            IntroCard(
                eyebrow = "寻径星野技能库",
                title = "户外技能",
                subtitle = "绳结、扎营、打包、天气和急救知识，出发前随时复习。",
            )
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::load) }
        if (state.loading) item { LoadingState() }
        if (!state.loading && state.categories.isEmpty()) item { EmptyState("暂无分类", "请稍后刷新或检查网络。") }
        items(state.categories, key = { it.id }) { category -> SkillCategoryCard(category, onOpen = { showKnots = true }) }
        if (showKnots) {
            item { SectionTitle("绳结技能") }
            items(state.knots, key = { it.id }) { knot -> KnotCard(knot, onClick = { onOpenKnot(knot.id) }) }
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
    }
}

@Composable
private fun SkillCategoryCard(category: SkillCategorySummary, onOpen: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(Modifier.fillMaxWidth()) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(44.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.controlBackground),
                contentAlignment = Alignment.Center,
            ) {
                Text("结", fontWeight = FontWeight.ExtraBold, color = palette.brandSoftText)
            }
            Column(Modifier.weight(1f)) {
                Text(category.slug.replaceFirstChar { it.uppercase() }, color = palette.accent, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold)
                Text(category.title.removeSuffix("技能"), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text(category.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
        }
        CompactPillAction("查看绳结列表", onOpen, Modifier.align(Alignment.Start))
    }
}

@Composable
private fun KnotCard(knot: KnotSummary, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(
                Modifier
                    .size(width = 82.dp, height = 62.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.controlBackground),
                contentAlignment = Alignment.Center,
            ) {
                Text("绳结", color = palette.textMuted, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f)) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Badge(knot.categories.firstOrNull()?.title ?: "绳结", tone = BadgeTone.Info)
                    Badge(knot.difficulty ?: "入门", tone = BadgeTone.Warning)
                }
                Text(knot.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                Text(knot.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
                if (knot.types.isNotEmpty()) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        knot.types.take(2).forEach { item ->
                            AssistChip(onClick = {}, label = { Text(item.title) })
                        }
                    }
                }
            }
        }
    }
}
