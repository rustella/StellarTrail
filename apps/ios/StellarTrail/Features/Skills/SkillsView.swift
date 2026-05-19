import SwiftUI

struct SkillsView: View {
    @StateObject private var viewModel: SkillsViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: SkillsViewModel(repository: environment.skillRepository, mediaCache: environment.mediaCache))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野技能库",
                    title: "户外技能",
                    subtitle: "按分类学习常用绳结，步骤清晰，出发前快速复习。",
                    chips: ["绳结库", "媒体缓存", viewModel.state.selectedCategoryTitle]
                ) {
                    TrailPrimaryButton(title: viewModel.state.loading ? "处理中…" : "缓存全部") {
                        Task { await viewModel.cacheAllKnots() }
                    }
                }

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }
                if let message = viewModel.state.cacheMessage {
                    TrailSurfaceCard {
                        Text(message)
                        if let progress = viewModel.state.cacheProgress, progress.total > 0 {
                            ProgressView(value: progress.fraction)
                        }
                    }
                }

                TrailSectionTitle(title: "技能分类", subtitle: "先从主题分类进入，再查看具体绳结步骤。")
                ForEach(viewModel.state.categories) { category in
                    SkillCategoryCard(category: category)
                }

                TrailSurfaceCard {
                    TrailSectionTitle(title: "搜索筛选")
                    TextField("搜索绳结名称或用途", text: Binding(
                        get: { viewModel.state.searchQuery },
                        set: { viewModel.updateSearchQuery($0) }
                    ))
                    .trailFormField()
                    TrailPrimaryButton(title: "搜索绳结") { Task { await viewModel.submitSearch() } }
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            TrailPillButton(title: "全部", isSelected: viewModel.state.selectedCategoryID == nil) {
                                Task { await viewModel.selectCategory(nil) }
                            }
                            ForEach(viewModel.state.filterOptions) { option in
                                TrailPillButton(title: option.title, isSelected: viewModel.state.selectedCategoryID == option.id) {
                                    Task { await viewModel.selectCategory(option.id) }
                                }
                            }
                        }
                    }
                }

                TrailSectionTitle(title: "绳结库", subtitle: "点击条目查看步骤和媒体展示。")
                ForEach(viewModel.state.knots) { knot in
                    Button { Task { await viewModel.openKnot(knot.id) } } label: {
                        KnotCard(knot: knot, thumbnailURL: knot.media.thumbnailAsset.flatMap { viewModel.resolvedMediaURL(for: $0) })
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(knot.title)
                }
                if viewModel.state.nextOffset != nil {
                    TrailPrimaryButton(title: viewModel.state.loadingMore ? "加载中…" : "加载更多绳结") {
                        Task { await viewModel.loadMoreKnots() }
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("技能")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
        .sheet(item: Binding<KnotDetail?>(
            get: { viewModel.state.selectedKnot },
            set: { if $0 == nil { viewModel.closeKnot() } }
        )) { knot in
            KnotDetailSheet(knot: knot, resolvedURL: { viewModel.resolvedMediaURL(for: $0) }, onClose: viewModel.closeKnot)
        }
    }
}

private struct SkillCategoryCard: View {
    @Environment(\.trailPalette) private var palette
    let category: SkillCategorySummary

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: "技能分类", tone: .brand)
                Spacer()
                TrailBadge(text: "\(category.itemCount) 项", tone: .info)
            }
            Text(category.title)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            Text(category.summary)
                .font(.subheadline)
                .foregroundStyle(palette.textMuted)
        }
    }
}

private struct KnotCard: View {
    @Environment(\.trailPalette) private var palette
    let knot: KnotSummary
    let thumbnailURL: URL?

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top, spacing: 12) {
                AsyncImage(url: thumbnailURL) { phase in
                    switch phase {
                    case .success(let image):
                        image.resizable().scaledToFill()
                    default:
                        Image(systemName: "link")
                            .font(.title2)
                            .foregroundStyle(palette.brand)
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                }
                .frame(width: 72, height: 72)
                .background(palette.controlBackground)
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: knot.difficulty ?? "绳结", tone: .info)
                        Spacer()
                        TrailBadge(text: "查看步骤", tone: .brand)
                    }
                    Text(knot.title)
                        .font(.headline.weight(.heavy))
                        .foregroundStyle(palette.textPrimary)
                    Text(knot.summary)
                        .font(.subheadline)
                        .foregroundStyle(palette.textMuted)
                    if !knot.categories.isEmpty || !knot.types.isEmpty {
                        TrailTagRow(tags: Array((knot.categories.map(\.title) + knot.types.map(\.title)).prefix(3)))
                    }
                }
            }
            if knot.mediaCount > 0 {
                Text("素材 \(knot.mediaCount) 个")
                    .font(.caption.weight(.bold))
                    .foregroundStyle(palette.textMuted)
            }
        }
    }
}

private struct KnotDetailSheet: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(\.trailPalette) private var palette
    let knot: KnotDetail
    let resolvedURL: (KnotMediaAsset) -> URL?
    let onClose: () -> Void
    @State private var selectedMediaID: String?

    var body: some View {
        NavigationStack {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 16) {
                    TrailHeroCard(
                        eyebrow: knot.difficulty ?? "绳结",
                        title: knot.title,
                        subtitle: knot.description ?? knot.summary,
                        chips: ["步骤 \(knot.steps.count)", "素材 \(knot.mediaCount)"]
                    )
                    mediaStage
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "步骤")
                        ForEach(Array(knot.steps.enumerated()), id: \.offset) { index, step in
                            HStack(alignment: .top, spacing: 12) {
                                Text("\(index + 1)")
                                    .font(.caption.weight(.heavy))
                                    .foregroundStyle(palette.brandText)
                                    .frame(width: 26, height: 26)
                                    .background(palette.brand)
                                    .clipShape(Circle())
                                Text(step)
                                    .foregroundStyle(palette.textPrimary)
                            }
                        }
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "素材")
                        if knot.media.isEmpty {
                            Text("当前没有可展示素材。")
                                .foregroundStyle(palette.textMuted)
                        } else {
                            ForEach(knot.media) { media in
                                TrailInfoRow(label: media.mediaType, value: media.mimeType)
                            }
                        }
                    }
                }
                .padding(16)
            }
            .trailScreenBackground()
            .navigationTitle(knot.title)
            .toolbar { ToolbarItem(placement: .confirmationAction) { Button("关闭") { onClose(); dismiss() } } }
        }
    }

    private var mediaStage: some View {
        TrailSurfaceCard {
            TrailSectionTitle(title: "媒体展示", subtitle: "缩略图、打法动图和预览图会优先显示。")
            let detailMedia = knot.media.filter { ["preview", "thumbnail", "draw_gif", "turntable_gif"].contains($0.mediaType) }
            if detailMedia.isEmpty {
                Text("当前没有可展示素材。")
                    .foregroundStyle(palette.textMuted)
            } else {
                let active = detailMedia.first { $0.id == (selectedMediaID ?? detailMedia.first?.id) } ?? detailMedia[0]
                AsyncImage(url: resolvedURL(active)) { phase in
                    switch phase {
                    case .success(let image):
                        image.resizable().scaledToFit()
                    default:
                        ZStack {
                            RoundedRectangle(cornerRadius: 16, style: .continuous)
                                .fill(palette.controlBackground)
                            Image(systemName: "photo")
                                .font(.largeTitle)
                                .foregroundStyle(palette.brand)
                        }
                    }
                }
                .frame(maxWidth: .infinity)
                .frame(height: 220)
                .background(palette.controlBackground)
                .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
                TrailInfoRow(label: "类型", value: active.displayName)
                TrailInfoRow(label: "格式", value: active.mimeType)
                TrailInfoRow(label: "大小", value: Formatters.bytes(active.sizeBytes))
                if !detailMedia.isEmpty {
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(detailMedia) { media in
                                TrailPillButton(title: media.displayName, isSelected: media.id == active.id) {
                                    selectedMediaID = media.id
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
