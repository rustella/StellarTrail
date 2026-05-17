import SwiftUI

struct SkillsView: View {
    @StateObject private var viewModel: SkillsViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: SkillsViewModel(repository: environment.skillRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "寻径星野技能库",
                    title: "户外技能",
                    subtitle: "按分类学习常用绳结，步骤清晰，出发前快速复习。",
                    chips: ["先看内容", "步骤化学习"]
                )

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }

                TrailSectionTitle(title: "技能分类", subtitle: "先从主题分类进入，再查看具体绳结步骤。")
                ForEach(viewModel.state.categories) { category in
                    SkillCategoryCard(category: category)
                }

                TrailSectionTitle(title: "绳结库", subtitle: "点击条目查看步骤，底部弹层展示详情。")
                ForEach(viewModel.state.knots) { knot in
                    Button { Task { await viewModel.openKnot(knot.id) } } label: {
                        KnotCard(knot: knot)
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
        .navigationTitle("技能")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
        .sheet(item: Binding<KnotDetail?>(
            get: { viewModel.state.selectedKnot },
            set: { if $0 == nil { viewModel.closeKnot() } }
        )) { knot in
            KnotDetailSheet(knot: knot, onClose: viewModel.closeKnot)
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

    var body: some View {
        TrailSurfaceCard {
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
    let onClose: () -> Void

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
            .navigationTitle(knot.title)
            .toolbar { ToolbarItem(placement: .confirmationAction) { Button("关闭") { onClose(); dismiss() } } }
        }
    }
}
