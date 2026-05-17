package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.feature.profile.ProfileViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.HeroButton
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.NoticeCard
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun ProfileScreen(viewModel: ProfileViewModel, onLogin: () -> Unit, modifier: Modifier = Modifier) {
    val session by viewModel.session.collectAsStateWithLifecycle()
    val theme by viewModel.theme.collectAsStateWithLifecycle()
    val config by viewModel.config.collectAsStateWithLifecycle()
    var baseUrl by remember(config.baseUrl) { mutableStateOf(config.baseUrl) }
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        HeroCard(
            eyebrow = "寻径星野账号",
            title = "我的",
            subtitle = "管理登录状态、主题和本地调试地址，保持与微信端一致的轻卡片界面。",
            chips = listOf(theme.label(), if (session == null) "待登录" else "已登录"),
            actions = {
                if (session == null) {
                    HeroButton("去登录", onLogin)
                }
            },
        )
        if (session == null) {
            NoticeCard(
                title = "登录后保存自己的准备进度",
                body = "当前仍可浏览首页、装备参考和技能内容。",
                action = { PrimaryPillButton("去登录", onLogin) },
            )
        } else {
            SurfaceCard {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Badge("已登录")
                    Badge(theme.label(), tone = BadgeTone.Info)
                }
                Text(
                    session?.user?.nickname ?: session?.user?.username ?: "已登录用户",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.ExtraBold,
                )
                MetadataRow("用户 ID", session?.user?.id.orEmpty())
                MetadataRow("邮箱", session?.user?.email ?: "未绑定")
            }
        }
        SurfaceCard {
            SectionTitle("主题", "所有移动端默认共享微信端品牌配色，避免被动态取色冲淡。")
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ThemeMode.entries.forEach { mode ->
                    FilterChip(
                        selected = theme == mode,
                        onClick = { viewModel.setTheme(mode) },
                        label = { Text(mode.label(), fontWeight = FontWeight.Bold) },
                    )
                }
            }
        }
        if (viewModel.canEditBaseUrl) {
            SurfaceCard {
                SectionTitle("本地调试地址")
                OutlinedTextField(
                    value = baseUrl,
                    onValueChange = { baseUrl = it },
                    label = { Text("地址") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    PrimaryPillButton("保存", { viewModel.updateBaseUrl(baseUrl) }, Modifier.weight(1f))
                    SoftPillButton("恢复默认", viewModel::resetBaseUrl, Modifier.weight(1f))
                }
            }
        }
        if (session != null) {
            SoftPillButton("退出登录", viewModel::logout, Modifier.fillMaxWidth())
        }
    }
}

private fun ThemeMode.label(): String = when (this) {
    ThemeMode.LIGHT -> "浅色"
    ThemeMode.DARK -> "深色"
    ThemeMode.SYSTEM -> "跟随系统"
}
