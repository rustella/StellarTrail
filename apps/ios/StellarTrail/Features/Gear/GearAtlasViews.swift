import SwiftUI

struct GearAtlasListView: View {
    @Environment(\.trailPalette) private var palette
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: GearAtlasListViewModel
    @State private var showingAuth = false

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearAtlasListViewModel(sessionStore: environment.sessionStore, repository: environment.gearAtlasRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "装备图鉴",
                    title: "装备图鉴",
                    subtitle: "浏览已收录装备的公开字段，投稿时不会提交个人备注、存放位置或购入渠道。",
                    chips: [viewModel.state.isLoggedIn ? "可投稿" : "登录后投稿", viewModel.state.sort.label]
                ) {
                    if viewModel.state.isLoggedIn {
                        NavigationLink(destination: GearAtlasSubmitView(environment: environment)) {
                            Text("投稿装备")
                                .font(.headline.weight(.bold))
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 13)
                        }
                        .buttonStyle(.plain)
                        .foregroundStyle(palette.brandText)
                        .background(palette.brand)
                        .clipShape(Capsule())
                    } else {
                        TrailPrimaryButton(title: "登录后投稿") { showingAuth = true }
                    }
                }

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }

                TrailSurfaceCard {
                    TrailSectionTitle(title: "搜索筛选")
                    TextField("搜索品牌、型号或名称", text: Binding(
                        get: { viewModel.state.query },
                        set: { query in viewModel.setQuery(query) }
                    ))
                    .trailFormField()
                    TrailPrimaryButton(title: "搜索图鉴") { Task { await viewModel.load() } }
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            TrailPillButton(title: "全部", isSelected: viewModel.state.selectedCategory == nil) {
                                Task { await viewModel.selectCategory(nil) }
                            }
                            ForEach(GearCategory.allCases) { category in
                                TrailPillButton(title: category.label, isSelected: viewModel.state.selectedCategory == category) {
                                    Task { await viewModel.selectCategory(category) }
                                }
                            }
                        }
                    }
                    Picker("排序", selection: Binding(
                        get: { viewModel.state.sort },
                        set: { sort in Task { await viewModel.selectSort(sort) } }
                    )) {
                        ForEach(GearAtlasSort.allCases) { sort in Text(sort.label).tag(sort) }
                    }
                    .pickerStyle(.menu)
                }

                ForEach(viewModel.state.items) { item in
                    NavigationLink(destination: GearAtlasDetailView(environment: environment, id: item.id)) {
                        GearAtlasCard(item: item)
                    }
                    .buttonStyle(.plain)
                }

                if viewModel.state.items.isEmpty && !viewModel.state.loading {
                    TrailEmptyState(title: "暂无图鉴内容", subtitle: "换个关键词或分类再试试。")
                }

                if viewModel.state.nextCursor != nil {
                    TrailPrimaryButton(title: viewModel.state.loadingMore ? "加载中…" : "加载更多") {
                        Task { await viewModel.loadMore() }
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("装备图鉴")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .password)
        }
    }
}

struct GearAtlasDetailView: View {
    @StateObject private var viewModel: GearAtlasDetailViewModel

    init(environment: AppEnvironment, id: String) {
        _viewModel = StateObject(wrappedValue: GearAtlasDetailViewModel(id: id, repository: environment.gearAtlasRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if viewModel.loading { TrailLoadingState() }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if let item = viewModel.item {
                    TrailHeroCard(
                        eyebrow: item.categoryLabel,
                        title: item.name,
                        subtitle: item.description ?? "这条图鉴还没有描述。",
                        chips: [item.formattedWeight, item.formattedOfficialPrice, Formatters.date(item.approvedAt)]
                    )
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "基本信息")
                        TrailInfoRow(label: "品牌型号", value: item.brandModel.nilIfBlank ?? "未填写")
                        TrailInfoRow(label: "分类", value: item.categoryLabel)
                        TrailInfoRow(label: "描述", value: item.description ?? "未填写")
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "公开规格")
                        TrailInfoRow(label: "重量", value: item.formattedWeight)
                        TrailInfoRow(label: "官方价格", value: item.formattedOfficialPrice)
                        let specs = item.specs ?? [:]
                        ForEach(GearOptions.specFieldViews(for: item.category, specs: specs).filter { specs[$0.key]?.nilIfBlank != nil }) { field in
                            TrailInfoRow(label: field.label, value: specs[field.key] ?? "未填写")
                        }
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "收录信息")
                        TrailInfoRow(label: "收录时间", value: Formatters.date(item.approvedAt))
                        TrailInfoRow(label: "更新时间", value: Formatters.date(item.updatedAt))
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle(viewModel.item?.name ?? "图鉴详情")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

struct GearAtlasSubmitView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: GearAtlasSubmitViewModel
    @State private var showingAuth = false

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearAtlasSubmitViewModel(repository: environment.gearAtlasRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "图鉴投稿",
                    title: "提交公开装备信息",
                    subtitle: "只填写可以公开展示的品牌、型号、重量、官方价和规格。",
                    chips: [viewModel.draft.category.label, "审核后收录"]
                )

                if !environment.sessionStore.isLoggedIn {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "登录后投稿", subtitle: "图鉴可以浏览，投稿需要账号登录。")
                        TrailPrimaryButton(title: "去登录") { showingAuth = true }
                    }
                } else {
                    submitForm
                }

                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("图鉴投稿")
        .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭") { dismiss() } } }
        .sheet(isPresented: $showingAuth) { AuthView(environment: environment, mode: .password) }
        .onChange(of: viewModel.completed) { _, completed in
            if completed { dismiss() }
        }
    }

    private var submitForm: some View {
        Group {
            TrailSurfaceCard {
                TrailSectionTitle(title: "基本信息")
                TextField("装备名称", text: $viewModel.draft.name)
                    .trailFormField()
                Picker("分类", selection: Binding(
                    get: { viewModel.draft.category },
                    set: { category in viewModel.selectCategory(category) }
                )) {
                    ForEach(GearCategory.allCases) { category in Text(category.label).tag(category) }
                }
                TextField("品牌", text: $viewModel.draft.brand)
                    .trailFormField()
                TextField("型号", text: $viewModel.draft.model)
                    .trailFormField()
                TextField("描述", text: $viewModel.draft.description, axis: .vertical)
                    .trailFormField()
            }

            TrailSurfaceCard {
                TrailSectionTitle(title: "公开规格")
                HStack {
                    TextField("重量", text: $viewModel.draft.weightText)
                        .keyboardType(.decimalPad)
                        .trailFormField()
                    Picker("单位", selection: $viewModel.draft.weightUnit) {
                        ForEach(GearWeightUnit.allCases) { unit in Text(unit.label).tag(unit) }
                    }
                    .pickerStyle(.menu)
                    .frame(width: 96)
                }
                HStack {
                    TextField("官方价", text: $viewModel.draft.officialPriceText)
                        .keyboardType(.decimalPad)
                        .trailFormField()
                    Picker("币种", selection: $viewModel.draft.officialPriceCurrency) {
                        ForEach(GearCurrency.allCases) { currency in Text(currency.label).tag(currency) }
                    }
                    .pickerStyle(.menu)
                    .frame(width: 116)
                }
                ForEach(Array(viewModel.specFields.enumerated()), id: \.element.id) { index, field in
                    HStack {
                        TextField(field.label, text: Binding(
                            get: { field.valueText },
                            set: { value in viewModel.updateSpecValue(key: field.key, value: value, unit: field.unitLabel) }
                        ))
                        .keyboardType(field.inputType == "number" ? .decimalPad : .default)
                        .trailFormField()
                        if !field.units.isEmpty {
                            Picker("单位", selection: Binding(
                                get: { field.unitIndex },
                                set: { unitIndex in viewModel.updateSpecUnit(key: field.key, fieldIndex: index, unitIndex: unitIndex) }
                            )) {
                                ForEach(Array(field.unitLabels.enumerated()), id: \.offset) { optionIndex, label in
                                    Text(label.isEmpty ? "无" : label).tag(optionIndex)
                                }
                            }
                            .pickerStyle(.menu)
                            .frame(width: 112)
                        }
                    }
                }
            }

            TrailPrimaryButton(title: viewModel.loading ? "提交中…" : "提交审核") {
                Task { await viewModel.submit() }
            }
            .disabled(viewModel.loading)
        }
    }
}

private struct GearAtlasCard: View {
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
            HStack {
                Text(item.formattedWeight)
                Text("收录 \(Formatters.date(item.approvedAt))")
            }
            .font(.caption.weight(.bold))
            .foregroundStyle(palette.textMuted)
        }
    }
}
