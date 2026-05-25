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
import com.rustella.stellartrail.ui.common.HeroVisualContract
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
        verticalArrangement = Arrangement.spacedBy(HeroVisualContract.followingSectionGapDp.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        HeroCard(
            eyebrow = "寻径星野",
            title = "山野出发前的准备台",
            subtitle = "装备准备、户外技能与路线知识，登录后同步保存你的出行进度。",
        )
        SurfaceCard(Modifier.fillMaxWidth()) {
            AuthModePicker(state.mode, viewModel)
            if (state.error != null) ErrorState(message = state.error!!)
            if (state.notice != null) Text(state.notice!!, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            when (state.mode) {
                AuthMode.LOGIN -> LoginForm(viewModel)
                AuthMode.EMAIL_CODE -> EmailCodeLoginForm(viewModel)
                AuthMode.REGISTER -> RegisterForm(viewModel)
                AuthMode.RESET_PASSWORD -> ResetPasswordForm(viewModel)
            }
            if (state.loading) LoadingState()
        }
    }
}

@Composable
private fun AuthModePicker(mode: AuthMode, viewModel: AuthViewModel) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
            AuthModeChip("账号登录", mode == AuthMode.LOGIN, Modifier.weight(1f)) { viewModel.switchMode(AuthMode.LOGIN) }
            AuthModeChip("邮箱验证码", mode == AuthMode.EMAIL_CODE, Modifier.weight(1f)) { viewModel.switchMode(AuthMode.EMAIL_CODE) }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
            AuthModeChip("注册", mode == AuthMode.REGISTER, Modifier.weight(1f)) { viewModel.switchMode(AuthMode.REGISTER) }
            AuthModeChip("找回密码", mode == AuthMode.RESET_PASSWORD, Modifier.weight(1f)) { viewModel.switchMode(AuthMode.RESET_PASSWORD) }
        }
    }
}

@Composable
private fun AuthModeChip(label: String, selected: Boolean, modifier: Modifier, onClick: () -> Unit) {
    FilterChip(
        selected = selected,
        onClick = onClick,
        label = { Text(label, fontWeight = FontWeight.Bold) },
        modifier = modifier,
    )
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
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            OutlinedButton(
                onClick = { viewModel.switchMode(AuthMode.EMAIL_CODE) },
                enabled = !state.loading,
                modifier = Modifier.weight(1f),
            ) { Text("邮箱验证码") }
            OutlinedButton(
                onClick = { viewModel.switchMode(AuthMode.RESET_PASSWORD) },
                enabled = !state.loading,
                modifier = Modifier.weight(1f),
            ) { Text("找回密码") }
        }
    }
}

@Composable
private fun EmailCodeLoginForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("收取邮箱验证码，不输入密码也能进入。", color = MaterialTheme.colorScheme.onSurfaceVariant)
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
            OutlinedButton(onClick = viewModel::sendEmailLoginCode, enabled = !state.loading) { Text("获取") }
        }
        PrimaryPillButton("邮箱验证码登录", viewModel::loginWithEmailCode, Modifier.fillMaxWidth(), enabled = !state.loading)
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

@Composable
private fun ResetPasswordForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("收取邮箱验证码，确认后设置新密码。", color = MaterialTheme.colorScheme.onSurfaceVariant)
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
            OutlinedButton(onClick = viewModel::sendPasswordResetCode, enabled = !state.loading) { Text("获取") }
        }
        OutlinedTextField(
            value = state.resetPassword,
            onValueChange = viewModel::updateResetPassword,
            label = { Text("新密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = state.resetConfirmPassword,
            onValueChange = viewModel::updateResetConfirmPassword,
            label = { Text("确认新密码") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
        )
        PrimaryPillButton("重设密码并登录", viewModel::resetPassword, Modifier.fillMaxWidth(), enabled = !state.loading)
    }
}
