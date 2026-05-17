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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FloatingActionButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearSort
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearSummary
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.apiValue
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.gear.list.GearListViewModel
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
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.StatCard
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailPillShape

@Composable
fun GearListScreen(
    viewModel: GearListViewModel,
    onOpenGear: (String) -> Unit,
    onCreateGear: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = onCreateGear,
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("新增装备") },
                shape = TrailPillShape,
                elevation = FloatingActionButtonDefaults.elevation(defaultElevation = 6.dp),
            )
        },
    ) { innerPadding ->
        LazyColumn(
            Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background),
            contentPadding = PaddingValues(
                start = 16.dp,
                top = innerPadding.calculateTopPadding() + 16.dp,
                end = 16.dp,
                bottom = innerPadding.calculateBottomPadding() + 96.dp,
            ),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                HeroCard(
                    eyebrow = "寻径星野装备库",
                    title = "我的户外装备",
                    subtitle = "按分类、状态和关键词快速管理装备，为下一次出行做准备。",
                    action = { HeroButton("+ 添加", onCreateGear) },
                )
            }
            item {
                SurfaceCard(contentPadding = PaddingValues(8.dp)) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        GearTab.entries.forEach { tab ->
                            FilterChip(
                                selected = state.tab == tab,
                                onClick = { viewModel.setTab(tab) },
                                label = { Text(tab.label, fontWeight = FontWeight.Bold) },
                                modifier = Modifier.weight(1f),
                            )
                        }
                    }
                }
            }
            item {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    StatCard("当前", state.stats.currentCount.toString(), Modifier.weight(1f), hint = "可用装备")
                    StatCard("历史", state.stats.archivedCount.toString(), Modifier.weight(1f), hint = "归档装备")
                }
            }
            item {
                SurfaceCard {
                    OutlinedTextField(
                        value = state.query,
                        onValueChange = viewModel::updateQuery,
                        label = { Text("搜索装备名、品牌、型号") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        trailingIcon = { TextButton(onClick = viewModel::submitSearch) { Text("搜索") } },
                    )
                }
            }
            item { CategoryStrip(state.selectedCategory, state.categories.items, viewModel::setCategory) }
            item {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    StatusDropdown(state.selectedStatus, viewModel::setStatus, Modifier.weight(1f))
                    SortDropdown(state.sort, viewModel::setSort, Modifier.weight(1f))
                }
            }
            item {
                SoftPillButton("刷新列表", viewModel::refresh, Modifier.fillMaxWidth())
            }
            if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::refresh) }
            if (state.loading) item { LoadingState() }
            if (!state.loading && state.gears.isEmpty()) {
                item { EmptyState("暂无装备", "点击右下角新增你的第一件装备。") }
            }
            items(state.gears, key = { it.id }) { gear ->
                GearSummaryCard(
                    gear = gear,
                    onClick = { onOpenGear(gear.id) },
                    onArchive = { viewModel.archive(gear.id) },
                    onRestore = { viewModel.restore(gear.id) },
                    showRestore = state.tab == GearTab.HISTORY,
                )
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
private fun CategoryStrip(
    selectedCategory: GearCategory?,
    categories: List<com.rustella.stellartrail.domain.gear.GearCategoryFilter>,
    onChange: (GearCategory?) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            FilterChip(
                selected = selectedCategory == null,
                onClick = { onChange(null) },
                label = { Text("全部分类", fontWeight = FontWeight.Bold) },
            )
        }
        items(categories, key = { it.id }) { filter ->
            val category = GearCategory.entries.firstOrNull { it.apiValue() == filter.id }
            if (category != null) {
                FilterChip(
                    selected = selectedCategory == category,
                    onClick = { onChange(category) },
                    label = { Text("${filter.label} ${filter.count}", fontWeight = FontWeight.Bold) },
                )
            }
        }
    }
}

@Composable
private fun GearSummaryCard(
    gear: GearSummary,
    onClick: () -> Unit,
    onArchive: () -> Unit,
    onRestore: () -> Unit,
    showRestore: Boolean,
) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(gear.categoryLabel)
            Badge(gear.statusLabel, tone = statusTone(gear.status))
        }
        Text(gear.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(gear.brand, gear.model), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(gear.weightG), Modifier.weight(1f))
            MetricTile("价格", formatPrice(gear.purchasePriceCents), Modifier.weight(1f))
            MetricTile("购买日期", gear.purchaseDate ?: "未记录", Modifier.weight(1f))
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            if (showRestore) {
                OutlinedButton(onClick = onRestore, shape = TrailPillShape) { Text("恢复") }
            } else {
                OutlinedButton(
                    onClick = onArchive,
                    shape = TrailPillShape,
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                ) { Text("移入历史") }
            }
        }
    }
}

@Composable
private fun CategoryDropdown(value: GearCategory?, onChange: (GearCategory?) -> Unit, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    Column(modifier) {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth(), shape = TrailPillShape) { Text(value?.label ?: "全部分类") }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("全部分类") }, onClick = { onChange(null); expanded = false })
            GearCategory.entries.forEach { category ->
                DropdownMenuItem(text = { Text(category.label) }, onClick = { onChange(category); expanded = false })
            }
        }
    }
}

@Composable
private fun StatusDropdown(value: GearStatus?, onChange: (GearStatus?) -> Unit, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    Column(modifier) {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth(), shape = TrailPillShape) { Text(value?.label ?: "全部状态") }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("全部状态") }, onClick = { onChange(null); expanded = false })
            GearStatus.entries.forEach { status ->
                DropdownMenuItem(text = { Text(status.label) }, onClick = { onChange(status); expanded = false })
            }
        }
    }
}

@Composable
private fun SortDropdown(value: GearSort, onChange: (GearSort) -> Unit, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    Column(modifier) {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth(), shape = TrailPillShape) { Text(value.label) }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            GearSort.entries.forEach { sort ->
                DropdownMenuItem(text = { Text(sort.label) }, onClick = { onChange(sort); expanded = false })
            }
        }
    }
}

private fun statusTone(status: GearStatus): BadgeTone = when (status) {
    GearStatus.AVAILABLE, GearStatus.IN_USE -> BadgeTone.Success
    GearStatus.MAINTENANCE, GearStatus.IDLE -> BadgeTone.Warning
    GearStatus.DAMAGED, GearStatus.LOST -> BadgeTone.Danger
    GearStatus.RETIRED, GearStatus.SOLD -> BadgeTone.Neutral
}
