package com.rustella.stellartrail.ui.screens

import androidx.compose.animation.core.animateDpAsState
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
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.feature.profile.ProfileCacheKind
import com.rustella.stellartrail.feature.profile.ProfileCacheViewModel
import com.rustella.stellartrail.feature.profile.ProfileViewModel
import com.rustella.stellartrail.ui.common.AvatarImage
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel,
    onLogin: () -> Unit,
    onOpenCache: () -> Unit,
    onOpenAbout: () -> Unit,
    onOpenSettings: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val session by viewModel.session.collectAsStateWithLifecycle()
    val theme by viewModel.theme.collectAsStateWithLifecycle()
    var dialog by remember { mutableStateOf<ProfileHelpAction?>(null) }
    LaunchedEffect(session?.user?.id) {
        viewModel.refreshCurrentProfile()
    }
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
        viewModel.refreshCurrentProfile()
    }
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(start = 16.dp, top = 0.dp, end = 16.dp, bottom = 16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        ProfileAccountCard(
            session = session,
            onLogin = onLogin,
            onOpenSettings = onOpenSettings,
            onLogout = viewModel::logout,
        )
        SurfaceCard {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("设置与帮助", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                ProfileThemeSwitch(theme = theme, onThemeChange = viewModel::setTheme)
            }
            ProfileVisualContract.helpItems.forEach { item ->
                ProfileHelpRow(
                    item = item,
                    onClick = {
                        when (item.action) {
                            ProfileHelpAction.Cache -> onOpenCache()
                            ProfileHelpAction.AboutHub -> onOpenAbout()
                            ProfileHelpAction.Feedback -> dialog = item.action
                        }
                    },
                )
            }
        }
    }
    dialog?.let { action ->
        ProfileInfoDialog(action = action, onDismiss = { dialog = null })
    }
}

@Composable
fun ProfileCacheScreen(
    viewModel: ProfileCacheViewModel,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val busy = state.cachingSelected || state.deletingSelected || state.cachingKnots || state.clearingKnots || state.clearingVisitedData
    val palette = currentTrailPalette()
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        SurfaceCard {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.weight(1f)) {
                    Text(
                        ProfileVisualContract.cacheTitle,
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.ExtraBold,
                    )
                    Text(
                        ProfileVisualContract.cacheDescription,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
                ProfileBackAction(onBack)
            }
        }
        SurfaceCard {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(ProfileVisualContract.cacheSectionTitle, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                if (state.selectionMode) {
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                        CompactPillAction(
                            text = ProfileVisualContract.cacheSelectAllAction,
                            onClick = viewModel::selectAllCacheKinds,
                            filled = false,
                            enabled = !busy,
                        )
                        CompactPillAction(
                            text = ProfileVisualContract.cacheInvertSelectionAction,
                            onClick = viewModel::invertCacheSelection,
                            filled = false,
                            enabled = !busy,
                        )
                        CompactPillAction(
                            text = ProfileVisualContract.cacheDoneAction,
                            onClick = viewModel::exitSelectionMode,
                            filled = true,
                            enabled = !busy,
                        )
                    }
                } else {
                    Box(
                        modifier = Modifier.semantics {
                            contentDescription = ProfileVisualContract.cacheSelectAction
                        }
                            .size(48.dp)
                            .clip(CircleShape)
                            .background(palette.softControlBackground)
                            .clickable(enabled = !busy, onClick = viewModel::enterSelectionMode),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            Icons.Filled.Menu,
                            contentDescription = null,
                            modifier = Modifier.size(24.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            if (state.selectionMode) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    PrimaryPillButton(
                        text = if (state.cachingSelected) "缓存中..." else ProfileVisualContract.cacheSelectedAction,
                        onClick = viewModel::cacheSelectedCaches,
                        modifier = Modifier.weight(1f),
                        enabled = !busy && state.selectedCacheKinds.isNotEmpty(),
                    )
                    SoftPillButton(
                        text = if (state.deletingSelected) "删除中..." else ProfileVisualContract.deleteSelectedAction,
                        onClick = viewModel::deleteSelectedCaches,
                        modifier = Modifier.weight(1f),
                        enabled = !busy && state.selectedCacheKinds.isNotEmpty(),
                    )
                }
            }
            ProfileVisualContract.cacheItems.forEach { item ->
                val isKnotCache = item.kind == ProfileCacheKind.Knots
                ProfileCacheRow(
                    item = item,
                    status = if (isKnotCache) {
                        ProfileVisualContract.knotCacheStatusLabel(state.status.cachedKnotCount)
                    } else {
                        ProfileVisualContract.visitedDataCacheStatusLabel(state.offlineStatus.cachedResponseCount)
                    },
                    selectionMode = state.selectionMode,
                    selected = item.kind in state.selectedCacheKinds,
                    onToggleSelection = { viewModel.toggleCacheKind(item.kind) },
                    cacheActionText = when {
                        isKnotCache && state.cachingKnots -> "缓存中..."
                        isKnotCache -> ProfileVisualContract.cacheKnotsAction
                        else -> ProfileVisualContract.autoCacheAction
                    },
                    clearActionText = when {
                        isKnotCache && state.clearingKnots -> "清空中..."
                        !isKnotCache && state.clearingVisitedData -> "清空中..."
                        isKnotCache -> ProfileVisualContract.cacheClearKnotsAction
                        else -> ProfileVisualContract.cacheClearVisitedDataAction
                    },
                    onCache = if (isKnotCache) viewModel::cacheKnots else ({ }),
                    onClear = if (isKnotCache) viewModel::clearKnotCache else viewModel::clearVisitedDataCache,
                    cacheEnabled = !busy && isKnotCache,
                    clearEnabled = !busy && if (isKnotCache) state.status.cachedKnotCount > 0 else state.offlineStatus.cachedResponseCount > 0,
                )
            }
            state.message?.let { message ->
                Text(message, color = palette.accent, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.SemiBold)
            }
            state.error?.let { error ->
                Text(error, color = palette.dangerText, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

@Composable
fun ProfileAboutScreen(
    onBack: () -> Unit,
    onOpenRoadmap: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var dialog by remember { mutableStateOf<ProfileAboutAction?>(null) }
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        SurfaceCard {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp), modifier = Modifier.weight(1f)) {
                    Text(
                        ProfileVisualContract.aboutBrandEyebrow,
                        color = currentTrailPalette().brandSoftText,
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.ExtraBold,
                    )
                    Text(ProfileVisualContract.aboutBrandTitle, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
                }
                ProfileBackAction(onBack)
            }
            Text(
                ProfileVisualContract.aboutBrandDescription,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
            )
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                ProfileVisualContract.aboutIntroItems.forEach { item ->
                    ProfileAboutIntroRow(item)
                }
            }
        }
        SurfaceCard {
            Text(ProfileVisualContract.aboutTitle, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
            ProfileVisualContract.aboutItems.forEach { item ->
                ProfileAboutRow(
                    item = item,
                    onClick = {
                        when (item.action) {
                            ProfileAboutAction.Roadmap -> onOpenRoadmap()
                            ProfileAboutAction.VersionInfo -> dialog = item.action
                        }
                    },
                )
            }
        }
    }
    dialog?.let { action ->
        ProfileAboutInfoDialog(action = action, onDismiss = { dialog = null })
    }
}

@Composable
private fun ProfileBackAction(onBack: () -> Unit) {
    Row(
        modifier = Modifier
            .clip(RoundedCornerShape(999.dp))
            .clickable(onClick = onBack)
            .semantics { contentDescription = "返回" }
            .padding(horizontal = 6.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(3.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text("‹", color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
        Text("返回", color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun ProfileThemeSwitch(theme: ThemeMode, onThemeChange: (ThemeMode) -> Unit) {
    val palette = currentTrailPalette()
    val checked = theme == ThemeMode.DARK
    val thumbOffset by animateDpAsState(
        targetValue = if (checked) 42.dp else 4.dp,
        label = "profileThemeSwitchThumb",
    )
    Box(
        modifier = Modifier
            .width(76.dp)
            .height(38.dp)
            .clip(RoundedCornerShape(999.dp))
            .background(if (checked) palette.brandSoft else palette.softControlBackground)
            .clickable { onThemeChange(if (checked) ThemeMode.LIGHT else ThemeMode.DARK) }
            .semantics {
                contentDescription = ProfileVisualContract.nightModeTitle
                stateDescription = ProfileVisualContract.nightModeDescription(theme)
            },
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 11.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                ProfileVisualContract.themeLightIcon,
                color = if (checked) palette.textMuted else palette.heroSun,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.ExtraBold,
            )
            Text(
                ProfileVisualContract.themeDarkIcon,
                color = if (checked) palette.brandSoftText else palette.textMuted,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.ExtraBold,
            )
        }
        Box(
            modifier = Modifier
                .offset(x = thumbOffset, y = 4.dp)
                .size(30.dp)
                .clip(CircleShape)
                .background(if (checked) palette.brand else MaterialTheme.colorScheme.surface),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                if (checked) ProfileVisualContract.themeDarkIcon else ProfileVisualContract.themeLightIcon,
                color = if (checked) palette.brandText else palette.heroSun,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.ExtraBold,
            )
        }
    }
}

@Composable
private fun ProfileAccountCard(
    session: UserSession?,
    onLogin: () -> Unit,
    onOpenSettings: () -> Unit,
    onLogout: () -> Unit,
) {
    val user = session?.user
    SurfaceCard(contentPadding = PaddingValues(14.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            AvatarImage(
                avatarUrl = user?.avatarUrl,
                fallbackText = user.avatarInitial(),
                modifier = Modifier.size(48.dp),
                backgroundColor = currentTrailPalette().brand,
                contentColor = currentTrailPalette().brandText,
                textStyle = MaterialTheme.typography.titleMedium,
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .then(if (session != null) Modifier.clickable(onClick = onOpenSettings) else Modifier),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = if (session == null) "未登录" else user.displayName(),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.ExtraBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = if (session == null) "登录后同步装备、计划和个人资料" else ProfileVisualContract.accountSettingsEntryLabel,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            if (session == null) {
                CompactPillAction("登录 / 注册", onLogin)
            } else {
                Text(
                    ">",
                    modifier = Modifier.clickable(onClick = onOpenSettings),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.ExtraBold,
                )
                CompactPillAction("退出", onLogout, filled = false)
            }
        }
    }
}

@Composable
private fun ProfileHelpRow(item: ProfileHelpItem, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(TrailInnerCardShape)
            .background(palette.controlBackground)
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .clip(TrailInnerCardShape)
                .background(palette.brandSoft)
                .padding(horizontal = 8.dp, vertical = 5.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(item.icon, color = palette.brandSoftText, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.ExtraBold)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(item.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
            Text(
                item.description,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Text(">", color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.ExtraBold)
    }
}

@Composable
private fun ProfileCacheRow(
    item: ProfileCacheItem,
    status: String,
    selectionMode: Boolean,
    selected: Boolean,
    onToggleSelection: () -> Unit,
    cacheActionText: String,
    clearActionText: String,
    onCache: () -> Unit,
    onClear: () -> Unit,
    cacheEnabled: Boolean,
    clearEnabled: Boolean,
) {
    val palette = currentTrailPalette()
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(TrailInnerCardShape)
            .background(palette.controlBackground)
            .then(
                if (selectionMode) {
                    Modifier.clickable(onClick = onToggleSelection)
                } else {
                    Modifier
                },
            )
            .padding(horizontal = 10.dp, vertical = 10.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .clip(TrailInnerCardShape)
                    .background(palette.brandSoft)
                    .padding(horizontal = 8.dp, vertical = 5.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(item.icon, color = palette.brandSoftText, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.ExtraBold)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                Text(item.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
                Text(
                    item.description,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            if (selectionMode) {
                Checkbox(
                    checked = selected,
                    onCheckedChange = { onToggleSelection() },
                    modifier = Modifier.semantics {
                        contentDescription = "${item.title}选择状态"
                    },
                )
            } else {
                Text(
                    status,
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(palette.brandSoft)
                        .padding(horizontal = 9.dp, vertical = 5.dp),
                    color = palette.brandSoftText,
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.ExtraBold,
                )
            }
        }
        if (!selectionMode) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                PrimaryPillButton(
                    text = cacheActionText,
                    onClick = onCache,
                    modifier = Modifier.weight(1f),
                    enabled = cacheEnabled,
                )
                SoftPillButton(
                    text = clearActionText,
                    onClick = onClear,
                    modifier = Modifier.weight(1f),
                    enabled = clearEnabled,
                )
            }
        }
    }
}

@Composable
private fun ProfileAboutIntroRow(item: ProfileAboutIntroItem) {
    val palette = currentTrailPalette()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(TrailInnerCardShape)
            .background(palette.controlBackground)
            .padding(horizontal = 9.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.Top,
    ) {
        Box(
            modifier = Modifier
                .size(32.dp)
                .clip(TrailInnerCardShape)
                .background(palette.brandSoft),
            contentAlignment = Alignment.Center,
        ) {
            Text(item.icon, color = palette.brandSoftText, style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.ExtraBold)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(item.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
            Text(
                item.description,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall.copy(lineHeight = 18.sp),
            )
        }
    }
}

@Composable
private fun ProfileAboutRow(item: ProfileAboutItem, onClick: () -> Unit) {
    val palette = currentTrailPalette()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(TrailInnerCardShape)
            .background(palette.controlBackground)
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .clip(TrailInnerCardShape)
                .background(palette.brandSoft)
                .padding(horizontal = 8.dp, vertical = 5.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(item.icon, color = palette.brandSoftText, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.ExtraBold)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(item.title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.ExtraBold)
            Text(
                item.description,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Text(">", color = MaterialTheme.colorScheme.onSurfaceVariant, fontWeight = FontWeight.ExtraBold)
    }
}

@Composable
private fun ProfileInfoDialog(action: ProfileHelpAction, onDismiss: () -> Unit) {
    val (title, body) = ProfileVisualContract.helpDialog(action)
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title, fontWeight = FontWeight.ExtraBold) },
        text = { Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant) },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("知道了")
            }
        },
    )
}

@Composable
private fun ProfileAboutInfoDialog(action: ProfileAboutAction, onDismiss: () -> Unit) {
    val (title, body) = ProfileVisualContract.aboutDialog(action)
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title, fontWeight = FontWeight.ExtraBold) },
        text = { Text(body, color = MaterialTheme.colorScheme.onSurfaceVariant) },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("知道了")
            }
        },
    )
}

private fun LoginUser?.displayName(): String =
    this?.nickname?.takeIf { it.isNotBlank() } ?: this?.username?.takeIf { it.isNotBlank() } ?: "已登录用户"

private fun LoginUser?.avatarInitial(): String =
    this?.displayName()?.firstOrNull()?.toString() ?: "我"
