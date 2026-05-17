package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
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
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.LoadingState

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
        topBar = {
            TopAppBar(
                title = { Text(if (state.id == null) "新增装备" else "编辑装备") },
                navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回") } },
            )
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier.fillMaxSize().padding(innerPadding).verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (state.loading) LoadingState()
            if (state.error != null) ErrorState(state.error!!)
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("基础信息", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    OutlinedTextField(state.form.name, viewModel::updateName, label = { Text("装备名称 *") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    CategoryPicker(state.form.category, viewModel::updateCategory)
                    StatusPicker(state.form.status, viewModel::updateStatus)
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                        OutlinedTextField(state.form.brand, viewModel::updateBrand, label = { Text("品牌") }, singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(state.form.model, viewModel::updateModel, label = { Text("型号") }, singleLine = true, modifier = Modifier.weight(1f))
                    }
                    OutlinedTextField(state.form.description, viewModel::updateDescription, label = { Text("描述") }, minLines = 3, modifier = Modifier.fillMaxWidth())
                }
            }
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("规格与购买", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                        OutlinedTextField(state.form.weightG, viewModel::updateWeight, label = { Text("重量 g") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number), singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(state.form.purchasePrice, viewModel::updatePrice, label = { Text("价格 ¥") }, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal), singleLine = true, modifier = Modifier.weight(1f))
                    }
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                        OutlinedTextField(state.form.purchaseDate, viewModel::updatePurchaseDate, label = { Text("购买日期") }, singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(state.form.expiryOrWarrantyDate, viewModel::updateWarrantyDate, label = { Text("保修/过期") }, singleLine = true, modifier = Modifier.weight(1f))
                    }
                    OutlinedTextField(state.form.purchaseLocation, viewModel::updatePurchaseLocation, label = { Text("购买地点") }, singleLine = true, modifier = Modifier.fillMaxWidth())
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
            }
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("库存管理", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    OutlinedTextField(state.form.storageLocation, viewModel::updateStorageLocation, label = { Text("存放位置") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(state.form.tags, viewModel::updateTags, label = { Text("标签，用逗号或空格分隔") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Row(modifier = Modifier.fillMaxWidth()) {
                        Checkbox(checked = state.form.shareEnabled, onCheckedChange = viewModel::updateShareEnabled)
                        Text("允许在共享装备池中展示", modifier = Modifier.padding(top = 12.dp))
                    }
                    OutlinedTextField(state.form.notes, viewModel::updateNotes, label = { Text("备注") }, minLines = 3, modifier = Modifier.fillMaxWidth())
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                OutlinedButton(onClick = onBack, modifier = Modifier.weight(1f)) { Text("取消") }
                Button(onClick = viewModel::submit, enabled = !state.submitting, modifier = Modifier.weight(1f)) {
                    Text(if (state.submitting) "保存中..." else "保存")
                }
            }
        }
    }
}

@Composable
private fun CategoryPicker(value: GearCategory, onChange: (GearCategory) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Column {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth()) { Text("分类：${value.label}") }
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
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth()) { Text("状态：${value.label}") }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            GearStatus.entries.forEach { status ->
                DropdownMenuItem(text = { Text(status.label) }, onClick = { onChange(status); expanded = false })
            }
        }
    }
}
