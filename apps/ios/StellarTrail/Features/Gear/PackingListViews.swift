import SwiftUI

struct GearStatsView: View {
    @StateObject private var viewModel: GearStatsViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: GearStatsViewModel(repository: environment.gearRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "装备统计",
                    title: "出发前先看总览",
                    subtitle: "对齐小程序端的装备统计信息，帮助判断重量、价值和分类覆盖。"
                )
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                TrailSurfaceCard {
                    TrailSectionTitle(title: "关键指标")
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                        TrailMetricTile(value: "\(viewModel.stats.currentCount)", label: "可用装备")
                        TrailMetricTile(value: "\(viewModel.stats.archivedCount)", label: "历史记录")
                        TrailMetricTile(value: Formatters.weight(viewModel.stats.totalWeightG), label: "总重量")
                        TrailMetricTile(value: Formatters.price(viewModel.stats.totalValueCents), label: "估算价值")
                    }
                }
                TrailSurfaceCard {
                    TrailSectionTitle(title: "分类")
                    if viewModel.stats.byCategory.isEmpty {
                        Text("暂无分类统计。")
                    } else {
                        ForEach(viewModel.stats.byCategory) { item in
                            TrailInfoRow(label: item.label, value: "\(item.count)")
                        }
                    }
                }
                TrailSurfaceCard {
                    TrailSectionTitle(title: "状态")
                    if viewModel.stats.byStatus.isEmpty {
                        Text("暂无状态统计。")
                    } else {
                        ForEach(viewModel.stats.byStatus) { item in
                            TrailInfoRow(label: item.label, value: "\(item.count)")
                        }
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("装备统计")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

struct PackingListView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: PackingListViewModel
    @State private var showingAuth = false
    @State private var showingCreate = false

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: PackingListViewModel(sessionStore: environment.sessionStore, repository: environment.gearPackingRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "出发前准备",
                    title: "打包清单",
                    subtitle: "按清单挑选个人装备，出发前逐项确认。",
                    chips: [viewModel.isLoggedIn ? "已同步" : "登录后保存", "\(viewModel.lists.count) 份清单"]
                ) {
                    TrailPrimaryButton(title: viewModel.isLoggedIn ? "新建清单" : "账号登录") {
                        viewModel.isLoggedIn ? (showingCreate = true) : (showingAuth = true)
                    }
                }

                if !viewModel.isLoggedIn {
                    TrailEmptyState(title: "登录后使用打包清单", subtitle: "清单会保存到你的账号里，方便重复准备。")
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                if viewModel.isLoggedIn && viewModel.lists.isEmpty && !viewModel.loading {
                    TrailEmptyState(title: "还没有打包清单", subtitle: "例如“一日武功山”，只挑当天需要带的小包装备。")
                }
                ForEach(viewModel.lists) { list in
                    NavigationLink(destination: PackingDetailView(environment: environment, id: list.id)) {
                        PackingListCard(list: list)
                    }
                    .buttonStyle(.plain)
                    .accessibilityIdentifier("packing-row-\(list.id)")
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("打包清单")
        .toolbar {
            if viewModel.isLoggedIn {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("新建") { showingCreate = true }
                }
            }
        }
        .task { await viewModel.load() }
        .onReceive(environment.sessionStore.$currentSession) { _ in Task { await viewModel.load() } }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .password)
        }
        .sheet(isPresented: $showingCreate, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack {
                PackingListFormView(viewModel: viewModel) { showingCreate = false }
            }
            .presentationDetents([.medium, .large])
        }
    }
}

private struct PackingListFormView: View {
    @ObservedObject var viewModel: PackingListViewModel
    let close: () -> Void

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "新建", title: "制作打包清单", subtitle: "名称、路线和天数会显示在清单卡片上。")
                TrailSurfaceCard {
                    TextField("清单名称", text: $viewModel.nameDraft)
                        .trailFormField()
                    TextField("路线名称", text: $viewModel.routeNameDraft)
                        .trailFormField()
                    TextField("时长，例如 2 天 1 夜", text: $viewModel.durationDraft)
                        .trailFormField()
                    TrailPrimaryButton(title: "保存") {
                        Task {
                            _ = await viewModel.create()
                            close()
                        }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("新建清单")
        .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭", action: close) } }
    }
}

struct PackingDetailView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: PackingDetailViewModel

    init(environment: AppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: PackingDetailViewModel(id: id, repository: environment.gearPackingRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if let detail = viewModel.detail {
                    TrailHeroCard(
                        eyebrow: "打包清单",
                        title: detail.name,
                        subtitle: detail.metaText.nilIfBlank ?? "逐项确认个人装备。",
                        chips: [detail.progressText, detail.weightText]
                    )
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "进度")
                        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                            TrailMetricTile(value: "\(detail.stats.itemCount)", label: "装备")
                            TrailMetricTile(value: "\(detail.stats.packedCount)", label: "已打包")
                            TrailMetricTile(value: detail.weightText, label: "总重量")
                        }
                    }
                    NavigationLink(destination: PackingSelectGearView(environment: environment, packingListID: detail.id)) {
                        TrailSurfaceCard {
                            HStack {
                                TrailSectionTitle(title: "挑选装备", subtitle: "从现有装备库加入这份清单。")
                                Spacer()
                                Image(systemName: "chevron.right")
                            }
                        }
                    }
                    .buttonStyle(.plain)
                    if detail.items.isEmpty {
                        TrailEmptyState(title: "暂无装备", subtitle: "先从装备库挑选要带的物品。")
                    } else {
                        ForEach(detail.items) { item in
                            PackingItemCard(item: item) {
                                Task { await viewModel.toggle(item) }
                            } remove: {
                                Task { await viewModel.remove(item) }
                            }
                        }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("清单详情")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

struct PackingSelectGearView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: PackingSelectGearViewModel

    init(environment: AppEnvironment, packingListID: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: PackingSelectGearViewModel(packingListID: packingListID, gearRepository: environment.gearRepository, packingRepository: environment.gearPackingRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "挑选装备", title: "从装备库加入清单", subtitle: "可一次选择多件可用装备。", chips: ["已选 \(viewModel.selectedIDs.count)"])
                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
                if viewModel.loading { TrailLoadingState() }
                ForEach(viewModel.gears) { gear in
                    Button { viewModel.toggle(id: gear.id) } label: {
                        GearSelectCard(gear: gear, selected: viewModel.selectedIDs.contains(gear.id))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("挑选装备")
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("加入") {
                    Task {
                        if await viewModel.addSelected() != nil {
                            dismiss()
                        }
                    }
                }
            }
        }
        .task { await viewModel.load() }
    }
}

private struct PackingListCard: View {
    let list: GearPackingListSummary

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    TrailBadge(text: "打包清单", tone: .brand)
                    Text(list.name)
                        .font(.headline.weight(.heavy))
                    if !list.metaText.isEmpty {
                        Text(list.metaText)
                            .font(.subheadline)
                    }
                }
                Spacer()
                TrailBadge(text: list.progressText, tone: .info)
            }
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                TrailMetricTile(value: "\(list.itemCount)", label: "装备")
                TrailMetricTile(value: "\(list.packedCount)", label: "已打包")
                TrailMetricTile(value: list.weightText, label: "总重量")
            }
        }
    }
}

private struct PackingItemCard: View {
    let item: GearPackingListItem
    let toggle: () -> Void
    let remove: () -> Void

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: item.gear.categoryLabel, tone: .info)
                        TrailBadge(text: item.packed ? "已打包" : "待确认", tone: item.packed ? .success : .warning)
                    }
                    Text(item.gear.name)
                        .font(.headline.weight(.heavy))
                    Text(item.gear.brandModel.nilIfBlank ?? "未填写品牌型号")
                        .font(.subheadline)
                    Text("\(item.plannedText) · \(item.packedText) · \(item.gear.formattedWeight)")
                        .font(.caption.weight(.bold))
                }
                Spacer()
            }
            HStack(spacing: 10) {
                TrailPrimaryButton(title: item.packed ? "取消打包" : "标记已打包", action: toggle)
                TrailSoftButton(title: "移除", action: remove)
            }
        }
    }
}

private struct GearSelectCard: View {
    let gear: GearSummary
    let selected: Bool

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                Image(systemName: selected ? "checkmark.circle.fill" : "circle")
                    .font(.title3.weight(.bold))
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: gear.categoryLabel, tone: .brand)
                        TrailBadge(text: gear.statusLabel, tone: gear.status.badgeTone)
                    }
                    Text(gear.name)
                        .font(.headline.weight(.heavy))
                    Text("\(gear.brandModel.nilIfBlank ?? "未填写品牌型号") · \(gear.formattedWeight)")
                        .font(.subheadline)
                }
                Spacer()
            }
        }
    }
}
