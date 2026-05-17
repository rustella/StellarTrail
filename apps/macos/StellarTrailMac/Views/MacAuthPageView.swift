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
                layout(size: proxy.size)
            }
        }
        .navigationTitle(viewModel.mode.title)
        .onChange(of: viewModel.completed) { _, completed in
            if completed {
                onAuthenticated()
            }
        }
    }

    @ViewBuilder
    private func layout(size: CGSize) -> some View {
        if size.width >= 780 {
            wideLayout(size: size)
        } else {
            compactLayout(size: size)
        }
    }

    private func wideLayout(size: CGSize) -> some View {
        let shellWidth = min(size.width - 64, 1040)
        let innerWidth = shellWidth - 56
        let brandWidth = innerWidth * 0.38
        let formWidth = min(max(innerWidth - brandWidth - 30, 430), 560)
        let shellMinHeight: CGFloat = viewModel.mode == .register ? 590 : 540

        return centeredPage(size: size) {
            authShell {
                HStack(alignment: .center, spacing: 30) {
                    brandPanel
                        .frame(width: brandWidth, alignment: .leading)
                    formPanel
                        .frame(width: formWidth, alignment: .leading)
                }
                .padding(28)
                .frame(width: shellWidth)
                .frame(minHeight: shellMinHeight, alignment: .center)
            }
        }
    }

    private func compactLayout(size: CGSize) -> some View {
        centeredPage(size: size) {
            authShell {
                VStack(alignment: .leading, spacing: 24) {
                    brandPanel
                    formPanel
                }
                .padding(22)
                .frame(maxWidth: 620, alignment: .leading)
            }
        }
    }

    private func centeredPage<Content: View>(size: CGSize, @ViewBuilder content: () -> Content) -> some View {
        content()
            .padding(.horizontal, 32)
            .padding(.vertical, 34)
            .frame(maxWidth: .infinity)
            .frame(minHeight: size.height, alignment: .center)
    }

    private func authShell<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        content()
            .background(
                RoundedRectangle(cornerRadius: 28, style: .continuous)
                    .fill(palette.surface.opacity(palette.isDark ? 0.76 : 0.90))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 28, style: .continuous)
                    .stroke(palette.softBorder, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 28, style: .continuous))
            .shadow(color: !palette.isDark ? Color.black.opacity(0.07) : Color.clear, radius: 26, x: 0, y: 16)
    }

    private var brandPanel: some View {
        VStack(alignment: .leading, spacing: 18) {
            brandVisual
            VStack(alignment: .leading, spacing: 12) {
                AuthBenefitRow(
                    icon: "eye.fill",
                    title: "免登录浏览",
                    subtitle: "先查看首页、装备参考和技能内容。"
                )
                AuthBenefitRow(
                    icon: "icloud.fill",
                    title: "登录后同步",
                    subtitle: "保存装备、统计和个人设置。"
                )
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            Text("也可以先浏览公开内容，之后再登录保存自己的出行准备。")
                .font(.subheadline)
                .foregroundStyle(palette.textMuted)
                .fixedSize(horizontal: false, vertical: true)
        }
    }

    private var brandVisual: some View {
        ZStack(alignment: .bottomLeading) {
            LinearGradient(
                colors: [palette.heroStart, palette.heroMid.opacity(0.92), palette.heroEnd],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            AuthBrandDecoration()
            VStack(alignment: .leading, spacing: 11) {
                Text("寻径星野账号")
                    .font(.caption.weight(.heavy))
                    .foregroundStyle(palette.brandSoftText)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(.white.opacity(!palette.isDark ? 0.66 : 0.12))
                    .clipShape(Capsule())
                Text("把出行准备留在同一个地方")
                    .font(.title2.weight(.heavy))
                    .foregroundStyle(!palette.isDark ? palette.textPrimary : .white)
                    .fixedSize(horizontal: false, vertical: true)
                Text(viewModel.mode == .login ? "登录后保存装备、统计和个人设置。" : "创建账号后可保存自己的出行准备。")
                    .font(.subheadline)
                    .foregroundStyle(!palette.isDark ? palette.textMuted : .white.opacity(0.84))
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(22)
        }
        .frame(height: 292)
        .frame(maxWidth: .infinity, alignment: .center)
        .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
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

            formFieldGroup {
                authFields
            }
            .textFieldStyle(.roundedBorder)

            if let message = viewModel.message {
                messagePanel(message)
            }

            actionButtons
        }
        .frame(maxWidth: 560, alignment: .leading)
    }

    private func formFieldGroup<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            content()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(16)
        .background(palette.controlBackground.opacity(palette.isDark ? 0.50 : 0.72))
        .overlay(
            RoundedRectangle(cornerRadius: 20, style: .continuous)
                .stroke(palette.softBorder, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private func messagePanel(_ message: String) -> some View {
        Text(message)
            .font(.subheadline)
            .foregroundStyle(palette.textMuted)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(14)
            .background(palette.controlBackground.opacity(palette.isDark ? 0.45 : 0.64))
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }

    private var actionButtons: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 12) {
                submitButton
                guestButton
            }
            VStack(spacing: 10) {
                submitButton
                guestButton
            }
        }
    }

    private var submitButton: some View {
        TrailPrimaryButton(
            title: viewModel.loading ? "处理中..." : viewModel.mode.title,
            action: { Task { await viewModel.submit() } },
            isDisabled: viewModel.loading || !viewModel.canSubmit
        )
    }

    private var guestButton: some View {
        TrailSoftButton(title: "暂不登录，先浏览", action: onContinueAsGuest)
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
}

private struct AuthBrandDecoration: View {
    @Environment(\.trailPalette) private var palette

    var body: some View {
        GeometryReader { proxy in
            let width = proxy.size.width
            let height = proxy.size.height
            ZStack {
                Circle()
                    .fill(palette.heroSun.opacity(0.86))
                    .frame(width: 48, height: 48)
                    .position(x: width - 48, y: 48)
                Circle()
                    .fill(palette.heroSun.opacity(0.22))
                    .frame(width: 92, height: 92)
                    .position(x: width - 48, y: 48)
                Path { path in
                    path.move(to: CGPoint(x: width * 0.26, y: height))
                    path.addQuadCurve(to: CGPoint(x: width, y: height * 0.68), control: CGPoint(x: width * 0.70, y: height * 0.38))
                    path.addLine(to: CGPoint(x: width, y: height))
                    path.closeSubpath()
                }
                .fill(palette.heroHill.opacity(0.42))
                Path { path in
                    path.move(to: CGPoint(x: 0, y: height))
                    path.addQuadCurve(to: CGPoint(x: width, y: height * 0.82), control: CGPoint(x: width * 0.55, y: height * 0.58))
                    path.addLine(to: CGPoint(x: width, y: height))
                    path.closeSubpath()
                }
                .fill(palette.brandSoft.opacity(0.36))
            }
        }
        .allowsHitTesting(false)
    }
}

private struct AuthBenefitRow: View {
    @Environment(\.trailPalette) private var palette
    let icon: String
    let title: String
    let subtitle: String

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 14, weight: .bold))
                .foregroundStyle(palette.brandSoftText)
                .frame(width: 28, height: 28)
                .background(palette.brandSoft)
                .clipShape(Circle())
            VStack(alignment: .leading, spacing: 3) {
                Text(title)
                    .font(.subheadline.weight(.heavy))
                    .foregroundStyle(palette.textPrimary)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(palette.textMuted)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}
