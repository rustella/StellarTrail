import SwiftUI

struct HomeView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: HomeViewModel
    @State private var showingAuth = false
    @State private var showingCreateGear = false

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: HomeViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            skillRepository: environment.skillRepository,
            contentRepository: environment.contentRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野 · 出发前检查",
                    title: "今天准备好出发了吗？",
                    subtitle: "跟着清单确认背包、技能和个人设置，轻松开始下一段路线。",
                    chips: [viewModel.state.isLoggedIn ? "我的装备已保存" : "可先浏览清单", "绳结教学可直接看"]
                ) {
                    HStack(spacing: 10) {
                        TrailPrimaryButton(title: "查看装备") {}
                        TrailSoftButton(title: "学习技能") {}
                    }
                }

                quickActions

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading {
                    TrailLoadingState()
                }

                GearOverviewCard(stats: viewModel.state.stats, isLoggedIn: viewModel.state.isLoggedIn) {
                    showingAuth = true
                }

                if viewModel.state.isLoggedIn && !viewModel.state.recentGears.isEmpty {
                    TrailSectionTitle(title: "最近装备", subtitle: "快速查看近期更新。")
                    ForEach(viewModel.state.recentGears) { gear in
                        GearPreviewCard(gear: gear)
                    }
                }

                TrailSectionTitle(title: "出行装备参考", subtitle: "按场景准备背包，登录后保存自己的清单。")
                ForEach(viewModel.state.templates) { template in
                    GearTemplateCard(template: template)
                }

                TrailSectionTitle(title: "户外技能", subtitle: "出发前先掌握常用绳结与营地技能。")
                ForEach(viewModel.state.skills) { skill in
                    SkillPreviewCard(skill: skill)
                }
            }
            .padding(16)
        }
        .navigationTitle("首页")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .login)
        }
        .sheet(isPresented: $showingCreateGear, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack { GearFormView(environment: environment) }
        }
    }

    private var quickActions: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
            FeatureTile(title: "装备库", subtitle: "整理背包", systemImage: "backpack.fill") {}
            FeatureTile(title: "新增装备", subtitle: viewModel.state.isLoggedIn ? "记录新物品" : "登录后可用", systemImage: "plus.circle.fill") {
                viewModel.state.isLoggedIn ? (showingCreateGear = true) : (showingAuth = true)
            }
            FeatureTile(title: "技能", subtitle: "绳结步骤", systemImage: "figure.hiking") {}
            FeatureTile(title: "我的", subtitle: viewModel.state.isLoggedIn ? "已登录" : "待登录", systemImage: "person.crop.circle") {
                if !viewModel.state.isLoggedIn { showingAuth = true }
            }
        }
    }
}

private struct FeatureTile: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let subtitle: String
    let systemImage: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            TrailSurfaceCard(padding: 14) {
                Image(systemName: systemImage)
                    .font(.title3.weight(.bold))
                    .foregroundStyle(palette.brand)
                Text(title)
                    .font(.headline.weight(.heavy))
                    .foregroundStyle(palette.textPrimary)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(palette.textMuted)
            }
        }
        .buttonStyle(.plain)
    }
}

private struct GearOverviewCard: View {
    @Environment(\.trailPalette) private var palette
    let stats: GearStatsResponse
    let isLoggedIn: Bool
    let login: () -> Void

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                TrailSectionTitle(title: "装备概览", subtitle: isLoggedIn ? "本次出行前先看关键指标。" : "登录后保存自己的装备进度。")
                Spacer()
                TrailBadge(text: isLoggedIn ? "已同步" : "先浏览", tone: isLoggedIn ? .success : .info)
            }
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                TrailMetricTile(value: "\(stats.currentCount)", label: "可用装备")
                TrailMetricTile(value: Formatters.weight(stats.totalWeightG), label: "总重量")
                TrailMetricTile(value: Formatters.price(stats.totalValueCents), label: "估算价值")
                TrailMetricTile(value: "\(stats.archivedCount)", label: "历史记录")
            }
            if !isLoggedIn {
                TrailPrimaryButton(title: "账号登录", action: login)
            }
        }
    }
}

private struct GearPreviewCard: View {
    @Environment(\.trailPalette) private var palette
    let gear: GearSummary

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    TrailBadge(text: gear.categoryLabel, tone: .brand)
                    Text(gear.name)
                        .font(.headline.weight(.heavy))
                        .foregroundStyle(palette.textPrimary)
                    Text(gear.brandModel.nilIfBlank ?? "未填写品牌型号")
                        .font(.subheadline)
                        .foregroundStyle(palette.textMuted)
                }
                Spacer()
                TrailBadge(text: gear.statusLabel, tone: gear.status.badgeTone)
            }
        }
    }
}

struct GearTemplateCard: View {
    @Environment(\.trailPalette) private var palette
    let template: GearTemplate

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: "出行装备参考", tone: .info)
                Spacer()
                TrailBadge(text: "\(template.categories.reduce(0) { $0 + $1.items.count }) 项", tone: .neutral)
            }
            Text(template.title)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            ForEach(template.categories.prefix(3)) { category in
                VStack(alignment: .leading, spacing: 4) {
                    Text(category.name)
                        .font(.subheadline.weight(.bold))
                        .foregroundStyle(palette.headingMuted)
                    Text(category.items.joined(separator: "、"))
                        .font(.caption)
                        .foregroundStyle(palette.textMuted)
                }
            }
        }
    }
}

private struct SkillPreviewCard: View {
    @Environment(\.trailPalette) private var palette
    let skill: SkillCategorySummary

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: "技能分类", tone: .brand)
                Spacer()
                TrailBadge(text: "\(skill.itemCount) 项", tone: .info)
            }
            Text(skill.title)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            Text(skill.summary)
                .font(.subheadline)
                .foregroundStyle(palette.textMuted)
        }
    }
}
