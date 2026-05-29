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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.packing.GearPackingListDetail
import com.rustella.stellartrail.domain.packing.GearPackingListSummary
import com.rustella.stellartrail.feature.packing.PackingViewModel
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
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun PackingListsScreen(
    viewModel: PackingViewModel,
    isLoggedIn: Boolean,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(isLoggedIn) { viewModel.refresh(isLoggedIn) }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "出发物品",
                title = "打包清单",
                subtitle = "按行程和场景逐项确认装备，减少出发前遗漏。",
                actions = {
                    HeroSoftButton("返回", onBack, Modifier.weight(1f))
                    HeroButton("新建", viewModel::createDefault, Modifier.weight(1f))
                },
            )
        }
        if (!isLoggedIn) {
            item {
                SurfaceCard {
                    Text("登录后管理打包清单", fontWeight = FontWeight.ExtraBold)
                    Text("清单会保存到账号中，也可导入到行程个人装备。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    PrimaryPillButton("去登录", onLogin)
                }
            }
        } else {
            if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.refresh(true) }) }
            if (state.loading) item { LoadingState() }
            state.selectedDetail?.let {
                item { PackingDetailCard(it, onClose = viewModel::closeDetail, onToggle = viewModel::toggleItem) }
            }
            if (!state.loading && state.lists.isEmpty()) item { EmptyState("还没有打包清单", "新建一份清单后，从装备库添加需要准备的物品。") }
            items(state.lists, key = { it.id }) { list ->
                PackingSummaryCard(list = list, onClick = { viewModel.open(list.id) })
            }
        }
    }
}
@Composable
private fun PackingSummaryCard(list: GearPackingListSummary, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge("打包清单")
            Badge("${list.packedItems}/${list.totalItems}", tone = BadgeTone.Info)
        }
        Text(list.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(list.description ?: "逐项确认出发物品", color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            MetricTile("总重量", formatWeight(list.totalWeightG), Modifier.weight(1f))
            MetricTile("已打包", formatWeight(list.packedWeightG), Modifier.weight(1f))
        }
    }
}

@Composable
private fun PackingDetailCard(detail: GearPackingListDetail, onClose: () -> Unit, onToggle: (String, Int) -> Unit) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(detail.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            SoftPillButton("收起", onClose)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            MetricTile("总物品", detail.stats.totalItems.toString(), Modifier.weight(1f))
            MetricTile("已打包", detail.stats.packedItems.toString(), Modifier.weight(1f))
            MetricTile("总重量", formatWeight(detail.stats.totalWeightG), Modifier.weight(1f))
        }
        detail.items.forEach { item ->
            SurfaceCard(Modifier.fillMaxWidth()) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Column(Modifier.weight(1f)) {
                        Text(item.name, fontWeight = FontWeight.ExtraBold)
                        Text("${item.categoryLabel} · ${item.plannedQuantity} 件", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    SoftPillButton(
                        if (item.packedQuantity >= item.plannedQuantity) "取消" else "打包",
                        { onToggle(item.id, if (item.packedQuantity >= item.plannedQuantity) 0 else item.plannedQuantity) },
                    )
                }
            }
        }
    }
}
