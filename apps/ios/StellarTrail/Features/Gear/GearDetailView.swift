import SwiftUI

struct GearDetailView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: GearDetailViewModel
    @State private var showingEdit = false

    init(environment: AppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearDetailViewModel(id: id, repository: environment.gearRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if viewModel.loading { TrailLoadingState() }
                if let error = viewModel.error { TrailErrorState(message: error) { Task { await viewModel.load() } } }
                if let item = viewModel.item {
                    detailContent(item)
                }
            }
            .padding(16)
        }
        .navigationTitle(viewModel.item?.name ?? "装备详情")
        .task { await viewModel.load() }
        .sheet(isPresented: $showingEdit, onDismiss: { Task { await viewModel.load() } }) {
            if let item = viewModel.item {
                NavigationStack { GearFormView(environment: environment, item: item) }
            }
        }
    }

    @ViewBuilder
    private func detailContent(_ item: GearItem) -> some View {
        TrailHeroCard(
            eyebrow: item.category.label,
            title: item.name,
            subtitle: item.description ?? "这件装备还没有补充说明。",
            chips: [item.status.label, item.formattedWeight, item.formattedPrice]
        ) {
            HStack(spacing: 10) {
                TrailPrimaryButton(title: "编辑装备") { showingEdit = true }
                TrailSoftButton(title: item.isArchived ? "恢复可用" : "移入历史") {
                    Task { item.isArchived ? await viewModel.restore() : await viewModel.archive() }
                }
            }
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "基础信息")
            TrailInfoRow(label: "品牌型号", value: item.brandModel.nilIfBlank ?? "未填写")
            TrailInfoRow(label: "颜色", value: item.color ?? "未填写")
            TrailInfoRow(label: "材质", value: item.material ?? "未填写")
            TrailInfoRow(label: "容量", value: item.capacity ?? "未填写")
            TrailInfoRow(label: "尺码", value: item.size ?? "未填写")
            TrailInfoRow(label: "存放位置", value: item.storageLocation ?? "未填写")
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "规格购买")
            TrailInfoRow(label: "重量", value: item.formattedWeight)
            TrailInfoRow(label: "保暖", value: item.warmthIndex ?? "未填写")
            TrailInfoRow(label: "防水", value: item.waterproofIndex ?? "未填写")
            TrailInfoRow(label: "购买日期", value: item.purchaseDate ?? "未填写")
            TrailInfoRow(label: "购买地点", value: item.purchaseLocation ?? "未填写")
            TrailInfoRow(label: "保修到期", value: item.expiryOrWarrantyDate ?? "未填写")
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "标签与共享")
            if item.tags.isEmpty {
                Text("暂无标签")
            } else {
                TrailTagRow(tags: item.tags)
            }
            TrailInfoRow(label: "共享状态", value: item.shareStatus.label)
            TrailInfoRow(label: "备注", value: item.notes ?? "未填写")
        }
    }
}
