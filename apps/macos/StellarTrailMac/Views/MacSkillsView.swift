import AVKit
import SwiftUI

struct MacSkillsView: View {
    @StateObject private var viewModel: SkillsViewModel

    init(environment: MacAppEnvironment) {
        _viewModel = StateObject(wrappedValue: SkillsViewModel(repository: environment.skillRepository))
    }

    var body: some View {
        HSplitView {
            knotBrowser
                .frame(minWidth: 440, idealWidth: 520)

            MacKnotDetailPane(
                knot: viewModel.state.selectedKnot,
                loading: viewModel.state.detailLoading,
                error: viewModel.state.detailError
            )
            .frame(minWidth: 520)
            .padding(24)
        }
        .navigationTitle("绳结")
        .task { await viewModel.load() }
    }

    private var knotBrowser: some View {
        VStack(spacing: 0) {
            knotToolbar
                .padding(.horizontal, 22)
                .padding(.top, 22)
                .padding(.bottom, 14)

            Divider()

            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    if let error = viewModel.state.error {
                        TrailErrorState(message: error) { Task { await viewModel.load() } }
                    }
                    if viewModel.state.loading {
                        TrailLoadingState()
                    }
                    if !viewModel.state.loading && viewModel.state.knots.isEmpty && viewModel.state.error == nil {
                        TrailSurfaceCard {
                            TrailSectionTitle(title: "没有找到相关绳结", subtitle: "换一个关键词或分类再试试。")
                        }
                    }
                    ForEach(viewModel.state.knots) { knot in
                        Button {
                            Task { await viewModel.openKnot(knot.id) }
                        } label: {
                            MacKnotRow(
                                knot: knot,
                                selected: viewModel.state.selectedKnotID == knot.id
                            )
                        }
                        .buttonStyle(.plain)
                    }
                    if let nextOffset = viewModel.state.nextOffset {
                        MacKnotAutoLoadFooter(loading: viewModel.state.loadingMore)
                            .id(nextOffset)
                            .task {
                                await viewModel.loadMoreKnots()
                            }
                    }
                }
                .padding(22)
            }
        }
    }

    private var knotToolbar: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("绳结")
                        .font(.largeTitle.weight(.heavy))
                    Text(viewModel.state.categorySummary?.summary ?? "按用途、名称和场景快速找到绳结打法。")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text(viewModel.state.categorySummary.map { "\($0.itemCount) 个" } ?? "\(viewModel.state.knots.count) 个")
                    .font(.headline.weight(.heavy))
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)
                    .background(.thinMaterial, in: Capsule())
            }

            HStack(spacing: 10) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField(
                    "搜索绳结、用途或场景",
                    text: Binding(
                        get: { viewModel.state.searchQuery },
                        set: { viewModel.updateSearchQuery($0) }
                    )
                )
                .textFieldStyle(.plain)
                .onSubmit { Task { await viewModel.submitSearch() } }
                Button {
                    Task { await viewModel.submitSearch() }
                } label: {
                    Image(systemName: "arrow.right")
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .background(Color(nsColor: .textBackgroundColor), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(.quaternary, lineWidth: 1)
            }

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    filterChip(title: "全部", id: nil)
                    ForEach(viewModel.state.filterOptions) { option in
                        filterChip(title: option.title, id: option.id)
                    }
                }
                .padding(.vertical, 2)
            }
        }
    }

    private func filterChip(title: String, id: String?) -> some View {
        let isSelected = viewModel.state.selectedCategoryID == id
        return Button {
            Task { await viewModel.selectCategory(id) }
        } label: {
            Text(title)
                .font(.caption.weight(.bold))
                .padding(.horizontal, 12)
                .padding(.vertical, 7)
                .foregroundStyle(isSelected ? Color.white : Color.primary)
                .background(isSelected ? Color.accentColor : Color(nsColor: .controlBackgroundColor), in: Capsule())
        }
        .buttonStyle(.plain)
    }
}

private struct MacKnotAutoLoadFooter: View {
    let loading: Bool

    var body: some View {
        HStack {
            Spacer()
            if loading {
                ProgressView()
                    .controlSize(.small)
                Text("加载中")
                    .font(.footnote.weight(.medium))
                    .foregroundStyle(.secondary)
            } else {
                Color.clear
                    .frame(width: 1, height: 1)
            }
            Spacer()
        }
        .frame(minHeight: 28)
        .padding(.vertical, 4)
    }
}

private struct MacKnotRow: View {
    let knot: KnotSummary
    let selected: Bool

    var body: some View {
        HStack(spacing: 14) {
            MacKnotThumbnail(asset: knot.media.thumbnailAsset)
                .frame(width: 92, height: 92)

            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline) {
                    Text(knot.title)
                        .font(.headline.weight(.heavy))
                        .lineLimit(1)
                    Spacer(minLength: 8)
                    TrailBadge(text: difficultyLabel(knot.difficulty), tone: .info)
                }
                Text(knot.summary)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                TrailTagRow(tags: Array((knot.categories.map(\.title) + knot.types.map(\.title)).prefix(4)))
            }
            Image(systemName: "chevron.right")
                .font(.caption.weight(.bold))
                .foregroundStyle(.tertiary)
        }
        .padding(12)
        .background(selected ? Color.accentColor.opacity(0.12) : Color(nsColor: .controlBackgroundColor), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(selected ? Color.accentColor.opacity(0.45) : Color.primary.opacity(0.06), lineWidth: 1)
        }
    }
}

private struct MacKnotThumbnail: View {
    let asset: KnotMediaAsset?

    var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.black.opacity(0.86))
            if let url = asset.flatMap({ URL(string: $0.url) }) {
                AsyncImage(url: url) { phase in
                    switch phase {
                    case let .success(image):
                        image
                            .resizable()
                            .scaledToFit()
                            .padding(8)
                    case .failure:
                        placeholder
                    case .empty:
                        ProgressView()
                            .controlSize(.small)
                    @unknown default:
                        placeholder
                    }
                }
            } else {
                placeholder
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var placeholder: some View {
        Image(systemName: "point.3.connected.trianglepath.dotted")
            .font(.title2)
            .foregroundStyle(.white.opacity(0.72))
    }
}

private struct MacKnotDetailPane: View {
    let knot: KnotDetail?
    let loading: Bool
    let error: String?

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 18) {
                if loading {
                    TrailLoadingState()
                }
                if let error {
                    TrailErrorState(message: error)
                }
                if let knot {
                    MacKnotHeroViewer(knot: knot)

                    TrailSurfaceCard {
                        TrailSectionTitle(title: "用途说明")
                        Text(knot.description ?? knot.summary)
                            .foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                } else if !loading {
                    TrailSurfaceCard {
                        TrailSectionTitle(title: "选择一个绳结", subtitle: "左侧列表会只显示绳结内容，点击条目后这里查看高清图、打法和旋转演示。")
                    }
                }
            }
        }
    }
}

private enum KnotViewerMode: String, CaseIterable, Identifiable {
    case still
    case tying
    case turntable

    var id: String { rawValue }

    var title: String {
        switch self {
        case .still: return "高清"
        case .tying: return "打法"
        case .turntable: return "旋转"
        }
    }

    var systemImage: String {
        switch self {
        case .still: return "photo"
        case .tying: return "play.circle"
        case .turntable: return "rotate.3d"
        }
    }
}

private struct MacKnotHeroViewer: View {
    let knot: KnotDetail

    @State private var mode: KnotViewerMode = .still
    @State private var mirrored = false
    @State private var looping = true
    @State private var rotation: Double = 0
    @State private var playing = false
    @State private var player = AVPlayer()

    private var activeAsset: KnotMediaAsset? {
        switch mode {
        case .still:
            return knot.media.previewAsset
        case .tying:
            return knot.media.drawPlayableAsset ?? knot.media.previewAsset
        case .turntable:
            return knot.media.turntablePlayableAsset ?? knot.media.previewAsset
        }
    }

    private var activeURL: URL? {
        activeAsset.flatMap { URL(string: $0.url) }
    }

    private var isVideo: Bool {
        activeAsset?.mimeType.hasPrefix("video/") == true
    }

    var body: some View {
        TrailSurfaceCard(padding: 0) {
            VStack(alignment: .leading, spacing: 0) {
                VStack(spacing: 0) {
                    ZStack {
                        Color.black
                        viewerContent
                            .scaleEffect(x: mirrored ? -1 : 1, y: 1)
                            .rotationEffect(.degrees(rotation))
                            .animation(.easeInOut(duration: 0.18), value: mirrored)
                            .animation(.easeInOut(duration: 0.18), value: rotation)
                    }
                    .frame(minHeight: 360, idealHeight: 460)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        if mode == .still {
                            mode = .tying
                        }
                        playing.toggle()
                    }

                    viewerControls
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .background(Color.black)
                }
                .background(Color.black)
                .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))

                VStack(alignment: .leading, spacing: 12) {
                    HStack(alignment: .firstTextBaseline) {
                        VStack(alignment: .leading, spacing: 5) {
                            Text(knot.title)
                                .font(.title.weight(.heavy))
                            Text(knot.summary)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        TrailBadge(text: difficultyLabel(knot.difficulty), tone: .info)
                    }
                    TrailTagRow(tags: Array((knot.categories.map(\.title) + knot.types.map(\.title)).prefix(6)))
                    if let activeAsset {
                        Text([activeAsset.displayName, activeAsset.attribution].compactMap { $0 }.joined(separator: " · "))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(18)
            }
        }
        .onChange(of: mode) { _, _ in
            resetPlaybackForMode()
        }
        .onChange(of: playing) { _, isPlaying in
            if isPlaying {
                player.play()
            } else {
                player.pause()
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .AVPlayerItemDidPlayToEndTime)) { notification in
            guard let currentItem = player.currentItem,
                  notification.object as? AVPlayerItem === currentItem else { return }
            if looping {
                player.seek(to: .zero)
                player.play()
                playing = true
            } else {
                playing = false
            }
        }
    }

    @ViewBuilder
    private var viewerContent: some View {
        if let activeURL {
            if isVideo {
                MacAVPlayerView(player: player)
                    .onAppear { configurePlayer(url: activeURL) }
            } else {
                AsyncImage(url: activeURL) { phase in
                    switch phase {
                    case let .success(image):
                        image
                            .resizable()
                            .scaledToFit()
                            .padding(18)
                    case .failure:
                        mediaPlaceholder
                    case .empty:
                        ProgressView()
                            .tint(.white)
                    @unknown default:
                        mediaPlaceholder
                    }
                }
            }
        } else {
            mediaPlaceholder
        }
    }

    private var viewerControls: some View {
        HStack(spacing: 0) {
            ForEach(KnotViewerMode.allCases) { item in
                Button {
                    mode = item
                    playing = item != .still
                } label: {
                    Label(item.title, systemImage: item.systemImage)
                        .labelStyle(.iconOnly)
                }
                .help(item.title)
                .buttonStyle(.borderless)
                .frame(width: 42, height: 34)
                .background(mode == item ? Color.white.opacity(0.22) : Color.clear)
            }
            Divider().frame(height: 22)
            Button {
                mirrored.toggle()
            } label: {
                Image(systemName: "arrow.left.and.right")
            }
            .help("镜像")
            .buttonStyle(.borderless)
            .frame(width: 42, height: 34)
            Button {
                rotation -= 90
            } label: {
                Image(systemName: "rotate.left")
            }
            .help("左旋")
            .buttonStyle(.borderless)
            .frame(width: 42, height: 34)
            Button {
                rotation += 90
            } label: {
                Image(systemName: "rotate.right")
            }
            .help("右旋")
            .buttonStyle(.borderless)
            .frame(width: 42, height: 34)
            Button {
                if mode == .still { mode = .tying }
                playing.toggle()
            } label: {
                Image(systemName: playing ? "pause.fill" : "play.fill")
            }
            .help(playing ? "暂停" : "播放")
            .buttonStyle(.borderless)
            .frame(width: 42, height: 34)
            Button {
                looping.toggle()
            } label: {
                Image(systemName: looping ? "repeat.circle.fill" : "repeat")
            }
            .help("循环")
            .buttonStyle(.borderless)
            .frame(width: 42, height: 34)
        }
        .foregroundStyle(.white)
        .padding(5)
        .background(.black.opacity(0.58), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
    }

    private var mediaPlaceholder: some View {
        VStack(spacing: 10) {
            Image(systemName: "point.3.connected.trianglepath.dotted")
                .font(.system(size: 56))
            Text("暂无演示素材")
                .font(.headline)
        }
        .foregroundStyle(.white.opacity(0.72))
    }

    private func resetPlaybackForMode() {
        rotation = 0
        guard let activeURL, isVideo else {
            player.pause()
            playing = false
            return
        }
        configurePlayer(url: activeURL)
        if playing {
            player.play()
        }
    }

    private func configurePlayer(url: URL) {
        let currentURL = (player.currentItem?.asset as? AVURLAsset)?.url
        guard currentURL != url else { return }
        player.replaceCurrentItem(with: AVPlayerItem(url: url))
        if playing {
            player.play()
        }
    }
}

private struct MacAVPlayerView: NSViewRepresentable {
    let player: AVPlayer

    func makeNSView(context: Context) -> AVPlayerView {
        let view = AVPlayerView()
        view.controlsStyle = .inline
        view.videoGravity = .resizeAspect
        view.player = player
        return view
    }

    func updateNSView(_ nsView: AVPlayerView, context: Context) {
        nsView.player = player
    }
}

private func difficultyLabel(_ value: String?) -> String {
    switch value {
    case "leisure": return "入门"
    case "beginner": return "新手"
    case "intermediate": return "进阶"
    case "advanced": return "熟练"
    case "technical": return "技术"
    case let value?: return value
    case nil: return "常用"
    }
}
