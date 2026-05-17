import Foundation
import Combine

@MainActor
final class GearListViewModel: ObservableObject {
    struct State: Equatable {
        var loading = false
        var loadingMore = false
        var error: String?
        var isLoggedIn = false
        var tab: GearTab = .available
        var query = ""
        var selectedCategory: GearCategory?
        var selectedStatus: GearStatus?
        var sort: GearSort = .createdAtDesc
        var categories: [GearCategoryFilter] = []
        var stats = GearStatsResponse.empty
        var gears: [GearSummary] = []
        var templates: [GearTemplate] = []
        var nextCursor: String?
    }

    @Published private(set) var state = State()

    private let sessionStore: SessionStore
    private let gearRepository: any GearRepositorying
    private let contentRepository: any ContentRepositorying

    init(sessionStore: SessionStore, gearRepository: any GearRepositorying, contentRepository: any ContentRepositorying) {
        self.sessionStore = sessionStore
        self.gearRepository = gearRepository
        self.contentRepository = contentRepository
    }

    func load() async {
        state.loading = true
        state.error = nil
        state.isLoggedIn = sessionStore.isLoggedIn
        do {
            if sessionStore.isLoggedIn {
                let stats = try await gearRepository.stats()
                let categories = try await gearRepository.categories(tab: state.tab).items
                let response = try await gearRepository.list(currentRequest(cursor: nil))
                state.stats = stats
                state.categories = categories
                state.gears = response.items
                state.nextCursor = response.nextCursor
                state.templates = []
            } else {
                state.templates = try await contentRepository.gearTemplates().items
                state.gears = []
                state.categories = []
                state.stats = .empty
                state.nextCursor = nil
            }
            state.loading = false
            state.isLoggedIn = sessionStore.isLoggedIn
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func refreshWith(tab: GearTab? = nil, category: GearCategory? = nil, status: GearStatus? = nil, sort: GearSort? = nil, query: String? = nil) async {
        if let tab { state.tab = tab }
        if let category { state.selectedCategory = category }
        if let status { state.selectedStatus = status }
        if let sort { state.sort = sort }
        if let query { state.query = query }
        await load()
    }

    func clearCategory() async {
        state.selectedCategory = nil
        await load()
    }

    func loadMore() async {
        guard let cursor = state.nextCursor, !state.loadingMore else { return }
        state.loadingMore = true
        defer { state.loadingMore = false }
        do {
            let response = try await gearRepository.list(currentRequest(cursor: cursor))
            state.gears.append(contentsOf: response.items)
            state.nextCursor = response.nextCursor
        } catch {
            state.error = error.localizedDescription
        }
    }

    private func currentRequest(cursor: String?) -> ListGearsRequest {
        ListGearsRequest(tab: state.tab, category: state.selectedCategory, status: state.selectedStatus, q: state.query, sort: state.sort, limit: 20, cursor: cursor)
    }
}

@MainActor
final class GearDetailViewModel: ObservableObject {
    @Published private(set) var item: GearItem?
    @Published private(set) var loading = false
    @Published private(set) var error: String?

    private let id: String
    private let repository: any GearRepositorying

    init(id: String, repository: any GearRepositorying) {
        self.id = id
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        do {
            item = try await repository.get(id: id)
            loading = false
        } catch {
            loading = false
            error = error.localizedDescription
        }
    }

    func archive() async {
        do {
            try await repository.archive(id: id)
            await load()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func restore() async {
        do {
            item = try await repository.restore(id: id)
        } catch {
            self.error = error.localizedDescription
        }
    }
}
