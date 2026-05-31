import SwiftUI

struct TripsView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: TripsViewModel
    @State private var showingAuth = false
    @State private var showingCreate = false
    @State private var showingJoin = false
    @State private var createType: TripType = .solo

    init(environment: AppEnvironment) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: TripsViewModel(sessionStore: environment.sessionStore, repository: environment.tripRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(
                    eyebrow: "出发前计划",
                    title: "行程",
                    subtitle: viewModel.isLoggedIn ? "管理单人行程与组队协作，出发前准备更清晰。" : "登录后可以加入多人行程，也可以创建自己的单人计划。",
                    chips: [viewModel.isLoggedIn ? "已同步" : "登录后使用", "\(viewModel.trips.count) 条"]
                ) {
                    HStack(spacing: 10) {
                        TrailPrimaryButton(title: viewModel.isLoggedIn ? "创建行程" : "账号登录") {
                            viewModel.isLoggedIn ? (showingCreate = true) : (showingAuth = true)
                        }
                        if viewModel.isLoggedIn {
                            TrailSoftButton(title: "加入") { showingJoin = true }
                        }
                    }
                }

                if !viewModel.isLoggedIn {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "登录后继续", subtitle: "当前可先去装备图鉴查看公开装备，行程会保存到账号。")
                        NavigationLink(destination: GearAtlasListView(environment: environment)) {
                            Text("查看装备图鉴")
                                .font(.headline.weight(.bold))
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 13)
                        }
                        .buttonStyle(.plain)
                    }
                }

                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                if viewModel.isLoggedIn {
                    tripFilters
                    if viewModel.trips.isEmpty && !viewModel.loading {
                        TrailEmptyState(title: "还没有行程", subtitle: "先制作一个单人计划，或者通过邀请加入多人行程。")
                    }
                    ForEach(viewModel.trips) { trip in
                        NavigationLink(destination: TripDetailView(environment: environment, id: trip.id)) {
                            TripSummaryCard(trip: trip)
                        }
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("trip-row-\(trip.id)")
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("行程")
        .toolbar {
            if viewModel.isLoggedIn {
                ToolbarItemGroup(placement: .topBarTrailing) {
                    Button("加入") { showingJoin = true }
                    Button("创建") { showingCreate = true }
                }
            }
        }
        .task { await viewModel.load() }
        .onReceive(environment.sessionStore.$currentSession) { _ in Task { await viewModel.load() } }
        .refreshable { await viewModel.load() }
        .sheet(isPresented: $showingAuth, onDismiss: { Task { await viewModel.load() } }) {
            AuthView(environment: environment, mode: .password)
        }
        .sheet(isPresented: $showingCreate, onDismiss: { Task { await viewModel.load() } }) {
            TripCreateChoiceSheet(environment: environment, selectedType: $createType, close: { showingCreate = false })
                .presentationDetents([.medium, .large])
        }
        .sheet(isPresented: $showingJoin, onDismiss: { Task { await viewModel.load() } }) {
            NavigationStack { TripJoinView(environment: environment, close: { showingJoin = false }) }
                .presentationDetents([.medium])
        }
    }

    private var tripFilters: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach([TripTimeBucket.all, .upcoming, .ongoing, .past, .undated], id: \.rawValue) { bucket in
                    TrailPillButton(title: bucket.label, isSelected: viewModel.bucket == bucket) {
                        Task { await viewModel.selectBucket(bucket) }
                    }
                }
            }
        }
    }
}

private struct TripCreateChoiceSheet: View {
    @ObservedObject var environment: AppEnvironment
    @Binding var selectedType: TripType
    let close: () -> Void

    var body: some View {
        NavigationStack {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 16) {
                    TrailHeroCard(eyebrow: "制作行程计划", title: "选择行程类型", subtitle: "单人隐藏多人协作和公共装备分工；多人保留邀请和成员准备。")
                    TrailSurfaceCard {
                        TripTypeButton(type: .solo, title: "单人行程", subtitle: "只有自己的装备、行程、食品、医药和预算准备。") {
                            selectedType = .solo
                        }
                        TripTypeButton(type: .team, title: "多人行程", subtitle: "保留成员协作、邀请加入和公共装备分工。") {
                            selectedType = .team
                        }
                    }
                    NavigationLink(destination: TripFormView(environment: environment, tripType: selectedType, close: close)) {
                        Text("继续")
                            .font(.headline.weight(.bold))
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 13)
                    }
                    .buttonStyle(.plain)
                }
                .padding(16)
            }
            .trailScreenBackground()
            .navigationTitle("创建行程")
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭", action: close) } }
        }
    }

    private func TripTypeButton(type: TripType, title: String, subtitle: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: selectedType == type ? "checkmark.circle.fill" : "circle")
                    .font(.title3.weight(.bold))
                VStack(alignment: .leading, spacing: 6) {
                    Text(title)
                        .font(.headline.weight(.heavy))
                    Text(subtitle)
                        .font(.subheadline)
                }
                Spacer()
            }
        }
        .buttonStyle(.plain)
    }
}

private struct TripFormView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel: TripFormViewModel
    let close: () -> Void

    init(environment: AppEnvironment, tripType: TripType, close: @escaping () -> Void) {
        _viewModel = StateObject(wrappedValue: TripFormViewModel(tripType: tripType, repository: environment.tripRepository))
        self.close = close
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: viewModel.tripType.label, title: "填写行程信息", subtitle: "日期可先留空，后续在详情里继续完善。")
                TrailSurfaceCard {
                    HStack(spacing: 8) {
                        ForEach(TripType.allCases) { type in
                            TrailPillButton(title: type.label, isSelected: viewModel.tripType == type) {
                                viewModel.tripType = type
                            }
                        }
                    }
                    TextField("行程名称", text: $viewModel.title)
                        .trailFormField()
                    TextField("描述", text: $viewModel.description, axis: .vertical)
                        .trailFormField()
                    TextField("开始日期，例如 2026-06-06", text: $viewModel.startDate)
                        .trailFormField()
                    TextField("结束日期，例如 2026-06-07", text: $viewModel.endDate)
                        .trailFormField()
                    Toggle("启用坡度修正", isOn: $viewModel.useSlopeAdjustment)
                    Toggle("启用高海拔修正", isOn: $viewModel.useHighAltitudeAdjustment)
                    TextField("起点海拔（米）", text: $viewModel.startAltitude)
                        .keyboardType(.numberPad)
                        .trailFormField()
                    TrailPrimaryButton(title: "保存行程") {
                        Task {
                            if await viewModel.submit() != nil {
                                close()
                                dismiss()
                            }
                        }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("行程表单")
    }
}

private struct TripJoinView: View {
    @StateObject private var viewModel: TripJoinViewModel
    let close: () -> Void

    init(environment: AppEnvironment, close: @escaping () -> Void) {
        _viewModel = StateObject(wrappedValue: TripJoinViewModel(repository: environment.tripRepository))
        self.close = close
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "加入多人行程", title: "粘贴邀请口令", subtitle: "接受后会进入对应多人行程详情。")
                TrailSurfaceCard {
                    TextField("邀请口令", text: $viewModel.token)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .trailFormField()
                    TrailPrimaryButton(title: "加入行程") {
                        Task {
                            if await viewModel.join() != nil {
                                close()
                            }
                        }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("加入行程")
        .toolbar { ToolbarItem(placement: .cancellationAction) { Button("关闭", action: close) } }
    }
}

struct TripDetailView: View {
    @ObservedObject var environment: AppEnvironment
    @StateObject private var viewModel: TripDetailViewModel

    init(environment: AppEnvironment, id: String) {
        self.environment = environment
        _viewModel = StateObject(wrappedValue: TripDetailViewModel(id: id, repository: environment.tripRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                if let detail = viewModel.detail {
                    TrailHeroCard(
                        eyebrow: detail.trip.tripType.label,
                        title: detail.trip.title,
                        subtitle: detail.trip.description ?? detail.trip.enabledSections.map(\.label).joined(separator: "、"),
                        chips: [detail.trip.dayCount > 0 ? "\(detail.trip.dayCount)天" : "未定日期", detail.trip.tripType == .solo ? "单人模式" : "多人协作"]
                    ) {
                        HStack(spacing: 10) {
                            if detail.trip.tripType == .team {
                                TrailPrimaryButton(title: "邀请") { Task { await viewModel.createInvitation() } }
                            }
                            TrailSoftButton(title: "转为经历") { Task { await viewModel.convertToExperience() } }
                        }
                    }
                    TripSectionSelector(sections: viewModel.visibleSections, selected: $viewModel.selectedSection)
                    TripSectionOverview(detail: detail, section: viewModel.selectedSection)
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "板块开关", subtitle: "保存时会按行程类型过滤隐藏板块。")
                        ForEach(TripSectionKey.allowed(for: detail.trip.tripType)) { section in
                            Button {
                                Task { await viewModel.toggleSection(section) }
                            } label: {
                                HStack {
                                    Label(section.label, systemImage: section.systemImage)
                                    Spacer()
                                    Image(systemName: detail.sections.contains(section) ? "checkmark.circle.fill" : "circle")
                                }
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    if let message = viewModel.message {
                        TrailSurfaceCard { Text(message) }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("行程详情")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

private struct TripSummaryCard: View {
    @Environment(\.trailPalette) private var palette
    let trip: TripSummary

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: trip.tripType.label, tone: trip.tripType == .solo ? .info : .brand)
                        TrailBadge(text: trip.timeBucket.label, tone: .neutral)
                    }
                    Text(trip.title)
                        .font(.headline.weight(.heavy))
                    Text("\(trip.dateText) · \(trip.durationText)")
                        .font(.subheadline)
                        .foregroundStyle(palette.textMuted)
                    Text(trip.readinessText)
                        .font(.caption.weight(.bold))
                        .foregroundStyle(palette.textMuted)
                }
                Spacer()
                TrailBadge(text: "\(trip.memberCount) 人", tone: .info)
            }
        }
    }
}

private struct TripSectionSelector: View {
    let sections: [TripSectionKey]
    @Binding var selected: TripSectionKey

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(sections) { section in
                    TrailPillButton(title: section.label, isSelected: selected == section) {
                        selected = section
                    }
                }
            }
        }
    }
}

private struct TripSectionOverview: View {
    let detail: TripDetail
    let section: TripSectionKey

    var body: some View {
        switch section {
        case .members:
            CountSection(title: "成员", subtitle: "成员资料、角色、联系方式和背负摘要。", countLabel: "\(detail.members.count) 人") {
                ForEach(detail.members) { member in
                    TrailInfoRow(label: member.profile.displayName, value: member.isOwner ? "队长" : (member.profile.roleLabel ?? "队员"))
                }
            }
        case .personalGear:
            CountSection(title: "个人装备", subtitle: "从装备库和打包清单导入的个人装备。", countLabel: "\(detail.personalGear.count) 项") {
                ForEach(detail.personalGear) { item in
                    TrailInfoRow(label: item.name, value: "\(item.packedQuantity)/\(item.plannedQuantity)")
                }
            }
        case .sharedGear:
            CountSection(title: "公共装备", subtitle: "多人行程的需求、负责人和具体装备分工。", countLabel: "\(detail.sharedGearDemands.count) 项") {
                ForEach(detail.sharedGearDemands) { item in
                    TrailInfoRow(label: item.demandName ?? item.name, value: item.concreteName ?? "待填写")
                }
            }
        case .itinerary:
            CountSection(title: "行程安排", subtitle: "行程日、路段、营地和估算耗时。", countLabel: "\(detail.itineraryDays.count) 天") {
                ForEach(detail.routeSegments) { item in
                    TrailInfoRow(label: item.name, value: String(format: "%.1f km", item.distanceKm))
                }
            }
        case .foodPlan:
            CountSection(title: "食品计划", subtitle: "餐次和公共食材会进入背负与预算视图。", countLabel: "\(detail.foodMeals.count) 餐") {
                ForEach(detail.foodMeals) { meal in
                    TrailInfoRow(label: meal.mealType ?? meal.mealKey, value: meal.dishName ?? "未填写")
                }
            }
        case .medicalKit:
            CountSection(title: "医药包", subtitle: "按必备药品和负责人确认携带。", countLabel: "\(detail.medicalItems.count) 项") {
                ForEach(detail.medicalItems) { item in
                    TrailInfoRow(label: item.name, value: "\(item.packedQuantity)/\(item.requiredQuantity)")
                }
            }
        case .safetyPlan:
            CountSection(title: "安全预案", subtitle: "风险、预防措施和应对方案。", countLabel: "\(detail.safetyRisks.count) 项") {
                ForEach(detail.safetyRisks) { item in
                    TrailInfoRow(label: item.riskType, value: item.response ?? "未填写")
                }
            }
        case .rescueInfo:
            CountSection(title: "救援信息", subtitle: "附近救援站、联系电话和下撤信息。", countLabel: "\(detail.rescueContacts.count) 条") {
                ForEach(detail.rescueContacts) { item in
                    TrailInfoRow(label: item.organization, value: item.phone ?? "未填写")
                }
            }
        case .budget:
            CountSection(title: "预算", subtitle: "交通、住宿、食品和公共装备费用。", countLabel: "\(detail.budgetItems.count) 项") {
                ForEach(detail.budgetItems) { item in
                    TrailInfoRow(label: item.name, value: Formatters.price(item.totalPriceCents))
                }
            }
        case .goals:
            CountSection(title: "目标", subtitle: "团队或个人目标，便于复盘转经历。", countLabel: "\(detail.goals.count) 条") {
                ForEach(detail.goals) { item in
                    TrailInfoRow(label: item.scope, value: item.content)
                }
            }
        }
    }
}

private struct CountSection<Content: View>: View {
    let title: String
    let subtitle: String
    let countLabel: String
    let content: Content

    init(title: String, subtitle: String, countLabel: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.subtitle = subtitle
        self.countLabel = countLabel
        self.content = content()
    }

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                TrailSectionTitle(title: title, subtitle: subtitle)
                Spacer()
                TrailBadge(text: countLabel, tone: .info)
            }
            content
        }
    }
}
