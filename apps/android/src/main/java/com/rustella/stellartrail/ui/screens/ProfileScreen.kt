package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.feature.profile.ProfileViewModel
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailInnerCardShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel,
    onLogin: () -> Unit,
    onOpenRoadmap: () -> Unit,
    onOpenOutdoorProfile: () -> Unit,
    onOpenOutdoorExperiences: () -> Unit,
    onOpenSettings: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val session by viewModel.session.collectAsStateWithLifecycle()
    val theme by viewModel.theme.collectAsStateWithLifecycle()
    var dialog by remember { mutableStateOf<ProfileHelpAction?>(null) }
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
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
                Switch(
                    checked = theme == ThemeMode.DARK,
                    onCheckedChange = { checked -> viewModel.setTheme(if (checked) ThemeMode.DARK else ThemeMode.LIGHT) },
                )
            }
            ProfileVisualContract.helpItems.forEach { item ->
                ProfileHelpRow(
                    item = item,
                    onClick = {
                        when (item.action) {
                            ProfileHelpAction.Roadmap -> onOpenRoadmap()
                            else -> dialog = item.action
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
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(currentTrailPalette().brand),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = user.avatarInitial(),
                    color = currentTrailPalette().brandText,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.ExtraBold,
                )
            }
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
private fun ProfileInfoDialog(action: ProfileHelpAction, onDismiss: () -> Unit) {
    val (title, body) = when (action) {
        ProfileHelpAction.CachedKnots -> "绳结离线缓存" to "Android 端会沿用小程序的离线缓存入口；当前可先在技能页查看在线绳结内容。"
        ProfileHelpAction.Feedback -> "意见反馈" to "反馈入口会继续对齐小程序样式；当前 Android 端先保留轻量提示，不在一级页放置复杂表单。"
        ProfileHelpAction.VersionInfo -> "版本信息" to "当前版本信息会随 Android 包更新展示。后续版本记录将继续对齐小程序弹层。"
        ProfileHelpAction.About -> "关于寻径星野" to "寻径星野为户外爱好者准备装备、行程与技能工具，帮助出发前更从容。"
        ProfileHelpAction.Roadmap -> "产品路线图" to ""
    }
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
