import SwiftUI

struct MacHomeView: View {
    @ObservedObject var environment: MacAppEnvironment
    @StateObject private var viewModel: HomeViewModel

    init(environment: MacAppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: HomeViewModel(
            sessionStore: environment.sessionStore,
            gearRepository: environment.gearRepository,
            gearAtlasRepository: environment.gearAtlasRepository,
            skillRepository: environment.skillRepository,
            contentRepository: environment.contentRepository
        ))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 18) {
                TrailHeroCard(
                    eyebrow: "寻径星野 · 桌面工作台",
                    title: "整理装备，复习技能，准备下一次出发",
                    subtitle: "首页只保留最常用的桌面入口：装备概览、最近记录、公开参考和绳结学习。"
                )

                if let error = viewModel.state.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.state.loading { TrailLoadingState() }

                LazyVGrid(columns: dashboardColumns, spacing: 14) {
                    TrailMetricTile(value: "\(viewModel.state.stats.currentCount)", label: "可用装备", hint: viewModel.state.isLoggedIn ? "当前装备库" : "登录后同步")
                    TrailMetricTile(value: Formatters.weight(viewModel.state.stats.totalWeightG), label: "总重量", hint: "按装备记录统计")
                    TrailMetricTile(value: "\(viewModel.state.templates.count)", label: "公开清单", hint: "免登录可浏览")
                    TrailMetricTile(value: "\(viewModel.state.skills.count)", label: "技能分类", hint: "优先学习绳结")
                }

                HStack(alignment: .top, spacing: 18) {
                    TrailSurfaceCard {
                        TrailSectionTitle(
                            title: viewModel.state.isLoggedIn ? "最近装备" : "公开装备参考",
                            subtitle: viewModel.state.isLoggedIn ? "快速回到刚整理过的装备。" : "先按场景查看常见出行准备。"
                        )
                        if viewModel.state.isLoggedIn {
                            if viewModel.state.recentGears.isEmpty {
                                Text("还没有装备记录。")
                                    .foregroundStyle(.secondary)
                            } else {
                                ForEach(viewModel.state.recentGears) { gear in
                                    RecentGearRow(gear: gear)
                                }
                            }
                        } else {
                            ForEach(viewModel.state.templates.prefix(3)) { template in
                                GearTemplateRow(template: template)
                            }
                        }
                    }
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "绳结与技能", subtitle: "桌面端优先展示分类和重点绳结入口。")
                        ForEach(viewModel.state.skills.prefix(4)) { skill in
                            SkillSummaryRow(skill: skill)
                        }
                    }
                }

                if !viewModel.state.atlasItems.isEmpty {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "公开装备图鉴", subtitle: "从社区公开资料中快速参考品牌型号和重量。")
                        LazyVGrid(columns: dashboardColumns, spacing: 12) {
                            ForEach(viewModel.state.atlasItems) { item in
                                GearAtlasPreview(item: item)
                            }
                        }
                    }
                }
            }
            .padding(28)
        }
        .navigationTitle("首页")
        .task { await viewModel.load() }
    }

    private var dashboardColumns: [GridItem] {
        [GridItem(.adaptive(minimum: 180), spacing: 14)]
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

private struct RecentGearRow: View {
    let gear: GearSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(gear.name).font(.headline.weight(.heavy))
                Spacer()
                TrailBadge(text: gear.categoryLabel, tone: .brand)
            }
            Text([gear.brandModel.nilIfBlank, gear.formattedWeight].compactMap { $0 }.joined(separator: " · "))
                .font(.subheadline)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 6)
    }
}

private struct GearAtlasPreview: View {
    let item: GearAtlasPublicItem

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TrailBadge(text: item.categoryLabel, tone: .info)
            Text(item.name)
                .font(.headline.weight(.heavy))
                .lineLimit(1)
            Text([item.brandModel.nilIfBlank, item.formattedWeight].compactMap { $0 }.joined(separator: " · "))
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
        .padding(14)
        .background(Color(nsColor: .controlBackgroundColor), in: RoundedRectangle(cornerRadius: 16, style: .continuous))
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
