import SwiftUI

struct MacGearView: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: GearListViewModel
    @State private var selectedGearID: String?
    @State private var showingCreate = false
    private let onRequestLogin: () -> Void
    private let onRequestRegister: () -> Void

    init(
        environment: MacAppEnvironment,
        onRequestLogin: @escaping () -> Void = {},
        onRequestRegister: @escaping () -> Void = {}
    ) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearListViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            gearAtlasRepository: environment.gearAtlasRepository,
            contentRepository: environment.contentRepository
        ))
        self.onRequestLogin = onRequestLogin
        self.onRequestRegister = onRequestRegister
    }

    var body: some View {
        HSplitView {
            VStack(alignment: .leading, spacing: 16) {
                header
                filters
                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }
                gearList
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            }
            .frame(minWidth: 420, idealWidth: 520, maxHeight: .infinity, alignment: .topLeading)
            .padding(24)

            detailPane
                .frame(minWidth: 420, maxHeight: .infinity, alignment: .topLeading)
                .padding(24)
        }
        .navigationTitle("装备")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button { showingCreate = true } label: {
                    Label("添加装备", systemImage: "plus")
                }
                .disabled(!viewModel.state.isLoggedIn)
            }
        }
        .task { await viewModel.load() }
        .sheet(isPresented: $showingCreate, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack { GearFormViewForMac(environment: environment) }
                .frame(minWidth: 560, minHeight: 640)
        }
    }

    private var header: some View {
        TrailHeroCard(
            eyebrow: "装备库",
            title: viewModel.state.isLoggedIn ? "桌面端装备管理" : "先看看常见装备参考",
            subtitle: viewModel.state.isLoggedIn ? "左侧筛选列表，右侧查看详情，适合桌面快速整理。" : "公开清单和装备图鉴可直接浏览，登录后再保存自己的装备库。"
        )
    }

    private var filters: some View {
        VStack(alignment: .leading, spacing: 10) {
            if viewModel.state.isLoggedIn {
                Picker("分组", selection: Binding(
                    get: { viewModel.state.tab },
                    set: { tab in Task { await viewModel.refreshWith(tab: tab) } }
                )) {
                    ForEach(GearTab.allCases) { tab in Text(tab.label).tag(tab) }
                }
                .pickerStyle(.segmented)
            }

            TextField("搜索装备", text: Binding(
                get: { viewModel.state.query },
                set: { query in Task { await viewModel.refreshWith(query: query) } }
            ))
            .textFieldStyle(.roundedBorder)

            if viewModel.state.isLoggedIn && !viewModel.state.categories.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        filterChip(title: "全部", selected: viewModel.state.selectedCategory == nil) {
                            Task { await viewModel.clearCategory() }
                        }
                        ForEach(viewModel.state.categories) { filter in
                            if let category = filter.category {
                                filterChip(
                                    title: "\(filter.label) \(filter.count)",
                                    selected: viewModel.state.selectedCategory == category
                                ) {
                                    Task { await viewModel.refreshWith(category: category) }
                                }
                            }
                        }
                    }
                    .padding(.vertical, 2)
                }
            }
        }
    }

    private func filterChip(title: String, selected: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title)
                .font(.caption.weight(.bold))
                .padding(.horizontal, 12)
                .padding(.vertical, 7)
        }
        .buttonStyle(.plain)
        .foregroundStyle(selected ? Color.white : Color.primary)
        .background(selected ? Color.accentColor : Color(nsColor: .controlBackgroundColor), in: Capsule())
    }

    @ViewBuilder
    private var gearList: some View {
        if viewModel.state.isLoggedIn {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 10) {
                    ForEach(viewModel.state.gears) { gear in
                        Button { selectedGearID = gear.id } label: {
                            MacGearRow(gear: gear, selected: selectedGearID == gear.id)
                        }
                        .buttonStyle(.plain)
                    }
                    if viewModel.state.gears.isEmpty && !viewModel.state.loading {
                        TrailEmptyState(title: "暂无装备", subtitle: "添加第一件装备，开始整理出行清单。")
                    }
                    if viewModel.state.nextCursor != nil {
                        MacGearLoadMoreFooter(loading: viewModel.state.loadingMore)
                            .task { await viewModel.loadMore() }
                    }
                }
            }
        } else {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 10) {
                    ForEach(viewModel.state.templates) { template in
                        TrailSurfaceCard {
                            TrailSectionTitle(title: template.title)
                            ForEach(template.categories) { category in
                                Text("\(category.name)：\(category.items.joined(separator: "、"))")
                                    .font(.subheadline)
                            }
                        }
                    }
                    ForEach(viewModel.state.atlasItems) { item in
                        TrailSurfaceCard {
                            HStack {
                                TrailBadge(text: item.categoryLabel, tone: .info)
                                Spacer()
                                Text(item.formattedWeight)
                                    .font(.caption.weight(.bold))
                                    .foregroundStyle(.secondary)
                            }
                            Text(item.name).font(.headline.weight(.heavy))
                            Text(item.brandModel.nilIfBlank ?? "未填写品牌型号")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
    }

    @ViewBuilder
    private var detailPane: some View {
        if let selectedGearID {
            MacGearDetailPane(environment: environment, id: selectedGearID)
        } else {
            TrailSurfaceCard {
                if viewModel.state.isLoggedIn {
                    TrailSectionTitle(title: "选择一件装备", subtitle: "在左侧列表中选择条目后，这里会显示详细信息和操作。")
                    Text("桌面端使用左右分栏，避免把手机页面直接放大。")
                        .foregroundStyle(.secondary)
                } else {
                    TrailSectionTitle(title: "登录后整理自己的装备库", subtitle: "公开参考可以先看，自己的重量、价格、标签和存放位置需要登录后保存。")
                    HStack(spacing: 10) {
                        TrailPrimaryButton(title: "账号登录", action: onRequestLogin)
                        TrailSoftButton(title: "注册账号", action: onRequestRegister)
                    }
                    .frame(maxWidth: 420)
                }
            }
        }
    }
}

private struct MacGearRow: View {
    @Environment(\.trailPalette) private var palette
    let gear: GearSummary
    let selected: Bool

    var body: some View {
        TrailSurfaceCard(padding: 14) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: gear.categoryLabel, tone: .brand)
                        TrailBadge(text: gear.statusLabel, tone: gear.status.badgeTone)
                    }
                    Text(gear.name).font(.headline.weight(.heavy))
                    Text(gear.brandModel.nilIfBlank ?? "未填写品牌型号")
                        .font(.subheadline)
                        .foregroundStyle(palette.textMuted)
                }
                Spacer()
                VStack(alignment: .trailing, spacing: 6) {
                    Text(gear.formattedWeight).font(.caption.weight(.bold))
                    Text(gear.formattedPrice).font(.caption)
                }
                .foregroundStyle(palette.textMuted)
            }
        }
        .overlay(
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(selected ? palette.brand : .clear, lineWidth: 2)
        )
    }
}

private struct MacGearLoadMoreFooter: View {
    let loading: Bool

    var body: some View {
        HStack {
            Spacer()
            if loading {
                ProgressView()
                    .controlSize(.small)
                Text("加载中")
                    .font(.footnote.weight(.medium))
                    .foregroundStyle(.secondary)
            } else {
                Color.clear
                    .frame(width: 1, height: 1)
            }
            Spacer()
        }
        .frame(minHeight: 28)
        .padding(.vertical, 4)
    }
}

private struct MacGearDetailPane: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: GearDetailViewModel

    init(environment: MacAppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearDetailViewModel(
            id: id,
            repository: environment.gearRepository,
            atlasRepository: environment.gearAtlasRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if viewModel.loading { TrailLoadingState() }
                if let error = viewModel.error { TrailErrorState(message: error) { Task { await viewModel.load() } } }
                if let item = viewModel.item {
                    TrailHeroCard(
                        eyebrow: item.category.label,
                        title: item.name,
                        subtitle: item.description ?? "这件装备还没有补充说明。",
                        chips: [item.status.label, item.formattedWeight, item.formattedPrice]
                    )
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "基础信息")
                        TrailInfoRow(label: "品牌型号", value: item.brandModel.nilIfBlank ?? "未填写")
                        TrailInfoRow(label: "颜色", value: item.color ?? "未填写")
                        TrailInfoRow(label: "材质", value: item.material ?? "未填写")
                        TrailInfoRow(label: "存放位置", value: item.storageLocation ?? "未填写")
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "标签与共享")
                        TrailTagRow(tags: item.tags)
                        TrailInfoRow(label: "共享状态", value: item.shareStatus.label)
                        TrailInfoRow(label: "备注", value: item.notes ?? "未填写")
                    }
                }
            }
        }
        .task { await viewModel.load() }
    }
}
