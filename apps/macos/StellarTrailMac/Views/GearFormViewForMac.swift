import SwiftUI

struct GearFormViewForMac: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel: GearFormViewModel

    init(environment: MacAppEnvironment, item: GearItem? = nil) {
        _viewModel = StateObject(wrappedValue: GearFormViewModel(item: item, repository: environment.gearRepository))
    }

    var body: some View {
        Form {
            Section("基础信息") {
                TextField("装备名称", text: $viewModel.draft.name)
                Picker("分类", selection: $viewModel.draft.category) {
                    ForEach(GearCategory.allCases) { category in Text(category.label).tag(category) }
                }
                Picker("状态", selection: Binding(
                    get: { viewModel.draft.status ?? .available },
                    set: { viewModel.draft.status = $0 }
                )) {
                    ForEach(GearStatus.allCases) { status in Text(status.label).tag(status) }
                }
                TextField("品牌", text: optionalText($viewModel.draft.brand))
                TextField("型号", text: optionalText($viewModel.draft.model))
                TextField("颜色", text: optionalText($viewModel.draft.color))
            }
            Section("规格购买") {
                TextField("重量 g", text: $viewModel.weightText)
                TextField("价格 元", text: $viewModel.priceText)
                TextField("容量", text: optionalText($viewModel.draft.capacity))
                TextField("尺码", text: optionalText($viewModel.draft.size))
                TextField("购买地点", text: optionalText($viewModel.draft.purchaseLocation))
            }
            Section("库存管理") {
                TextField("存放位置", text: optionalText($viewModel.draft.storageLocation))
                TextField("标签，用逗号分隔", text: $viewModel.tagsText)
                Toggle("允许公开展示", isOn: Binding(
                    get: { viewModel.draft.shareEnabled ?? false },
                    set: { viewModel.draft.shareEnabled = $0 }
                ))
                TextField("备注", text: optionalText($viewModel.draft.notes), axis: .vertical)
            }
            if let error = viewModel.error {
                Section { Text(error).foregroundStyle(.red) }
            }
        }
        .navigationTitle(viewModel.title)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("关闭") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) {
                Button(viewModel.loading ? "保存中…" : "保存装备") { Task { await viewModel.save() } }
                    .disabled(viewModel.loading)
            }
        }
        .onChange(of: viewModel.savedItem) { _, item in
            if item != nil { dismiss() }
        }
    }

    private func optionalText(_ value: Binding<String?>) -> Binding<String> {
        Binding(
            get: { value.wrappedValue ?? "" },
            set: { value.wrappedValue = $0.nilIfBlank }
        )
    }
}
