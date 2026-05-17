import SwiftUI

struct GearListView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: GearListViewModel
    @State private var showingAuth = false
    @State private var showingCreate = false

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearListViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            contentRepository: environment.contentRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野装备库",
                    title: "装备",
                    subtitle: viewModel.state.isLoggedIn ? "按分类、状态和重量管理自己的出行装备。" : "先看看常见清单，登录后保存自己的装备。",
                    chips: [viewModel.state.isLoggedIn ? "可用 \(viewModel.state.stats.currentCount)" : "先浏览", "历史 \(viewModel.state.stats.archivedCount)"]
                ) {
                    if viewModel.state.isLoggedIn {
                        TrailPrimaryButton(title: "添加装备") { showingCreate = true }
                    } else {
                        TrailPrimaryButton(title: "账号登录") { showingAuth = true }
                    }
                }

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }

                if viewModel.state.isLoggedIn {
                    signedInContent
                } else {
                    guestContent
                }
            }
            .padding(16)
        }
        .navigationTitle("装备")
        .toolbar {
            if viewModel.state.isLoggedIn {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("添加") { showingCreate = true }
                }
            }
        }
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .login)
        }
        .sheet(isPresented: $showingCreate, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack { GearFormView(environment: environment) }
        }
    }

    private var guestContent: some View {
        Group {
            TrailSectionTitle(title: "出行装备参考", subtitle: "按场景准备背包，登录后保存自己的清单。")
            ForEach(viewModel.state.templates) { template in
                GearTemplateCard(template: template)
            }
        }
    }

    private var signedInContent: some View {
        Group {
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                TrailMetricTile(value: "\(viewModel.state.stats.currentCount)", label: "可用装备")
                TrailMetricTile(value: "\(viewModel.state.stats.archivedCount)", label: "历史记录")
                TrailMetricTile(value: Formatters.weight(viewModel.state.stats.totalWeightG), label: "总重量")
                TrailMetricTile(value: Formatters.price(viewModel.state.stats.totalValueCents), label: "估算价值")
            }

            Picker("分组", selection: Binding(
                get: { viewModel.state.tab },
                set: { tab in Task { await viewModel.refreshWith(tab: tab) } }
            )) {
                ForEach(GearTab.allCases) { tab in Text(tab.label).tag(tab) }
            }
            .pickerStyle(.segmented)

            TextField("搜索装备", text: Binding(
                get: { viewModel.state.query },
                set: { query in Task { await viewModel.refreshWith(query: query) } }
            ))
            .textFieldStyle(.roundedBorder)

            filterRows

            if viewModel.state.gears.isEmpty && !viewModel.state.loading {
                TrailEmptyState(title: "暂无装备", subtitle: "添加第一件装备，开始整理你的出行清单。")
            }

            ForEach(viewModel.state.gears) { gear in
                NavigationLink(destination: GearDetailView(environment: environment, id: gear.id)) {
                    GearListCard(gear: gear)
                }
                .buttonStyle(.plain)
            }

            if viewModel.state.nextCursor != nil {
                TrailPrimaryButton(title: viewModel.state.loadingMore ? "加载中…" : "加载更多") {
                    Task { await viewModel.loadMore() }
                }
            }
        }
    }

    private var filterRows: some View {
        VStack(alignment: .leading, spacing: 10) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    Button("全部装备") { Task { await viewModel.clearCategory() } }
                        .buttonStyle(.borderedProminent)
                    ForEach(GearCategory.allCases) { category in
                        Button(category.label) { Task { await viewModel.refreshWith(category: category) } }
                            .buttonStyle(.bordered)
                    }
                }
            }
            Picker("排序", selection: Binding(
                get: { viewModel.state.sort },
                set: { sort in Task { await viewModel.refreshWith(sort: sort) } }
            )) {
                ForEach(GearSort.allCases) { sort in Text(sort.label).tag(sort) }
            }
            .pickerStyle(.menu)
        }
    }
}

private struct GearListCard: View {
    @Environment(\.trailPalette) private var palette
    let gear: GearSummary

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: gear.categoryLabel, tone: .brand)
                        TrailBadge(text: gear.statusLabel, tone: gear.status.badgeTone)
                    }
                    Text(gear.name)
                        .font(.headline.weight(.heavy))
                        .foregroundStyle(palette.textPrimary)
                    Text(gear.brandModel.nilIfBlank ?? "未填写品牌型号")
                        .font(.subheadline)
                        .foregroundStyle(palette.textMuted)
                    HStack {
                        Text(gear.formattedWeight)
                        Text(gear.formattedPrice)
                    }
                    .font(.caption.weight(.bold))
                    .foregroundStyle(palette.textMuted)
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .foregroundStyle(palette.textMuted)
            }
        }
        .accessibilityLabel(gear.name)
    }
}
