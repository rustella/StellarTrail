import SwiftUI

struct MacHomeView: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: HomeViewModel

    init(environment: MacAppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: HomeViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            skillRepository: environment.skillRepository,
            contentRepository: environment.contentRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 18) {
                TrailHeroCard(
                    eyebrow: "寻径星野 · 桌面准备台",
                    title: "把出行准备放到更大的屏幕上",
                    subtitle: "在 Mac 上查看装备、学习绳结，并同步管理个人设置。"
                )

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }

                LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 14), count: 4), spacing: 14) {
                    TrailMetricTile(value: "\(viewModel.state.stats.currentCount)", label: "可用装备")
                    TrailMetricTile(value: "\(viewModel.state.stats.archivedCount)", label: "历史记录")
                    TrailMetricTile(value: Formatters.weight(viewModel.state.stats.totalWeightG), label: "总重量")
                    TrailMetricTile(value: Formatters.price(viewModel.state.stats.totalValueCents), label: "估算价值")
                }

                HStack(alignment: .top, spacing: 18) {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "出行装备参考", subtitle: "按场景检查背包，登录后继续整理自己的物品。")
                        ForEach(viewModel.state.templates.prefix(3)) { template in
                            GearTemplateRow(template: template)
                        }
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "户外技能", subtitle: "桌面端优先展示分类与重点绳结。")
                        ForEach(viewModel.state.skills.prefix(4)) { skill in
                            SkillSummaryRow(skill: skill)
                        }
                    }
                }
            }
            .padding(28)
        }
        .navigationTitle("首页")
        .task { await viewModel.load() }
    }
}

private struct GearTemplateRow: View {
    let template: GearTemplate

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(template.title).font(.headline.weight(.heavy))
            Text(template.categories.flatMap(\.items).prefix(4).joined(separator: " · "))
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .lineLimit(2)
        }
        .padding(.vertical, 6)
    }
}

private struct SkillSummaryRow: View {
    let skill: SkillCategorySummary

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(skill.title).font(.headline.weight(.heavy))
                Spacer()
                TrailBadge(text: "\(skill.itemCount) 项", tone: .info)
            }
            Text(skill.summary)
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 6)
    }
}
