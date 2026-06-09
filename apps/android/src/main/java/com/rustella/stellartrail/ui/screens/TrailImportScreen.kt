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
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.dateText
import com.rustella.stellartrail.feature.trails.TrailImportMode
import com.rustella.stellartrail.feature.trails.TrailImportViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun TrailImportScreen(
    viewModel: TrailImportViewModel,
    isLoggedIn: Boolean,
    onLogin: () -> Unit,
    onBack: () -> Unit,
    onOpenTrailLibrary: () -> Unit,
    onOpenTrip: (String) -> Unit,
    onOpenOutdoorExperiences: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!isLoggedIn) {
        LoginRequiredScreen(
            title = "登录后导入轨迹",
            body = "轨迹会先保存到你的个人轨迹库，再关联到行程或户外经历。",
            onLogin = onLogin,
            modifier = modifier,
        )
        return
    }
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            SurfaceCard {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("导入轨迹", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                        Text("先保存为独立轨迹资产，再选择关联目标。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    SoftPillButton("返回", onBack)
                }
            }
        }
        state.error?.let { item { ErrorState(it) } }
        state.notice?.let { item { SurfaceCard { Text(it, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold) } } }
        state.pending?.let { pending ->
            item {
                SurfaceCard {
                    Text(pending.filename, fontWeight = FontWeight.ExtraBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${pending.contentType ?: "未知类型"} · ${pending.sizeBytes.formatBytes()}", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    if (state.uploadedTrail == null) {
                        PrimaryPillButton(
                            if (state.mutating) "导入中" else "导入到轨迹库",
                            viewModel::uploadToLibrary,
                            Modifier.fillMaxWidth(),
                            enabled = !state.mutating,
                        )
                    }
                }
            }
        }
        state.uploadedTrail?.let { trail ->
            item {
                SurfaceCard {
                    Badge("已保存到轨迹库")
                    Text(trail.displayName, fontWeight = FontWeight.ExtraBold)
                    Text("${(trail.distanceM / 1000.0).formatOne()} km · ${trail.pointCount} 点", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            when (state.mode) {
                TrailImportMode.Actions -> item {
                    ImportActionCard(
                        mutating = state.mutating,
                        onExistingTrip = viewModel::showExistingTrip,
                        onNewTrip = viewModel::showNewTrip,
                        onOutdoorExperience = viewModel::showOutdoorExperience,
                        onSaveOnly = { viewModel.saveOnly(onOpenTrailLibrary) },
                    )
                }
                TrailImportMode.ExistingTrip -> {
                    item { ImportModeHeader("添加到已有行程", viewModel::showActions) }
                    if (state.loading) item { LoadingState() }
                    items(state.trips, key = { it.id }) { trip ->
                        ExistingTripRow(
                            trip = trip,
                            enabled = !state.mutating,
                            onClick = { viewModel.linkToExistingTrip(trip.id, onOpenTrip) },
                        )
                    }
                }
                TrailImportMode.NewTrip -> item {
                    NewTripImportCard(
                        title = state.newTripTitle,
                        type = state.newTripType,
                        mutating = state.mutating,
                        onTitleChange = viewModel::updateNewTripTitle,
                        onTypeChange = viewModel::updateNewTripType,
                        onBack = viewModel::showActions,
                        onCreate = { viewModel.createTripAndLink(onOpenTrip) },
                    )
                }
                TrailImportMode.OutdoorExperience -> item {
                    OutdoorExperienceImportCard(
                        title = state.outdoorExperienceTitle,
                        mutating = state.mutating,
                        onTitleChange = viewModel::updateOutdoorExperienceTitle,
                        onBack = viewModel::showActions,
                        onCreate = { viewModel.createOutdoorExperienceAndLink(onOpenOutdoorExperiences) },
                    )
                }
            }
        }
    }
}

@Composable
private fun ImportActionCard(
    mutating: Boolean,
    onExistingTrip: () -> Unit,
    onNewTrip: () -> Unit,
    onOutdoorExperience: () -> Unit,
    onSaveOnly: () -> Unit,
) {
    SurfaceCard {
        Text("选择用途", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        PrimaryPillButton("添加到已有行程", onExistingTrip, Modifier.fillMaxWidth(), enabled = !mutating)
        SoftPillButton("新建行程并添加", onNewTrip, Modifier.fillMaxWidth(), enabled = !mutating)
        SoftPillButton("记录为户外经历", onOutdoorExperience, Modifier.fillMaxWidth(), enabled = !mutating)
        SoftPillButton("仅保存到轨迹库", onSaveOnly, Modifier.fillMaxWidth(), enabled = !mutating)
    }
}

@Composable
private fun ImportModeHeader(title: String, onBack: () -> Unit) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            SoftPillButton("返回选项", onBack)
        }
    }
}

@Composable
private fun ExistingTripRow(trip: TripSummary, enabled: Boolean, onClick: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(enabled = enabled, onClick = onClick)) {
        Text(trip.displayName, fontWeight = FontWeight.ExtraBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(trip.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun NewTripImportCard(
    title: String,
    type: TripType,
    mutating: Boolean,
    onTitleChange: (String) -> Unit,
    onTypeChange: (TripType) -> Unit,
    onBack: () -> Unit,
    onCreate: () -> Unit,
) {
    SurfaceCard {
        Text("新建行程并添加", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        OutlinedTextField(title, onTitleChange, label = { Text("行程名称") }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (type == TripType.SOLO) PrimaryPillButton("单人", { onTypeChange(TripType.SOLO) }, Modifier.weight(1f)) else SoftPillButton("单人", { onTypeChange(TripType.SOLO) }, Modifier.weight(1f))
            if (type == TripType.TEAM) PrimaryPillButton("多人", { onTypeChange(TripType.TEAM) }, Modifier.weight(1f)) else SoftPillButton("多人", { onTypeChange(TripType.TEAM) }, Modifier.weight(1f))
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SoftPillButton("返回选项", onBack, Modifier.weight(1f), enabled = !mutating)
            PrimaryPillButton(if (mutating) "创建中" else "创建并添加", onCreate, Modifier.weight(1f), enabled = !mutating)
        }
    }
}

@Composable
private fun OutdoorExperienceImportCard(
    title: String,
    mutating: Boolean,
    onTitleChange: (String) -> Unit,
    onBack: () -> Unit,
    onCreate: () -> Unit,
) {
    SurfaceCard {
        Text("记录为户外经历", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        OutlinedTextField(title, onTitleChange, label = { Text("经历标题") }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SoftPillButton("返回选项", onBack, Modifier.weight(1f), enabled = !mutating)
            PrimaryPillButton(if (mutating) "记录中" else "记录并关联", onCreate, Modifier.weight(1f), enabled = !mutating)
        }
    }
}

private fun Long.formatBytes(): String = when {
    this >= 1024L * 1024L -> "${this / (1024L * 1024L)} MB"
    this >= 1024L -> "${this / 1024L} KB"
    else -> "$this B"
}

private fun Double.formatOne(): String = "%.1f".format(this)
