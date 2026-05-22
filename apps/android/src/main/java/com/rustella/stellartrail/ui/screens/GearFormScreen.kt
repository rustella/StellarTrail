package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.feature.gear.form.GearFormViewModel
import com.rustella.stellartrail.ui.common.Badge
import com.rustella.stellartrail.ui.common.BadgeTone
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SectionTitle
import com.rustella.stellartrail.ui.common.SoftPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GearFormScreen(
    viewModel: GearFormViewModel,
    onBack: () -> Unit,
    onSaved: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(state.savedId) {
        state.savedId?.let(onSaved)
    }
    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                title = { Text(if (state.id == null) "新增装备" else "编辑装备", fontWeight = FontWeight.ExtraBold) },
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
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            if (state.loading) LoadingState()
            if (state.error != null) ErrorState(state.error!!)
            SurfaceCard(Modifier.fillMaxWidth()) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Badge("基础信息")
                    Badge(if (state.id == null) "新装备" else "编辑中", BadgeTone.Info)
                }
                OutlinedTextField(state.form.name, viewModel::updateName, label = { Text("装备名称 *") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                CategoryPicker(state.form.category, viewModel::updateCategory)
                StatusPicker(state.form.status, viewModel::updateStatus)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.brand, viewModel::updateBrand, label = { Text("品牌") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.model, viewModel::updateModel, label = { Text("型号") }, singleLine = true, modifier = Modifier.weight(1f))
                }
                OutlinedTextField(state.form.description, viewModel::updateDescription, label = { Text("描述") }, minLines = 3, modifier = Modifier.fillMaxWidth())
            }
            SurfaceCard(Modifier.fillMaxWidth()) {
                SectionTitle("规格与购买", "将重量、价格和日期整理成微信端同款轻量信息层级。")
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.weightG, viewModel::updateWeight, label = { Text("重量 g") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.officialPrice, viewModel::updateOfficialPrice, label = { Text("官方价格 ¥") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal), singleLine = true, modifier = Modifier.weight(1f))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.purchasePrice, viewModel::updatePrice, label = { Text("购入价格 ¥") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal), singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.purchaseLocation, viewModel::updatePurchaseLocation, label = { Text("购买渠道") }, singleLine = true, modifier = Modifier.weight(1f))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.purchaseDate, viewModel::updatePurchaseDate, label = { Text("购买日期") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.expiryOrWarrantyDate, viewModel::updateWarrantyDate, label = { Text("保修/过期") }, singleLine = true, modifier = Modifier.weight(1f))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.color, viewModel::updateColor, label = { Text("颜色") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.material, viewModel::updateMaterial, label = { Text("材质") }, singleLine = true, modifier = Modifier.weight(1f))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.capacity, viewModel::updateCapacity, label = { Text("容量") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.size, viewModel::updateSize, label = { Text("尺码") }, singleLine = true, modifier = Modifier.weight(1f))
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    OutlinedTextField(state.form.warmthIndex, viewModel::updateWarmth, label = { Text("保暖指数") }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(state.form.waterproofIndex, viewModel::updateWaterproof, label = { Text("防水指数") }, singleLine = true, modifier = Modifier.weight(1f))
                }
            }
            SurfaceCard(Modifier.fillMaxWidth()) {
                SectionTitle("库存管理", "存放位置、标签和共享状态集中在一个轻卡片中。")
                OutlinedTextField(state.form.storageLocation, viewModel::updateStorageLocation, label = { Text("存放位置") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                OutlinedTextField(state.form.tags, viewModel::updateTags, label = { Text("标签，用逗号或空格分隔") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                Row(modifier = Modifier.fillMaxWidth()) {
                    Checkbox(checked = state.form.shareEnabled, onCheckedChange = viewModel::updateShareEnabled)
                    Text("允许在共享装备池中展示", modifier = Modifier.padding(top = 12.dp))
                }
                OutlinedTextField(state.form.notes, viewModel::updateNotes, label = { Text("备注") }, minLines = 3, modifier = Modifier.fillMaxWidth())
            }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                SoftPillButton("取消", onBack, Modifier.weight(1f))
                PrimaryPillButton(if (state.submitting) "保存中..." else "保存", viewModel::submit, Modifier.weight(1f), enabled = !state.submitting)
            }
        }
    }
}

@Composable
private fun CategoryPicker(value: GearCategory, onChange: (GearCategory) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Column {
        SoftPillButton("分类：${value.label}", { expanded = true }, Modifier.fillMaxWidth())
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            GearCategory.entries.forEach { category ->
                DropdownMenuItem(text = { Text(category.label) }, onClick = { onChange(category); expanded = false })
            }
        }
    }
}

@Composable
private fun StatusPicker(value: GearStatus, onChange: (GearStatus) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Column {
        SoftPillButton("状态：${value.label}", { expanded = true }, Modifier.fillMaxWidth())
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            GearStatus.entries.forEach { status ->
                DropdownMenuItem(text = { Text(status.label) }, onClick = { onChange(status); expanded = false })
            }
        }
    }
}
