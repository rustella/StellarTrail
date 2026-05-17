import SwiftUI

struct MacGearView: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: GearListViewModel
    @State private var selectedGearID: String?
    @State private var showingCreate = false

    init(environment: MacAppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearListViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            contentRepository: environment.contentRepository
        ))
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
            }
            .frame(minWidth: 420, idealWidth: 520)
            .padding(24)

            detailPane
                .frame(minWidth: 420)
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
            subtitle: viewModel.state.isLoggedIn ? "左侧筛选列表，右侧查看详情，适合桌面快速整理。" : "登录后保存自己的装备和历史记录。",
            chips: [viewModel.state.isLoggedIn ? "可用 \(viewModel.state.stats.currentCount)" : "可先浏览", "历史 \(viewModel.state.stats.archivedCount)"]
        )
    }

    private var filters: some View {
        VStack(alignment: .leading, spacing: 10) {
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
        }
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
                TrailSectionTitle(title: "选择一件装备", subtitle: "在左侧列表中选择条目后，这里会显示详细信息和操作。")
                Text("桌面端使用左右分栏，避免把手机页面直接放大。")
                    .foregroundStyle(.secondary)
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

private struct MacGearDetailPane: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: GearDetailViewModel

    init(environment: MacAppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearDetailViewModel(id: id, repository: environment.gearRepository))
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
