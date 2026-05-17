import SwiftUI

struct MacAuthPageView: View {
    @StateObject private var viewModel: AuthViewModel

    init(environment: MacAppEnvironment, mode: AuthMode) {
        _viewModel = StateObject(wrappedValue: AuthViewModel(mode: mode, repository: environment.authRepository, sessionStore: environment.sessionStore))
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                TrailHeroCard(
                    eyebrow: "寻径星野账号",
                    title: viewModel.mode.title,
                    subtitle: viewModel.mode == .login ? "登录后保存装备、统计和个人设置。" : "创建账号后可保存自己的出行准备。",
                    chips: ["安全保存", "桌面端"]
                )
                .frame(maxWidth: 760)

                Picker("方式", selection: $viewModel.mode) {
                    Text("账号登录").tag(AuthMode.login)
                    Text("注册账号").tag(AuthMode.register)
                }
                .pickerStyle(.segmented)
                .frame(maxWidth: 420)

                TrailSurfaceCard {
                    if viewModel.mode == .login {
                        TextField("用户名或邮箱", text: $viewModel.account)
                        SecureField("密码", text: $viewModel.password)
                        if viewModel.captchaTicket != nil {
                            TextField("验证码", text: $viewModel.captchaAnswer)
                            Text("多次登录失败时需要先完成验证码。")
                                .font(.caption)
                        } else {
                            Text("若多次输入错误，系统会提示验证码。")
                                .font(.caption)
                        }
                    } else {
                        TextField("用户名", text: $viewModel.username)
                        TextField("邮箱", text: $viewModel.email)
                        HStack {
                            TextField("邮箱验证码", text: $viewModel.verificationCode)
                            Button("发送验证码") { Task { await viewModel.sendVerificationCode() } }
                                .font(.subheadline.weight(.bold))
                        }
                        SecureField("密码", text: $viewModel.password)
                        SecureField("确认密码", text: $viewModel.confirmPassword)
                    }
                }
                .textFieldStyle(.roundedBorder)
                .frame(maxWidth: 520)

                if let message = viewModel.message {
                    TrailSurfaceCard { Text(message) }
                        .frame(maxWidth: 520)
                }

                TrailPrimaryButton(title: viewModel.loading ? "处理中…" : viewModel.mode.title, action: { Task { await viewModel.submit() } }, isDisabled: viewModel.loading || !viewModel.canSubmit)
                    .frame(maxWidth: 240)
            }
            .padding(32)
        }
        .navigationTitle(viewModel.mode.title)
    }
}
