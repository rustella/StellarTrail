import SwiftUI

struct GearFormViewForMac: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel: GearFormViewModel

    init(environment: MacAppEnvironment, item: GearItem? = nil) {
        _viewModel = StateObject(wrappedValue: GearFormViewModel(item: item, repository: environment.gearRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "装备记录",
                    title: viewModel.title,
                    subtitle: "记录分类规格、价格币种、购买渠道、彩色标签和存放位置。"
                )

                basicSection
                specsSection
                purchaseSection
                tagsSection
                notesSection

                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
            }
            .padding(22)
        }
        .navigationTitle(viewModel.title)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("关闭") { dismiss() }
            }
            ToolbarItem(placement: .confirmationAction) {
                Button(viewModel.saving ? "保存中..." : "保存装备") {
                    Task { await viewModel.save() }
                }
                .disabled(viewModel.saving)
            }
        }
        .task { await viewModel.loadOptions() }
        .onChange(of: viewModel.savedItem) { _, item in
            if item != nil { dismiss() }
        }
    }

    private var basicSection: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "基础信息")
            TextField("装备名称", text: $viewModel.draft.name)
                .trailFormField()
            Picker("分类", selection: Binding(
                get: { viewModel.draft.category },
                set: { category in Task { await viewModel.selectCategory(category) } }
            )) {
                ForEach(GearCategory.allCases) { category in
                    Text("\(category.label) · \(category.hint)").tag(category)
                }
            }
            Picker("状态", selection: $viewModel.draft.status) {
                ForEach(GearStatus.allCases) { status in Text(status.label).tag(status) }
            }
            .pickerStyle(.menu)
            TextField("品牌", text: $viewModel.draft.brand)
                .trailFormField()
            TextField("型号", text: $viewModel.draft.model)
                .trailFormField()
            TextField("描述", text: $viewModel.draft.description, axis: .vertical)
                .trailFormField()
        }
    }

    private var specsSection: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "分类规格", subtitle: viewModel.draft.category.hint)
            HStack {
                TextField("重量", text: $viewModel.draft.weightText)
                    .trailFormField()
                Picker("单位", selection: $viewModel.draft.weightUnit) {
                    ForEach(GearWeightUnit.allCases) { unit in Text(unit.label).tag(unit) }
                }
                .pickerStyle(.menu)
                .frame(width: 96)
            }

            ForEach(Array(viewModel.specFields.enumerated()), id: \.element.id) { index, field in
                VStack(alignment: .leading, spacing: 8) {
                    Text(field.label)
                        .font(.subheadline.weight(.bold))
                    if field.choiceOnly {
                        Picker(field.label, selection: Binding(
                            get: { field.unitIndex },
                            set: { unitIndex in viewModel.updateSpecUnit(key: field.key, fieldIndex: index, unitIndex: unitIndex) }
                        )) {
                            ForEach(Array(field.unitLabels.enumerated()), id: \.offset) { optionIndex, label in
                                Text(label.isEmpty ? "未选择" : label).tag(optionIndex)
                            }
                        }
                        .pickerStyle(.menu)
                    } else {
                        HStack {
                            TextField(field.placeholder, text: Binding(
                                get: { field.valueText },
                                set: { value in viewModel.updateSpecValue(key: field.key, value: value, unit: field.unitLabel) }
                            ))
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
            }
        }
    }

    private var purchaseSection: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "价格与购买")
            HStack {
                TextField("官方价", text: $viewModel.draft.officialPriceText)
                    .trailFormField()
                currencyPicker(selection: $viewModel.draft.officialPriceCurrency)
            }
            HStack {
                TextField("购入价", text: $viewModel.draft.purchasePriceText)
                    .trailFormField()
                currencyPicker(selection: $viewModel.draft.purchasePriceCurrency)
            }
            TextField("购买日期 YYYY-MM-DD", text: $viewModel.draft.purchaseDate)
                .trailFormField()
            Picker("购买渠道", selection: $viewModel.draft.purchaseLocation) {
                Text("未记录").tag("")
                ForEach(GearOptions.purchaseLocations, id: \.self) { location in
                    Text(location).tag(location)
                }
            }
            .pickerStyle(.menu)
            TextField("存放位置", text: $viewModel.draft.storageLocation)
                .trailFormField()
        }
    }

    private var tagsSection: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "标签颜色", subtitle: "最多 20 个标签，会随装备一起保存。")
            if viewModel.draft.tags.isEmpty {
                Text("暂无标签")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            } else {
                TrailColorTagRow(tags: viewModel.draft.tags)
                ForEach(viewModel.draft.tags) { tag in
                    Button("移除 \(tag.name)") { viewModel.removeTag(tag) }
                        .font(.caption.weight(.bold))
                }
            }
            TagColorSwatchPicker(selection: $viewModel.selectedTagColor)
            HStack {
                TextField("输入标签，用逗号分隔", text: $viewModel.tagInput)
                    .trailFormField()
                TrailPillButton(title: "添加", isSelected: true) {
                    viewModel.addTagsFromInput()
                }
            }
            if !viewModel.tagSuggestions.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(viewModel.tagSuggestions) { suggestion in
                            Button {
                                viewModel.addSuggestion(suggestion)
                            } label: {
                                TrailColorTag(tag: GearTagView(name: suggestion.name, color: suggestion.color))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
            Toggle("允许公开展示", isOn: $viewModel.draft.shareEnabled)
        }
    }

    private var notesSection: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "备注")
            TextField("保养记录、存放提醒或使用心得", text: $viewModel.draft.notes, axis: .vertical)
                .trailFormField()
        }
    }

    private func currencyPicker(selection: Binding<GearCurrency>) -> some View {
        Picker("币种", selection: selection) {
            ForEach(GearCurrency.allCases) { currency in Text(currency.label).tag(currency) }
        }
        .pickerStyle(.menu)
        .frame(width: 116)
    }
}

private struct TagColorSwatchPicker: View {
    @Environment(\.trailPalette) private var palette
    @Binding var selection: GearTagColor

    var body: some View {
        HStack(spacing: 10) {
            ForEach(GearTagColor.allCases) { color in
                Button {
                    selection = color
                } label: {
                    ZStack {
                        Circle()
                            .fill(color.swatchColor)
                        if selection == color {
                            Image(systemName: "checkmark")
                                .font(.caption.weight(.black))
                                .foregroundStyle(.white)
                        }
                    }
                    .frame(width: 32, height: 32)
                    .overlay {
                        Circle()
                            .stroke(selection == color ? palette.textPrimary : palette.border, lineWidth: selection == color ? 2 : 1)
                    }
                }
                .buttonStyle(.plain)
                .accessibilityLabel(color.label)
            }
        }
    }
}

private extension GearTagColor {
    var swatchColor: Color {
        switch self {
        case .teal: return Color(red: 0.05, green: 0.46, blue: 0.42)
        case .blue: return Color(red: 0.13, green: 0.32, blue: 0.75)
        case .violet: return Color(red: 0.43, green: 0.25, blue: 0.76)
        case .rose: return Color(red: 0.80, green: 0.18, blue: 0.34)
        case .orange: return Color(red: 0.76, green: 0.30, blue: 0.06)
        case .amber: return Color(red: 0.70, green: 0.40, blue: 0.02)
        case .green: return Color(red: 0.12, green: 0.45, blue: 0.20)
        case .slate: return Color(red: 0.29, green: 0.33, blue: 0.39)
        }
    }
}
