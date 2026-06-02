package com.rustella.stellartrail.ui.common

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DatePickerField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    placeholder: String = "选择日期",
    allowClear: Boolean = true,
) {
    var showPicker by remember { mutableStateOf(false) }

    Box(modifier = modifier) {
        OutlinedTextField(
            value = value,
            onValueChange = {},
            readOnly = true,
            label = { Text(label) },
            placeholder = { Text(placeholder) },
            singleLine = true,
            trailingIcon = {
                Icon(Icons.Filled.DateRange, contentDescription = "$label 日期选择")
            },
            modifier = Modifier.fillMaxWidth(),
        )
        Box(
            Modifier
                .matchParentSize()
                .clip(TrailInnerCardShape)
                .clickable { showPicker = true },
        )
    }

    if (showPicker) {
        val pickerState = rememberDatePickerState(
            initialSelectedDateMillis = dateStringToPickerMillis(value),
        )
        DatePickerDialog(
            onDismissRequest = { showPicker = false },
            confirmButton = {
                TextButton(
                    onClick = {
                        pickerState.selectedDateMillis?.let { onValueChange(pickerMillisToDateString(it)) }
                        showPicker = false
                    },
                ) {
                    Text("确定", fontWeight = FontWeight.Bold)
                }
            },
            dismissButton = {
                Row {
                    if (allowClear && value.isNotBlank()) {
                        TextButton(
                            onClick = {
                                onValueChange("")
                                showPicker = false
                            },
                        ) {
                            Text("清除")
                        }
                    }
                    TextButton(onClick = { showPicker = false }) {
                        Text("取消")
                    }
                }
            },
        ) {
            DatePicker(state = pickerState)
        }
    }
}

internal fun dateStringToPickerMillis(value: String): Long? =
    runCatching {
        LocalDate.parse(value.trim()).atStartOfDay(ZoneOffset.UTC).toInstant().toEpochMilli()
    }.getOrNull()

internal fun pickerMillisToDateString(value: Long): String =
    Instant.ofEpochMilli(value).atZone(ZoneOffset.UTC).toLocalDate().toString()
