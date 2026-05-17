import SwiftUI

struct MacProfileView: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: ProfileViewModel

    init(environment: MacAppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: ProfileViewModel(settingsStore: environment.settingsStore, sessionStore: environment.sessionStore))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 18) {
                TrailHeroCard(
                    eyebrow: "账号与设置",
                    title: "我的",
                    subtitle: "管理登录状态、主题和本地连接地址。",
                    chips: [viewModel.themeMode.label, viewModel.session == nil ? "待登录" : "已登录"]
                )

                if let session = viewModel.session {
                    TrailSurfaceCard {
                        HStack {
                            TrailBadge(text: "已登录", tone: .success)
                            Spacer()
                            TrailBadge(text: viewModel.themeMode.label, tone: .info)
                        }
                        Text(session.user.displayName).font(.title2.weight(.heavy))
                        TrailInfoRow(label: "用户 ID", value: session.user.id)
                        TrailInfoRow(label: "邮箱", value: session.user.email ?? "未绑定")
                    }
                } else {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "登录后保存自己的准备进度", subtitle: "当前仍可浏览首页、装备参考和技能内容。")
                    }
                }

                TrailSurfaceCard {
                    TrailSectionTitle(title: "主题", subtitle: "浅色清爽卡片与深色星空渐变都已准备好。")
                    HStack(spacing: 8) {
                        ForEach(AppThemeMode.allCases) { mode in
                            if viewModel.themeMode == mode {
                                Button(mode.label) { viewModel.themeMode = mode }
                                    .buttonStyle(.borderedProminent)
                            } else {
                                Button(mode.label) { viewModel.themeMode = mode }
                                    .buttonStyle(.bordered)
                            }
                        }
                    }
                }

                if viewModel.canEditBaseURL {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "本地调试地址")
                        TextField("连接地址", text: $viewModel.baseURLString)
                            .textFieldStyle(.roundedBorder)
                        HStack(spacing: 10) {
                            TrailPrimaryButton(title: "保存") { viewModel.updateBaseURL() }
                            TrailSoftButton(title: "恢复默认") { viewModel.resetBaseURL() }
                        }
                    }
                }

                if viewModel.session != nil {
                    TrailSoftButton(title: "退出登录") { viewModel.logout() }
                }
            }
            .padding(28)
        }
        .navigationTitle("我的")
    }
}
