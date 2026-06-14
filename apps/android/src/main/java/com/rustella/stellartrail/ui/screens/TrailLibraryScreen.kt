package com.rustella.stellartrail.ui.screens

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
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
                if (!selecting) {
                    TrailLibraryPreviewHint(enabled = enabled, onPreview = onPreview)
                }
            }
        }
    }
}

@Composable
private fun TrailLibraryPreviewHint(enabled: Boolean, onPreview: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(TrailInnerCardShape)
            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.09f))
            .clickable(enabled = enabled, onClick = onPreview)
            .padding(horizontal = 12.dp, vertical = 9.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            "点按卡片预览地图与3D地形",
            modifier = Modifier.weight(1f),
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            ">",
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
        )
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

internal enum class TrailMapPreviewMode {
    FlatMap,
    Map3d,
}

internal data class TrailMapPreviewState(
    val mode: TrailMapPreviewMode = TrailMapPreviewMode.FlatMap,
)

internal fun defaultTrailMapPreviewState(): TrailMapPreviewState = TrailMapPreviewState()

internal fun enterTrailMapPreview3d(state: TrailMapPreviewState): TrailMapPreviewState =
    state.copy(mode = TrailMapPreviewMode.Map3d)

internal fun exitTrailMapPreview3d(state: TrailMapPreviewState): TrailMapPreviewState =
    state.copy(mode = TrailMapPreviewMode.FlatMap)

internal fun trailPreviewTerrainEnabled(state: TrailMapPreviewState): Boolean =
    state.mode == TrailMapPreviewMode.Map3d

@Composable
private fun TrailPreviewDialog(
    trail: Trail,
    selectedStyleId: String?,
    onSelectMapStyle: (String) -> Unit,
    mapConfig: com.rustella.stellartrail.domain.trip.MapConfigResponse?,
    onDismiss: () -> Unit,
) {
    var previewState by remember(trail.id) { mutableStateOf(defaultTrailMapPreviewState()) }
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
                            trailPreviewHeaderSummary(trail.distanceM),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    TextButton(onClick = onDismiss) { Text("关闭") }
                }
                BoxWithConstraints(
                    Modifier
                        .fillMaxWidth()
                        .weight(1f),
                ) {
                    if (mapConfig != null) {
                        TrailAssetPreviewMap(
                            map = mapConfig,
                            trail = trail,
                            selectedStyleId = selectedStyleId,
                            onSelectMapStyle = onSelectMapStyle,
                            modifier = Modifier.fillMaxWidth(),
                            height = maxHeight,
                            zoomGesturesEnabled = true,
                            terrain3dEnabled = trailPreviewTerrainEnabled(previewState),
                            showStyleSelector = previewState.mode == TrailMapPreviewMode.FlatMap,
                            bottomStartControls = {
                                TrailPreviewDimensionButton(
                                    state = previewState,
                                    onEnter3d = { previewState = enterTrailMapPreview3d(previewState) },
                                    onExit3d = { previewState = exitTrailMapPreview3d(previewState) },
                                )
                            },
                        )
                    } else {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            LoadingState()
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TrailPreviewDimensionButton(
    state: TrailMapPreviewState,
    onEnter3d: () -> Unit,
    onExit3d: () -> Unit,
) {
    val isMap3d = state.mode == TrailMapPreviewMode.Map3d
    Surface(
        shape = RoundedCornerShape(8.dp),
        color = if (isMap3d) {
            MaterialTheme.colorScheme.primary.copy(alpha = 0.94f)
        } else {
            MaterialTheme.colorScheme.surface.copy(alpha = 0.94f)
        },
        tonalElevation = 2.dp,
        shadowElevation = 2.dp,
    ) {
        Box(
            Modifier
                .size(28.dp)
                .clip(RoundedCornerShape(6.dp))
                .clickable(onClick = if (isMap3d) onExit3d else onEnter3d),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = if (isMap3d) "2D" else "3D",
                color = if (isMap3d) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 1,
            )
        }
    }
}

internal fun trailPreviewHeaderSummary(distanceM: Double): String =
    "${(distanceM / 1000.0).formatOne()} km"

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
