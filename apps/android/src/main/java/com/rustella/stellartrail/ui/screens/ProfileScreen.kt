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
import com.rustella.stellartrail.ui.common.CompactPillAction
import com.rustella.stellartrail.ui.common.FeatureTile
import com.rustella.stellartrail.ui.common.MetadataRow
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

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
    val config by viewModel.config.collectAsStateWithLifecycle()
    val canEditBaseUrl = viewModel.canEditBaseUrl
    var baseUrl by remember(config.baseUrl) { mutableStateOf(config.baseUrl) }
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        SurfaceCard {
            Text("我的寻径星野", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.ExtraBold)
            Text(
                "管理装备、学习技能，也可以切换黑夜模式。",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        if (session == null) {
            SurfaceCard {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("账号状态", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.ExtraBold)
                        Text(
                            "未登录也可以先看装备图鉴和绳结教学。",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                    CompactPillAction("登录 / 注册", onLogin)
                }
            }
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
        SectionTitle("我的工具")
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            FeatureTile("图", "功能路线图", "查看计划和投票", onOpenRoadmap, Modifier.weight(1f), compact = true)
            FeatureTile("山", "户外资料", "健康与联系人", onOpenOutdoorProfile, Modifier.weight(1f), compact = true)
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            FeatureTile("历", "户外经历", "历史行程沉淀", onOpenOutdoorExperiences, Modifier.weight(1f), compact = true)
            FeatureTile("设", "设置", "账号与资料", onOpenSettings, Modifier.weight(1f), compact = true)
        }
        SurfaceCard {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    SectionTitle("黑夜模式")
                    Text(
                        "开启后切换为紫色星空暗色界面，夜间浏览更舒服。",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                CompactPillAction(
                    text = if (theme == ThemeMode.DARK) "浅色模式" else "黑夜模式",
                    onClick = { viewModel.setTheme(if (theme == ThemeMode.DARK) ThemeMode.LIGHT else ThemeMode.DARK) },
                    filled = false,
                )
            }
        }
        if (canEditBaseUrl) {
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
