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
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.feature.auth.AuthMode
import com.rustella.stellartrail.feature.auth.AuthViewModel
import com.rustella.stellartrail.ui.common.ErrorState
import com.rustella.stellartrail.ui.common.HeroCard
import com.rustella.stellartrail.ui.common.LoadingState
import com.rustella.stellartrail.ui.common.PrimaryPillButton
import com.rustella.stellartrail.ui.common.SurfaceCard

@Composable
fun AuthScreen(viewModel: AuthViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        HeroCard(
            eyebrow = "寻径星野",
            title = "山野出发前的准备台",
            subtitle = "装备准备、户外技能与路线知识，延续微信端深色渐变头图与圆角卡片。",
        )
        SurfaceCard(Modifier.fillMaxWidth()) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
                FilterChip(
                    selected = state.mode == AuthMode.LOGIN,
                    onClick = { viewModel.switchMode(AuthMode.LOGIN) },
                    label = { Text("登录", fontWeight = FontWeight.Bold) },
                    modifier = Modifier.weight(1f),
                )
                FilterChip(
                    selected = state.mode == AuthMode.REGISTER,
                    onClick = { viewModel.switchMode(AuthMode.REGISTER) },
                    label = { Text("注册", fontWeight = FontWeight.Bold) },
                    modifier = Modifier.weight(1f),
                )
            }
            if (state.error != null) ErrorState(message = state.error!!)
            if (state.notice != null) Text(state.notice!!, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            if (state.mode == AuthMode.LOGIN) {
                LoginForm(viewModel)
            } else {
                RegisterForm(viewModel)
            }
            if (state.loading) LoadingState()
        }
    }
}

@Composable
private fun LoginForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        OutlinedTextField(
            value = state.account,
            onValueChange = viewModel::updateAccount,
            label = { Text("用户名或邮箱") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.password,
            onValueChange = viewModel::updatePassword,
            label = { Text("密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )
        if (state.captchaTicket.isNotBlank()) {
            Text("验证码图片已返回。当前版本以调试文本方式展示，后续可接入 SVG 渲染。")
            if (state.debugCaptchaAnswer != null) {
                Text("调试验证码：${state.debugCaptchaAnswer}", color = MaterialTheme.colorScheme.tertiary)
            }
            OutlinedTextField(
                value = state.captchaAnswer,
                onValueChange = viewModel::updateCaptchaAnswer,
                label = { Text("验证码") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedButton(onClick = viewModel::refreshCaptcha, enabled = !state.loading) { Text("刷新验证码") }
        }
        PrimaryPillButton("登录", viewModel::login, Modifier.fillMaxWidth(), enabled = !state.loading)
    }
}

@Composable
private fun RegisterForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        OutlinedTextField(
            value = state.username,
            onValueChange = viewModel::updateUsername,
            label = { Text("用户名") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.email,
            onValueChange = viewModel::updateEmail,
            label = { Text("邮箱") },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedTextField(
                value = state.emailCode,
                onValueChange = viewModel::updateEmailCode,
                label = { Text("邮箱验证码") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
            OutlinedButton(onClick = viewModel::sendEmailCode, enabled = !state.loading) { Text("获取") }
        }
        OutlinedTextField(
            value = state.password,
            onValueChange = viewModel::updatePassword,
            label = { Text("密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.confirmPassword,
            onValueChange = viewModel::updateConfirmPassword,
            label = { Text("确认密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )
        PrimaryPillButton("注册并登录", viewModel::register, Modifier.fillMaxWidth(), enabled = !state.loading)
    }
}
