import SwiftUI

struct GearDetailView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: GearDetailViewModel
    @State private var showingEdit = false

    init(environment: AppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: GearDetailViewModel(id: id, repository: environment.gearRepository, atlasRepository: environment.gearAtlasRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if viewModel.loading { TrailLoadingState() }
                if let error = viewModel.error { TrailErrorState(message: error) { Task { await viewModel.load() } } }
                if let item = viewModel.item {
                    detailContent(item)
                }
                if let message = viewModel.message {
                    TrailSurfaceCard { Text(message) }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle(viewModel.item?.name ?? "装备详情")
        .task { await viewModel.load() }
        .sheet(isPresented: $showingEdit, onDismiss: { Task { await viewModel.load() } }) {
            if let item = viewModel.item {
                NavigationStack { GearFormView(environment: environment, item: item) }
                    .presentationDetents([.large])
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
            HStack {
                TrailSectionTitle(title: "图鉴投稿", subtitle: "只提交品牌、型号、规格、重量和官方价等公开字段。")
                Spacer()
                if let submission = viewModel.submission {
                    TrailBadge(text: submission.status.label, tone: submission.status.badgeTone)
                } else {
                    TrailBadge(text: "未投稿", tone: .neutral)
                }
            }
            if let submission = viewModel.submission {
                TrailInfoRow(label: "来源", value: submission.sourceType.label)
                TrailInfoRow(label: "投稿时间", value: Formatters.date(submission.createdAt))
                if let reason = submission.rejectionReason {
                    TrailInfoRow(label: "审核说明", value: reason)
                }
            } else {
                TrailPrimaryButton(title: viewModel.submitting ? "提交中…" : "投稿到图鉴") {
                    Task { await viewModel.submitToAtlas() }
                }
            }
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "基础信息")
            TrailInfoRow(label: "品牌型号", value: item.brandModel.nilIfBlank ?? "未填写")
            TrailInfoRow(label: "官方价格", value: item.formattedOfficialPrice)
            TrailInfoRow(label: "购入价格", value: item.formattedPrice)
            TrailInfoRow(label: "存放位置", value: item.storageLocation ?? "未填写")
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "规格购买")
            TrailInfoRow(label: "重量", value: item.formattedWeight)
            TrailInfoRow(label: "购买日期", value: Formatters.date(item.purchaseDate))
            TrailInfoRow(label: "购买渠道", value: item.purchaseLocation ?? "未填写")
            let fields = GearOptions.specFieldViews(for: item.category, specs: item.specs ?? GearOptions.legacySpecs(from: item))
                .filter { (item.specs ?? GearOptions.legacySpecs(from: item))[$0.key]?.nilIfBlank != nil }
            ForEach(fields) { field in
                TrailInfoRow(label: field.label, value: (item.specs ?? GearOptions.legacySpecs(from: item))[field.key] ?? "未填写")
            }
        }

        TrailSurfaceCard {
            TrailSectionTitle(title: "标签与共享")
            if item.tags.isEmpty {
                Text("暂无标签")
            } else {
                TrailColorTagRow(tags: item.tagViews)
            }
            TrailInfoRow(label: "共享状态", value: item.shareStatus.label)
            TrailInfoRow(label: "备注", value: item.notes ?? "未填写")
        }
    }
}
