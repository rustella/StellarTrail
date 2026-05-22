package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.feature.skills.detail.SkillDetailViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.EmptyState
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.StepItem
import com.rustella.stellartrail.ui.common.SurfaceCard
import com.rustella.stellartrail.ui.common.TrailCardShape
import com.rustella.stellartrail.ui.common.currentTrailPalette

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun SkillDetailScreen(
    viewModel: SkillDetailViewModel,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = { Text(state.detail?.title ?: "技能详情", fontWeight = FontWeight.ExtraBold) },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回") } },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
            )
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            if (state.error != null) ErrorState(state.error!!, onRetry = viewModel::load)
            if (state.loading) LoadingState()
            state.detail?.let { SkillDetailContent(it) }
            if (!state.loading && state.detail == null && state.error == null) {
                EmptyState("暂无绳结详情", "返回列表后重新打开。")
            }
        }
    }
}

@Composable
private fun SkillDetailContent(detail: KnotDetail) {
    val palette = currentTrailPalette()
    SurfaceCard(contentPadding = PaddingValues(0.dp)) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(215.dp)
                .clip(TrailCardShape)
                .background(Color(0xFF020617)),
            contentAlignment = Alignment.Center,
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 36.dp)
                    .height(170.dp)
                    .background(Color(0xFFF8FAFC)),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    if (detail.media.isEmpty()) "暂无演示图" else "${detail.media.size} 个素材",
                    color = palette.textMuted,
                    fontWeight = FontWeight.ExtraBold,
                )
            }
        }
        Column(Modifier.fillMaxWidth().padding(17.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Top) {
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(detail.title, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.ExtraBold)
                    Text(detail.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        (detail.categories + detail.types).take(3).forEach { item ->
                            Badge(item.title, tone = BadgeTone.Info)
                        }
                    }
                }
                Badge(detail.difficulty ?: "入门", tone = BadgeTone.Warning)
            }
        }
    }
    if (!detail.description.isNullOrBlank()) {
        SurfaceCard {
            SectionTitle("用途说明")
            Text(detail.description, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
    if (detail.steps.isNotEmpty()) {
        SurfaceCard {
            SectionTitle("步骤")
            detail.steps.forEachIndexed { index, step -> StepItem(index, step) }
        }
    }
}
