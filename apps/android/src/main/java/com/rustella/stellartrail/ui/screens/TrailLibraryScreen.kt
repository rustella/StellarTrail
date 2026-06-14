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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.PointerInputChange
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
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.currentTrailPalette
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.hypot
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
    val palette = currentTrailPalette()
    SurfaceCard(
        Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = { if (selecting) onToggle() else onPreview() }),
        contentPadding = PaddingValues(14.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
            if (selecting) Checkbox(checked = selected, onCheckedChange = { onToggle() }, enabled = enabled)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Row(
                        Modifier.weight(1f),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Badge(trail.sourceFormat.label())
                        Text(
                            trail.displayName,
                            modifier = Modifier.weight(1f),
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.ExtraBold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    TrailLibraryEditMenu(enabled = enabled, onRename = onRename, onDelete = onDelete)
                }
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clip(TrailInnerCardShape)
                        .background(palette.controlBackground)
                        .padding(horizontal = 12.dp, vertical = 10.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        TrailLibraryFact("长度", (trail.distanceM / 1000.0).formatOne() + " km", Modifier.weight(1f))
                        TrailLibraryFact("爬升", trail.ascentM.formatMeters(), Modifier.weight(1f))
                        TrailLibraryFact("下降", trail.descentM.formatMeters(), Modifier.weight(1f))
                    }
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        TrailLibraryFact("起点", trail.startElevationM?.formatMeters() ?: "暂无", Modifier.weight(1f))
                        TrailLibraryFact("终点", trail.endElevationM?.formatMeters() ?: "暂无", Modifier.weight(1f))
                        TrailLibraryFact("更新", trail.updatedAt.datePart(), Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

@Composable
private fun TrailLibraryEditMenu(
    enabled: Boolean,
    onRename: () -> Unit,
    onDelete: () -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        IconButton(
            onClick = { expanded = true },
            enabled = enabled,
            modifier = Modifier.size(40.dp),
        ) {
            Icon(
                imageVector = Icons.Filled.Edit,
                contentDescription = "编辑轨迹",
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(
                text = { Text("重命名") },
                onClick = {
                    expanded = false
                    onRename()
                },
            )
            DropdownMenuItem(
                text = { Text("删除") },
                onClick = {
                    expanded = false
                    onDelete()
                },
            )
        }
    }
}

@Composable
private fun TrailLibraryFact(label: String, value: String, modifier: Modifier = Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Text(
            label,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
        )
        Text(
            value,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

private enum class Trail3dPreviewMode {
    Map,
    Terrain3d,
    Track3d,
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
                        Trail3dPreviewMode.Track3d -> {
                            Trail3dTrackPanel(trail = trail, modifier = Modifier.fillMaxSize())
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
private fun Trail3dTrackPanel(trail: Trail, modifier: Modifier = Modifier) {
    val model = remember(trail.normalizedPoints) { buildTrail3dTrackModel(trail.normalizedPoints) }
    var camera by remember(model) { mutableStateOf(resetTrail3dCamera()) }
    var canvasWidthPx by remember(model) { mutableStateOf(1) }
    var canvasHeightPx by remember(model) { mutableStateOf(1) }
    var selectedTrackIndex by remember(model) { mutableStateOf(model.points.lastIndex.coerceAtLeast(0)) }
    if (!model.hasEnoughData) {
        Box(modifier, contentAlignment = Alignment.Center) {
            EmptyState("暂无轨迹3D", "这条轨迹没有足够的带海拔轨迹点。")
        }
        return
    }
    val projection = remember(model, camera, canvasWidthPx, canvasHeightPx) {
        projectTrail3dScene(
            model = model,
            camera = camera,
            viewportWidthPx = canvasWidthPx.toFloat(),
            viewportHeightPx = canvasHeightPx.toFloat(),
        )
    }
    val safeSelectedIndex = selectedTrackIndex.coerceIn(0, model.points.lastIndex)
    val selectedPoint = model.points[safeSelectedIndex]
    val lineColor = MaterialTheme.colorScheme.primary
    val groundFillColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.68f)
    val groundStrokeColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.84f)
    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.58f)
    val shadowColor = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.18f)
    val startColor = MaterialTheme.colorScheme.secondary
    val endColor = MaterialTheme.colorScheme.tertiary
    val selectedColor = MaterialTheme.colorScheme.tertiary
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
                TrailTrendMetric("最低", model.minElevationM?.formatMeters().orEmpty(), Modifier.weight(1f))
                TrailTrendMetric("最高", model.maxElevationM?.formatMeters().orEmpty(), Modifier.weight(1f))
                TrailTrendMetric("长度", (model.distanceM / 1000.0).formatOne() + " km", Modifier.weight(1f))
            }
            Canvas(
                Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .onSizeChanged { size ->
                        canvasWidthPx = size.width.coerceAtLeast(1)
                        canvasHeightPx = size.height.coerceAtLeast(1)
                    }
                    .pointerInput(model) {
                        awaitPointerEventScope {
                            while (true) {
                                val event = awaitPointerEvent()
                                val pressedChanges = event.changes.filter { it.pressed }
                                when (pressedChanges.size) {
                                    0 -> Unit
                                    1 -> {
                                        val change = pressedChanges.single()
                                        val delta = change.position - change.previousPosition
                                        if (change.previousPressed && !delta.isZero()) {
                                            camera = panTrail3dCamera(
                                                camera = camera,
                                                panDeltaXPx = delta.x.toDouble(),
                                                panDeltaYPx = delta.y.toDouble(),
                                            )
                                            change.consume()
                                        }
                                    }
                                    else -> {
                                        val stableChanges = pressedChanges.filter { it.previousPressed }
                                        if (stableChanges.size >= 2) {
                                            val pan = stableChanges.currentCentroid() - stableChanges.previousCentroid()
                                            camera = updateTrail3dCamera(
                                                camera = camera,
                                                yawDeltaDegrees = pan.x * 0.28 + stableChanges.rotationDegrees() * 0.8,
                                                pitchDeltaDegrees = -pan.y * 0.22,
                                                zoomMultiplier = stableChanges.zoomMultiplier().toDouble(),
                                            )
                                            stableChanges.forEach { it.consume() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    .pointerInput(model, projection) {
                        detectTapGestures(
                            onDoubleTap = {
                                camera = zoomTrail3dCamera(camera, TRAIL_3D_DOUBLE_TAP_ZOOM_MULTIPLIER)
                            },
                            onTap = { offset ->
                                projection?.nearestTrackPointIndex(offset)?.let { selectedTrackIndex = it }
                            },
                        )
                    },
            ) {
                projection?.let {
                    drawTrail3dTrackProjection(
                        projection = it,
                        selectedIndex = safeSelectedIndex,
                        lineColor = lineColor,
                        groundFillColor = groundFillColor,
                        groundStrokeColor = groundStrokeColor,
                        gridColor = gridColor,
                        shadowColor = shadowColor,
                        startColor = startColor,
                        endColor = endColor,
                        selectedColor = selectedColor,
                    )
                }
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

private fun DrawScope.drawTrail3dTrackProjection(
    projection: Trail3dProjection,
    selectedIndex: Int,
    lineColor: Color,
    groundFillColor: Color,
    groundStrokeColor: Color,
    shadowColor: Color,
    gridColor: Color,
    startColor: Color,
    endColor: Color,
    selectedColor: Color,
) {
    if (projection.trackPoints.size < 2) return
    val groundPath = projection.groundOutline.toPath()
    drawPath(groundPath, groundFillColor)
    drawPath(groundPath, groundStrokeColor, style = Stroke(width = 1.5f))
    projection.gridLines.sortedBy { it.depth }.forEach { line ->
        drawLine(
            color = gridColor,
            start = Offset(line.start.x, line.start.y),
            end = Offset(line.end.x, line.end.y),
            strokeWidth = 1.2f,
        )
    }
    projection.shadowPoints.zipWithNext().forEach { (start, end) ->
        drawLine(
            color = shadowColor,
            start = Offset(start.x, start.y),
            end = Offset(end.x, end.y),
            strokeWidth = 4.5f,
            cap = StrokeCap.Round,
        )
    }
    val depthMin = projection.trackPoints.minOf { it.depth }
    val depthMax = projection.trackPoints.maxOf { it.depth }
    val depthRange = (depthMax - depthMin).takeIf { abs(it) > 0.0001 } ?: 1.0
    projection.trackPoints.zipWithNext().forEach { (start, end) ->
        val depthRatio = (((start.depth + end.depth) / 2.0 - depthMin) / depthRange).coerceIn(0.0, 1.0).toFloat()
        val segmentColor = lineColor.copy(alpha = 0.72f + depthRatio * 0.28f)
        drawLine(
            color = lineColor.copy(alpha = 0.22f),
            start = Offset(start.x, start.y),
            end = Offset(end.x, end.y),
            strokeWidth = 10f,
            cap = StrokeCap.Round,
        )
        drawLine(
            color = segmentColor,
            start = Offset(start.x, start.y),
            end = Offset(end.x, end.y),
            strokeWidth = 5.2f,
            cap = StrokeCap.Round,
        )
    }
    val start = projection.trackPoints.first()
    val end = projection.trackPoints.last()
    drawTrackPointMarker(start, startColor, radius = 5.5f)
    drawTrackPointMarker(end, endColor, radius = 6.5f)
    val selectedIndexSafe = selectedIndex.coerceIn(projection.trackPoints.indices)
    val selected = projection.trackPoints[selectedIndexSafe]
    val selectedShadow = projection.shadowPoints[selectedIndexSafe]
    drawLine(
        color = selectedColor.copy(alpha = 0.48f),
        start = Offset(selected.x, selected.y),
        end = Offset(selectedShadow.x, selectedShadow.y),
        strokeWidth = 2f,
    )
    drawCircle(selectedColor.copy(alpha = 0.20f), radius = 13f, center = Offset(selected.x, selected.y))
    drawTrackPointMarker(selected, selectedColor, radius = 6f)
}

private fun DrawScope.drawTrackPointMarker(point: Trail3dProjectedPoint, color: Color, radius: Float) {
    drawCircle(Color.White.copy(alpha = 0.86f), radius = radius + 2.5f, center = Offset(point.x, point.y))
    drawCircle(color, radius = radius, center = Offset(point.x, point.y))
}

private fun List<Trail3dProjectedPoint>.toPath(): Path = Path().apply {
    forEachIndexed { index, point ->
        if (index == 0) moveTo(point.x, point.y) else lineTo(point.x, point.y)
    }
    close()
}

private fun Trail3dProjection.nearestTrackPointIndex(offset: Offset): Int? =
    trackPoints.indices.minByOrNull { index ->
        val point = trackPoints[index]
        hypot((point.x - offset.x).toDouble(), (point.y - offset.y).toDouble())
    }

private fun Offset.isZero(): Boolean = x == 0f && y == 0f

private fun List<PointerInputChange>.currentCentroid(): Offset = centroid { it.position }

private fun List<PointerInputChange>.previousCentroid(): Offset = centroid { it.previousPosition }

private fun List<PointerInputChange>.centroid(position: (PointerInputChange) -> Offset): Offset {
    if (isEmpty()) return Offset.Zero
    val x = sumOf { position(it).x.toDouble() } / size
    val y = sumOf { position(it).y.toDouble() } / size
    return Offset(x.toFloat(), y.toFloat())
}

private fun List<PointerInputChange>.zoomMultiplier(): Float {
    val currentCentroid = currentCentroid()
    val previousCentroid = previousCentroid()
    val currentDistance = averageDistanceFrom(currentCentroid) { it.position }
    val previousDistance = averageDistanceFrom(previousCentroid) { it.previousPosition }
    if (previousDistance <= 0.0001f || !currentDistance.isFinite()) return 1f
    return (currentDistance / previousDistance).coerceIn(0.75f, 1.35f)
}

private fun List<PointerInputChange>.averageDistanceFrom(
    centroid: Offset,
    position: (PointerInputChange) -> Offset,
): Float {
    if (isEmpty()) return 0f
    val distance = sumOf { change ->
        val point = position(change)
        hypot((point.x - centroid.x).toDouble(), (point.y - centroid.y).toDouble())
    } / size
    return distance.toFloat()
}

private fun List<PointerInputChange>.rotationDegrees(): Double {
    if (size < 2) return 0.0
    val first = this[0]
    val second = this[1]
    val current = second.position - first.position
    val previous = second.previousPosition - first.previousPosition
    if (current.isZero() || previous.isZero()) return 0.0
    var degrees = (atan2(current.y, current.x) - atan2(previous.y, previous.x)) * 180.0 / PI
    if (degrees > 180.0) degrees -= 360.0
    if (degrees < -180.0) degrees += 360.0
    return degrees
}

private fun Trail3dPreviewMode.label(): String = when (this) {
    Trail3dPreviewMode.Map -> "地图"
    Trail3dPreviewMode.Terrain3d -> "地形3D"
    Trail3dPreviewMode.Track3d -> "轨迹3D"
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
