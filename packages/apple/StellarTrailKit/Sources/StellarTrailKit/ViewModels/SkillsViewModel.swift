import Foundation
import Combine

@MainActor
final class SkillsViewModel: ObservableObject {
    struct KnotFilterOption: Equatable, Identifiable {
        let id: String
        let title: String
    }

    struct State: Equatable {
        var loading = false
        var loadingMore = false
        var detailLoading = false
        var error: String?
        var detailError: String?
        var categories: [SkillCategorySummary] = []
        var categorySummary: SkillCategorySummary?
        var filterOptions: [KnotFilterOption] = SkillsViewModel.defaultFilterOptions
        var selectedCategoryID: String?
        var searchQuery = ""
        var activeQuery = ""
        var knots: [KnotSummary] = []
        var nextOffset: Int?
        var selectedKnotID: String?
        var selectedKnot: KnotDetail?
        var cacheProgress: MediaCacheProgress?
        var cacheResult: MediaCacheResult?
        var cacheMessage: String?

        var selectedCategoryTitle: String {
            guard let selectedCategoryID else { return "全部" }
            return filterOptions.first { $0.id == selectedCategoryID }?.title ?? "全部"
        }
    }

    @Published private(set) var state = State()
    private let repository: any SkillRepositorying
    private let mediaCache: any MediaCacheManaging

    init(repository: any SkillRepositorying, mediaCache: any MediaCacheManaging) {
        self.repository = repository
        self.mediaCache = mediaCache
    }

    func load() async {
        state.loading = true
        state.error = nil
        do {
            async let categoriesResponse = repository.categories()
            async let knotsResponse = repository.knots(currentRequest(offset: 0))
            let categoriesPayload = try await categoriesResponse
            let categories = categoriesPayload.items
            let knots = try await knotsResponse
            state.categories = categories
            state.categorySummary = categories.first { $0.slug == "knots" || $0.id == "knots" }
            apply(knots, appending: false)
            state.loading = false
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func updateSearchQuery(_ value: String) {
        state.searchQuery = value
    }

    func submitSearch() async {
        await reloadKnots()
    }

    func selectCategory(_ id: String?) async {
        guard state.selectedCategoryID != id else { return }
        state.selectedCategoryID = id
        await reloadKnots()
    }

    func reloadKnots() async {
        state.loading = true
        state.error = nil
        state.selectedKnot = nil
        state.selectedKnotID = nil
        state.activeQuery = state.searchQuery.trimmingCharacters(in: .whitespacesAndNewlines)
        do {
            let response = try await repository.knots(currentRequest(offset: 0))
            apply(response, appending: false)
            state.loading = false
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func loadMoreKnots() async {
        guard let offset = state.nextOffset, !state.loading, !state.loadingMore else { return }
        state.loadingMore = true
        defer { state.loadingMore = false }
        do {
            let response = try await repository.knots(currentRequest(offset: offset))
            apply(response, appending: true)
        } catch {
            state.error = error.localizedDescription
        }
    }

    func openKnot(_ id: String) async {
        let switchingKnot = state.selectedKnotID != id
        state.selectedKnotID = id
        state.detailLoading = true
        state.detailError = nil
        if switchingKnot {
            state.selectedKnot = nil
        }
        do {
            let detail = try await repository.knotDetail(id: id)
            guard state.selectedKnotID == id else { return }
            state.selectedKnot = detail
            state.detailLoading = false
        } catch {
            guard state.selectedKnotID == id else { return }
            state.detailLoading = false
            state.detailError = error.localizedDescription
        }
    }

    func closeKnot() {
        state.selectedKnotID = nil
        state.selectedKnot = nil
        state.detailError = nil
    }

    func cacheAllKnots() async {
        guard !state.loading else { return }
        state.loading = true
        state.error = nil
        state.cacheResult = nil
        state.cacheMessage = "正在准备离线缓存"
        defer { state.loading = false }
        do {
            let manifest = try await repository.offlineManifest()
            let urls = manifest.items
                .flatMap(\.media)
                .compactMap { URL(string: $0.url) }
            let result = await mediaCache.cache(urls: urls, estimatedBytes: manifest.estimatedBytes) { [weak self] progress in
                self?.state.cacheProgress = progress
                self?.state.cacheMessage = progress.label
            }
            state.cacheResult = result
            state.cacheMessage = result.label
        } catch {
            state.error = error.localizedDescription
            state.cacheMessage = nil
        }
    }

    func resolvedMediaURL(for asset: KnotMediaAsset) -> URL? {
        guard let url = URL(string: asset.url) else { return nil }
        return mediaCache.resolvedURL(for: url)
    }

    private func currentRequest(offset: Int) -> ListKnotsRequest {
        ListKnotsRequest(
            offset: offset,
            limit: 24,
            category: state.selectedCategoryID,
            q: offset == 0 ? state.searchQuery : state.activeQuery
        )
    }

    private func apply(_ response: KnotListResponse, appending: Bool) {
        if appending {
            state.knots.append(contentsOf: response.items)
        } else {
            state.knots = response.items
        }
        state.nextOffset = response.page.nextOffset
        updateFilterOptions(with: state.knots)
    }

    private func updateFilterOptions(with knots: [KnotSummary]) {
        var options = Self.defaultFilterOptions
        let existing = Set(options.map(\.id))
        let discovered = knots
            .flatMap(\.categories)
            .filter { !existing.contains($0.id) }
            .reduce(into: [String: String]()) { partialResult, category in
                partialResult[category.id] = category.title
            }
            .map { KnotFilterOption(id: $0.key, title: $0.value) }
            .sorted { $0.title.localizedStandardCompare($1.title) == .orderedAscending }
        options.append(contentsOf: discovered)
        state.filterOptions = options
    }

    private nonisolated static let defaultFilterOptions: [KnotFilterOption] = [
        KnotFilterOption(id: "essential-knots", title: "必备"),
        KnotFilterOption(id: "camping-knots", title: "露营"),
        KnotFilterOption(id: "climbing-knots", title: "攀岩"),
        KnotFilterOption(id: "fishing-knots", title: "钓鱼"),
        KnotFilterOption(id: "boating-knots", title: "划船"),
        KnotFilterOption(id: "fire-search-rescue-sar-knots", title: "消防救援"),
        KnotFilterOption(id: "arborist-knots", title: "树艺师"),
        KnotFilterOption(id: "caving-knots", title: "探洞"),
        KnotFilterOption(id: "scouting-knots", title: "童军"),
        KnotFilterOption(id: "decorative-knots", title: "装饰")
    ]
}

extension KnotMediaAsset {
    var displayName: String {
        switch mediaType {
        case "thumbnail": return "缩略图"
        case "preview": return "高清图"
        case "draw_gif": return "打法动图"
        case "turntable_gif": return "旋转动图"
        case "draw_mp4": return "打法视频"
        case "turntable_mp4": return "旋转视频"
        default: return "素材"
        }
    }
}

extension Collection where Element == KnotMediaAsset {
    func preferredAsset(for mediaType: String) -> KnotMediaAsset? {
        first { $0.mediaType == mediaType }
    }

    var thumbnailAsset: KnotMediaAsset? {
        preferredAsset(for: "thumbnail") ?? first { $0.mimeType.hasPrefix("image/") }
    }

    var previewAsset: KnotMediaAsset? {
        preferredAsset(for: "preview") ?? thumbnailAsset
    }

    var drawPlayableAsset: KnotMediaAsset? {
        preferredAsset(for: "draw_mp4") ?? preferredAsset(for: "draw_gif")
    }

    var turntablePlayableAsset: KnotMediaAsset? {
        preferredAsset(for: "turntable_mp4") ?? preferredAsset(for: "turntable_gif")
    }
}
