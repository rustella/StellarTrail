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
                        subtitle: subtitle,
                        chips: ["邮箱账号", "验证码", "统一登录态"]
                    )

                    modeSelector

                    TrailSurfaceCard {
                        formFields
                    }

                    if let message = viewModel.message {
                        TrailSurfaceCard {
                            Text(message)
                        }
                    }

                    TrailPrimaryButton(
                        title: viewModel.loading ? "处理中…" : viewModel.mode.submitTitle,
                        action: { Task { await viewModel.submit() } },
                        isDisabled: viewModel.loading || !viewModel.canSubmit
                    )
                }
                .padding(16)
            }
            .trailScreenBackground()
            .navigationTitle(viewModel.mode.title)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭") { dismiss() } } }
            .onChange(of: viewModel.completed) { _, completed in
                if completed { dismiss() }
            }
        }
    }

    private var subtitle: String {
        switch viewModel.mode {
        case .password: return "使用用户名或邮箱登录，验证码会在需要时出现。"
        case .email: return "不输入密码，使用邮箱一次性验证码登录。"
        case .register: return "创建账号后会立即刷新本机登录态。"
        case .reset: return "通过邮箱验证码重置密码并重新登录。"
        }
    }

    private var modeSelector: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(AuthMode.allCases) { mode in
                    TrailPillButton(title: mode.title, isSelected: viewModel.mode == mode) {
                        viewModel.switchMode(mode)
                    }
                }
            }
        }
    }

    @ViewBuilder
    private var formFields: some View {
        switch viewModel.mode {
        case .password:
            TextField("用户名或邮箱", text: $viewModel.account)
                .textContentType(.username)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .trailFormField()
            SecureField("密码", text: $viewModel.password)
                .textContentType(.password)
                .trailFormField()
            if viewModel.captchaTicket != nil {
                TextField("验证码", text: $viewModel.captchaAnswer)
                    .textInputAutocapitalization(.characters)
                    .trailFormField()
                Text("多次登录失败时需要先完成验证码。")
                    .font(.caption)
            } else {
                Text("若多次输入错误，系统会提示验证码。")
                    .font(.caption)
            }

        case .email:
            emailAndCodeFields

        case .register:
            TextField("用户名", text: $viewModel.username)
                .textContentType(.username)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .trailFormField()
            emailAndCodeFields
            SecureField("密码", text: $viewModel.password)
                .textContentType(.newPassword)
                .trailFormField()
            SecureField("确认密码", text: $viewModel.confirmPassword)
                .textContentType(.newPassword)
                .trailFormField()

        case .reset:
            emailAndCodeFields
            SecureField("新密码", text: $viewModel.password)
                .textContentType(.newPassword)
                .trailFormField()
            SecureField("确认新密码", text: $viewModel.confirmPassword)
                .textContentType(.newPassword)
                .trailFormField()
        }
    }

    private var emailAndCodeFields: some View {
        Group {
            TextField("邮箱", text: $viewModel.email)
                .keyboardType(.emailAddress)
                .textContentType(.emailAddress)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .trailFormField()
            HStack {
                TextField("邮箱验证码", text: $viewModel.verificationCode)
                    .keyboardType(.numberPad)
                    .trailFormField()
                TrailPillButton(title: viewModel.sendingCode ? "发送中…" : "发送验证码") {
                    Task { await viewModel.sendVerificationCode() }
                }
            }
        }
    }
}
