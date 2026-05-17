package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.formatPrice
import com.rustella.stellartrail.domain.gear.formatWeight
import com.rustella.stellartrail.domain.gear.joinBrandModel
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.gear.detail.GearDetailViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TagList

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GearDetailScreen(
    viewModel: GearDetailViewModel,
    onBack: () -> Unit,
    onEdit: () -> Unit,
    onClosed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = { Text(state.item?.name ?: "装备详情") },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回") } },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
            )
        },
    ) { innerPadding ->
        Column(
            Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            if (state.loading) LoadingState()
            if (state.error != null) ErrorState(state.error!!, onRetry = viewModel::load)
            state.item?.let { item -> GearDetailContent(item, onEdit, { viewModel.archive(onClosed) }, viewModel::restore) }
        }
    }
}

@Composable
private fun GearDetailContent(item: GearItem, onEdit: () -> Unit, onArchive: () -> Unit, onRestore: () -> Unit) {
    SurfaceCard(contentPadding = PaddingValues(22.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Badge(item.category.label)
            Badge(item.status.label, tone = statusTone(item.status))
        }
        Text(item.name, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
        Text(joinBrandModel(item.brand, item.model), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            MetricTile("重量", formatWeight(item.weightG), Modifier.weight(1f))
            MetricTile("价格", formatPrice(item.purchasePriceCents), Modifier.weight(1f))
            MetricTile("共享", item.shareStatus.label, Modifier.weight(1f))
        }
    }
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        PrimaryPillButton("编辑装备", onEdit, Modifier.weight(1f))
        if (item.status == GearStatus.RETIRED || item.archivedAt != null) {
            SoftPillButton("恢复可用", onRestore, Modifier.weight(1f))
        } else {
            SoftPillButton("移入历史", onArchive, Modifier.weight(1f))
        }
    }
    if (item.tags.isNotEmpty()) {
        SurfaceCard {
            SectionTitle("标签")
            TagList(item.tags)
        }
    }
    SurfaceCard {
        SectionTitle("基础信息")
        MetadataRow("分类", item.category.label)
        MetadataRow("状态", item.status.label)
        MetadataRow("购买日期", item.purchaseDate ?: "未记录")
        MetadataRow("存放位置", item.storageLocation ?: "未记录")
        MetadataRow("共享状态", item.shareStatus.label)
    }
    SurfaceCard {
        SectionTitle("详细信息")
        MetadataRow("颜色", item.color ?: "未记录")
        MetadataRow("材质", item.material ?: "未记录")
        MetadataRow("容量", item.capacity ?: "未记录")
        MetadataRow("尺码", item.size ?: "未记录")
        MetadataRow("保暖指数", item.warmthIndex ?: "未记录")
        MetadataRow("防水指数", item.waterproofIndex ?: "未记录")
        MetadataRow("保修/过期", item.expiryOrWarrantyDate ?: "未记录")
        MetadataRow("购买地点", item.purchaseLocation ?: "未记录")
        if (!item.description.isNullOrBlank()) Text(item.description)
        if (!item.notes.isNullOrBlank()) Text("备注：${item.notes}")
    }
}

private fun statusTone(status: GearStatus): BadgeTone = when (status) {
    GearStatus.AVAILABLE, GearStatus.IN_USE -> BadgeTone.Success
    GearStatus.MAINTENANCE, GearStatus.IDLE -> BadgeTone.Warning
    GearStatus.DAMAGED, GearStatus.LOST -> BadgeTone.Danger
    GearStatus.RETIRED, GearStatus.SOLD -> BadgeTone.Neutral
}
