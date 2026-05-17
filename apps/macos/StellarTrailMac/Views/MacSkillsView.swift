import SwiftUI

struct MacSkillsView: View {
    @StateObject private var viewModel: SkillsViewModel

    init(environment: MacAppEnvironment) {
        _viewModel = StateObject(wrappedValue: SkillsViewModel(repository: environment.skillRepository))
    }

    var body: some View {
        HSplitView {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    TrailHeroCard(
                        eyebrow: "技能库",
                        title: "户外技能",
                        subtitle: "桌面端左侧浏览分类和绳结，右侧查看步骤详情。",
                        chips: ["分类", "步骤", "素材"]
                    )
                    if let error = viewModel.state.error {
                        TrailErrorState(message: error) { Task { await viewModel.load() } }
                    }
                    if viewModel.state.loading { TrailLoadingState() }
                    TrailSectionTitle(title: "技能分类")
                    ForEach(viewModel.state.categories) { category in
                        TrailSurfaceCard {
                            HStack {
                                Text(category.title).font(.headline.weight(.heavy))
                                Spacer()
                                TrailBadge(text: "\(category.itemCount) 项", tone: .info)
                            }
                            Text(category.summary).foregroundStyle(.secondary)
                        }
                    }
                    TrailSectionTitle(title: "绳结库")
                    ForEach(viewModel.state.knots) { knot in
                        Button { Task { await viewModel.openKnot(knot.id) } } label: {
                            MacKnotRow(knot: knot)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(24)
            }
            .frame(minWidth: 430, idealWidth: 520)

            MacKnotDetailPane(knot: viewModel.state.selectedKnot)
                .frame(minWidth: 420)
                .padding(24)
        }
        .navigationTitle("技能")
        .task { await viewModel.load() }
    }
}

private struct MacKnotRow: View {
    let knot: KnotSummary

    var body: some View {
        TrailSurfaceCard(padding: 14) {
            HStack {
                VStack(alignment: .leading, spacing: 8) {
                    Text(knot.title).font(.headline.weight(.heavy))
                    Text(knot.summary).font(.subheadline).foregroundStyle(.secondary)
                    TrailTagRow(tags: Array((knot.categories.map(\.title) + knot.types.map(\.title)).prefix(3)))
                }
                Spacer()
                TrailBadge(text: knot.difficulty ?? "绳结", tone: .info)
            }
        }
    }
}

private struct MacKnotDetailPane: View {
    let knot: KnotDetail?

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if let knot {
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
                                Text("\(index + 1)").font(.caption.weight(.heavy)).frame(width: 26, height: 26).background(.tint).clipShape(Circle())
                                Text(step)
                            }
                        }
                    }
                } else {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "选择一个绳结", subtitle: "点击左侧条目后，这里展示步骤和素材信息。")
                    }
                }
            }
        }
    }
}
