package com.rustella.stellartrail.ui.screens

import android.widget.Toast
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.BuildConfig
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.profile.categoryLabel
import com.rustella.stellartrail.domain.profile.statusLabel
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.domain.trip.label
import com.rustella.stellartrail.feature.profile.OutdoorExperienceForm
import com.rustella.stellartrail.feature.profile.OutdoorExperiencesViewModel
import com.rustella.stellartrail.feature.profile.OutdoorProfileForm
import com.rustella.stellartrail.feature.profile.OutdoorProfileViewModel
import com.rustella.stellartrail.feature.profile.ProfileSettingsViewModel
import com.rustella.stellartrail.feature.profile.RoadmapViewModel
import com.rustella.stellartrail.feature.profile.toForm
import com.rustella.stellartrail.ui.common.AvatarImage
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroButton
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.HeroSoftButton
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.LoginPromptSheet
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun RoadmapScreen(
    viewModel: RoadmapViewModel,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LoginPromptSheet(
        visible = state.loginPromptMessage != null,
        message = state.loginPromptMessage.orEmpty(),
        onDismiss = viewModel::dismissLoginPrompt,
        onLogin = onLogin,
    )
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "寻径星野",
                title = "产品路线图",
                subtitle = "这里记录功能计划。你可以给关心的方向投票，也可以订阅站内进度。",
                chips = listOf("投票", "订阅"),
                actions = {
                    HeroSoftButton("返回", onBack, Modifier.weight(1f))
                    HeroButton("刷新", viewModel::load, Modifier.weight(1f))
                },
            )
        }
        item {
            SurfaceCard {
                SectionTitle("状态")
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(RoadmapStatusFilter.entries) { filter ->
                        FilterChip(
                            selected = state.selectedStatus == filter,
                            onClick = { viewModel.selectStatus(filter) },
                            label = { Text(filter.label, fontWeight = FontWeight.Bold) },
                        )
                    }
                }
            }
        }
        if (state.loading) item { LoadingState() }
        state.error?.let { item { ErrorState(it, onRetry = viewModel::load) } }
        if (!state.loading && state.error == null && state.items.isEmpty()) {
            item { EmptyState("暂时没有这个状态的路线图条目。", "") }
        }
        items(state.items, key = { it.id }) { item ->
            RoadmapItemCard(
                item = item,
                loading = state.actionLoadingId == item.id,
                onVote = { viewModel.toggleVote(item) },
                onSubscribe = { viewModel.toggleSubscription(item) },
            )
        }
    }
}

@Composable
private fun RoadmapItemCard(
    item: RoadmapItem,
    loading: Boolean,
    onVote: () -> Unit,
    onSubscribe: () -> Unit,
) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.weight(1f)) {
                Badge(item.categoryLabel(), tone = BadgeTone.Info)
                Badge(item.statusLabel(), tone = roadmapStatusTone(item.status))
            }
            Badge("P${item.priority}", tone = BadgeTone.Neutral)
        }
        Text(item.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
        Text(item.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (!item.details.isNullOrBlank()) {
            Text(item.details, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            PrimaryPillButton(
                text = "${if (item.isVoted) "已投票" else "投票"} · ${item.voteCount} 票",
                onClick = onVote,
                modifier = Modifier.weight(1f),
                enabled = !loading,
            )
            SoftPillButton(
                text = if (item.isSubscribed) "已订阅" else "订阅",
                onClick = onSubscribe,
                modifier = Modifier.weight(1f),
                enabled = !loading,
            )
        }
    }
}

@Composable
fun OutdoorProfileScreen(
    viewModel: OutdoorProfileViewModel,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "我的默认资料",
                title = "户外资料",
                subtitle = "可在组队计划书中一键导入到自己的成员信息。",
                chips = listOf("紧急联系人", "健康信息"),
                actions = {
                    HeroSoftButton("返回", onBack, Modifier.weight(1f))
                    HeroButton("保存", viewModel::save, Modifier.weight(1f))
                },
            )
        }
        if (!state.loggedIn) {
            item {
                SurfaceCard {
                    SectionTitle("登录后维护户外资料", "资料会保存到你的账号中，方便出行计划复用。")
                    PrimaryPillButton("登录 / 注册", onLogin, Modifier.fillMaxWidth())
                }
            }
            return@LazyColumn
        }
        if (state.loading) item { LoadingState() }
        state.error?.let { item { ErrorState(it, onRetry = viewModel::load) } }
        state.notice?.let { item { SurfaceCard { Badge(it, tone = BadgeTone.Success) } } }
        item {
            OutdoorProfileFormCard(
                form = state.form,
                saving = state.saving,
                onChange = viewModel::updateForm,
                onSave = viewModel::save,
            )
        }
    }
}

@Composable
private fun OutdoorProfileFormCard(
    form: OutdoorProfileForm,
    saving: Boolean,
    onChange: (OutdoorProfileForm) -> Unit,
    onSave: () -> Unit,
) {
    SurfaceCard {
        FormTextField("户外 ID", form.outdoorId, { onChange(form.copy(outdoorId = it)) }, "例如：星星")
        FormTextField("姓名", form.realName, { onChange(form.copy(realName = it)) }, "真实姓名")
        ChoiceRow(
            label = "性别",
            options = listOf("", "男", "女", "其他"),
            selected = form.gender,
            onSelect = { onChange(form.copy(gender = it)) },
            emptyLabel = "未填写",
        )
        FormTextField("出生日期", form.birthDate, { onChange(form.copy(birthDate = it)) }, "YYYY-MM-DD")
        FormTextField(
            "身高 cm",
            form.heightCm,
            { onChange(form.copy(heightCm = it)) },
            "例如：176",
            keyboardType = KeyboardType.Number,
        )
        ChoiceRow(
            label = "血型",
            options = listOf("", "A", "B", "AB", "O"),
            selected = form.bloodType,
            onSelect = { onChange(form.copy(bloodType = it)) },
            emptyLabel = "未填写",
        )
        FormTextField("联系电话", form.phone, { onChange(form.copy(phone = it)) }, "手机号", keyboardType = KeyboardType.Phone)
        FormTextField("紧急联系人", form.emergencyContact, { onChange(form.copy(emergencyContact = it)) }, "姓名")
        FormTextField(
            "紧急联系人关系",
            form.emergencyContactRelationship,
            { onChange(form.copy(emergencyContactRelationship = it)) },
            "例如：家属 / 朋友",
        )
        FormTextField(
            "紧急联系人电话",
            form.emergencyPhone,
            { onChange(form.copy(emergencyPhone = it)) },
            "联系电话",
            keyboardType = KeyboardType.Phone,
        )
        FormTextField("既往病", form.medicalHistory, { onChange(form.copy(medicalHistory = it)) }, "无或简要说明", singleLine = false)
        FormTextField("过敏史", form.allergyHistory, { onChange(form.copy(allergyHistory = it)) }, "无或简要说明", singleLine = false)
        FormTextField(
            "过敏 / 伤病处理方法",
            form.medicalResponseNote,
            { onChange(form.copy(medicalResponseNote = it)) },
            "例如：随身药、禁忌或处置方式",
            singleLine = false,
        )
        FormTextField("饮食习惯", form.dietPreference, { onChange(form.copy(dietPreference = it)) }, "例如：清真 / 素食 / 不吃牛羊肉")
        FormTextField("保险单号", form.insurancePolicyNo, { onChange(form.copy(insurancePolicyNo = it)) }, "户外保险单号")
        FormTextField("保险公司电话", form.insuranceCompanyPhone, { onChange(form.copy(insuranceCompanyPhone = it)) }, "保险报案电话")
        PrimaryPillButton("保存户外资料", onSave, Modifier.fillMaxWidth(), enabled = !saving)
    }
}

@Composable
fun OutdoorExperiencesScreen(
    viewModel: OutdoorExperiencesViewModel,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var editorVisible by remember { mutableStateOf(false) }
    var editorForm by remember { mutableStateOf(OutdoorExperienceForm()) }
    var editingId by remember { mutableStateOf<String?>(null) }
    var deleting by remember { mutableStateOf<OutdoorExperience?>(null) }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            HeroCard(
                eyebrow = "我的账号资料",
                title = "户外经历",
                subtitle = "记录完成过的路线、同行和准备经验。",
                chips = listOf("历史行程", "手动记录"),
                actions = {
                    HeroSoftButton("返回", onBack, Modifier.weight(1f))
                    HeroButton(
                        "新增经历",
                        {
                            editingId = null
                            editorForm = OutdoorExperienceForm()
                            editorVisible = true
                        },
                        Modifier.weight(1f),
                    )
                },
            )
        }
        if (!state.loggedIn) {
            item {
                SurfaceCard {
                    SectionTitle("登录后维护户外经历", "历史行程和手动记录会保存到你的账号中。")
                    PrimaryPillButton("登录 / 注册", onLogin, Modifier.fillMaxWidth())
                }
            }
            return@LazyColumn
        }
        state.error?.let { item { ErrorState(it, onRetry = viewModel::load) } }
        if (state.loading) item { LoadingState() }
        if (!state.loading && state.items.isEmpty()) {
            item {
                EmptyState("还没有户外经历", "可以从历史行程转入，也可以手动记录一次完成的路线。")
            }
        }
        items(state.items, key = { it.id }) { item ->
            OutdoorExperienceCard(
                item = item,
                onEdit = {
                    editingId = item.id
                    editorForm = item.toForm()
                    editorVisible = true
                },
                onDelete = { deleting = item },
            )
        }
    }
    if (editorVisible) {
        OutdoorExperienceEditorSheet(
            title = if (editingId == null) "新增经历" else "编辑经历",
            form = editorForm,
            saving = state.saving,
            onFormChange = { editorForm = it },
            onDismiss = { editorVisible = false },
            onSave = { viewModel.save(editingId, editorForm) { editorVisible = false } },
        )
    }
    deleting?.let { item ->
        AlertDialog(
            onDismissRequest = { deleting = null },
            title = { Text("删除经历") },
            text = { Text("确定删除「${item.title}」吗？") },
            confirmButton = {
                TextButton(onClick = {
                    viewModel.delete(item.id)
                    deleting = null
                }) { Text("删除") }
            },
            dismissButton = { TextButton(onClick = { deleting = null }) { Text("取消") } },
        )
    }
}

@Composable
private fun OutdoorExperienceCard(
    item: OutdoorExperience,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(item.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                Text(item.dateText(), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
                Text(item.metaText(), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
            }
            Badge(if (item.sourceTripId == null) "手动记录" else "历史行程", tone = BadgeTone.Neutral)
        }
        item.summaryLines().forEach { (label, value) ->
            MetadataRow(label, value)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            SoftPillButton("编辑", onEdit, Modifier.weight(1f))
            SoftPillButton("删除", onDelete, Modifier.weight(1f))
        }
    }
}

@Composable
fun ProfileSettingsScreen(
    viewModel: ProfileSettingsViewModel,
    onBack: () -> Unit,
    onLogin: () -> Unit,
    onOpenOutdoorProfile: () -> Unit,
    onOpenOutdoorExperiences: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val session by viewModel.session.collectAsStateWithLifecycle()
    val theme by viewModel.theme.collectAsStateWithLifecycle()
    val config by viewModel.config.collectAsStateWithLifecycle()
    val actionState by viewModel.actionState.collectAsStateWithLifecycle()
    var baseUrl by remember(config.baseUrl) { mutableStateOf(config.baseUrl) }
    var nicknameSheet by remember { mutableStateOf(false) }
    var emailSheet by remember { mutableStateOf(false) }
    var phoneSheet by remember { mutableStateOf(false) }
    var passwordSheet by remember { mutableStateOf(false) }
    var debugExpanded by remember { mutableStateOf(false) }
    val user = session?.user
    val palette = currentTrailPalette()
    val context = LocalContext.current
    LaunchedEffect(user?.id) {
        viewModel.refreshCurrentProfile()
    }
    LaunchedEffect(actionState.phoneBindingCompleted, actionState.phoneNotice, phoneSheet) {
        if (phoneSheet && actionState.phoneBindingCompleted) {
            Toast.makeText(context, actionState.phoneNotice ?: "绑定完成", Toast.LENGTH_SHORT).show()
            phoneSheet = false
            viewModel.consumePhoneBindingCompletion()
        }
    }
    LazyColumn(
        modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentPadding = PaddingValues(16.dp, 16.dp, 16.dp, 28.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item { SettingsHero(user) }
        if (session == null) {
            item {
                SurfaceCard {
                    SectionTitle("还没有登录", "登录后才能管理账号资料。")
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        SoftPillButton("返回", onBack, Modifier.weight(1f))
                        PrimaryPillButton("登录 / 注册", onLogin, Modifier.weight(1f))
                    }
                }
            }
            return@LazyColumn
        }
        item {
            SurfaceCard {
                SectionTitle("账号资料")
                SettingsRow("名", "修改名称", user.displayName(), { nicknameSheet = true })
                SettingsRow("邮", if (user?.email.isNullOrBlank()) "绑定邮箱" else "修改邮箱", user?.email ?: "未绑定", { emailSheet = true })
                SettingsRow("手", if (user?.phone.isNullOrBlank()) "绑定手机号" else "修改手机号", user?.phone?.maskedPhone() ?: "未绑定", { phoneSheet = true })
                SettingsRow(
                    "密",
                    "修改密码",
                    user.passwordResetDescription(),
                    { passwordSheet = true },
                )
                SettingsRow("户", "户外资料", "维护身高、血型、紧急联系人和饮食习惯。", onOpenOutdoorProfile)
                SettingsRow("历", "户外经历", "记录历史行程和手动补充的户外经历。", onOpenOutdoorExperiences)
                actionState.accountError?.let { Text(it, color = currentTrailPalette().dangerText) }
            }
        }
        item {
            SurfaceCard {
                SectionTitle("黑夜模式")
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(theme.label(), fontWeight = FontWeight.ExtraBold)
                        Text(
                            ProfileVisualContract.nightModeDescription(theme),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                    Switch(
                        checked = theme == ThemeMode.DARK,
                        onCheckedChange = { checked -> viewModel.setTheme(if (checked) ThemeMode.DARK else ThemeMode.LIGHT) },
                        colors = SwitchDefaults.colors(
                            checkedThumbColor = palette.brandText,
                            checkedTrackColor = palette.brand,
                            uncheckedThumbColor = MaterialTheme.colorScheme.surface,
                            uncheckedTrackColor = palette.border,
                            uncheckedBorderColor = palette.border,
                        ),
                    )
                }
            }
        }
        if (viewModel.canEditBaseUrl) {
            item {
                SurfaceCard {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            SectionTitle("连接设置")
                            Text(
                                ProfileVisualContract.debugEndpointSummary(config.baseUrl, BuildConfig.DEFAULT_API_BASE_URL),
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = MaterialTheme.typography.bodySmall,
                            )
                        }
                        CompactPillAction(
                            text = if (debugExpanded) "收起" else "展开",
                            onClick = { debugExpanded = !debugExpanded },
                            filled = false,
                        )
                    }
                    if (debugExpanded) {
                        OutlinedTextField(
                            value = baseUrl,
                            onValueChange = { baseUrl = it },
                            label = { Text("连接地址") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            PrimaryPillButton("保存", { viewModel.updateBaseUrl(baseUrl) }, Modifier.weight(1f))
                            SoftPillButton("恢复默认", viewModel::resetBaseUrl, Modifier.weight(1f))
                        }
                    }
                }
            }
        }
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                SoftPillButton("返回", onBack, Modifier.weight(1f))
                SoftPillButton("退出登录", viewModel::logout, Modifier.weight(1f))
            }
        }
    }
    if (nicknameSheet) {
        NicknameSheet(user = user, onDismiss = { nicknameSheet = false })
    }
    if (emailSheet) {
        EmailBindingSheet(
            currentEmail = user?.email,
            actionState = actionState,
            onDismiss = {
                viewModel.clearMessages()
                emailSheet = false
            },
            onSendCode = viewModel::sendBindEmailCode,
            onSubmit = viewModel::bindEmail,
        )
    }
    if (phoneSheet) {
        PhoneBindingSheet(
            currentPhone = user?.phone,
            actionState = actionState,
            onDismiss = {
                viewModel.clearMessages()
                phoneSheet = false
            },
            onSendNewCode = viewModel::sendBindPhoneCode,
            onSendCurrentCode = viewModel::sendRebindCurrentPhoneCode,
            onSubmit = viewModel::bindPhone,
        )
    }
    if (passwordSheet) {
        PasswordSheet(
            email = user?.email.orEmpty(),
            phone = user?.phone.orEmpty(),
            actionState = actionState,
            onDismiss = {
                viewModel.clearMessages()
                passwordSheet = false
            },
            onSendEmailCode = viewModel::sendPasswordResetCode,
            onSendSmsCode = viewModel::sendSmsPasswordResetCode,
            onSubmitEmail = viewModel::resetPassword,
            onSubmitPhone = viewModel::resetPasswordByPhone,
        )
    }
}

@Composable
private fun SettingsHero(user: LoginUser?) {
    SurfaceCard {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            AvatarImage(
                avatarUrl = user?.avatarUrl,
                fallbackText = user.avatarInitial(),
                modifier = Modifier.size(58.dp),
                backgroundColor = currentTrailPalette().brandSoft,
                contentColor = currentTrailPalette().brand,
                textStyle = MaterialTheme.typography.titleLarge,
            )
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(user.displayName(), style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                Text(user.contactSummary(), color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
private fun SettingsRow(icon: String, title: String, desc: String, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.medium)
            .clickable(onClick = onClick)
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier.size(34.dp).clip(MaterialTheme.shapes.medium).background(palette.brandSoft),
            contentAlignment = Alignment.Center,
        ) {
            Text(icon, fontWeight = FontWeight.ExtraBold, color = palette.brandSoftText)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(title, fontWeight = FontWeight.ExtraBold)
            Text(desc, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        }
        Text(">", color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun FormTextField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    singleLine: Boolean = true,
    keyboardType: KeyboardType = KeyboardType.Text,
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        label = { Text(label) },
        placeholder = { Text(placeholder) },
        singleLine = singleLine,
        minLines = if (singleLine) 1 else 3,
        keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
        modifier = modifier.fillMaxWidth(),
    )
}

@Composable
private fun ChoiceRow(
    label: String,
    options: List<String>,
    selected: String,
    onSelect: (String) -> Unit,
    emptyLabel: String,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(label, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold)
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(options) { value ->
                FilterChip(
                    selected = selected == value,
                    onClick = { onSelect(value) },
                    label = { Text(value.ifBlank { emptyLabel }) },
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun OutdoorExperienceEditorSheet(
    title: String,
    form: OutdoorExperienceForm,
    saving: Boolean,
    onFormChange: (OutdoorExperienceForm) -> Unit,
    onDismiss: () -> Unit,
    onSave: () -> Unit,
) {
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = currentTrailPalette().surface) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 640.dp)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 20.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            Text("经历会展示在账号资料中，可从历史行程转入。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            SectionTitle("基础信息")
            FormTextField("标题", form.title, { onFormChange(form.copy(title = it)) }, "经历标题，例如：三天两夜罗浮山重装")
            FormTextField("开始日期", form.startDate, { onFormChange(form.copy(startDate = it)) }, "YYYY-MM-DD")
            FormTextField("结束日期", form.endDate, { onFormChange(form.copy(endDate = it)) }, "YYYY-MM-DD")
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                FormTextField(
                    "天数",
                    form.dayCount,
                    { onFormChange(form.copy(dayCount = it)) },
                    "例如：3",
                    Modifier.weight(1f),
                    keyboardType = KeyboardType.Number,
                )
                FormTextField(
                    "同行人数",
                    form.companionCount,
                    { onFormChange(form.copy(companionCount = it)) },
                    "不含自己",
                    Modifier.weight(1f),
                    keyboardType = KeyboardType.Number,
                )
            }
            SectionTitle("经历摘要")
            FormTextField("路线摘要", form.routeSummary, { onFormChange(form.copy(routeSummary = it)) }, "路线摘要，例如：罗浮山环线", singleLine = false)
            FormTextField("装备经验", form.gearSummary, { onFormChange(form.copy(gearSummary = it)) }, "装备经验，例如：轻量雨衣够用", singleLine = false)
            FormTextField("食品经验", form.foodSummary, { onFormChange(form.copy(foodSummary = it)) }, "食品经验，例如：早餐偏少", singleLine = false)
            FormTextField("预算摘要", form.budgetSummary, { onFormChange(form.copy(budgetSummary = it)) }, "预算摘要，例如：包车 300", singleLine = false)
            FormTextField("其他备注", form.notes, { onFormChange(form.copy(notes = it)) }, "其他备注", singleLine = false)
            PrimaryPillButton("保存经历", onSave, Modifier.fillMaxWidth(), enabled = !saving)
            Spacer(Modifier.height(8.dp))
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun NicknameSheet(user: LoginUser?, onDismiss: () -> Unit) {
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = currentTrailPalette().surface) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("修改名称", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            MetadataRow("当前名称", user.displayName())
            Text("修改后会同步到账号资料。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            PrimaryPillButton("知道了", onDismiss, Modifier.fillMaxWidth())
            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
private fun VerificationCodeInputRow(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    onSendCode: () -> Unit,
    enabled: Boolean,
    keyboardType: KeyboardType = KeyboardType.Number,
) {
    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        FormTextField(label, value, onValueChange, "验证码", Modifier.weight(1f), keyboardType = keyboardType)
        CompactPillAction(
            "获取验证码",
            onSendCode,
            modifier = Modifier.heightIn(min = 44.dp),
            enabled = enabled,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EmailBindingSheet(
    currentEmail: String?,
    actionState: com.rustella.stellartrail.feature.profile.ProfileSettingsActionState,
    onDismiss: () -> Unit,
    onSendCode: (String) -> Unit,
    onSubmit: (String, String) -> Unit,
) {
    var email by remember(currentEmail) { mutableStateOf(currentEmail.orEmpty()) }
    var code by remember { mutableStateOf("") }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = currentTrailPalette().surface) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(if (currentEmail.isNullOrBlank()) "绑定邮箱" else "修改邮箱", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            Text(if (currentEmail.isNullOrBlank()) "绑定后可用邮箱登录，也可以找回密码。" else "输入新的邮箱地址，并用验证码确认。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            FormTextField("邮箱", email, { email = it }, "邮箱", keyboardType = KeyboardType.Email)
            VerificationCodeInputRow(
                label = "邮箱验证码",
                value = code,
                onValueChange = { code = it },
                onSendCode = { onSendCode(email) },
                enabled = !actionState.emailCodeLoading,
            )
            actionState.emailNotice?.let { Text(it, color = currentTrailPalette().successText) }
            actionState.accountError?.let { Text(it, color = currentTrailPalette().dangerText) }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                PrimaryPillButton("确定", { onSubmit(email, code) }, Modifier.weight(1f), enabled = !actionState.emailBindingLoading)
                SoftPillButton("取消", onDismiss, Modifier.weight(1f))
            }
            Spacer(Modifier.height(8.dp))
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun PhoneBindingSheet(
    currentPhone: String?,
    actionState: com.rustella.stellartrail.feature.profile.ProfileSettingsActionState,
    onDismiss: () -> Unit,
    onSendNewCode: (String) -> Unit,
    onSendCurrentCode: () -> Unit,
    onSubmit: (String, String, String?) -> Unit,
) {
    val hasCurrentPhone = !currentPhone.isNullOrBlank()
    var phone by remember(currentPhone) { mutableStateOf("") }
    var code by remember { mutableStateOf("") }
    var currentCode by remember { mutableStateOf("") }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = currentTrailPalette().surface) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(if (hasCurrentPhone) "修改手机号" else "绑定手机号", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            Text(if (hasCurrentPhone) "先验证当前手机号，再确认新的手机号。" else "绑定后可用手机号登录，也可以找回密码。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            if (hasCurrentPhone) {
                MetadataRow("当前手机号", currentPhone.orEmpty().maskedPhone())
                VerificationCodeInputRow(
                    label = "当前手机号验证码",
                    value = currentCode,
                    onValueChange = { currentCode = it },
                    onSendCode = onSendCurrentCode,
                    enabled = !actionState.currentPhoneCodeLoading,
                )
            }
            FormTextField("新手机号", phone, { phone = it }, "手机号", keyboardType = KeyboardType.Phone)
            VerificationCodeInputRow(
                label = "短信验证码",
                value = code,
                onValueChange = { code = it },
                onSendCode = { onSendNewCode(phone) },
                enabled = !actionState.phoneCodeLoading,
            )
            actionState.phoneNotice?.let { Text(it, color = currentTrailPalette().successText) }
            actionState.accountError?.let { Text(it, color = currentTrailPalette().dangerText) }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                PrimaryPillButton(
                    "确定",
                    { onSubmit(phone, code, if (hasCurrentPhone) currentCode else null) },
                    Modifier.weight(1f),
                    enabled = !actionState.phoneBindingLoading,
                )
                SoftPillButton("取消", onDismiss, Modifier.weight(1f))
            }
            Spacer(Modifier.height(8.dp))
        }
    }
}

private enum class PasswordVerificationMethod { EMAIL, PHONE }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun PasswordSheet(
    email: String,
    phone: String,
    actionState: com.rustella.stellartrail.feature.profile.ProfileSettingsActionState,
    onDismiss: () -> Unit,
    onSendEmailCode: (String) -> Unit,
    onSendSmsCode: (String) -> Unit,
    onSubmitEmail: (String, String, String, String) -> Unit,
    onSubmitPhone: (String, String, String, String) -> Unit,
) {
    val hasEmail = email.isNotBlank()
    val hasPhone = phone.isNotBlank()
    var method by remember(email, phone) {
        mutableStateOf(
            when {
                hasEmail -> PasswordVerificationMethod.EMAIL
                hasPhone -> PasswordVerificationMethod.PHONE
                else -> PasswordVerificationMethod.EMAIL
            },
        )
    }
    var code by remember(method) { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }
    val canUseSelectedMethod = when (method) {
        PasswordVerificationMethod.EMAIL -> hasEmail
        PasswordVerificationMethod.PHONE -> hasPhone
    }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = currentTrailPalette().surface) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("修改密码", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            if (!hasEmail && !hasPhone) {
                Text("需要先绑定邮箱或手机号。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                Text(
                    when (method) {
                        PasswordVerificationMethod.EMAIL -> "验证码会发送到 $email。"
                        PasswordVerificationMethod.PHONE -> "验证码会发送到 ${phone.maskedPhone()}。"
                    },
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            if (hasEmail && hasPhone) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterChip(
                        selected = method == PasswordVerificationMethod.EMAIL,
                        onClick = { method = PasswordVerificationMethod.EMAIL },
                        label = { Text("邮箱验证", fontWeight = FontWeight.Bold) },
                        modifier = Modifier.weight(1f),
                    )
                    FilterChip(
                        selected = method == PasswordVerificationMethod.PHONE,
                        onClick = { method = PasswordVerificationMethod.PHONE },
                        label = { Text("手机验证", fontWeight = FontWeight.Bold) },
                        modifier = Modifier.weight(1f),
                    )
                }
            }
            VerificationCodeInputRow(
                label = if (method == PasswordVerificationMethod.EMAIL) "邮箱验证码" else "短信验证码",
                value = code,
                onValueChange = { code = it },
                onSendCode = {
                    if (method == PasswordVerificationMethod.EMAIL) onSendEmailCode(email) else onSendSmsCode(phone)
                },
                enabled = canUseSelectedMethod && !actionState.passwordCodeLoading,
            )
            OutlinedTextField(
                value = password,
                onValueChange = { password = it },
                label = { Text("新密码") },
                placeholder = { Text("至少 8 位") },
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = confirmPassword,
                onValueChange = { confirmPassword = it },
                label = { Text("再次输入新密码") },
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
                modifier = Modifier.fillMaxWidth(),
            )
            actionState.passwordNotice?.let { Text(it, color = currentTrailPalette().successText) }
            actionState.accountError?.let { Text(it, color = currentTrailPalette().dangerText) }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                PrimaryPillButton(
                    "保存",
                    {
                        if (method == PasswordVerificationMethod.EMAIL) {
                            onSubmitEmail(email, code, password, confirmPassword)
                        } else {
                            onSubmitPhone(phone, code, password, confirmPassword)
                        }
                    },
                    Modifier.weight(1f),
                    enabled = canUseSelectedMethod && !actionState.passwordLoading,
                )
                SoftPillButton("取消", onDismiss, Modifier.weight(1f))
            }
            Spacer(Modifier.height(8.dp))
        }
    }
}

private fun LoginUser?.displayName(): String =
    this?.nickname?.takeIf { it.isNotBlank() } ?: this?.username?.takeIf { it.isNotBlank() } ?: "未登录"

private fun LoginUser?.contactSummary(): String =
    this?.email?.takeIf { it.isNotBlank() }
        ?: this?.phone?.takeIf { it.isNotBlank() }?.maskedPhone()
        ?: "未绑定联系方式"

private fun LoginUser?.passwordResetDescription(): String {
    val email = this?.email
    val phone = this?.phone
    return when {
        email.isNullOrBlank() && phone.isNullOrBlank() -> "需要先绑定邮箱或手机号"
        email.isNullOrBlank() -> "通过手机验证码更新密码"
        phone.isNullOrBlank() -> "通过邮箱验证码更新密码"
        else -> "通过邮箱或手机验证码更新密码"
    }
}

private fun String.maskedPhone(): String =
    if (length >= 7) "${take(3)}****${takeLast(4)}" else this

private fun LoginUser?.avatarInitial(): String = displayName().firstOrNull()?.toString() ?: "我"

private fun ThemeMode.label(): String = when (this) {
    ThemeMode.LIGHT -> "浅色"
    ThemeMode.DARK -> "深色"
    ThemeMode.SYSTEM -> "跟随系统"
}

private fun roadmapStatusTone(status: String): BadgeTone = when (status) {
    "building", "preview" -> BadgeTone.Warning
    "shipped" -> BadgeTone.Success
    "designing" -> BadgeTone.Info
    else -> BadgeTone.Neutral
}

private fun OutdoorExperience.dateText(): String = when {
    !startDate.isNullOrBlank() && !endDate.isNullOrBlank() -> "$startDate 至 $endDate"
    !startDate.isNullOrBlank() -> startDate
    !endDate.isNullOrBlank() -> endDate
    else -> "未设置日期"
}.orEmpty()

private fun OutdoorExperience.metaText(): String {
    val parts = listOfNotNull(
        dayCount?.takeIf { it > 0 }?.let { "${it}天" },
        companionCount?.let { "同行${it}人" },
        tripType.label(),
    )
    return parts.joinToString(" · ").ifBlank { "未填写同行信息" }
}

private fun OutdoorExperience.summaryLines(): List<Pair<String, String>> = listOfNotNull(
    routeSummary?.takeIf { it.isNotBlank() }?.let { "路线" to it },
    gearSummary?.takeIf { it.isNotBlank() }?.let { "装备" to it },
    foodSummary?.takeIf { it.isNotBlank() }?.let { "食品" to it },
    budgetSummary?.takeIf { it.isNotBlank() }?.let { "预算" to it },
    notes?.takeIf { it.isNotBlank() }?.let { "备注" to it },
)
