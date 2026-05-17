import SwiftUI

struct MacAuthPageView: View {
    @Environment(\.trailPalette) private var palette
    @StateObject private var viewModel: AuthViewModel
    private let onContinueAsGuest: () -> Void
    private let onAuthenticated: () -> Void

    init(
        environment: MacAppEnvironment,
        mode: AuthMode,
        onContinueAsGuest: @escaping () -> Void = {},
        onAuthenticated: @escaping () -> Void = {}
    ) {
        _viewModel = StateObject(wrappedValue: AuthViewModel(mode: mode, repository: environment.authRepository, sessionStore: environment.sessionStore))
        self.onContinueAsGuest = onContinueAsGuest
        self.onAuthenticated = onAuthenticated
    }

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                if proxy.size.width >= 940 {
                    wideLayout(size: proxy.size)
                } else {
                    compactLayout
                }
            }
        }
        .navigationTitle(viewModel.mode.title)
        .onChange(of: viewModel.completed) { _, completed in
            if completed {
                onAuthenticated()
            }
        }
    }

    private func wideLayout(size: CGSize) -> some View {
        HStack(spacing: 0) {
            authIntro
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.leading, 48)
                .padding(.trailing, 34)
                .padding(.vertical, 42)

            formPanel
                .frame(width: min(max(size.width * 0.39, 440), 540))
                .frame(minHeight: size.height, alignment: .center)
                .padding(.horizontal, 34)
                .background(authPanelBackground)
        }
        .frame(maxWidth: .infinity, minHeight: size.height, alignment: .center)
    }

    private var compactLayout: some View {
        VStack(alignment: .leading, spacing: 22) {
            authIntro
            formPanel
        }
        .frame(maxWidth: 620)
        .padding(28)
        .frame(maxWidth: .infinity, alignment: .center)
    }

    private var authIntro: some View {
        VStack(alignment: .leading, spacing: 18) {
            TrailHeroCard(
                eyebrow: "寻径星野账号",
                title: "把出行准备留在同一个地方",
                subtitle: viewModel.mode == .login ? "登录后保存装备、统计和个人设置。" : "创建账号后可保存自己的出行准备。",
                chips: ["安全保存", "桌面端", "可先浏览"]
            )
            .frame(maxWidth: 760)

            HStack(alignment: .top, spacing: 14) {
                TrailMetricTile(value: "公开", label: "免登录浏览", hint: "首页、装备参考和技能内容")
                TrailMetricTile(value: "同步", label: "登录后解锁", hint: "装备库、统计和个人设置")
            }
            .frame(maxWidth: 760)

            Text("也可以先浏览公开内容；保存装备、查看个人统计和同步设置需要登录。")
                .font(.subheadline)
                .foregroundStyle(palette.textMuted)
                .frame(maxWidth: 620, alignment: .leading)
        }
    }

    private var formPanel: some View {
        VStack(alignment: .leading, spacing: 18) {
            VStack(alignment: .leading, spacing: 6) {
                Text(viewModel.mode.title)
                    .font(.title.weight(.heavy))
                    .foregroundStyle(palette.textPrimary)
                Text(viewModel.mode == .login ? "继续管理自己的装备准备。" : "创建账号后即可保存出行准备。")
                    .font(.subheadline)
                    .foregroundStyle(palette.textMuted)
            }

            Picker("方式", selection: $viewModel.mode) {
                Text("账号登录").tag(AuthMode.login)
                Text("注册账号").tag(AuthMode.register)
            }
            .pickerStyle(.segmented)

            TrailSurfaceCard {
                authFields
            }
            .textFieldStyle(.roundedBorder)

            if let message = viewModel.message {
                TrailSurfaceCard { Text(message) }
            }

            HStack(spacing: 12) {
                TrailPrimaryButton(title: viewModel.loading ? "处理中..." : viewModel.mode.title, action: { Task { await viewModel.submit() } }, isDisabled: viewModel.loading || !viewModel.canSubmit)
                TrailSoftButton(title: "暂不登录，先浏览", action: onContinueAsGuest)
            }
        }
        .frame(maxWidth: 520, alignment: .leading)
        .padding(.vertical, 34)
    }

    @ViewBuilder
    private var authFields: some View {
        if viewModel.mode == .login {
            TextField("用户名或邮箱", text: $viewModel.account)
            SecureField("密码", text: $viewModel.password)
            if viewModel.captchaTicket != nil {
                TextField("验证码", text: $viewModel.captchaAnswer)
                Text("多次登录失败时需要先完成验证码。")
                    .font(.caption)
                    .foregroundStyle(palette.textMuted)
            } else {
                Text("若多次输入错误，系统会提示验证码。")
                    .font(.caption)
                    .foregroundStyle(palette.textMuted)
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

    private var authPanelBackground: some View {
        Rectangle()
            .fill(palette.surface.opacity(palette.isDark ? 0.72 : 0.78))
            .overlay(alignment: .leading) {
                Rectangle()
                    .fill(palette.softBorder)
                    .frame(width: 1)
            }
    }
}
