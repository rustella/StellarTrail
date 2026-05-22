package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearFormState
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.atlas.detail.GearAtlasDetailViewModel
import com.rustella.stellartrail.feature.atlas.list.GearAtlasListViewModel
import com.rustella.stellartrail.feature.atlas.submit.GearAtlasSubmitViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.CompactTextInput
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.IntroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TagList

@Composable
fun GearAtlasListScreen(
    viewModel: GearAtlasListViewModel,
    onOpenItem: (String) -> Unit,
    onSubmit: () -> Unit,
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
                eyebrow = "装备图鉴",
                title = "装备图鉴",
                subtitle = "浏览已审核收录的市面装备，也可以投稿补充新装备。",
                actionText = "投稿",
                onAction = onSubmit,
            )
        }
        item {
            SurfaceCard(Modifier.fillMaxWidth(), contentPadding = PaddingValues(12.dp)) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    CompactTextInput(
                        value = state.query,
                        onValueChange = viewModel::updateQuery,
                        placeholder = "搜索装备名、品牌、型号",
                        modifier = Modifier.weight(1f),
                    )
                    CompactPillAction("搜索", viewModel::submitSearch)
                    if (state.query.isNotBlank()) {
                        CompactPillAction("清除", viewModel::clearSearch, filled = false)
                    }
                }
            }
        }
        item { AtlasCategoryRow(state.selectedCategory, viewModel::selectCategory) }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::refresh) }
        if (state.loading) item { LoadingState() }
        if (!state.loading && state.items.isEmpty() && state.error == null) {
            item {
                EmptyState(
                    title = "还没有收录装备",
                    body = "可以先提交一件装备，审核通过后会展示在这里。",
                )
            }
        }
        items(state.items, key = { it.id }) { item ->
            GearAtlasCard(item, onClick = { onOpenItem(item.id) })
        }
        if (state.loadingMore) item { LoadingState() }
        if (state.nextCursor != null) {
            item { PrimaryPillButton("加载更多", viewModel::loadMore, Modifier.fillMaxWidth()) }
        } else if (state.items.isNotEmpty()) {
            item {
                Text(
                    "没有更多装备了",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth().padding(8.dp),
                )
            }
        }
    }
}

@Composable
private fun AtlasCategoryRow(selected: GearCategory?, onSelect: (GearCategory?) -> Unit) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        item {
            FilterChip(
                selected = selected == null,
                onClick = { onSelect(null) },
                label = { Text("全部", fontWeight = FontWeight.Bold) },
            )
        }
        items(GearCategory.entries, key = { it.name }) { category ->
            FilterChip(
                selected = selected == category,
                onClick = { onSelect(category) },
                label = { Text(category.label, fontWeight = FontWeight.Bold) },
            )
        }
    }
}

@Composable
private fun GearAtlasCard(item: GearAtlasPublicItem, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(item.categoryLabel ?: item.category.label)
            Badge("已收录", tone = BadgeTone.Success)
        }
        Text(item.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(item.brand, item.model), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        Text(item.description ?: "暂无描述", color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(item.weightG), Modifier.weight(1f))
            MetricTile("官方价", formatPrice(item.officialPriceCents, item.officialPriceCurrency), Modifier.weight(1f))
        }
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun GearAtlasDetailScreen(
    viewModel: GearAtlasDetailViewModel,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = { Text(state.item?.name ?: "图鉴详情", fontWeight = FontWeight.ExtraBold) },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回") } },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
            )
        },
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).padding(innerPadding),
            contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::load) }
            if (state.loading) item { LoadingState() }
            state.item?.let { item ->
                item { GearAtlasDetailContent(item) }
            }
            if (!state.loading && state.item == null && state.error == null) {
                item { EmptyState("暂无图鉴详情", "返回列表后重新打开。") }
            }
        }
    }
}

@Composable
private fun GearAtlasDetailContent(item: GearAtlasPublicItem) {
    SurfaceCard(contentPadding = PaddingValues(16.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Badge(item.categoryLabel ?: item.category.label)
            Badge("已收录", tone = BadgeTone.Success)
        }
        Text(item.name, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(item.brand, item.model), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(item.weightG), Modifier.weight(1f))
            MetricTile("官方价", formatPrice(item.officialPriceCents, item.officialPriceCurrency), Modifier.weight(1f))
            MetricTile("收录时间", item.approvedAt?.take(10) ?: "未记录", Modifier.weight(1f))
        }
    }
    SurfaceCard {
        SectionTitle("基本信息")
        MetadataRow("分类", item.categoryLabel ?: item.category.label)
        MetadataRow("品牌", item.brand ?: "未记录")
        MetadataRow("型号", item.model ?: "未记录")
        MetadataRow("描述", item.description ?: "暂无描述")
    }
    SurfaceCard {
        SectionTitle("可公开信息")
        MetadataRow("重量", formatWeight(item.weightG))
        MetadataRow("官方价格", formatPrice(item.officialPriceCents, item.officialPriceCurrency))
        item.specs.orEmpty().forEach { (key, value) -> MetadataRow(key, value) }
    }
}

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun GearAtlasSubmitScreen(
    viewModel: GearAtlasSubmitViewModel,
    isLoggedIn: Boolean,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    onSubmitted: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(isLoggedIn) { viewModel.setLoggedIn(isLoggedIn) }
    LaunchedEffect(state.submitted) {
        if (state.submitted) onSubmitted()
    }
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = { Text("投稿装备", fontWeight = FontWeight.ExtraBold) },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回") } },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
            )
        },
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).padding(innerPadding),
            contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            if (!state.isLoggedIn) {
                item {
                    SurfaceCard(
                        modifier = Modifier.fillMaxWidth(),
                        contentPadding = PaddingValues(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Text(
                            "登录后投稿装备",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.ExtraBold,
                            textAlign = TextAlign.Center,
                        )
                        Text(
                            "装备图鉴投稿需要进入审核，通过后才会展示在公共图鉴。",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.bodySmall,
                            textAlign = TextAlign.Center,
                        )
                        CompactPillAction("去登录", onLogin, Modifier.fillMaxWidth())
                    }
                }
                return@LazyColumn
            }
            item {
                IntroCard(
                    eyebrow = "装备图鉴",
                    title = "投稿到装备图鉴",
                    subtitle = "只填写适合公开展示的信息，审核通过后会出现在装备图鉴。",
                )
            }
            if (state.error != null) item { ErrorState(state.error!!) }
            item { AtlasSubmitForm(state.form, viewModel) }
            item {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    SoftPillButton("取消", onBack, Modifier.weight(1f))
                    PrimaryPillButton(if (state.submitting) "提交中..." else "提交审核", viewModel::submit, Modifier.weight(1f), enabled = !state.submitting)
                }
            }
        }
    }
}

@Composable
private fun AtlasSubmitForm(form: GearFormState, viewModel: GearAtlasSubmitViewModel) {
    SurfaceCard(Modifier.fillMaxWidth()) {
        SectionTitle("基本信息")
        AtlasCategoryRow(form.category) { category -> category?.let(viewModel::updateCategory) }
        OutlinedTextField(form.name, viewModel::updateName, label = { Text("装备名称 *") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(form.brand, viewModel::updateBrand, label = { Text("品牌") }, singleLine = true, modifier = Modifier.weight(1f))
            OutlinedTextField(form.model, viewModel::updateModel, label = { Text("型号") }, singleLine = true, modifier = Modifier.weight(1f))
        }
        OutlinedTextField(form.description, viewModel::updateDescription, label = { Text("装备描述") }, minLines = 3, modifier = Modifier.fillMaxWidth())
    }
    SurfaceCard(Modifier.fillMaxWidth()) {
        SectionTitle("可公开信息")
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(form.weightG, viewModel::updateWeight, label = { Text("重量 g") }, singleLine = true, modifier = Modifier.weight(1f))
            OutlinedTextField(form.officialPrice, viewModel::updateOfficialPrice, label = { Text("官方价格 ¥") }, singleLine = true, modifier = Modifier.weight(1f))
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(form.color, viewModel::updateColor, label = { Text("颜色") }, singleLine = true, modifier = Modifier.weight(1f))
            OutlinedTextField(form.material, viewModel::updateMaterial, label = { Text("材质") }, singleLine = true, modifier = Modifier.weight(1f))
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(form.capacity, viewModel::updateCapacity, label = { Text("容量") }, singleLine = true, modifier = Modifier.weight(1f))
            OutlinedTextField(form.size, viewModel::updateSize, label = { Text("尺码") }, singleLine = true, modifier = Modifier.weight(1f))
        }
    }
}
