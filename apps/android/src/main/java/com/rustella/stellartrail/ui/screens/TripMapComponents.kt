package com.rustella.stellartrail.ui.screens

import android.content.Context
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.maptiler.maptilersdk.MTConfig
import com.maptiler.maptilersdk.events.MTEvent
import com.maptiler.maptilersdk.map.LngLat
import com.maptiler.maptilersdk.map.MTMapOptions
import com.maptiler.maptilersdk.map.MTMapView
import com.maptiler.maptilersdk.map.MTMapViewController
import com.maptiler.maptilersdk.map.MTMapViewDelegate
import com.maptiler.maptilersdk.map.options.MTEventLevel
import com.maptiler.maptilersdk.map.options.MTFitBoundsOptions
import com.maptiler.maptilersdk.map.options.MTPaddingOptions
import com.maptiler.maptilersdk.map.style.MTMapReferenceStyle
import com.maptiler.maptilersdk.map.style.MTStyleError
import com.maptiler.maptilersdk.map.style.layer.line.MTLineLayer
import com.maptiler.maptilersdk.map.style.source.MTGeoJSONSource
import com.maptiler.maptilersdk.map.types.MTBounds
import com.maptiler.maptilersdk.map.types.MTData
import com.rustella.stellartrail.core.trail.readTrailUpload
import com.rustella.stellartrail.core.trail.trailDocumentMimeTypes
import com.rustella.stellartrail.domain.trip.MapAnnotation
import com.rustella.stellartrail.domain.trip.MapConfigResponse
import com.rustella.stellartrail.domain.trip.MapStyleOption
import com.rustella.stellartrail.domain.trip.MapTrailLink
import com.rustella.stellartrail.domain.trip.Trail
import com.rustella.stellartrail.domain.trip.TrailBounds
import com.rustella.stellartrail.domain.trip.TripOverviewMapTrail
import com.rustella.stellartrail.feature.trips.TripMapUiState
import com.rustella.stellartrail.feature.trips.TripsOverviewMapUiState
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.coroutines.delay
import java.net.URL
import kotlin.math.hypot

@Composable
fun TripsOverviewMapSection(
    state: TripsOverviewMapUiState,
    selectedStyleId: String?,
    onSelectMapStyle: (String) -> Unit,
    onOpenTrailLibrary: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val data = state.data
    if (state.loading && data == null) {
        SurfaceCard(modifier.fillMaxWidth()) { LoadingState() }
        return
    }
    if (data == null) {
        state.error?.let { error ->
            SurfaceCard(modifier.fillMaxWidth()) {
                Text("轨迹总览暂不可用", fontWeight = FontWeight.ExtraBold)
                Text(error, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        return
    }
    val trails = data.trails
    val canRenderMap = data.map.enabled && data.map.publicKey?.isNotBlank() == true
    var expandedMap by remember { mutableStateOf(false) }
    var compactStyleIdWhileExpanded by remember { mutableStateOf<String?>(null) }
    val featureCollection = rememberOverviewFeatureCollection(trails)
    val styleOptions = if (canRenderMap) resolveMapStyleOptions(data.map) else emptyList()
    val selectedStyle = if (canRenderMap) resolveSelectedMapStyle(data.map, selectedStyleId) else null
    val compactSelectedStyle = if (canRenderMap) {
        resolveSelectedMapStyle(data.map, compactStyleIdWhileExpanded ?: selectedStyleId)
    } else {
        null
    }
    SurfaceCard(modifier.fillMaxWidth(), contentPadding = PaddingValues(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("行程轨迹总览", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text(
                    if (trails.isEmpty()) "上传轨迹后会在这里汇总显示。" else "${data.stats.tripCount} 个行程 · ${data.stats.trailCount} 条轨迹",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                if (data.truncated) Badge("已简化显示", tone = BadgeTone.Info)
                CompactPillAction("轨迹库", onOpenTrailLibrary)
            }
        }
        if (canRenderMap && compactSelectedStyle != null) {
            MapTilerTrailMap(
                map = data.map,
                styleOptions = styleOptions,
                selectedStyle = compactSelectedStyle,
                onSelectStyle = onSelectMapStyle,
                bounds = data.bounds,
                featureCollection = featureCollection,
                height = 204.dp,
                lineColor = USER_TRAIL_COLOR,
                eventLevel = MTEventLevel.ESSENTIAL,
                zoomGesturesEnabled = false,
                onMapTap = { _, _ ->
                    compactStyleIdWhileExpanded = compactSelectedStyle.id
                    expandedMap = true
                },
            )
        } else {
            CompactMapFallback(
                title = "地图暂未启用",
                body = "后端未返回可用 MapTiler public key。",
                height = 120.dp,
            )
        }
    }
    if (expandedMap && canRenderMap && selectedStyle != null) {
        ExpandedTrailMapDialog(
            title = "行程轨迹总览",
            map = data.map,
            styleOptions = styleOptions,
            selectedStyle = selectedStyle,
            onSelectStyle = onSelectMapStyle,
            bounds = data.bounds,
            featureCollection = featureCollection,
            lineColor = USER_TRAIL_COLOR,
            eventLevel = MTEventLevel.ESSENTIAL,
            onDismiss = {
                expandedMap = false
                compactStyleIdWhileExpanded = null
            },
        )
    }
}

@Composable
fun TripDetailMapSection(
    state: TripMapUiState,
    selectedStyleId: String?,
    onSelectMapStyle: (String) -> Unit,
    onUploadTrail: (String, String?, ByteArray) -> Unit,
    onRemoveTrail: (String) -> Unit,
    onCreateAnnotation: (Double, Double, String, String) -> Unit,
    onUpdateAnnotation: (String, String, String) -> Unit,
    onDeleteAnnotation: (String) -> Unit,
    onOpenTrailLibrary: () -> Unit,
    onRefresh: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    var pendingPoint by remember { mutableStateOf<Pair<Double, Double>?>(null) }
    var editingAnnotation by remember { mutableStateOf<MapAnnotation?>(null) }
    var expandedMap by remember { mutableStateOf(false) }
    var compactStyleIdWhileExpanded by remember { mutableStateOf<String?>(null) }
    var showAddTrailDialog by remember { mutableStateOf(false) }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let { readTrailUpload(context, it)?.let { upload -> onUploadTrail(upload.filename, upload.contentType, upload.bytes) } }
    }
    SurfaceCard(modifier.fillMaxWidth(), contentPadding = PaddingValues(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("轨迹地图", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text("上传轨迹后可在地图上查看和备注。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            CompactPillAction("刷新", onRefresh)
        }
        if (state.loading && state.data == null) LoadingState()
        state.error?.let { ErrorState(it, onRetry = onRefresh) }
        val data = state.data
        if (data != null) {
            val canRenderMap = data.map.enabled && data.map.publicKey?.isNotBlank() == true
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                MetricTile("轨迹", "${data.trails.size}", Modifier.weight(1f))
                MetricTile("备注", "${data.annotations.size}", Modifier.weight(1f))
                MetricTile("点数", "${data.trails.sumOf { it.trail.pointCount }}", Modifier.weight(1f))
            }
            if (canRenderMap) {
                val styleOptions = resolveMapStyleOptions(data.map)
                val selectedStyle = resolveSelectedMapStyle(data.map, compactStyleIdWhileExpanded ?: selectedStyleId)
                val bounds = unionBounds(data.trails.mapNotNull { it.trail.bounds })
                val featureCollection = rememberTripFeatureCollection(data.trails)
                MapTilerTrailMap(
                    map = data.map,
                    styleOptions = styleOptions,
                    selectedStyle = selectedStyle,
                    onSelectStyle = onSelectMapStyle,
                    bounds = bounds,
                    featureCollection = featureCollection,
                    height = 260.dp,
                    lineColor = USER_TRAIL_COLOR,
                    eventLevel = MTEventLevel.ALL,
                    zoomGesturesEnabled = false,
                    onMapTap = { _, _ ->
                        compactStyleIdWhileExpanded = selectedStyle.id
                        expandedMap = true
                    },
                    onMapLongPress = { lng, lat -> pendingPoint = lng to lat },
                )
            } else {
                CompactMapFallback(
                    title = "地图暂未启用",
                    body = "后端未返回可用 MapTiler public key。",
                    height = 150.dp,
                )
            }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PrimaryPillButton(
                    if (state.mutating) "处理中" else "添加轨迹",
                    { showAddTrailDialog = true },
                    Modifier.weight(1f),
                    enabled = !state.mutating,
                )
                SoftPillButton("刷新地图", onRefresh, Modifier.weight(1f), enabled = !state.mutating)
            }
            data.trails.forEach { link ->
                TrailLinkRow(link = link, onRemove = { onRemoveTrail(link.trailId) }, enabled = !state.mutating)
            }
            if (data.annotations.isNotEmpty()) SectionTitle("地图备注")
            data.annotations.forEach { annotation ->
                AnnotationRow(
                    annotation = annotation,
                    onEdit = { editingAnnotation = annotation },
                    onDelete = { onDeleteAnnotation(annotation.id) },
                    enabled = !state.mutating,
                )
            }
        }
    }
    state.data?.let { data ->
        val canRenderMap = data.map.enabled && data.map.publicKey?.isNotBlank() == true
        if (expandedMap && canRenderMap) {
            val styleOptions = resolveMapStyleOptions(data.map)
            val selectedStyle = resolveSelectedMapStyle(data.map, selectedStyleId)
            ExpandedTrailMapDialog(
                title = "轨迹地图",
                map = data.map,
                styleOptions = styleOptions,
                selectedStyle = selectedStyle,
                onSelectStyle = onSelectMapStyle,
                bounds = unionBounds(data.trails.mapNotNull { it.trail.bounds }),
                featureCollection = rememberTripFeatureCollection(data.trails),
                lineColor = USER_TRAIL_COLOR,
                eventLevel = MTEventLevel.ALL,
                onDismiss = {
                    expandedMap = false
                    compactStyleIdWhileExpanded = null
                },
                onMapLongPress = { lng, lat ->
                    expandedMap = false
                    compactStyleIdWhileExpanded = null
                    pendingPoint = lng to lat
                },
            )
        }
    }
    pendingPoint?.let { (lng, lat) ->
        AnnotationDialog(
            title = "新增地图备注",
            initialTitle = "",
            initialNote = "",
            onDismiss = { pendingPoint = null },
            onSave = { title, note ->
                pendingPoint = null
                onCreateAnnotation(lng, lat, title, note)
            },
        )
    }
    editingAnnotation?.let { annotation ->
        AnnotationDialog(
            title = "编辑地图备注",
            initialTitle = annotation.title.orEmpty(),
            initialNote = annotation.note.orEmpty(),
            onDismiss = { editingAnnotation = null },
            onSave = { title, note ->
                editingAnnotation = null
                onUpdateAnnotation(annotation.id, title, note)
            },
        )
    }
    if (showAddTrailDialog) {
        AlertDialog(
            onDismissRequest = { showAddTrailDialog = false },
            title = { Text("添加轨迹") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    PrimaryPillButton(
                        "上传文件",
                        {
                            showAddTrailDialog = false
                            launcher.launch(trailDocumentMimeTypes)
                        },
                        Modifier.fillMaxWidth(),
                    )
                    SoftPillButton(
                        "从轨迹库选择",
                        {
                            showAddTrailDialog = false
                            onOpenTrailLibrary()
                        },
                        Modifier.fillMaxWidth(),
                    )
                }
            },
            confirmButton = {},
            dismissButton = { TextButton(onClick = { showAddTrailDialog = false }) { Text("取消") } },
        )
    }
}

@Composable
fun TrailAssetPreviewMap(
    map: MapConfigResponse,
    trail: Trail,
    selectedStyleId: String?,
    onSelectMapStyle: (String) -> Unit,
    modifier: Modifier = Modifier,
    height: Dp = 220.dp,
) {
    val canRenderMap = map.enabled && map.publicKey?.isNotBlank() == true
    if (!canRenderMap) {
        CompactMapFallback(
            title = "地图暂未启用",
            body = "后端未返回可用 MapTiler public key。",
            height = height,
        )
        return
    }
    val styleOptions = resolveMapStyleOptions(map)
    val selectedStyle = resolveSelectedMapStyle(map, selectedStyleId)
    MapTilerTrailMap(
        map = map,
        styleOptions = styleOptions,
        selectedStyle = selectedStyle,
        onSelectStyle = onSelectMapStyle,
        bounds = trail.bounds,
        featureCollection = featureCollectionJson(listOf(trail.simplifiedGeojson), DETAIL_MAP_MAX_RENDERED_POINTS),
        height = height,
        lineColor = USER_TRAIL_COLOR,
        eventLevel = MTEventLevel.ESSENTIAL,
        zoomGesturesEnabled = false,
        onMapTap = { _, _ -> },
        modifier = modifier,
    )
}

@Composable
private fun MapTilerTrailMap(
    map: MapConfigResponse,
    styleOptions: List<MapStyleOption>,
    selectedStyle: MapStyleOption,
    onSelectStyle: (String) -> Unit,
    bounds: TrailBounds?,
    featureCollection: String,
    height: Dp,
    lineColor: Color,
    eventLevel: MTEventLevel,
    zoomGesturesEnabled: Boolean,
    onMapTap: (Double, Double) -> Unit,
    onMapLongPress: (Double, Double) -> Unit = { _, _ -> },
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val publicKey = map.publicKey.orEmpty()
    val styleUrl = selectedStyle.styleUrl.withMapTilerKey(publicKey)
    var legendVisible by remember { mutableStateOf(false) }
    var styleSwitchLocked by remember { mutableStateOf(false) }
    LaunchedEffect(styleUrl) {
        if (styleSwitchLocked) {
            delay(MAP_STYLE_SWITCH_COOLDOWN_MILLIS)
            styleSwitchLocked = false
        }
    }
    val currentOnMapTap by rememberUpdatedState<(Double, Double) -> Unit> { lng, lat ->
        if (legendVisible) {
            legendVisible = false
        } else {
            onMapTap(lng, lat)
        }
    }
    val currentOnMapLongPress by rememberUpdatedState<(Double, Double) -> Unit> { lng, lat ->
        if (legendVisible) {
            legendVisible = false
        } else {
            onMapLongPress(lng, lat)
        }
    }
    val onSafeSelectStyle by rememberUpdatedState<(String) -> Unit> { styleId ->
        if (!styleSwitchLocked && styleId != selectedStyle.id) {
            legendVisible = false
            styleSwitchLocked = true
            onSelectStyle(styleId)
        }
    }
    Box(
        modifier
            .fillMaxWidth()
            .height(height)
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
    ) {
        key(styleUrl) {
            val controllerDelegate = remember(featureCollection, bounds, lineColor, eventLevel, zoomGesturesEnabled) {
                MTConfig.apiKey = publicKey
                TrailMapDelegate(
                    context = context,
                    featureCollection = featureCollection,
                    bounds = bounds,
                    lineColor = lineColor.toArgb(),
                    eventLevel = eventLevel,
                    zoomGesturesEnabled = zoomGesturesEnabled,
                    onTap = { lng, lat -> currentOnMapTap(lng, lat) },
                    onLongPress = { lng, lat -> currentOnMapLongPress(lng, lat) },
                )
            }
            MTMapView(
                referenceStyle = MTMapReferenceStyle.CUSTOM(URL(styleUrl)),
                options = MTMapOptions(
                    center = SHENZHEN_MAP_CENTER,
                    zoom = SHENZHEN_MAP_ZOOM,
                    minZoom = 2.0,
                    maxZoom = 18.0,
                    isInteractionEnabled = true,
                    dragPanIsEnabled = true,
                    dragRotateIsEnabled = zoomGesturesEnabled,
                    doubleTapShouldZoom = zoomGesturesEnabled,
                    shouldPinchToRotateAndZoom = zoomGesturesEnabled,
                    navigationControlIsVisible = false,
                    geolocateControlIsVisible = false,
                    terrainControlIsVisible = false,
                    scaleControlIsVisible = false,
                    minimapIsVisible = false,
                    eventLevel = eventLevel,
                    highFrequencyEventThrottleMs = 250,
                ),
                controller = controllerDelegate.controller,
                modifier = Modifier.fillMaxSize(),
            )
            MapZoomControls(
                onZoomIn = { controllerDelegate.controller.zoomIn() },
                onZoomOut = { controllerDelegate.controller.zoomOut() },
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = 8.dp, bottom = 12.dp),
            )
        }
        MapStyleSelector(
            styles = styleOptions,
            selectedStyleId = selectedStyle.id,
            enabled = !styleSwitchLocked,
            onSelectStyle = onSafeSelectStyle,
            modifier = Modifier
                .align(Alignment.TopEnd)
                .padding(8.dp),
        )
        MapLegendHelpButton(
            expanded = legendVisible,
            onToggle = { legendVisible = !legendVisible },
            modifier = Modifier
                .align(Alignment.BottomStart)
                .padding(start = 8.dp, bottom = 76.dp),
        )
        if (legendVisible) {
            MapLegendPopover(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = 46.dp, bottom = 76.dp),
            )
        }
    }
}

@Composable
private fun MapZoomControls(
    onZoomIn: () -> Unit,
    onZoomOut: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
        tonalElevation = 2.dp,
        shadowElevation = 2.dp,
    ) {
        Column(Modifier.padding(1.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            MapZoomButton("+", onZoomIn)
            Box(
                Modifier
                    .width(20.dp)
                    .height(1.dp)
                    .background(MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.6f)),
            )
            MapZoomButton("-", onZoomOut)
        }
    }
}

@Composable
private fun MapZoomButton(symbol: String, onClick: () -> Unit) {
    Box(
        Modifier
            .size(28.dp)
            .clip(RoundedCornerShape(6.dp))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            symbol,
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
        )
    }
}

@Composable
private fun MapLegendHelpButton(
    expanded: Boolean,
    onToggle: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(8.dp),
        color = if (expanded) MaterialTheme.colorScheme.primary.copy(alpha = 0.94f) else MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
        tonalElevation = 2.dp,
        shadowElevation = 2.dp,
    ) {
        Box(
            Modifier
                .size(28.dp)
                .clip(RoundedCornerShape(6.dp))
                .clickable(onClick = onToggle),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                "?",
                color = if (expanded) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 1,
            )
        }
    }
}

@Composable
private fun MapLegendPopover(modifier: Modifier = Modifier) {
    Surface(
        modifier = modifier.clickable { },
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
        tonalElevation = 1.dp,
        shadowElevation = 1.dp,
    ) {
        Row(
            Modifier.padding(horizontal = 8.dp, vertical = 5.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            MapLegendItem(color = USER_TRAIL_COLOR, label = "上传轨迹")
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    MapLegendLine(BASE_TRAIL_RED)
                    MapLegendLine(BASE_TRAIL_PURPLE)
                }
                Text(
                    "底图步道",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelSmall,
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun MapLegendItem(color: Color, label: String) {
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
        MapLegendLine(color)
        Text(
            label,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
        )
    }
}

@Composable
private fun MapLegendLine(color: Color) {
    Box(
        Modifier
            .width(16.dp)
            .height(3.dp)
            .clip(RoundedCornerShape(999.dp))
            .background(color),
    )
}

@Composable
private fun ExpandedTrailMapDialog(
    title: String,
    map: MapConfigResponse,
    styleOptions: List<MapStyleOption>,
    selectedStyle: MapStyleOption,
    onSelectStyle: (String) -> Unit,
    bounds: TrailBounds?,
    featureCollection: String,
    lineColor: Color,
    eventLevel: MTEventLevel,
    onDismiss: () -> Unit,
    onMapLongPress: (Double, Double) -> Unit = { _, _ -> },
) {
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            shape = RoundedCornerShape(18.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 6.dp,
        ) {
            Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                    TextButton(onClick = onDismiss) { Text("关闭") }
                }
                MapTilerTrailMap(
                    map = map,
                    styleOptions = styleOptions,
                    selectedStyle = selectedStyle,
                    onSelectStyle = onSelectStyle,
                    bounds = bounds,
                    featureCollection = featureCollection,
                    height = 480.dp,
                    lineColor = lineColor,
                    eventLevel = eventLevel,
                    zoomGesturesEnabled = true,
                    onMapTap = { _, _ -> },
                    onMapLongPress = onMapLongPress,
                )
            }
        }
    }
}

@Composable
private fun MapStyleSelector(
    styles: List<MapStyleOption>,
    selectedStyleId: String,
    enabled: Boolean,
    onSelectStyle: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (styles.size <= 1) return
    Row(
        modifier
            .clip(RoundedCornerShape(999.dp))
            .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.94f))
            .padding(2.dp),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        styles.forEach { style ->
            val selected = style.id == selectedStyleId
            Text(
                text = style.label,
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(if (selected) MaterialTheme.colorScheme.primary else Color.Transparent)
                    .clickable(enabled = enabled && !selected) { onSelectStyle(style.id) }
                    .padding(horizontal = 10.dp, vertical = 6.dp),
                color = if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = if (selected) FontWeight.ExtraBold else FontWeight.Bold,
                maxLines = 1,
            )
        }
    }
}

private class TrailMapDelegate(
    context: Context,
    private val featureCollection: String,
    private val bounds: TrailBounds?,
    private val lineColor: Int,
    private val eventLevel: MTEventLevel,
    private val zoomGesturesEnabled: Boolean,
    private val onTap: (Double, Double) -> Unit,
    private val onLongPress: (Double, Double) -> Unit,
) : MTMapViewDelegate {
    val controller = MTMapViewController(context).apply { delegate = this@TrailMapDelegate }
    private var touchCandidate: LongPressCandidate? = null
    private var suppressNextTap = false
    private var pinchGestureEnabled = false

    override fun onMapViewInitialized() {
        renderTrailLayer()
        enablePinchGestureIfNeeded()
    }

    override fun onEventTriggered(event: MTEvent, data: MTData?) {
        when (event) {
            MTEvent.ON_READY, MTEvent.ON_LOAD -> {
                enablePinchGestureIfNeeded()
            }
            MTEvent.ON_TAP -> {
                if (suppressNextTap) {
                    suppressNextTap = false
                    return
                }
                data?.coordinate?.let { onTap(it.lng, it.lat) }
            }
            MTEvent.ON_TOUCH_START -> {
                touchCandidate = data?.coordinate?.let { coordinate ->
                    val point = data.point
                    LongPressCandidate(
                        startedAtMillis = System.currentTimeMillis(),
                        lng = coordinate.lng,
                        lat = coordinate.lat,
                        x = point?.x,
                        y = point?.y,
                    )
                }
            }
            MTEvent.ON_TOUCH_MOVE -> {
                val candidate = touchCandidate
                val point = data?.point
                if (candidate != null && point != null && candidate.hasMovedPast(point.x, point.y)) {
                    touchCandidate = null
                }
            }
            MTEvent.ON_TOUCH_END -> {
                val candidate = touchCandidate
                touchCandidate = null
                val point = data?.point
                if (
                    candidate != null &&
                    System.currentTimeMillis() - candidate.startedAtMillis >= LONG_PRESS_MIN_MILLIS &&
                    (point == null || !candidate.hasMovedPast(point.x, point.y))
                ) {
                    suppressNextTap = true
                    onLongPress(candidate.lng, candidate.lat)
                }
            }
            MTEvent.ON_TOUCH_CANCEL -> {
                touchCandidate = null
            }
            else -> Unit
        }
    }

    private fun enablePinchGestureIfNeeded() {
        if (!zoomGesturesEnabled || pinchGestureEnabled) return
        val gestureService = controller.gestureService ?: return
        runCatching { gestureService.enablePinchRotateAndZoomGesture() }
            .onSuccess { pinchGestureEnabled = true }
    }

    private fun renderTrailLayer() {
        val style = controller.style ?: return
        runCatching {
            style.addSource(MTGeoJSONSource(TRAIL_SOURCE_ID, featureCollection))
        }.onFailure { if (it !is MTStyleError.SourceAlreadyExists) throw it }
        runCatching {
            style.addLayer(
                MTLineLayer(TRAIL_OUTLINE_LAYER_ID, TRAIL_SOURCE_ID).apply {
                    color = USER_TRAIL_OUTLINE_COLOR.toArgb()
                    width = if (eventLevel == MTEventLevel.ESSENTIAL) 6.0 else 7.0
                    opacity = 0.95
                },
            )
        }.onFailure { if (it !is MTStyleError.LayerAlreadyExists) throw it }
        runCatching {
            style.addLayer(
                MTLineLayer(TRAIL_LAYER_ID, TRAIL_SOURCE_ID).apply {
                    color = lineColor
                    width = if (eventLevel == MTEventLevel.ESSENTIAL) 3.5 else 4.5
                    opacity = 0.98
                },
            )
        }.onFailure { if (it !is MTStyleError.LayerAlreadyExists) throw it }
        bounds?.let {
            controller.fitBounds(
                MTBounds(it.minLng, it.minLat, it.maxLng, it.maxLat),
                MTFitBoundsOptions(
                    padding = MTPaddingOptions(left = 24.0, top = 24.0, right = 24.0, bottom = 24.0),
                    maxZoom = 14.0,
                    duration = 0.0,
                ),
            )
        }
    }
}

private data class LongPressCandidate(
    val startedAtMillis: Long,
    val lng: Double,
    val lat: Double,
    val x: Double?,
    val y: Double?,
) {
    fun hasMovedPast(nextX: Double, nextY: Double): Boolean =
        x != null && y != null && hypot(nextX - x, nextY - y) > LONG_PRESS_MOVE_TOLERANCE_PX
}

@Composable
private fun TrailLinkRow(link: MapTrailLink, onRemove: () -> Unit, enabled: Boolean) {
    SurfaceCard(contentPadding = PaddingValues(10.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(link.trail.displayName, fontWeight = FontWeight.ExtraBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text("${(link.trail.distanceM / 1000.0).formatOne()} km · ${link.trail.pointCount} 点", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            SoftPillButton("移除", onRemove, enabled = enabled)
        }
    }
}

@Composable
private fun AnnotationRow(annotation: MapAnnotation, onEdit: () -> Unit, onDelete: () -> Unit, enabled: Boolean) {
    SurfaceCard(contentPadding = PaddingValues(10.dp)) {
        Text(annotation.title?.takeIf { it.isNotBlank() } ?: "地图备注", fontWeight = FontWeight.ExtraBold)
        Text(annotation.note?.takeIf { it.isNotBlank() } ?: "${annotation.lng.formatCoord()}, ${annotation.lat.formatCoord()}", color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SoftPillButton("编辑", onEdit, Modifier.weight(1f), enabled = enabled)
            SoftPillButton("删除", onDelete, Modifier.weight(1f), enabled = enabled)
        }
    }
}

@Composable
private fun AnnotationDialog(
    title: String,
    initialTitle: String,
    initialNote: String,
    onDismiss: () -> Unit,
    onSave: (String, String) -> Unit,
) {
    var annotationTitle by remember { mutableStateOf(initialTitle) }
    var note by remember { mutableStateOf(initialNote) }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(annotationTitle, { annotationTitle = it }, label = { Text("标题") }, singleLine = true)
                OutlinedTextField(note, { note = it }, label = { Text("备注") }, minLines = 3)
            }
        },
        confirmButton = { TextButton(onClick = { onSave(annotationTitle, note) }) { Text("保存") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } },
    )
}

@Composable
private fun CompactMapFallback(title: String, body: String, height: Dp) {
    Box(
        Modifier
            .fillMaxWidth()
            .height(height)
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(16.dp),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(title, fontWeight = FontWeight.ExtraBold)
            Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun rememberOverviewFeatureCollection(trails: List<TripOverviewMapTrail>): String =
    remember(trails) { featureCollectionJson(trails.map { it.simplifiedGeojson }) }

@Composable
private fun rememberTripFeatureCollection(trails: List<MapTrailLink>): String =
    remember(trails) { featureCollectionJson(trails.map { it.simplifiedGeojson }, DETAIL_MAP_MAX_RENDERED_POINTS) }

internal fun featureCollectionJson(features: List<JsonElement>, maxRenderedPoints: Int? = null): String = mapJson.encodeToString(
    JsonObject(
        mapOf(
            "type" to JsonPrimitive("FeatureCollection"),
            "features" to JsonArray(maxRenderedPoints?.let { limitFeaturePointCount(features, it) } ?: features),
        ),
    ),
)

private fun limitFeaturePointCount(features: List<JsonElement>, maxRenderedPoints: Int): List<JsonElement> {
    if (maxRenderedPoints <= 0 || features.totalPointCount() <= maxRenderedPoints) return features
    var remainingBudget = maxRenderedPoints
    var remainingFeatures = features.size
    return features.map { feature ->
        val minimumPoints = if (remainingFeatures <= 1) remainingBudget else maxOf(2, remainingBudget / remainingFeatures)
        val limited = limitFeaturePointCount(feature, minimumPoints)
        remainingBudget = (remainingBudget - limited.pointCount()).coerceAtLeast(0)
        remainingFeatures--
        limited
    }
}

private fun List<JsonElement>.totalPointCount(): Int = sumOf { it.pointCount() }

private fun JsonElement.pointCount(): Int {
    val feature = this as? JsonObject ?: return 0
    val geometry = feature["geometry"] as? JsonObject ?: return 0
    return when ((geometry["type"] as? JsonPrimitive)?.content) {
        "LineString" -> (geometry["coordinates"] as? JsonArray)?.size ?: 0
        "MultiLineString" -> (geometry["coordinates"] as? JsonArray)?.sumOf { (it as? JsonArray)?.size ?: 0 } ?: 0
        else -> 0
    }
}

private fun limitFeaturePointCount(feature: JsonElement, maxPoints: Int): JsonElement {
    if (feature.pointCount() <= maxPoints) return feature
    val featureObject = feature as? JsonObject ?: return feature
    val geometry = featureObject["geometry"] as? JsonObject ?: return feature
    val limitedGeometry = when ((geometry["type"] as? JsonPrimitive)?.content) {
        "LineString" -> geometry.copyWithCoordinates(
            simplifyCoordinates(geometry["coordinates"] as? JsonArray, maxPoints),
        )
        "MultiLineString" -> geometry.copyWithCoordinates(
            simplifyMultiLineCoordinates(geometry["coordinates"] as? JsonArray, maxPoints),
        )
        else -> geometry
    }
    return JsonObject(featureObject.toMutableMap().also { it["geometry"] = limitedGeometry })
}

private fun JsonObject.copyWithCoordinates(coordinates: JsonArray): JsonObject =
    JsonObject(toMutableMap().also { it["coordinates"] = coordinates })

private fun simplifyMultiLineCoordinates(coordinates: JsonArray?, maxPoints: Int): JsonArray {
    val lines = coordinates ?: return JsonArray(emptyList())
    val nonEmpty = lines.mapNotNull { it as? JsonArray }.filter { it.isNotEmpty() }
    if (nonEmpty.isEmpty()) return JsonArray(emptyList())
    val perLine = maxOf(2, maxPoints / nonEmpty.size.coerceAtLeast(1))
    return JsonArray(nonEmpty.map { simplifyCoordinates(it, perLine) })
}

private fun simplifyCoordinates(coordinates: JsonArray?, maxPoints: Int): JsonArray {
    val points = coordinates ?: return JsonArray(emptyList())
    if (points.size <= maxPoints || maxPoints < 2) return points
    val lastIndex = points.lastIndex
    val simplified = (0 until maxPoints).map { index ->
        points[((index * lastIndex).toDouble() / (maxPoints - 1)).toInt().coerceIn(0, lastIndex)]
    }.distinct()
    return JsonArray(simplified)
}

private fun unionBounds(bounds: List<TrailBounds>): TrailBounds? = bounds.reduceOrNull { current, next ->
    TrailBounds(
        minLng = minOf(current.minLng, next.minLng),
        minLat = minOf(current.minLat, next.minLat),
        maxLng = maxOf(current.maxLng, next.maxLng),
        maxLat = maxOf(current.maxLat, next.maxLat),
    )
}

internal fun resolveMapStyleOptions(map: MapConfigResponse): List<MapStyleOption> {
    val configuredStyles = map.styles.mapNotNull { style ->
        val id = style.id.trim()
        val styleUrl = style.styleUrl.trim()
        if (id.isEmpty() || styleUrl.isEmpty()) {
            null
        } else {
            style.copy(
                id = id,
                label = style.label.trim().ifEmpty { fallbackMapStyleLabel(id) },
                styleUrl = styleUrl,
            )
        }
    }
    if (configuredStyles.isNotEmpty()) return configuredStyles
    val fallbackId = map.defaultStyleId.trim().ifEmpty { DEFAULT_MAP_STYLE_ID }
    return listOf(
        MapStyleOption(
            id = fallbackId,
            label = fallbackMapStyleLabel(fallbackId),
            styleUrl = map.styleUrl,
        ),
    )
}

internal fun resolveSelectedMapStyle(map: MapConfigResponse, selectedStyleId: String?): MapStyleOption {
    val styles = resolveMapStyleOptions(map)
    val selectedId = selectedStyleId?.trim()
    return styles.firstOrNull { it.id == selectedId }
        ?: styles.firstOrNull { it.id == map.defaultStyleId.trim() }
        ?: styles.first()
}

private fun String.withMapTilerKey(publicKey: String): String = when {
    publicKey.isBlank() || contains("key=") -> this
    contains('?') -> "$this&key=$publicKey"
    else -> "$this?key=$publicKey"
}

private fun Double.formatOne(): String = "%.1f".format(this)
private fun Double.formatCoord(): String = "%.5f".format(this)
private fun fallbackMapStyleLabel(styleId: String): String = when (styleId) {
    "streets" -> "街道"
    "satellite" -> "卫星"
    else -> "户外"
}

private val mapJson = Json { encodeDefaults = false }
private val USER_TRAIL_COLOR = Color(0xFF0B7CFF)
private val USER_TRAIL_OUTLINE_COLOR = Color.White
private val BASE_TRAIL_RED = Color(0xFFE9361F)
private val BASE_TRAIL_PURPLE = Color(0xFFE63DCD)
private val SHENZHEN_MAP_CENTER = LngLat(114.0579, 22.5431)
private const val SHENZHEN_MAP_ZOOM = 10.5
private const val LONG_PRESS_MIN_MILLIS = 550L
private const val LONG_PRESS_MOVE_TOLERANCE_PX = 18.0
private const val MAP_STYLE_SWITCH_COOLDOWN_MILLIS = 700L
private const val DETAIL_MAP_MAX_RENDERED_POINTS = 8000
private const val DEFAULT_MAP_STYLE_ID = "outdoor"
private const val TRAIL_SOURCE_ID = "stellartrail-trails"
private const val TRAIL_OUTLINE_LAYER_ID = "stellartrail-trails-outline"
private const val TRAIL_LAYER_ID = "stellartrail-trails-line"
