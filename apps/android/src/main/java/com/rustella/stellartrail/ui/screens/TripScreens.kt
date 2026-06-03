package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.trip.TripBudgetItem
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripGoalItem
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripTimeBucket
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.dateText
import com.rustella.stellartrail.domain.trip.durationText
import com.rustella.stellartrail.domain.trip.label
import com.rustella.stellartrail.feature.trips.TripDetailViewModel
import com.rustella.stellartrail.feature.trips.TripFormViewModel
import com.rustella.stellartrail.feature.trips.TripJoinViewModel
import com.rustella.stellartrail.feature.trips.TripListViewModel
import com.rustella.stellartrail.feature.trips.visibleSections
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.DatePickerField
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroButton
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.HeroSoftButton
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.MetricTile
import com.rustella.stellartrail.ui.common.NoticeCard
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun TripsScreen(
    viewModel: TripListViewModel,
    isLoggedIn: Boolean,
    onLogin: () -> Unit,
    onCreateTrip: (TripType) -> Unit,
    onJoinTrip: () -> Unit,
    onOpenTrip: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var showCreateSheet by remember { mutableStateOf(false) }
    LaunchedEffect(isLoggedIn) { viewModel.refresh(isLoggedIn) }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(start = 16.dp, top = 0.dp, end = 16.dp, bottom = 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "我的户外行程",
                title = "我的行程",
                subtitle = "管理单人行程与组队协作，出发前准备更清晰。",
                chips = listOf("单人准备", "多人协作"),
                actions = {
                    HeroSoftButton("加入", onJoinTrip, Modifier.weight(1f))
                    HeroButton("制作行程", { showCreateSheet = true }, Modifier.weight(1f))
                },
            )
        }
        state.highlight?.let { item { TripHighlightCard(it, onOpenTrip) } }
        if (!isLoggedIn) {
            item {
                NoticeCard(
                    title = "登录后查看我的行程",
                    body = "行程计划和装备分工会保存到账号中。",
                    action = { CompactPillAction("登录", onLogin) },
                )
            }
        } else {
            if (state.error != null) item { ErrorState(state.error!!, onRetry = { viewModel.refresh(true) }) }
            if (state.loading) item { LoadingState() }
            if (!state.loading && state.trips.isEmpty()) {
                item {
                    EmptyState("还没有行程", "制作一个单人行程，或和队友一起准备多人行程。")
                }
                item {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        PrimaryPillButton("制作第一份行程", { showCreateSheet = true }, Modifier.weight(1f))
                        SoftPillButton("加入多人行程", onJoinTrip, Modifier.weight(1f))
                    }
                }
            }
            tripGroups(state.trips).forEach { group ->
                item { SectionTitle(group.title, group.subtitle) }
                items(group.trips, key = { it.id }) { trip ->
                    TripSummaryCard(
                        trip = trip,
                        onClick = { onOpenTrip(trip.id) },
                        onDelete = { viewModel.deleteTrip(trip.id) },
                        onConvert = { viewModel.convertToExperience(trip.id) },
                        mutating = state.mutatingId == trip.id,
                    )
                }
            }
            if (state.loadingMore) item { LoadingState() }
            if (state.nextCursor != null) item { PrimaryPillButton("加载更多", viewModel::loadMore, Modifier.fillMaxWidth()) }
            if (state.trips.isNotEmpty() && state.nextCursor == null) item { Text("没有更多行程了", color = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
    }
    CreateTripSheet(
        visible = showCreateSheet,
        onDismiss = { showCreateSheet = false },
        onCreateTrip = {
            showCreateSheet = false
            onCreateTrip(it)
        },
    )
}

@Composable
private fun TripHighlightCard(highlight: TripHomeHighlightItem, onOpenTrip: (String) -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth().clickable { onOpenTrip(highlight.trip.id) }) {
        Badge("近期行程", tone = BadgeTone.Info)
        Text(highlight.trip.displayName, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(highlight.trip.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            MetricTile("装备", "${highlight.trip.readiness.completionPercent}%", Modifier.weight(1f))
            MetricTile("成员", "${highlight.trip.memberCount}", Modifier.weight(1f))
            MetricTile("安排", highlight.trip.durationText(), Modifier.weight(1f))
        }
        Text("出发前检查：装备、技能、天气和安全预案。", color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun TripSummaryCard(
    trip: TripSummary,
    onClick: () -> Unit,
    onDelete: () -> Unit,
    onConvert: () -> Unit,
    mutating: Boolean,
) {
    SurfaceCard(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Badge(trip.tripType.label())
            Badge(trip.durationText(), tone = BadgeTone.Info)
        }
        Text(trip.displayName, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(trip.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(
            readinessText(trip),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            if (trip.timeBucket == TripTimeBucket.PAST && trip.outdoorExperienceId == null) {
                SoftPillButton(if (mutating) "处理中" else "转为经历", onConvert, Modifier.weight(1f), enabled = !mutating)
            }
            SoftPillButton(if (mutating) "处理中" else "删除", onDelete, Modifier.weight(1f), enabled = !mutating)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CreateTripSheet(
    visible: Boolean,
    onDismiss: () -> Unit,
    onCreateTrip: (TripType) -> Unit,
) {
    if (!visible) return
    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            SectionTitle("制作行程计划")
            SurfaceCard(Modifier.fillMaxWidth().clickable { onCreateTrip(TripType.SOLO) }) {
                Text("单人行程", fontWeight = FontWeight.ExtraBold)
                Text("只有自己的装备、行程、食品、医药和预算准备。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            SurfaceCard(Modifier.fillMaxWidth().clickable { onCreateTrip(TripType.TEAM) }) {
                Text("多人行程", fontWeight = FontWeight.ExtraBold)
                Text("保留成员协作、邀请加入和公共装备分工。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            SoftPillButton("取消", onDismiss, Modifier.fillMaxWidth())
            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
fun TripFormScreen(
    viewModel: TripFormViewModel,
    onBack: () -> Unit,
    onSaved: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { viewModel.load() }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            SurfaceCard {
                Badge(if (state.tripType == TripType.SOLO) "单人行程" else "多人行程")
                Text(if (state.isEdit) "编辑行程" else "制作行程", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                Text("名称、日期和备注会保存到行程计划中。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        if (state.error != null) item { ErrorState(state.error!!) }
        if (state.loading) item { LoadingState() }
        item {
            SurfaceCard {
                OutlinedTextField(
                    value = state.title,
                    onValueChange = viewModel::updateTitle,
                    label = { Text("行程名称") },
                    placeholder = { Text("例如：端午武功山重装") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                DatePickerField("开始日期", state.startDate, viewModel::updateStartDate, Modifier.fillMaxWidth())
                DatePickerField("结束日期", state.endDate, viewModel::updateEndDate, Modifier.fillMaxWidth())
                OutlinedTextField(
                    value = state.description,
                    onValueChange = viewModel::updateDescription,
                    label = { Text("备注") },
                    placeholder = { Text("集合方式、天气预案、风险提示") },
                    minLines = 3,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                SoftPillButton("返回", onBack, Modifier.weight(1f))
                PrimaryPillButton(if (state.saving) "保存中" else "保存行程", { viewModel.save(onSaved) }, Modifier.weight(1f), enabled = !state.saving)
            }
        }
    }
}

@Composable
fun TripJoinScreen(
    viewModel: TripJoinViewModel,
    onBack: () -> Unit,
    onAccepted: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            SurfaceCard {
                Text("加入组队计划", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                Text("输入队长发来的邀请口令，加入后可一起编辑计划内容。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                OutlinedTextField(
                    value = state.token,
                    onValueChange = viewModel::updateToken,
                    label = { Text("邀请口令或邀请文案") },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        if (state.error != null) item { ErrorState(state.error!!) }
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                SoftPillButton("返回", onBack, Modifier.weight(1f))
                PrimaryPillButton(if (state.loading) "加入中" else "加入计划", { viewModel.accept(onAccepted) }, Modifier.weight(1f), enabled = !state.loading)
            }
        }
    }
}

@Composable
fun TripDetailScreen(
    viewModel: TripDetailViewModel,
    onBack: () -> Unit,
    onEdit: (String) -> Unit,
    onDeleted: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { viewModel.load() }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        if (state.loading) item { LoadingState() }
        if (state.error != null) item { ErrorState(state.error!!, onRetry = viewModel::load) }
        state.detail?.let { detail ->
            item { TripDetailHeader(detail, onBack, onEdit, onDeleted = { viewModel.deleteTrip(onDeleted) }, onConvert = viewModel::convertToExperience) }
            item { TripSectionTabs(detail, state.selectedSection, viewModel::selectSection, viewModel::toggleSection) }
            item {
                when (state.selectedSection) {
                    TripSectionKey.MEMBERS -> MembersSection(detail, viewModel)
                    TripSectionKey.PERSONAL_GEAR -> PersonalGearSection(detail, viewModel)
                    TripSectionKey.SHARED_GEAR -> SharedGearSection(detail, viewModel)
                    TripSectionKey.ITINERARY -> ItinerarySection(detail, viewModel)
                    TripSectionKey.FOOD_PLAN -> FoodSection(detail, viewModel)
                    TripSectionKey.MEDICAL_KIT -> MedicalSection(detail, viewModel)
                    TripSectionKey.SAFETY_PLAN -> SafetySection(detail, viewModel)
                    TripSectionKey.RESCUE_INFO -> RescueSection(detail, viewModel)
                    TripSectionKey.BUDGET -> BudgetSection(detail, viewModel)
                    TripSectionKey.GOALS -> GoalsSection(detail, viewModel)
                }
            }
        }
    }
    state.conflict?.let { conflict ->
        AlertDialog(
            onDismissRequest = viewModel::clearConflict,
            title = { Text("内容已被更新") },
            text = { Text(conflict.message.ifBlank { "同一字段被其他成员更新，请刷新后再编辑，或覆盖本次字段。" }) },
            confirmButton = { TextButton(onClick = viewModel::forceConflictOverwrite) { Text("覆盖本次字段") } },
            dismissButton = { TextButton(onClick = { viewModel.clearConflict(); viewModel.load() }) { Text("刷新") } },
        )
    }
}

@Composable
private fun TripDetailHeader(
    detail: TripDetail,
    onBack: () -> Unit,
    onEdit: (String) -> Unit,
    onDeleted: () -> Unit,
    onConvert: () -> Unit,
) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Badge(detail.trip.tripType.label())
                Text(detail.trip.displayName, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                Text(detail.trip.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            SoftPillButton("返回", onBack)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            MetricTile("成员", "${detail.members.size}", Modifier.weight(1f))
            MetricTile("完成", "${detail.trip.readiness.completionPercent}%", Modifier.weight(1f))
            MetricTile("安排", detail.trip.durationText(), Modifier.weight(1f))
        }
        if (detail.trip.readiness.missingLabels.isNotEmpty()) {
            Text("待补齐：${detail.trip.readiness.missingLabels.joinToString("、")}", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SoftPillButton("编辑", { onEdit(detail.trip.id) }, Modifier.weight(1f))
            SoftPillButton("转为经历", onConvert, Modifier.weight(1f))
            SoftPillButton("删除", onDeleted, Modifier.weight(1f))
        }
    }
}

@Composable
private fun TripSectionTabs(
    detail: TripDetail,
    selected: TripSectionKey,
    onSelect: (TripSectionKey) -> Unit,
    onToggle: (TripSectionKey) -> Unit,
) {
    SurfaceCard(contentPadding = PaddingValues(12.dp)) {
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(TripSectionKey.entries, key = { it.name }) { section ->
                val enabled = section in detail.visibleSections()
                val text = if (enabled) section.label() else "+ ${section.label()}"
                if (enabled) {
                    if (section == selected) {
                        PrimaryPillButton(text, { onSelect(section) })
                    } else {
                        SoftPillButton(text, { onSelect(section) })
                    }
                } else {
                    SoftPillButton(text, { onToggle(section) })
                }
            }
        }
    }
}

@Composable
private fun MembersSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "成员信息", actionText = "邀请", onAction = viewModel::createInvitation) {
        detail.members.forEach { member ->
            SurfaceCard(Modifier.fillMaxWidth()) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text(member.profile.displayName.ifBlank { "未命名成员" }, fontWeight = FontWeight.ExtraBold)
                    if (member.isOwner) Badge("队长") else Badge("队员", tone = BadgeTone.Info)
                }
                Text(listOfNotNull(member.profile.roleLabel, member.profile.phone, member.profile.emergencyPhone).joinToString(" · ").ifBlank { "联系方式待补充" }, color = MaterialTheme.colorScheme.onSurfaceVariant)
                val summary = detail.weightSummaries.firstOrNull { it.memberId == member.id }
                if (summary != null) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        MetricTile("总重量", "${summary.allWeightG}g", Modifier.weight(1f))
                        MetricTile("实际背负", "${summary.actualWeightG}g", Modifier.weight(1f))
                    }
                }
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    SoftPillButton("标记已确认", { viewModel.updateMember(member.id, member.profile.displayName.ifBlank { "队员" }, member.fieldVersions) }, Modifier.weight(1f))
                    if (!member.isOwner) SoftPillButton("删除", { viewModel.removeMember(member.id) }, Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun PersonalGearSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "个人装备", actionText = "新增", onAction = { viewModel.addRecord(TripRecordKind.PersonalGear) }) {
        val myView = detail.memberGearViews.firstOrNull { it.memberId == detail.myMemberId }
        if (myView != null) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                MetricTile("所有装备", "${myView.allWeightG}g", Modifier.weight(1f))
                MetricTile("实际重量", "${myView.actualWeightG}g", Modifier.weight(1f))
            }
        }
        detail.personalGear.forEach { item ->
            RecordCard(
                title = item.name,
                meta = "${item.categoryLabel} · ${item.plannedQuantity} 件 · 已打包 ${item.packedQuantity}",
                onEdit = { viewModel.updateRecord(TripRecordKind.PersonalGear, item.id, item.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.PersonalGear, item.id) },
            )
        }
    }
}

@Composable
private fun SharedGearSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "公共装备", actionText = "新增需求", onAction = { viewModel.addRecord(TripRecordKind.SharedGear) }) {
        detail.sharedGearDemandTemplates.takeIf { it.isNotEmpty() }?.let {
            Text("模板需求：${it.joinToString("、") { template -> template.demandName }}", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        detail.sharedGearDemands.forEach { item ->
            RecordCard(
                title = item.slotName ?: item.demandName ?: item.name,
                meta = "${item.categoryLabel} · 需求 ${item.plannedQuantity} · ${item.concreteName ?: "待填写具体装备"}",
                onEdit = { viewModel.updateRecord(TripRecordKind.SharedGear, item.id, item.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.SharedGear, item.id) },
            )
        }
    }
}

@Composable
private fun ItinerarySection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "行程安排", actionText = "新增一天", onAction = { viewModel.addRecord(TripRecordKind.ItineraryDay) }) {
        SoftPillButton("添加路段", { viewModel.addRecord(TripRecordKind.RouteSegment) }, Modifier.fillMaxWidth())
        detail.itineraryDays.forEach { day ->
            RecordCard(
                title = day.title ?: "第 ${day.dayIndex} 天",
                meta = "${day.dateLabel ?: "日期待定"} · 预计 ${day.estimateMinutes} 分钟 · ${day.timeSlots.size} 个时段",
                onEdit = { viewModel.updateRecord(TripRecordKind.ItineraryDay, day.id, day.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.ItineraryDay, day.id) },
            )
        }
        detail.routeSegments.forEach { segment ->
            RecordCard(
                title = segment.name,
                meta = "${segment.distanceKm} km · 爬升 ${segment.ascentM}m · 预计 ${segment.finalEstimateMinutes} 分钟",
                onEdit = { viewModel.updateRecord(TripRecordKind.RouteSegment, segment.id, segment.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.RouteSegment, segment.id) },
            )
        }
    }
}

@Composable
private fun FoodSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "食品计划", actionText = "新增餐次", onAction = { viewModel.addRecord(TripRecordKind.FoodMeal) }) {
        SoftPillButton("添加公共食材", { viewModel.addRecord(TripRecordKind.FoodSupply) }, Modifier.fillMaxWidth())
        detail.foodMeals.forEach { meal ->
            RecordCard(
                title = meal.dishName ?: meal.mealKey,
                meta = "${meal.items.size} 个食材 · ${if (meal.skipped) "跳过" else "需要准备"}",
                onEdit = { viewModel.updateRecord(TripRecordKind.FoodMeal, meal.id, meal.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.FoodMeal, meal.id) },
            )
        }
        detail.foodSupplies.forEach { supply ->
            RecordCard(
                title = supply.name,
                meta = "${supply.amountG ?: 0}g · ${supply.supplyType ?: "公共食材"}",
                onEdit = { viewModel.updateRecord(TripRecordKind.FoodSupply, supply.id, supply.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.FoodSupply, supply.id) },
            )
        }
    }
}

@Composable
private fun MedicalSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "医药包", actionText = "新增", onAction = { viewModel.addRecord(TripRecordKind.MedicalItem) }) {
        detail.medicalItems.forEach { item ->
            RecordCard(
                title = item.name,
                meta = "${item.itemType ?: "医药"} · 需要 ${item.requiredQuantity} · 已带 ${item.packedQuantity}",
                onEdit = { viewModel.updateRecord(TripRecordKind.MedicalItem, item.id, item.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.MedicalItem, item.id) },
            )
        }
    }
}

@Composable
private fun SafetySection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "安全预案", actionText = "新增风险", onAction = { viewModel.addRecord(TripRecordKind.SafetyRisk) }) {
        SoftPillButton("添加分段分工", { viewModel.addRecord(TripRecordKind.SegmentAssignment) }, Modifier.fillMaxWidth())
        detail.safetyRisks.forEach { risk ->
            RecordCard(
                title = risk.riskType,
                meta = risk.prevention ?: risk.response ?: "预案待补充",
                onEdit = { viewModel.updateRecord(TripRecordKind.SafetyRisk, risk.id, risk.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.SafetyRisk, risk.id) },
            )
        }
        detail.segmentAssignments.forEach { assignment ->
            RecordCard(
                title = assignment.checkpoint ?: "分段分工",
                meta = assignment.notes ?: "领队、导航、安全、收队等角色待确认",
                onEdit = { viewModel.updateRecord(TripRecordKind.SegmentAssignment, assignment.id, assignment.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.SegmentAssignment, assignment.id) },
            )
        }
    }
}

@Composable
private fun RescueSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "救援信息", actionText = "新增", onAction = { viewModel.addRecord(TripRecordKind.RescueContact) }) {
        detail.rescueContacts.forEach { contact ->
            RecordCard(
                title = contact.organization,
                meta = listOfNotNull(contact.phone, contact.address).joinToString(" · ").ifBlank { "联系方式待补充" },
                onEdit = { viewModel.updateRecord(TripRecordKind.RescueContact, contact.id, contact.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.RescueContact, contact.id) },
            )
        }
    }
}

@Composable
private fun BudgetSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "财务预算", actionText = "新增", onAction = { viewModel.addRecord(TripRecordKind.BudgetItem) }) {
        detail.budgetItems.forEach { item: TripBudgetItem ->
            RecordCard(
                title = item.name,
                meta = "${item.category ?: "费用"} · 数量 ${item.quantity} · 总价 ${item.totalPriceCents?.let { "¥${it / 100.0}" } ?: "待定"}",
                onEdit = { viewModel.updateRecord(TripRecordKind.BudgetItem, item.id, item.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.BudgetItem, item.id) },
            )
        }
    }
}

@Composable
private fun GoalsSection(detail: TripDetail, viewModel: TripDetailViewModel) {
    SectionWorkbench(title = "目标", actionText = "新增", onAction = { viewModel.addRecord(TripRecordKind.Goal) }) {
        detail.goals.forEach { goal: TripGoalItem ->
            RecordCard(
                title = goal.content,
                meta = "${goal.scope} · ${goal.notes ?: "备注待补充"}",
                onEdit = { viewModel.updateRecord(TripRecordKind.Goal, goal.id, goal.fieldVersions) },
                onDelete = { viewModel.deleteRecord(TripRecordKind.Goal, goal.id) },
            )
        }
    }
}

@Composable
private fun SectionWorkbench(
    title: String,
    actionText: String,
    onAction: () -> Unit,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            SectionTitle(title)
            CompactPillAction(actionText, onAction)
        }
        content()
    }
}

@Composable
private fun RecordCard(title: String, meta: String, onEdit: () -> Unit, onDelete: () -> Unit) {
    SurfaceCard(Modifier.fillMaxWidth()) {
        Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
        Text(meta, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SoftPillButton("快速编辑", onEdit, Modifier.weight(1f))
            SoftPillButton("删除", onDelete, Modifier.weight(1f))
        }
    }
}

@Composable
fun PlaceholderParityScreen(title: String, body: String, onBack: () -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).padding(16.dp), contentAlignment = Alignment.Center) {
        SurfaceCard(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant)
            PrimaryPillButton("返回", onBack)
        }
    }
}

private data class TripGroup(val title: String, val subtitle: String, val trips: List<TripSummary>)

private fun tripGroups(trips: List<TripSummary>): List<TripGroup> = listOf(
    TripTimeBucket.ONGOING to ("进行中" to "正在路上的行程"),
    TripTimeBucket.UPCOMING to ("未来行程" to "准备中的出发计划"),
    TripTimeBucket.UNDATED to ("未定日期" to "还未设置时间"),
    TripTimeBucket.PAST to ("历史行程" to "已结束，可转为户外经历"),
).mapNotNull { (bucket, copy) ->
    val items = trips.filter { it.timeBucket == bucket }
    if (items.isEmpty()) null else TripGroup(copy.first, copy.second, items)
}

private fun readinessText(trip: TripSummary): String =
    if (trip.readiness.missingLabels.isEmpty()) {
        "准备度 ${trip.readiness.completionPercent}%"
    } else {
        "准备度 ${trip.readiness.completionPercent}% · 待补 ${trip.readiness.missingLabels.joinToString("、")}"
    }
