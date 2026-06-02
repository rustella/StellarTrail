package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.allGearSorts
import com.rustella.stellartrail.domain.gear.allGearStatuses
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.gear.list.GearListViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.IntroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.StatCard
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.TrailPillShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun GearListScreen(
    viewModel: GearListViewModel,
    onOpenGear: (String) -> Unit,
    onCreateGear: () -> Unit,
    onOpenAtlas: () -> Unit,
    onOpenPackingLists: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
            item {
                IntroCard(
                    eyebrow = "寻径星野装备库",
                    title = "户外装备库",
                    subtitle = if (state.isLoggedIn) {
                        "记录自己的装备、重量、价格和历史状态。"
                    } else {
                        "登录后记录自己的装备、重量、价格和历史状态。"
                    },
                    actionText = "+ 添加",
                    onAction = if (state.isLoggedIn) onCreateGear else onLogin,
                )
            }
            item { GearQuickEntries(onOpenAtlas = onOpenAtlas, onOpenPackingLists = onOpenPackingLists, showPackingLists = state.isLoggedIn) }
            if (!state.isLoggedIn) {
                item { GuestGearLoginCard(onLogin) }
            }
            if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.refresh(state.isLoggedIn) }) }
            if (state.loading) item { LoadingState() }
            if (state.isLoggedIn) {
                item { GearControls(viewModel) }
                item {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        StatCard("可用装备", state.stats.currentCount.toString(), Modifier.weight(1f), hint = "当前库存")
                        StatCard("历史装备", state.stats.archivedCount.toString(), Modifier.weight(1f), hint = "归档记录")
                    }
                }
                item {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        StatCard("总重量", formatWeight(state.stats.totalWeightG), Modifier.weight(1f), hint = "背包负重")
                        StatCard("总价值", formatPrice(state.stats.totalValueCents), Modifier.weight(1f), hint = "预算参考")
                    }
                }
                item { SectionTitle("我的装备") }
                if (!state.loading && state.gears.isEmpty()) {
                    item { EmptyState("暂无装备", "点击右下角新增第一件装备。") }
                }
                items(state.gears, key = { it.id }) { gear ->
                    GearCard(gear = gear, onClick = { onOpenGear(gear.id) })
                }
                if (state.nextCursor != null) {
                    item {
                        PrimaryPillButton(
                            text = if (state.loadingMore) "加载中..." else "加载更多",
                            onClick = viewModel::loadMore,
                            enabled = !state.loadingMore,
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
        }
    }
}

@Composable
private fun GearQuickEntries(onOpenAtlas: () -> Unit, onOpenPackingLists: () -> Unit, showPackingLists: Boolean) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        GearEntryCard(
            title = "装备图鉴",
            body = "浏览已审核收录的市面装备",
            icon = "册",
            onClick = onOpenAtlas,
            modifier = Modifier.weight(1f),
        )
        if (showPackingLists) {
            GearEntryCard(
                title = "打包清单",
                body = "准备出发物品",
                icon = "✓",
                onClick = onOpenPackingLists,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
private fun GearEntryCard(title: String, body: String, icon: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    SurfaceCard(
        modifier = modifier.fillMaxWidth().clickable(onClick = onClick),
        contentPadding = PaddingValues(horizontal = 13.dp, vertical = 11.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(32.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.brandSoft),
                contentAlignment = Alignment.Center,
            ) {
                Text(icon, color = palette.brandSoftText, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
private fun GuestGearLoginCard(onLogin: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(
        modifier = Modifier.fillMaxWidth(),
        contentPadding = PaddingValues(horizontal = 13.dp, vertical = 14.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(38.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.brandSoft),
                contentAlignment = Alignment.Center,
            ) {
                Text("包", color = palette.brandSoftText, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                Text("登录后管理装备", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                Text(
                    "保存自己的装备、重量和估值，出发前整理清单更安心。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            CompactPillAction("登录", onLogin)
        }
    }
}

@Composable
private fun GearControls(viewModel: GearListViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            FilterChip(
                selected = state.tab == GearTab.AVAILABLE,
                onClick = { viewModel.setTab(GearTab.AVAILABLE) },
                label = { Text(GearTab.AVAILABLE.label, fontWeight = FontWeight.Bold) },
            )
            FilterChip(
                selected = state.tab == GearTab.HISTORY,
                onClick = { viewModel.setTab(GearTab.HISTORY) },
                label = { Text(GearTab.HISTORY.label, fontWeight = FontWeight.Bold) },
            )
        }
        OutlinedTextField(
            value = state.query,
            onValueChange = viewModel::updateQuery,
            label = { Text("搜索装备、品牌或型号") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            trailingIcon = { TextButton(onClick = viewModel::submitSearch) { Text("搜索") } },
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            item {
                FilterChip(
                    selected = state.selectedCategory == null,
                    onClick = { viewModel.setCategory(null) },
                    label = { Text("全部") },
                )
            }
            items(GearCategory.entries, key = { it.name }) { category ->
                FilterChip(
                    selected = state.selectedCategory == category,
                    onClick = { viewModel.setCategory(category) },
                    label = { Text(category.label) },
                )
            }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            StatusPicker(state.selectedStatus, viewModel::setStatus, Modifier.weight(1f))
            SortPicker(state.sort, viewModel::setSort, Modifier.weight(1f))
        }
    }
}

@Composable
private fun StatusPicker(value: GearStatus?, onChange: (GearStatus?) -> Unit, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    Column(modifier) {
        SoftPillButton(value?.label ?: "全部状态", { expanded = true }, Modifier.fillMaxWidth())
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("全部状态") }, onClick = { onChange(null); expanded = false })
            allGearStatuses.forEach { status ->
                DropdownMenuItem(text = { Text(status.label) }, onClick = { onChange(status); expanded = false })
            }
        }
    }
}

@Composable
private fun SortPicker(value: GearSort, onChange: (GearSort) -> Unit, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    Column(modifier) {
        SoftPillButton(value.label, { expanded = true }, Modifier.fillMaxWidth())
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            allGearSorts.forEach { sort ->
                DropdownMenuItem(text = { Text(sort.label) }, onClick = { onChange(sort); expanded = false })
            }
        }
    }
}

@Composable
private fun GearCard(gear: GearSummary, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(gear.categoryLabel)
            Badge(gear.statusLabel, tone = statusTone(gear.status))
        }
        Text(gear.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(gear.brand, gear.model), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(gear.weightG), Modifier.weight(1f))
            MetricTile("价格", formatPrice(gear.purchasePriceCents), Modifier.weight(1f))
            MetricTile("购买", gear.purchaseDate ?: "未记录", Modifier.weight(1f))
        }
    }
}

private fun statusTone(status: GearStatus): BadgeTone = when (status) {
    GearStatus.AVAILABLE, GearStatus.IN_USE -> BadgeTone.Success
    GearStatus.MAINTENANCE, GearStatus.IDLE -> BadgeTone.Warning
    GearStatus.DAMAGED, GearStatus.LOST -> BadgeTone.Danger
    GearStatus.RETIRED, GearStatus.SOLD -> BadgeTone.Neutral
}
