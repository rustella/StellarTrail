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
            gearAtlasRepository: environment.gearAtlasRepository,
            skillRepository: environment.skillRepository,
            contentRepository: environment.contentRepository,
            tripRepository: environment.tripRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野 · 出发前检查",
                    title: "今天准备好出发了吗？",
                    subtitle: "跟着清单确认背包、技能和个人设置，轻松开始下一段路线。",
                    chips: [viewModel.state.isLoggedIn ? "我的装备已保存" : "可浏览装备图鉴", "行程提醒", "绳结教学"]
                ) {
                    TrailBadge(text: "首页 · 装备 · 行程 · 技能 · 我的", tone: .brand)
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

                if viewModel.state.isLoggedIn {
                    TripHighlightCard(item: viewModel.state.tripHighlight)
                } else {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "登录后继续准备", subtitle: "当前先开放装备图鉴和技能浏览；个人装备、打包清单和行程会保存到账号。")
                        TrailPrimaryButton(title: "账号登录") { showingAuth = true }
                    }
                }

                if !viewModel.state.atlasItems.isEmpty {
                    TrailSectionTitle(title: "装备图鉴", subtitle: "参考已收录装备的公开规格。")
                    ForEach(viewModel.state.atlasItems) { item in
                        NavigationLink(destination: GearAtlasDetailView(environment: environment, id: item.id)) {
                            AtlasPreviewCard(item: item)
                        }
                        .buttonStyle(.plain)
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
        .trailScreenBackground()
        .navigationTitle("首页")
        .task { await viewModel.load() }
        .onReceive(environment.sessionStore.$currentSession) { _ in
            Task { await viewModel.load() }
        }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .password)
        }
        .sheet(isPresented: $showingCreateGear, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack { GearFormView(environment: environment) }
                .presentationDetents([.large])
        }
    }

    private var quickActions: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
            FeatureTile(title: "装备库", subtitle: "整理背包", systemImage: "backpack.fill") {}
            FeatureTile(title: "新增装备", subtitle: viewModel.state.isLoggedIn ? "记录新物品" : "登录后可用", systemImage: "plus.circle.fill") {
                viewModel.state.isLoggedIn ? (showingCreateGear = true) : (showingAuth = true)
            }
            NavigationLink(destination: TripsView(environment: environment)) {
                FeatureTileCard(title: "行程", subtitle: "出发计划", systemImage: "map.fill")
            }
            .buttonStyle(.plain)
            NavigationLink(destination: PackingListView(environment: environment)) {
                FeatureTileCard(title: "打包清单", subtitle: "逐项确认", systemImage: "checklist")
            }
            .buttonStyle(.plain)
            NavigationLink(destination: GearAtlasListView(environment: environment)) {
                FeatureTileCard(title: "装备图鉴", subtitle: "公开规格", systemImage: "square.grid.2x2.fill")
            }
            .buttonStyle(.plain)
        }
    }
}

private struct FeatureTileCard: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let subtitle: String
    let systemImage: String

    var body: some View {
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
}

private struct FeatureTile: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let subtitle: String
    let systemImage: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            FeatureTileCard(title: title, subtitle: subtitle, systemImage: systemImage)
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

private struct TripHighlightCard: View {
    @Environment(\.trailPalette) private var palette
    let item: TripHomeHighlightItem?

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                TrailSectionTitle(title: "近期行程", subtitle: item == nil ? "暂无即将开始的行程。" : "小程序同款行程提醒。")
                Spacer()
                TrailBadge(text: item?.status.label ?? "空态", tone: item == nil ? .neutral : .warning)
            }
            if let item {
                Text(item.trip.title)
                    .font(.headline.weight(.heavy))
                    .foregroundStyle(palette.textPrimary)
                Text("\(item.trip.tripType.label) · \(item.trip.dateText) · \(item.trip.durationText)")
                    .font(.subheadline)
                    .foregroundStyle(palette.textMuted)
                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                    TrailMetricTile(value: "\(item.trip.readiness.completionPercent)%", label: "准备进度")
                    TrailMetricTile(value: item.trip.readiness.missingCount == 0 ? "完成" : "\(item.trip.readiness.missingCount) 项", label: "待确认")
                }
            } else {
                Text("创建行程后，这里会显示正在进行或即将出发的提醒。")
                    .font(.subheadline)
                    .foregroundStyle(palette.textMuted)
            }
        }
    }
}

private struct AtlasPreviewCard: View {
    @Environment(\.trailPalette) private var palette
    let item: GearAtlasPublicItem

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: item.categoryLabel, tone: .info)
                Spacer()
                TrailBadge(text: item.formattedOfficialPrice, tone: .brand)
            }
            Text(item.name)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            Text(item.brandModel.nilIfBlank ?? "未填写品牌型号")
                .font(.subheadline)
                .foregroundStyle(palette.textMuted)
            Text(item.description ?? "暂无描述")
                .font(.caption)
                .foregroundStyle(palette.textMuted)
                .lineLimit(2)
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
