package com.rustella.stellartrail.ui.screens

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.map.MapStylePreferenceRepository
import com.rustella.stellartrail.core.trail.readTrailUpload
import com.rustella.stellartrail.core.trail.trailDocumentMimeTypes
import com.rustella.stellartrail.domain.trip.Trail
import com.rustella.stellartrail.domain.trip.TrailSourceFormat
import com.rustella.stellartrail.domain.trip.TrailSummary
import com.rustella.stellartrail.feature.trails.TrailLibrarySort
import com.rustella.stellartrail.feature.trails.TrailLibraryViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import kotlin.math.abs
import kotlin.math.roundToInt

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TrailLibraryScreen(
    viewModel: TrailLibraryViewModel,
    mapStylePreferenceRepository: MapStylePreferenceRepository,
    isLoggedIn: Boolean,
    selectingForTrip: Boolean,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    onLinkedToTrip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!isLoggedIn) {
        LoginRequiredScreen(
            title = "登录后查看轨迹库",
            body = "轨迹文件会保存到你的个人轨迹库，请先登录。",
            onLogin = onLogin,
            modifier = modifier,
        )
        return
    }
    val context = LocalContext.current
    val state by viewModel.state.collectAsStateWithLifecycle()
    val selectedStyleId by mapStylePreferenceRepository.selectedStyleId.collectAsStateWithLifecycle()
    val visibleTrails = remember(state.trails, state.formatFilter, state.sort) {
        state.trails.filteredAndSorted(state.formatFilter, state.sort)
    }
    var renaming by remember { mutableStateOf<TrailSummary?>(null) }
    var deleting by remember { mutableStateOf<TrailSummary?>(null) }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let {
            readTrailUpload(context, it)?.let { upload ->
                viewModel.uploadTrailFile(upload.filename, upload.contentType, upload.bytes)
            }
        }
    }

    LaunchedEffect(Unit) { viewModel.load() }

    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            SurfaceCard {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("我的轨迹库", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                        Text(
                            if (selectingForTrip) {
                                "选择一条或多条轨迹，添加到当前行程。"
                            } else {
                                "保存和复用你的户外轨迹，可添加到行程或户外经历。"
                            },
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    SoftPillButton("返回", onBack)
                }
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    PrimaryPillButton(
                        if (state.mutating) "处理中" else "上传轨迹",
                        { launcher.launch(trailDocumentMimeTypes) },
                        Modifier.weight(1f),
                        enabled = !state.mutating,
                    )
                    if (selectingForTrip) {
                        SoftPillButton(
                            "添加选中",
                            { viewModel.linkSelectedToTrip(onLinkedToTrip) },
                            Modifier.weight(1f),
                            enabled = !state.mutating,
                        )
                    }
                }
            }
        }
        state.notice?.let { item { SurfaceCard { Text(it, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold) } } }
        state.error?.let { item { ErrorState(it, onRetry = viewModel::load) } }
        if (state.loading) item { LoadingState() }
        if (!state.loading && state.trails.isNotEmpty()) {
            item {
                TrailLibraryFilterBar(
                    totalCount = state.trails.size,
                    visibleCount = visibleTrails.size,
                    selectedFormat = state.formatFilter,
                    sort = state.sort,
                    onFormatChange = viewModel::setFormatFilter,
                    onSortChange = viewModel::setSort,
                )
            }
        }
        if (!state.loading && state.trails.isEmpty()) {
            item { EmptyState("还没有轨迹", "上传 GPX、KML/KMZ 或 FIT 文件后，可以复用到行程和户外经历。") }
        }
        if (!state.loading && state.trails.isNotEmpty() && visibleTrails.isEmpty()) {
            item {
                SurfaceCard(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("当前筛选下没有轨迹", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                    Text("换个文件格式筛选，或清除筛选后查看全部轨迹。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    CompactPillAction("清除筛选", viewModel::clearFilters, filled = false)
                }
            }
        }
        items(visibleTrails, key = { it.id }) { trail ->
            TrailLibraryRow(
                trail = trail,
                selected = trail.id in state.selectedTrailIds,
                selecting = selectingForTrip,
                enabled = !state.mutating,
                onToggle = { viewModel.toggleTrailSelection(trail.id) },
                onPreview = { viewModel.previewTrail(trail.id) },
                onRename = { renaming = trail },
                onDelete = { deleting = trail },
            )
        }
    }

    state.preview?.let { preview ->
        TrailPreviewDialog(
            trail = preview,
            selectedStyleId = selectedStyleId,
            onSelectMapStyle = mapStylePreferenceRepository::selectStyle,
            mapConfig = state.mapConfig,
            onDismiss = viewModel::dismissPreview,
        )
    }
    renaming?.let { trail ->
        RenameTrailDialog(
            trail = trail,
            onDismiss = { renaming = null },
            onSave = { name ->
                renaming = null
                viewModel.renameTrail(trail.id, name)
            },
        )
    }
    deleting?.let { trail ->
        DeleteTrailDialog(
            trail = trail,
            onDismiss = { deleting = null },
            onConfirm = {
                deleting = null
                viewModel.deleteTrail(trail.id)
            },
        )
    }
}

@Composable
private fun TrailLibraryFilterBar(
    totalCount: Int,
    visibleCount: Int,
    selectedFormat: TrailSourceFormat?,
    sort: TrailLibrarySort,
    onFormatChange: (TrailSourceFormat?) -> Unit,
    onSortChange: (TrailLibrarySort) -> Unit,
) {
    SurfaceCard(contentPadding = PaddingValues(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text("共 $totalCount 条 · 当前 $visibleCount 条", color = MaterialTheme.colorScheme.onSurfaceVariant)
            TrailSortMenu(sort = sort, onSortChange = onSortChange)
        }
        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            TrailFormatFilterChip("全部", selectedFormat == null) { onFormatChange(null) }
            TrailSourceFormat.entries.forEach { format ->
                TrailFormatFilterChip(format.label(), selectedFormat == format) { onFormatChange(format) }
            }
        }
    }
}

@Composable
private fun TrailFormatFilterChip(label: String, selected: Boolean, onClick: () -> Unit) {
    FilterChip(selected = selected, onClick = onClick, label = { Text(label) })
}

@Composable
private fun TrailSortMenu(sort: TrailLibrarySort, onSortChange: (TrailLibrarySort) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        CompactPillAction("排序 · ${sort.label()}", { expanded = true }, filled = false)
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            TrailLibrarySort.entries.forEach { option ->
                DropdownMenuItem(
                    text = { Text(option.label()) },
                    onClick = {
                        onSortChange(option)
                        expanded = false
                    },
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TrailLibraryRow(
    trail: TrailSummary,
    selected: Boolean,
    selecting: Boolean,
    enabled: Boolean,
    onToggle: () -> Unit,
    onPreview: () -> Unit,
    onRename: () -> Unit,
    onDelete: () -> Unit,
) {
    SurfaceCard(
        Modifier
            .fillMaxWidth()
            .clickable(enabled = selecting && enabled, onClick = onToggle),
        contentPadding = PaddingValues(12.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
            if (selecting) Checkbox(checked = selected, onCheckedChange = { onToggle() }, enabled = enabled)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    Badge(trail.sourceFormat.label())
                    Text(trail.displayName, fontWeight = FontWeight.ExtraBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                Text(
                    trail.metricLine(),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(trail.elevationLine(), color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text(
                    "更新 ${trail.updatedAt.datePart()}",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    CompactPillAction("预览", onPreview, enabled = enabled)
                    CompactPillAction("重命名", onRename, enabled = enabled)
                    CompactPillAction("删除", onDelete, enabled = enabled)
                }
            }
        }
    }
}

private enum class Trail3dPreviewMode {
    Map,
    Terrain3d,
    ElevationTrend3d,
}

@Composable
private fun TrailPreviewDialog(
    trail: Trail,
    selectedStyleId: String?,
    onSelectMapStyle: (String) -> Unit,
    mapConfig: com.rustella.stellartrail.domain.trip.MapConfigResponse?,
    onDismiss: () -> Unit,
) {
    var previewMode by remember(trail.id) { mutableStateOf(Trail3dPreviewMode.Map) }
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Surface(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 12.dp, vertical = 24.dp),
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 6.dp,
        ) {
            Column(
                Modifier
                    .fillMaxSize()
                    .padding(14.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.Top,
                ) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(
                            trail.displayName,
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.ExtraBold,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            "${(trail.distanceM / 1000.0).formatOne()} km · ${trail.pointCount} 点",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    TextButton(onClick = onDismiss) { Text("关闭") }
                }
                TrailPreviewModeSelector(selectedMode = previewMode, onSelectMode = { previewMode = it })
                BoxWithConstraints(
                    Modifier
                        .fillMaxWidth()
                        .weight(1f),
                ) {
                    when (previewMode) {
                        Trail3dPreviewMode.Map,
                        Trail3dPreviewMode.Terrain3d -> {
                            if (mapConfig != null) {
                                TrailAssetPreviewMap(
                                    map = mapConfig,
                                    trail = trail,
                                    selectedStyleId = selectedStyleId,
                                    onSelectMapStyle = onSelectMapStyle,
                                    modifier = Modifier.fillMaxWidth(),
                                    height = maxHeight,
                                    zoomGesturesEnabled = true,
                                    terrain3dEnabled = previewMode == Trail3dPreviewMode.Terrain3d,
                                )
                            } else {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    LoadingState()
                                }
                            }
                        }
                        Trail3dPreviewMode.ElevationTrend3d -> {
                            TrailElevationTrend3dPanel(trail = trail, modifier = Modifier.fillMaxSize())
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TrailPreviewModeSelector(
    selectedMode: Trail3dPreviewMode,
    onSelectMode: (Trail3dPreviewMode) -> Unit,
) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(999.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(2.dp),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Trail3dPreviewMode.entries.forEach { mode ->
            val selected = mode == selectedMode
            Text(
                text = mode.label(),
                modifier = Modifier
                    .weight(1f)
                    .clip(RoundedCornerShape(999.dp))
                    .background(if (selected) MaterialTheme.colorScheme.primary else Color.Transparent)
                    .clickable(enabled = !selected) { onSelectMode(mode) }
                    .padding(vertical = 8.dp),
                color = if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = if (selected) FontWeight.ExtraBold else FontWeight.Bold,
                textAlign = TextAlign.Center,
                maxLines = 1,
            )
        }
    }
}

@Composable
private fun TrailElevationTrend3dPanel(trail: Trail, modifier: Modifier = Modifier) {
    val profile = remember(trail.normalizedPoints) { buildTrailElevationProfile(trail.normalizedPoints) }
    var chartWidthPx by remember(profile) { mutableStateOf(1) }
    var selectedIndex by remember(profile) { mutableStateOf(profile.points.lastIndex.coerceAtLeast(0)) }
    if (!profile.hasEnoughData) {
        Box(modifier, contentAlignment = Alignment.Center) {
            EmptyState("暂无海拔走势", "这条轨迹没有足够的海拔点。")
        }
        return
    }
    val safeSelectedIndex = selectedIndex.coerceIn(0, profile.points.lastIndex)
    val selectedPoint = profile.points[safeSelectedIndex]
    val lineColor = MaterialTheme.colorScheme.primary
    val shadowColor = MaterialTheme.colorScheme.secondary.copy(alpha = 0.48f)
    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.72f)
    val baseColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.68f)
    val markerColor = MaterialTheme.colorScheme.tertiary
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.42f),
    ) {
        Column(
            Modifier
                .fillMaxSize()
                .padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TrailTrendMetric("最低", profile.minElevationM?.formatMeters().orEmpty(), Modifier.weight(1f))
                TrailTrendMetric("最高", profile.maxElevationM?.formatMeters().orEmpty(), Modifier.weight(1f))
                TrailTrendMetric("长度", (profile.distanceM / 1000.0).formatOne() + " km", Modifier.weight(1f))
            }
            Canvas(
                Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .onSizeChanged { chartWidthPx = it.width.coerceAtLeast(1) }
                    .pointerInput(profile) {
                        detectTapGestures { offset ->
                            selectedIndex = profile.nearestPointIndex(offset.x / chartWidthPx.toFloat())
                        }
                    },
            ) {
                drawTrailElevationProfile(
                    profile = profile,
                    selectedIndex = safeSelectedIndex,
                    lineColor = lineColor,
                    shadowColor = shadowColor,
                    gridColor = gridColor,
                    baseColor = baseColor,
                    fillColor = lineColor.copy(alpha = 0.16f),
                    markerColor = markerColor,
                )
            }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "第 ${selectedPoint.pointIndex + 1} 点",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelMedium,
                )
                Text(
                    "${(selectedPoint.distanceM / 1000.0).formatOne()} km · ${selectedPoint.elevationM.formatMeters()}",
                    fontWeight = FontWeight.ExtraBold,
                )
            }
        }
    }
}

@Composable
private fun TrailTrendMetric(label: String, value: String, modifier: Modifier = Modifier) {
    Column(
        modifier
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.72f))
            .padding(horizontal = 10.dp, vertical = 8.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.labelSmall)
        Text(value.ifBlank { "暂无" }, fontWeight = FontWeight.ExtraBold, maxLines = 1)
    }
}

private fun DrawScope.drawTrailElevationProfile(
    profile: TrailElevationProfile,
    selectedIndex: Int,
    lineColor: Color,
    shadowColor: Color,
    gridColor: Color,
    baseColor: Color,
    fillColor: Color,
    markerColor: Color,
) {
    if (profile.points.size < 2 || size.width <= 0f || size.height <= 0f) return
    val left = 44f
    val right = 18f
    val top = 18f
    val bottom = 34f
    val depth = 22f
    val chartWidth = size.width - left - right - depth
    val chartHeight = size.height - top - bottom - depth
    if (chartWidth <= 0f || chartHeight <= 0f) return
    val baseY = top + chartHeight
    val iso = Offset(depth, depth)
    val minElevation = profile.minElevationM ?: return
    val maxElevation = profile.maxElevationM ?: return
    val elevationRange = (maxElevation - minElevation).takeIf { abs(it) > 0.0001 } ?: 1.0
    val distanceRange = profile.points.maxOf { it.distanceM }.takeIf { it > 0.0 } ?: 1.0
    fun pointOffset(point: TrailElevationProfilePoint, index: Int): Offset {
        val distanceRatio = if (distanceRange > 1.0) {
            (point.distanceM / distanceRange).toFloat()
        } else {
            index.toFloat() / profile.points.lastIndex.coerceAtLeast(1)
        }
        val elevationRatio = ((point.elevationM - minElevation) / elevationRange).toFloat()
        return Offset(
            x = left + chartWidth * distanceRatio.coerceIn(0f, 1f),
            y = top + chartHeight * (1f - elevationRatio.coerceIn(0f, 1f)),
        )
    }
    val offsets = profile.points.mapIndexed { index, point -> pointOffset(point, index) }
    val basePath = Path().apply {
        moveTo(left + iso.x, top + iso.y)
        lineTo(left + chartWidth + iso.x, top + iso.y)
        lineTo(left + chartWidth + iso.x, baseY + iso.y)
        lineTo(left + iso.x, baseY + iso.y)
        close()
    }
    drawPath(basePath, baseColor)
    repeat(4) { step ->
        val y = top + chartHeight * (step + 1) / 5f
        drawLine(gridColor, Offset(left, y), Offset(left + chartWidth, y), strokeWidth = 1f)
        drawLine(
            gridColor.copy(alpha = 0.36f),
            Offset(left + iso.x, y + iso.y),
            Offset(left + chartWidth + iso.x, y + iso.y),
            strokeWidth = 1f,
        )
    }
    val areaPath = Path().apply {
        moveTo(offsets.first().x, baseY)
        offsets.forEach { lineTo(it.x, it.y) }
        lineTo(offsets.last().x, baseY)
        close()
    }
    drawPath(path = areaPath, brush = Brush.verticalGradient(listOf(fillColor, Color.Transparent)))
    val shadowPath = Path().apply {
        offsets.forEachIndexed { index, offset ->
            val shifted = offset + iso
            if (index == 0) moveTo(shifted.x, shifted.y) else lineTo(shifted.x, shifted.y)
        }
    }
    drawPath(shadowPath, shadowColor, style = Stroke(width = 5f, cap = StrokeCap.Round))
    val linePath = Path().apply {
        offsets.forEachIndexed { index, offset ->
            if (index == 0) moveTo(offset.x, offset.y) else lineTo(offset.x, offset.y)
        }
    }
    drawPath(linePath, lineColor, style = Stroke(width = 4f, cap = StrokeCap.Round))
    val selected = offsets[selectedIndex.coerceIn(offsets.indices)]
    drawLine(markerColor.copy(alpha = 0.54f), Offset(selected.x, top), Offset(selected.x, baseY + iso.y), strokeWidth = 2f)
    drawCircle(markerColor.copy(alpha = 0.24f), radius = 11f, center = selected)
    drawCircle(markerColor, radius = 5f, center = selected)
    drawRect(
        color = gridColor.copy(alpha = 0.5f),
        topLeft = Offset(left, top),
        size = Size(chartWidth, chartHeight),
        style = Stroke(width = 1f),
    )
}

private fun TrailElevationProfile.nearestPointIndex(fraction: Float): Int {
    val targetDistance = points.maxOf { it.distanceM } * fraction.coerceIn(0f, 1f)
    return points.indices.minByOrNull { index -> abs(points[index].distanceM - targetDistance) } ?: 0
}

private fun Trail3dPreviewMode.label(): String = when (this) {
    Trail3dPreviewMode.Map -> "地图"
    Trail3dPreviewMode.Terrain3d -> "地形3D"
    Trail3dPreviewMode.ElevationTrend3d -> "走势3D"
}

@Composable
private fun RenameTrailDialog(trail: TrailSummary, onDismiss: () -> Unit, onSave: (String) -> Unit) {
    var name by remember(trail.id) { mutableStateOf(trail.displayName) }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("重命名轨迹") },
        text = {
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("轨迹名称") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
        },
        confirmButton = { TextButton(onClick = { onSave(name) }) { Text("保存") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } },
    )
}

@Composable
private fun DeleteTrailDialog(trail: TrailSummary, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("删除轨迹？") },
        text = { Text("删除后，引用这条轨迹的行程和户外经历都将不再显示它。") },
        confirmButton = { TextButton(onClick = onConfirm) { Text("删除") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } },
    )
}

private fun TrailSourceFormat.label(): String = when (this) {
    TrailSourceFormat.GPX -> "GPX"
    TrailSourceFormat.KML -> "KML"
    TrailSourceFormat.FIT -> "FIT"
}

private fun TrailLibrarySort.label(): String = when (this) {
    TrailLibrarySort.UpdatedDesc -> "最近更新"
    TrailLibrarySort.UpdatedAsc -> "最早更新"
}

private fun List<TrailSummary>.filteredAndSorted(
    formatFilter: TrailSourceFormat?,
    sort: TrailLibrarySort,
): List<TrailSummary> {
    val filtered = formatFilter?.let { format -> filter { it.sourceFormat == format } } ?: this
    return when (sort) {
        TrailLibrarySort.UpdatedDesc -> filtered.sortedWith(compareByDescending<TrailSummary> { it.updatedAt }.thenByDescending { it.id })
        TrailLibrarySort.UpdatedAsc -> filtered.sortedWith(compareBy<TrailSummary> { it.updatedAt }.thenBy { it.id })
    }
}

private fun TrailSummary.metricLine(): String =
    "总长度 ${(distanceM / 1000.0).formatOne()} km · 爬升 ${ascentM.formatMeters()} · 下降 ${descentM.formatMeters()}"

private fun TrailSummary.elevationLine(): String {
    val start = startElevationM?.formatMeters()
    val end = endElevationM?.formatMeters()
    if (start != null || end != null) {
        return "起点海拔 ${start ?: "暂无"} · 终点海拔 ${end ?: "暂无"}"
    }
    val min = minElevationM?.formatMeters()
    val max = maxElevationM?.formatMeters()
    return when {
        min != null && max != null -> "最低海拔 $min · 最高海拔 $max"
        min != null -> "最低海拔 $min"
        max != null -> "最高海拔 $max"
        else -> "海拔暂无"
    }
}

private fun String.datePart(): String = take(10)

private fun Double.formatOne(): String = "%.1f".format(this)

private fun Double.formatMeters(): String = "${roundToInt()}m"
