import SwiftUI
import PhotosUI

struct ProfileView: View {
    @Environment(\.trailPalette) private var palette
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: ProfileViewModel
    @State private var showingAuth = false
    @State private var selectedPhoto: PhotosPickerItem?

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: ProfileViewModel(settingsStore: environment.settingsStore, sessionStore: environment.sessionStore, authRepository: environment.authRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野账号",
                    title: "我的",
                    subtitle: "管理账号资料、头像、邮箱绑定、主题和本地调试地址。",
                    chips: [viewModel.themeMode.label, viewModel.session == nil ? "待登录" : "已登录"]
                ) {
                    if viewModel.session == nil {
                        TrailPrimaryButton(title: "去登录") { showingAuth = true }
                    }
                }

                if let session = viewModel.session {
                    TrailSurfaceCard {
                        HStack(alignment: .top, spacing: 14) {
                            AvatarView(user: session.user)
                            VStack(alignment: .leading, spacing: 8) {
                                Text(session.user.displayName)
                                    .font(.title3.weight(.heavy))
                                TrailInfoRow(label: "用户 ID", value: session.user.id)
                                TrailInfoRow(label: "邮箱", value: session.user.email ?? "未绑定")
                            }
                        }
                        HStack {
                            TrailBadge(text: "已登录", tone: .success)
                            Spacer()
                            TrailBadge(text: viewModel.themeMode.label, tone: .info)
                        }
                        PhotosPicker(selection: $selectedPhoto, matching: .images) {
                            Text("选择系统照片作为头像")
                                .font(.headline.weight(.bold))
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 12)
                        }
                        .buttonStyle(.plain)
                        .foregroundStyle(palette.brandSoftText)
                        .background(palette.brandSoft)
                        .clipShape(Capsule())
                    }

                    TrailSurfaceCard {
                        TrailSectionTitle(title: "昵称")
                        TextField("昵称", text: $viewModel.nicknameDraft)
                            .trailFormField()
                        TrailPrimaryButton(title: "保存昵称") { viewModel.saveNicknameLocally() }
                    }

                    TrailSurfaceCard {
                        TrailSectionTitle(title: "邮箱绑定")
                        TextField("邮箱", text: $viewModel.emailDraft)
                            .keyboardType(.emailAddress)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .trailFormField()
                        HStack {
                            TextField("验证码", text: $viewModel.emailCode)
                                .keyboardType(.numberPad)
                                .trailFormField()
                            TrailPillButton(title: "发送验证码") {
                                Task { await viewModel.sendBindEmailCode() }
                            }
                        }
                        TrailPrimaryButton(title: "绑定邮箱") { Task { await viewModel.bindEmail() } }
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
                            TrailPillButton(title: mode.label, isSelected: viewModel.themeMode == mode) {
                                viewModel.themeMode = mode
                            }
                        }
                    }
                }

                if viewModel.canEditBaseURL {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "本地调试地址")
                        TextField("连接地址", text: $viewModel.baseURLString)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .trailFormField()
                        HStack(spacing: 10) {
                            TrailPrimaryButton(title: "保存") { viewModel.updateBaseURL() }
                            TrailSoftButton(title: "恢复默认") { viewModel.resetBaseURL() }
                        }
                    }
                }

                if viewModel.session != nil {
                    if let message = viewModel.message {
                        TrailSurfaceCard { Text(message) }
                    }
                    TrailSoftButton(title: "退出登录") { viewModel.logout() }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("我的")
        .task { await viewModel.refreshProfile() }
        .onChange(of: selectedPhoto) { _, item in
            guard let item else { return }
            Task {
                if let data = try? await item.loadTransferable(type: Data.self) {
                    await viewModel.uploadAvatar(data: data)
                }
            }
        }
        .sheet(isPresented: $showingAuth) {
            AuthView(environment: environment, mode: .password)
        }
    }
}

private struct AvatarView: View {
    @Environment(\.trailPalette) private var palette
    let user: UserProfile

    var body: some View {
        ZStack {
            Circle().fill(palette.brandSoft)
            if let avatarUrl = user.avatarUrl, let url = URL(string: avatarUrl) {
                AsyncImage(url: url) { phase in
                    switch phase {
                    case .success(let image):
                        image.resizable().scaledToFill()
                    default:
                        Text(user.avatarInitial)
                            .font(.title2.weight(.heavy))
                            .foregroundStyle(palette.brandSoftText)
                    }
                }
            } else {
                Text(user.avatarInitial)
                    .font(.title2.weight(.heavy))
                    .foregroundStyle(palette.brandSoftText)
            }
        }
        .frame(width: 64, height: 64)
        .clipShape(Circle())
    }
}
