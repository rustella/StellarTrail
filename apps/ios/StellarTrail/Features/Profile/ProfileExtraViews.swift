import SwiftUI

struct RoadmapView: View {
    @StateObject private var viewModel: RoadmapViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: RoadmapViewModel(sessionStore: environment.sessionStore, repository: environment.roadmapRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "寻径星野", title: "产品路线图", subtitle: "查看 iOS 后续计划，并对你关心的事项投票或订阅。", chips: ["client_key=ios"])
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        TrailPillButton(title: "全部", isSelected: viewModel.selectedStatus == nil) {
                            Task { await viewModel.selectStatus(nil) }
                        }
                        ForEach(RoadmapStatus.allCases) { status in
                            TrailPillButton(title: status.label, isSelected: viewModel.selectedStatus == status) {
                                Task { await viewModel.selectStatus(status) }
                            }
                        }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                if viewModel.items.isEmpty && !viewModel.loading {
                    TrailEmptyState(title: "暂无 iOS 路线图", subtitle: "后台发布 iOS 数据后会显示在这里。")
                }
                ForEach(viewModel.items) { item in
                    RoadmapCard(item: item, vote: { Task { await viewModel.toggleVote(item) } }, subscribe: { Task { await viewModel.toggleSubscription(item) } })
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("产品路线图")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

struct OutdoorProfileView: View {
    @StateObject private var viewModel: OutdoorProfileViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: OutdoorProfileViewModel(repository: environment.profileRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "户外资料", title: "成员默认资料", subtitle: "维护身高、血型、紧急联系人和饮食习惯，创建行程时复用。")
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                TrailSurfaceCard {
                    TextField("户外 ID", text: textBinding(\.outdoorId))
                        .trailFormField()
                    TextField("真实姓名", text: textBinding(\.realName))
                        .trailFormField()
                    TextField("性别", text: textBinding(\.gender))
                        .trailFormField()
                    TextField("出生日期，例如 1995-05-01", text: textBinding(\.birthDate))
                        .trailFormField()
                    TextField("身高 cm", text: heightBinding)
                        .keyboardType(.numberPad)
                        .trailFormField()
                    TextField("手机号", text: textBinding(\.phone))
                        .keyboardType(.phonePad)
                        .trailFormField()
                }
                TrailSurfaceCard {
                    TrailSectionTitle(title: "紧急与健康")
                    TextField("紧急联系人", text: textBinding(\.emergencyContact))
                        .trailFormField()
                    TextField("关系", text: textBinding(\.emergencyContactRelationship))
                        .trailFormField()
                    TextField("紧急电话", text: textBinding(\.emergencyPhone))
                        .keyboardType(.phonePad)
                        .trailFormField()
                    TextField("血型", text: textBinding(\.bloodType))
                        .trailFormField()
                    TextField("病史", text: textBinding(\.medicalHistory), axis: .vertical)
                        .trailFormField()
                    TextField("过敏史", text: textBinding(\.allergyHistory), axis: .vertical)
                        .trailFormField()
                    TextField("医疗应对说明", text: textBinding(\.medicalResponseNote), axis: .vertical)
                        .trailFormField()
                }
                TrailSurfaceCard {
                    TrailSectionTitle(title: "饮食与保险")
                    TextField("饮食习惯", text: textBinding(\.dietPreference))
                        .trailFormField()
                    TextField("保险单号", text: textBinding(\.insurancePolicyNo))
                        .trailFormField()
                    TextField("保险公司电话", text: textBinding(\.insuranceCompanyPhone))
                        .trailFormField()
                    TextField("经验备注", text: textBinding(\.experienceNote), axis: .vertical)
                        .trailFormField()
                    TrailPrimaryButton(title: "保存户外资料") { Task { await viewModel.save() } }
                }
                if let message = viewModel.message {
                    TrailSurfaceCard { Text(message) }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("户外资料")
        .task { await viewModel.load() }
    }

    private func textBinding(_ keyPath: WritableKeyPath<OutdoorProfile, String?>) -> Binding<String> {
        Binding(
            get: { viewModel.profile[keyPath: keyPath] ?? "" },
            set: { viewModel.profile[keyPath: keyPath] = $0.nilIfBlank }
        )
    }

    private var heightBinding: Binding<String> {
        Binding(
            get: { viewModel.profile.heightCm.map(String.init) ?? "" },
            set: { viewModel.profile.heightCm = Int($0) }
        )
    }
}

struct OutdoorExperiencesView: View {
    @StateObject private var viewModel: OutdoorExperiencesViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: OutdoorExperiencesViewModel(repository: environment.tripRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "户外经历", title: "历史行程和复盘", subtitle: "从历史行程转换，或手动补充自己的户外经历。")
                TrailSurfaceCard {
                    TrailSectionTitle(title: "手动补充")
                    TextField("标题", text: $viewModel.titleDraft)
                        .trailFormField()
                    TextField("路线摘要", text: $viewModel.routeDraft, axis: .vertical)
                        .trailFormField()
                    TextField("复盘备注", text: $viewModel.notesDraft, axis: .vertical)
                        .trailFormField()
                    TrailPrimaryButton(title: "添加经历") { Task { await viewModel.create() } }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                if viewModel.items.isEmpty && !viewModel.loading {
                    TrailEmptyState(title: "暂无户外经历", subtitle: "历史行程转为经历后会出现在这里。")
                }
                ForEach(viewModel.items) { item in
                    OutdoorExperienceCard(item: item) {
                        Task { await viewModel.delete(id: item.id) }
                    }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("户外经历")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

struct FeedbackView: View {
    @StateObject private var viewModel: FeedbackViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: FeedbackViewModel(repository: environment.feedbackRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "反馈", title: "告诉我们你的问题", subtitle: "iOS 使用邮箱登录体系，反馈会提交到现有后端。")
                TrailSurfaceCard {
                    Picker("反馈类型", selection: $viewModel.category) {
                        ForEach(FeedbackCategory.allCases) { category in
                            Text(category.label).tag(category)
                        }
                    }
                    .pickerStyle(.segmented)
                    TextField("反馈内容", text: $viewModel.content, axis: .vertical)
                        .lineLimit(5, reservesSpace: true)
                        .trailFormField()
                    TextField("联系方式（选填）", text: $viewModel.contact)
                        .trailFormField()
                    TrailPrimaryButton(title: viewModel.loading ? "提交中…" : "提交反馈") {
                        Task { await viewModel.submit() }
                    }
                }
                if let error = viewModel.error {
                    TrailErrorState(message: error)
                }
                if let message = viewModel.message {
                    TrailSurfaceCard { Text(message) }
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("反馈")
    }
}

struct ClientVersionsView: View {
    @StateObject private var viewModel: ClientVersionViewModel

    init(environment: AppEnvironment) {
        _viewModel = StateObject(wrappedValue: ClientVersionViewModel(repository: environment.clientVersionRepository))
    }

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 16) {
                TrailHeroCard(eyebrow: "版本信息", title: "iOS 版本公告", subtitle: "仅展示 client_key=ios 的版本信息；后台暂无数据时显示空态。")
                if let error = viewModel.error {
                    TrailErrorState(message: error) { Task { await viewModel.load() } }
                }
                if viewModel.loading { TrailLoadingState() }
                if viewModel.versions.isEmpty && !viewModel.loading {
                    TrailEmptyState(title: "暂无 iOS 版本公告", subtitle: "后台发布后会显示在这里。")
                }
                ForEach(viewModel.versions) { version in
                    ClientVersionCard(version: version)
                }
            }
            .padding(16)
        }
        .trailScreenBackground()
        .navigationTitle("版本信息")
        .task { await viewModel.load() }
        .refreshable { await viewModel.load() }
    }
}

private struct RoadmapCard: View {
    let item: RoadmapItem
    let vote: () -> Void
    let subscribe: () -> Void

    var body: some View {
        TrailSurfaceCard {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        TrailBadge(text: item.categoryLabel, tone: .brand)
                        TrailBadge(text: item.statusLabel, tone: .info)
                        TrailBadge(text: "P\(item.priority)", tone: .neutral)
                    }
                    Text(item.title)
                        .font(.headline.weight(.heavy))
                    Text(item.summary)
                        .font(.subheadline)
                    if let details = item.details {
                        Text(details)
                            .font(.caption)
                    }
                }
                Spacer()
            }
            HStack(spacing: 10) {
                TrailPrimaryButton(title: "\(item.isVoted ? "已投票" : "投票") · \(item.voteCount)", action: vote)
                TrailSoftButton(title: "\(item.isSubscribed ? "已订阅" : "订阅") · \(item.subscriptionCount)", action: subscribe)
            }
        }
    }
}

private struct OutdoorExperienceCard: View {
    let item: OutdoorExperience
    let delete: () -> Void

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: item.tripType.label, tone: item.tripType == .solo ? .info : .brand)
                Spacer()
                TrailBadge(text: item.dateText, tone: .neutral)
            }
            Text(item.title)
                .font(.headline.weight(.heavy))
            Text(item.routeSummary ?? "暂无路线摘要")
                .font(.subheadline)
            if let notes = item.notes {
                Text(notes)
                    .font(.caption)
            }
            TrailSoftButton(title: "删除", action: delete)
        }
    }
}

private struct ClientVersionCard: View {
    let version: ClientVersion

    var body: some View {
        TrailSurfaceCard {
            HStack {
                TrailBadge(text: version.version, tone: .brand)
                Spacer()
                TrailBadge(text: version.status.rawValue, tone: .neutral)
            }
            Text(version.title)
                .font(.headline.weight(.heavy))
            if !version.releaseNotes.isEmpty {
                ForEach(version.releaseNotes, id: \.self) { note in
                    Text("• \(note)")
                        .font(.subheadline)
                }
            }
            ForEach(version.releaseNoteSections) { section in
                TrailSectionTitle(title: section.title)
                ForEach(section.items, id: \.self) { note in
                    Text("• \(note)")
                        .font(.subheadline)
                }
            }
        }
    }
}
