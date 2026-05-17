import SwiftUI

struct AuthView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel: AuthViewModel

    init(environment: AppEnvironment, mode: AuthMode) {
        _viewModel = StateObject(wrappedValue: AuthViewModel(mode: mode, repository: environment.authRepository, sessionStore: environment.sessionStore))
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    TrailHeroCard(
                        eyebrow: "寻径星野账号",
                        title: viewModel.mode.title,
                        subtitle: viewModel.mode == .login ? "登录后保存装备、统计和个人设置。" : "创建账号后可保存自己的出行准备。",
                        chips: ["验证码门槛", "安全保存"]
                    )

                    Picker("方式", selection: $viewModel.mode) {
                        Text("账号登录").tag(AuthMode.login)
                        Text("注册账号").tag(AuthMode.register)
                    }
                    .pickerStyle(.segmented)

                    TrailSurfaceCard {
                        if viewModel.mode == .login {
                            TextField("用户名或邮箱", text: $viewModel.account)
                                .textContentType(.username)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                            SecureField("密码", text: $viewModel.password)
                                .textContentType(.password)
                            if viewModel.captchaTicket != nil {
                                TextField("验证码", text: $viewModel.captchaAnswer)
                                    .textInputAutocapitalization(.characters)
                                Text("多次登录失败时需要先完成验证码。")
                                    .font(.caption)
                            } else {
                                Text("若多次输入错误，系统会提示验证码。")
                                    .font(.caption)
                            }
                        } else {
                            TextField("用户名", text: $viewModel.username)
                                .textContentType(.username)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                            TextField("邮箱", text: $viewModel.email)
                                .keyboardType(.emailAddress)
                                .textContentType(.emailAddress)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                            HStack {
                                TextField("邮箱验证码", text: $viewModel.verificationCode)
                                    .keyboardType(.numberPad)
                                Button("发送验证码") { Task { await viewModel.sendVerificationCode() } }
                                    .font(.subheadline.weight(.bold))
                            }
                            SecureField("密码", text: $viewModel.password)
                                .textContentType(.newPassword)
                            SecureField("确认密码", text: $viewModel.confirmPassword)
                                .textContentType(.newPassword)
                        }
                    }
                    .textFieldStyle(.roundedBorder)

                    if let message = viewModel.message {
                        TrailSurfaceCard {
                            Text(message)
                        }
                    }

                    TrailPrimaryButton(title: viewModel.loading ? "处理中…" : viewModel.mode.title, action: { Task { await viewModel.submit() } }, isDisabled: viewModel.loading || !viewModel.canSubmit)
                }
                .padding(16)
            }
            .navigationTitle(viewModel.mode.title)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭") { dismiss() } } }
            .onChange(of: viewModel.completed) { _, completed in
                if completed { dismiss() }
            }
        }
    }
}
