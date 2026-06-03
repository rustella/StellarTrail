package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.allGearSorts
import com.rustella.stellartrail.domain.gear.allGearStatuses
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.gear.list.GearListUiState
import com.rustella.stellartrail.feature.gear.list.GearListViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.CompactTextInput
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.TrailPillShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun GearListScreen(
    viewModel: GearListViewModel,
    onOpenGear: (String) -> Unit,
    onEditGear: (String) -> Unit,
    onCreateGear: () -> Unit,
    onOpenAtlas: () -> Unit,
    onOpenPackingLists: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var pendingDelete by remember { mutableStateOf<GearSummary?>(null) }
    LazyColumn(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(start = 16.dp, top = 0.dp, end = 16.dp, bottom = 28.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        item {
            GearQuickEntries(
                onOpenAtlas = onOpenAtlas,
                onOpenPackingLists = onOpenPackingLists,
                showPackingLists = state.isLoggedIn,
            )
        }
        if (!state.isLoggedIn) {
            item { GuestGearLoginCard(onLogin) }
        }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.refresh(state.isLoggedIn) }) }
        if (state.loading) item { LoadingState() }
        if (state.isLoggedIn) {
            item { GearStatsPanel(state = state) }
            item { GearToolbar(viewModel = viewModel, onCreateGear = onCreateGear) }
            item { GearFilterSummary(state = state, onReset = { viewModel.resetFilters() }) }
            if (!state.loading && state.gears.isEmpty()) {
                item { EmptyState("还没有装备", "用装备卡片记录重量、价格、存放位置和标签备注。") }
            }
            items(state.gears, key = { it.id }) { gear ->
                GearCard(
                    gear = gear,
                    onClick = { onOpenGear(gear.id) },
                    onEdit = { onEditGear(gear.id) },
                    onDelete = { pendingDelete = gear },
                )
            }
            if (state.nextCursor != null) {
                item {
                    PrimaryPillButton(
                        text = if (state.loadingMore) "继续加载..." else "加载更多",
                        onClick = viewModel::loadMore,
                        enabled = !state.loadingMore,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }
        }
    }
    pendingDelete?.let { gear ->
        AlertDialog(
            onDismissRequest = { pendingDelete = null },
            title = { Text("删除这件装备？") },
            text = { Text("删除后不会出现在装备列表中，已有打包清单会保留历史条目。") },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.delete(gear.id)
                        pendingDelete = null
                    },
                ) {
                    Text("删除", color = currentTrailPalette().dangerText, fontWeight = FontWeight.ExtraBold)
                }
            },
            dismissButton = {
                TextButton(onClick = { pendingDelete = null }) {
                    Text("取消")
                }
            },
        )
    }
}

@Composable
private fun GearQuickEntries(onOpenAtlas: () -> Unit, onOpenPackingLists: () -> Unit, showPackingLists: Boolean) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        GearEntryCard(
            title = "装备图鉴",
            body = "浏览市面装备",
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
        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 10.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(34.dp)
                    .clip(TrailInnerCardShape)
                    .background(palette.brandSoft),
                contentAlignment = Alignment.Center,
            ) {
                Text(icon, color = palette.brandSoftText, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold, maxLines = 1)
                Text(
                    body,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
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
                Text("未登录也可先浏览", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                Text(
                    "可以先看装备图鉴；要保存自己的装备时再登录。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            CompactPillAction("登录", onLogin)
        }
    }
}

@Composable
private fun GearStatsPanel(state: GearListUiState) {
    SurfaceCard(contentPadding = PaddingValues(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(
                "当前库存汇总",
                color = currentTrailPalette().headingMuted,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.ExtraBold,
            )
            GearSoftLabel("详细统计")
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            GearStatTile(state.stats.currentCount.toString(), "装备数量", Modifier.weight(1f))
            GearStatTile(categoryCount(state).toString(), "分类数", Modifier.weight(1f))
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            GearStatTile(formatWeight(state.stats.totalWeightG), "总重量", Modifier.weight(1f))
            GearStatTile(formatPrice(state.stats.totalValueCents), "价值", Modifier.weight(1f))
        }
    }
}

@Composable
private fun GearStatTile(value: String, label: String, modifier: Modifier = Modifier) {
    val palette = currentTrailPalette()
    Column(
        modifier = modifier
            .clip(TrailInnerCardShape)
            .background(palette.controlBackground)
            .padding(horizontal = 10.dp, vertical = 10.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(value, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold, maxLines = 1)
        Text(label, color = palette.textMuted, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold)
    }
}

@Composable
private fun GearToolbar(viewModel: GearListViewModel, onCreateGear: () -> Unit) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val palette = currentTrailPalette()
    SurfaceCard(contentPadding = PaddingValues(8.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            CompactTextInput(
                value = state.query,
                onValueChange = viewModel::updateQuery,
                placeholder = "搜索装备名、品牌、型号",
                modifier = Modifier.weight(1f),
            )
            GearFilterButton(viewModel)
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(TrailPillShape)
                    .background(palette.brand)
                    .clickable(onClick = onCreateGear),
                contentAlignment = Alignment.Center,
            ) {
                Text("+", color = palette.brandText, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
            }
        }
    }
}

@Composable
private fun GearFilterButton(viewModel: GearListViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val palette = currentTrailPalette()
    var expanded by remember { mutableStateOf(false) }
    val active = activeFilterCount(state) > 0
    Box {
        Text(
            text = "筛选",
            modifier = Modifier
                .height(40.dp)
                .clip(TrailPillShape)
                .background(if (active) palette.brandSoft else palette.softControlBackground)
                .clickable { expanded = true }
                .padding(horizontal = 16.dp, vertical = 10.dp),
            color = if (active) palette.brandSoftText else palette.softControlText,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.ExtraBold,
        )
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("全部装备") }, onClick = { viewModel.setCategory(null); expanded = false })
            GearCategory.entries.forEach { category ->
                DropdownMenuItem(text = { Text("分类 · ${category.label}") }, onClick = { viewModel.setCategory(category); expanded = false })
            }
            DropdownMenuItem(text = { Text("全部状态") }, onClick = { viewModel.setStatus(null); expanded = false })
            allGearStatuses.forEach { status ->
                DropdownMenuItem(text = { Text("状态 · ${status.label}") }, onClick = { viewModel.setStatus(status); expanded = false })
            }
            allGearSorts.forEach { sort ->
                DropdownMenuItem(text = { Text("排序 · ${sort.label}") }, onClick = { viewModel.setSort(sort); expanded = false })
            }
        }
    }
}

@Composable
private fun GearFilterSummary(state: GearListUiState, onReset: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            activeFilterText(state),
            modifier = Modifier.weight(1f),
            color = currentTrailPalette().textMuted,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (activeFilterCount(state) > 0) {
            GearActionPill("重置", onClick = onReset, filled = false)
        }
    }
}

@Composable
private fun GearCard(gear: GearSummary, onClick: () -> Unit, onEdit: () -> Unit, onDelete: () -> Unit) {
    val palette = currentTrailPalette()
    SurfaceCard(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick),
        contentPadding = PaddingValues(14.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
            Text(
                gear.name,
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.ExtraBold,
                lineHeight = MaterialTheme.typography.titleMedium.lineHeight,
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Badge(gear.categoryLabel)
            Badge(gear.statusLabel, tone = statusTone(gear.status))
        }
        Text(
            joinBrandModel(gear.brand, gear.model),
            color = palette.textMuted,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Row(
            Modifier
                .fillMaxWidth()
                .clip(TrailInnerCardShape)
                .background(palette.controlBackground)
                .padding(horizontal = 10.dp, vertical = 9.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            GearFact("重量", formatWeight(gear.weightG))
            GearFact("估价", formatPrice(gear.purchasePriceCents))
            GearFact("购入", gear.purchaseDate ?: "未记录")
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            GearActionPill("编辑", onClick = onEdit)
            Spacer(Modifier.size(8.dp))
            GearActionPill("删除", onClick = onDelete, danger = true)
        }
    }
}

@Composable
private fun GearFact(label: String, value: String) {
    Text(
        "$label $value",
        color = currentTrailPalette().headingMuted,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.ExtraBold,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun GearSoftLabel(text: String) {
    val palette = currentTrailPalette()
    Text(
        text = text,
        modifier = Modifier
            .clip(TrailPillShape)
            .background(palette.brandSoft)
            .padding(horizontal = 12.dp, vertical = 7.dp),
        color = palette.brandSoftText,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.ExtraBold,
    )
}

@Composable
private fun GearActionPill(text: String, onClick: () -> Unit, filled: Boolean = true, danger: Boolean = false) {
    val palette = currentTrailPalette()
    val background = when {
        danger -> palette.dangerBackground
        filled -> palette.brandSoft
        else -> palette.brandSoft
    }
    val foreground = when {
        danger -> palette.dangerText
        else -> palette.brandSoftText
    }
    Text(
        text = text,
        modifier = Modifier
            .clip(TrailPillShape)
            .background(background)
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 7.dp),
        color = foreground,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.ExtraBold,
    )
}

private fun categoryCount(state: GearListUiState): Int {
    val statsCount = state.stats.byCategory.count { it.count > 0 }
    if (statsCount > 0) return statsCount
    return state.categories.items.count { it.id != "all" && it.count > 0 }
}

private fun activeFilterText(state: GearListUiState): String {
    val category = state.selectedCategory?.label ?: "全部装备"
    val status = state.selectedStatus?.label ?: "全部状态"
    val sort = when (state.sort) {
        GearSort.CREATED_AT_DESC -> "最近添加"
        GearSort.CREATED_AT_ASC -> "最早添加"
        GearSort.PURCHASE_DATE_DESC -> "最近购入"
        GearSort.NAME_ASC -> "名称排序"
        GearSort.WEIGHT_DESC -> "重量优先"
        GearSort.PRICE_DESC -> "价格优先"
    }
    return "$category · $status · $sort"
}

private fun activeFilterCount(state: GearListUiState): Int {
    var count = 0
    if (state.selectedCategory != null) count += 1
    if (state.selectedStatus != null) count += 1
    if (state.sort != GearSort.CREATED_AT_DESC) count += 1
    return count
}

private fun statusTone(status: GearStatus): BadgeTone = when (status) {
    GearStatus.AVAILABLE, GearStatus.IN_USE -> BadgeTone.Success
    GearStatus.MAINTENANCE, GearStatus.IDLE -> BadgeTone.Warning
    GearStatus.DAMAGED, GearStatus.LOST -> BadgeTone.Danger
    GearStatus.RETIRED, GearStatus.SOLD -> BadgeTone.Neutral
}
