package com.rustella.stellartrail.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.rustella.stellartrail.feature.auth.AuthMode
import com.rustella.stellartrail.feature.auth.AuthRegisterMethod
import com.rustella.stellartrail.feature.auth.AuthResetMethod
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
            eyebrow = AuthVisualContract.heroEyebrow,
            title = AuthVisualContract.heroTitle,
            subtitle = AuthVisualContract.heroSubtitle,
        )
        SurfaceCard(Modifier.fillMaxWidth()) {
            AuthHeader(state.mode, viewModel)
            if (state.error != null) ErrorState(message = state.error!!)
            if (state.notice != null) Text(state.notice!!, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            when (state.mode) {
                AuthMode.LOGIN -> LoginForm(viewModel)
                AuthMode.PHONE_CODE -> PhoneCodeLoginForm(viewModel)
                AuthMode.EMAIL_CODE -> EmailCodeLoginForm(viewModel)
                AuthMode.REGISTER -> RegisterForm(viewModel)
                AuthMode.RESET_PASSWORD -> ResetPasswordForm(viewModel)
            }
            if (state.loading) LoadingState()
        }
    }
}

@Composable
private fun AuthHeader(mode: AuthMode, viewModel: AuthViewModel) {
    Column(verticalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = when (mode) {
                    AuthMode.REGISTER -> "创建账号"
                    AuthMode.RESET_PASSWORD -> "找回密码"
                    else -> AuthVisualContract.loginSectionTitle
                },
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.ExtraBold,
            )
            if (mode == AuthMode.REGISTER || mode == AuthMode.RESET_PASSWORD) {
                TextButton(onClick = { viewModel.switchMode(AuthMode.LOGIN) }) {
                    Text(AuthVisualContract.backToLogin, fontWeight = FontWeight.Bold)
                }
            }
        }
        if (mode in AuthVisualContract.loginTabModes) {
            AuthModePicker(mode, viewModel)
        }
    }
}

@Composable
private fun AuthModePicker(mode: AuthMode, viewModel: AuthViewModel) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
        AuthVisualContract.loginTabModes.forEach { tabMode ->
            AuthModeChip(
                label = AuthVisualContract.loginTabLabels.getValue(tabMode),
                selected = mode == tabMode,
                modifier = Modifier.weight(1f),
            ) {
                viewModel.switchMode(tabMode)
            }
        }
    }
}

@Composable
private fun AuthModeChip(label: String, selected: Boolean, modifier: Modifier, onClick: () -> Unit) {
    FilterChip(
        selected = selected,
        onClick = onClick,
        label = {
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                Text(label, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        },
        modifier = modifier,
    )
}

@Composable
private fun AuthChoiceRow(labels: List<Pair<String, Boolean>>, onSelect: (String) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
        labels.forEach { (label, selected) ->
            AuthModeChip(label = label, selected = selected, modifier = Modifier.weight(1f)) { onSelect(label) }
        }
    }
}

@Composable
private fun LoginForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        AuthTextField(
            value = state.account,
            onValueChange = viewModel::updateAccount,
            label = "用户名 / 邮箱 / 手机号",
        )
        AuthTextField(
            value = state.password,
            onValueChange = viewModel::updatePassword,
            label = "密码",
            password = true,
        )
        if (state.captchaTicket.isNotBlank()) {
            Text("请输入验证码后继续。")
            if (state.debugCaptchaAnswer != null) {
                Text("验证码提示：${state.debugCaptchaAnswer}", color = MaterialTheme.colorScheme.tertiary)
            }
            AuthTextField(
                value = state.captchaAnswer,
                onValueChange = viewModel::updateCaptchaAnswer,
                label = "验证码",
            )
            OutlinedButton(onClick = viewModel::refreshCaptcha, enabled = !state.loading) { Text("刷新验证码") }
        }
        PrimaryPillButton(AuthVisualContract.passwordPrimaryAction, viewModel::login, Modifier.fillMaxWidth(), enabled = !state.loading)
        AuthSecondaryActions(viewModel)
    }
}

@Composable
private fun PhoneCodeLoginForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("使用手机短信验证码登录。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        AuthTextField(
            value = state.phone,
            onValueChange = viewModel::updatePhone,
            label = "手机号",
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
        )
        SmsCodeRow(
            value = state.smsCode,
            onValueChange = viewModel::updateSmsCode,
            onSend = viewModel::sendSmsLoginCode,
            enabled = !state.loading,
        )
        PrimaryPillButton(AuthVisualContract.phoneCodePrimaryAction, viewModel::loginWithSmsCode, Modifier.fillMaxWidth(), enabled = !state.loading)
        AuthSecondaryActions(viewModel)
    }
}

@Composable
private fun EmailCodeLoginForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("使用邮箱验证码登录。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        AuthTextField(
            value = state.email,
            onValueChange = viewModel::updateEmail,
            label = "邮箱",
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
            AuthTextField(
                value = state.emailCode,
                onValueChange = viewModel::updateEmailCode,
                label = "邮箱验证码",
                modifier = Modifier.weight(1f),
            )
            OutlinedButton(
                onClick = viewModel::sendEmailLoginCode,
                enabled = !state.loading,
                modifier = Modifier.height(56.dp).widthIn(min = 112.dp),
            ) { Text(AuthVisualContract.sendCodeAction, fontWeight = FontWeight.Bold) }
        }
        PrimaryPillButton(AuthVisualContract.emailCodePrimaryAction, viewModel::loginWithEmailCode, Modifier.fillMaxWidth(), enabled = !state.loading)
        AuthSecondaryActions(viewModel)
    }
}

@Composable
private fun RegisterForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("创建账号后可以保存装备、行程和个人资料。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        AuthChoiceRow(
            labels = listOf(
                AuthVisualContract.phoneRegisterMethod to (state.registerMethod == AuthRegisterMethod.PHONE),
                AuthVisualContract.emailRegisterMethod to (state.registerMethod == AuthRegisterMethod.EMAIL),
            ),
            onSelect = { label ->
                viewModel.setRegisterMethod(
                    if (label == AuthVisualContract.phoneRegisterMethod) AuthRegisterMethod.PHONE else AuthRegisterMethod.EMAIL,
                )
            },
        )
        AuthTextField(
            value = state.username,
            onValueChange = viewModel::updateUsername,
            label = "用户名",
        )
        if (state.registerMethod == AuthRegisterMethod.PHONE) {
            AuthTextField(
                value = state.nickname,
                onValueChange = viewModel::updateNickname,
                label = "昵称",
            )
            AuthTextField(
                value = state.phone,
                onValueChange = viewModel::updatePhone,
                label = "手机号",
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
            )
            SmsCodeRow(
                value = state.smsCode,
                onValueChange = viewModel::updateSmsCode,
                onSend = viewModel::sendSmsRegistrationCode,
                enabled = !state.loading,
            )
        } else {
            AuthTextField(
                value = state.email,
                onValueChange = viewModel::updateEmail,
                label = "邮箱",
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                AuthTextField(
                    value = state.emailCode,
                    onValueChange = viewModel::updateEmailCode,
                    label = "邮箱验证码",
                    modifier = Modifier.weight(1f),
                )
                OutlinedButton(
                    onClick = viewModel::sendEmailCode,
                    enabled = !state.loading,
                    modifier = Modifier.height(56.dp).widthIn(min = 112.dp),
                ) { Text(AuthVisualContract.sendCodeAction, fontWeight = FontWeight.Bold) }
            }
        }
        AuthTextField(
            value = state.password,
            onValueChange = viewModel::updatePassword,
            label = "密码",
            password = true,
        )
        AuthTextField(
            value = state.confirmPassword,
            onValueChange = viewModel::updateConfirmPassword,
            label = "确认密码",
            password = true,
        )
        PrimaryPillButton("注册并登录", viewModel::register, Modifier.fillMaxWidth(), enabled = !state.loading)
    }
}

@Composable
private fun ResetPasswordForm(viewModel: AuthViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Column(verticalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        Text("通过验证码确认身份后设置新密码。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        AuthChoiceRow(
            labels = listOf(
                AuthVisualContract.phoneResetMethod to (state.resetMethod == AuthResetMethod.PHONE),
                AuthVisualContract.emailResetMethod to (state.resetMethod == AuthResetMethod.EMAIL),
            ),
            onSelect = { label ->
                viewModel.setResetMethod(
                    if (label == AuthVisualContract.phoneResetMethod) AuthResetMethod.PHONE else AuthResetMethod.EMAIL,
                )
            },
        )
        if (state.resetMethod == AuthResetMethod.PHONE) {
            AuthTextField(
                value = state.phone,
                onValueChange = viewModel::updatePhone,
                label = "手机号",
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
            )
            SmsCodeRow(
                value = state.smsCode,
                onValueChange = viewModel::updateSmsCode,
                onSend = viewModel::sendSmsPasswordResetCode,
                enabled = !state.loading,
            )
        } else {
            AuthTextField(
                value = state.email,
                onValueChange = viewModel::updateEmail,
                label = "邮箱",
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
                AuthTextField(
                    value = state.emailCode,
                    onValueChange = viewModel::updateEmailCode,
                    label = "邮箱验证码",
                    modifier = Modifier.weight(1f),
                )
                OutlinedButton(
                    onClick = viewModel::sendPasswordResetCode,
                    enabled = !state.loading,
                    modifier = Modifier.height(56.dp).widthIn(min = 112.dp),
                ) { Text(AuthVisualContract.sendCodeAction, fontWeight = FontWeight.Bold) }
            }
        }
        AuthTextField(
            value = state.resetPassword,
            onValueChange = viewModel::updateResetPassword,
            label = "新密码",
            password = true,
        )
        AuthTextField(
            value = state.resetConfirmPassword,
            onValueChange = viewModel::updateResetConfirmPassword,
            label = "确认新密码",
            password = true,
        )
        PrimaryPillButton("重设密码并登录", viewModel::resetPassword, Modifier.fillMaxWidth(), enabled = !state.loading)
    }
}

@Composable
private fun SmsCodeRow(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit,
    enabled: Boolean,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp), modifier = Modifier.fillMaxWidth()) {
        AuthTextField(
            value = value,
            onValueChange = onValueChange,
            label = "短信验证码",
            modifier = Modifier.weight(1f),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        )
        OutlinedButton(
            onClick = onSend,
            enabled = enabled,
            modifier = Modifier.height(56.dp).widthIn(min = 112.dp),
        ) { Text(AuthVisualContract.sendCodeAction, fontWeight = FontWeight.Bold) }
    }
}

@Composable
private fun AuthTextField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    modifier: Modifier = Modifier,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    password: Boolean = false,
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        label = { Text(label) },
        keyboardOptions = keyboardOptions,
        singleLine = true,
        visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None,
        modifier = modifier.fillMaxWidth(),
    )
}

@Composable
private fun AuthSecondaryActions(viewModel: AuthViewModel, showForgotPassword: Boolean = true) {
    Row(
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth(),
    ) {
        TextButton(onClick = { viewModel.switchMode(AuthMode.REGISTER) }) {
            Text(AuthVisualContract.registerLink, fontWeight = FontWeight.Bold)
        }
        if (showForgotPassword) {
            Text("·", color = MaterialTheme.colorScheme.onSurfaceVariant)
            TextButton(onClick = { viewModel.switchMode(AuthMode.RESET_PASSWORD) }) {
                Text(AuthVisualContract.forgotPasswordLink, fontWeight = FontWeight.Bold)
            }
        }
    }
}
