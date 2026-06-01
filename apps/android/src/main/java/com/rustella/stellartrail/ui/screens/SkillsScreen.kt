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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.feature.skills.SkillsMode
import com.rustella.stellartrail.feature.skills.SkillsViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.IntroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
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
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            IntroCard(
                eyebrow = SkillsVisualContract.heroEyebrow,
                title = SkillsVisualContract.heroTitle,
                subtitle = SkillsVisualContract.heroSubtitle,
            )
        }
        if (state.error != null) {
            item {
                ErrorState(
                    message = state.error!!,
                    onRetry = when (state.mode) {
                        SkillsMode.Catalog -> viewModel::load
                        SkillsMode.Knots -> viewModel::loadKnots
                        SkillsMode.Favorites -> viewModel::loadFavoriteSkills
                    },
                )
            }
        }
        if (state.loading) item { LoadingState() }

        when (state.mode) {
            SkillsMode.Catalog -> {
                item { FavoriteEntryCard(onOpen = viewModel::openFavorites) }
                items(SkillsVisualContract.catalogCategories, key = { it.id }) { category ->
                    SkillCategoryCard(category, onOpen = viewModel::openKnots)
                }
            }
            SkillsMode.Knots -> {
                item {
                    SkillListHeader(
                        eyebrow = "绳结技能",
                        title = "绳结列表",
                        onBack = viewModel::openCatalog,
                    )
                }
                item {
                    SurfaceCard(containerColor = currentTrailPalette().chipBackground) {
                        Text("缓存全部后，离线模式也能查询绳结详情和动图。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                if (!state.loading && state.knots.isEmpty() && state.error == null) {
                    item { EmptyState("绳结内容准备中", "稍后会展示常用绳结。") }
                }
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
                } else if (state.knots.isNotEmpty()) {
                    item { Text("没有更多绳结了", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.fillMaxWidth()) }
                }
            }
            SkillsMode.Favorites -> {
                item {
                    SkillListHeader(
                        eyebrow = "我的收藏",
                        title = "收藏清单",
                        onBack = viewModel::openCatalog,
                    )
                }
                if (!state.loading && state.favoriteKnots.isEmpty() && state.error == null) {
                    item { EmptyState("还没有收藏技能", "在绳结列表或详情页点星标，就会出现在这里。") }
                }
                items(state.favoriteKnots, key = { it.knot.id }) { favorite ->
                    KnotCard(favorite.knot, onClick = { onOpenKnot(favorite.knot.id) })
                }
                if (state.favoriteNextOffset != null) {
                    item {
                        PrimaryPillButton(
                            text = if (state.loadingMore) "加载中..." else "加载更多收藏",
                            onClick = viewModel::loadMoreFavoriteSkills,
                            enabled = !state.loadingMore,
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                } else if (state.favoriteKnots.isNotEmpty()) {
                    item { Text("没有更多收藏了", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.fillMaxWidth()) }
                }
            }
        }
    }
}

@Composable
private fun FavoriteEntryCard(onOpen: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onOpen)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(44.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.warningBackground),
                contentAlignment = Alignment.Center,
            ) {
                Text("★", color = palette.warningText, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(SkillsVisualContract.favoriteTitle, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text(
                    SkillsVisualContract.favoriteDescription,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Text(SkillsVisualContract.favoriteAction, color = palette.accent, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.ExtraBold)
        }
    }
}

@Composable
private fun SkillCategoryCard(category: SkillCatalogCategory, onOpen: () -> Unit) {
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
                Text(category.icon, fontWeight = FontWeight.ExtraBold, color = palette.brandSoftText)
            }
            Column(Modifier.weight(1f)) {
                Text(category.subtitle, color = palette.accent, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold)
                Text(category.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text(category.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
        }
        CompactPillAction(category.actionText, onOpen, Modifier.align(Alignment.Start))
    }
}

@Composable
private fun SkillListHeader(
    eyebrow: String,
    title: String,
    onBack: () -> Unit,
) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(eyebrow, color = currentTrailPalette().accent, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold)
            Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        }
        CompactPillAction("返回", onBack, filled = false)
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
                }
                Text(knot.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                if (knot.aliases.isNotEmpty()) {
                    Text(
                        knot.aliases.joinToString(" / "),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
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
