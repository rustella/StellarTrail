import Foundation
import Combine

@MainActor
final class GearAtlasListViewModel: ObservableObject {
    struct State: Equatable {
        var loading = false
        var loadingMore = false
        var error: String?
        var isLoggedIn = false
        var query = ""
        var activeQuery = ""
        var selectedCategory: GearCategory?
        var sort: GearAtlasSort = .approvedAtDesc
        var items: [GearAtlasPublicItem] = []
        var nextCursor: String?
    }

    @Published private(set) var state = State()

    private let sessionStore: SessionStore
    private let repository: any GearAtlasRepositorying

    init(sessionStore: SessionStore, repository: any GearAtlasRepositorying) {
        self.sessionStore = sessionStore
        self.repository = repository
    }

    func load() async {
        state.loading = true
        state.error = nil
        state.isLoggedIn = sessionStore.isLoggedIn
        state.activeQuery = state.query
        do {
            let response = try await repository.list(request(cursor: nil))
            state.items = response.items
            state.nextCursor = response.nextCursor
            state.loading = false
            state.isLoggedIn = sessionStore.isLoggedIn
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func setQuery(_ query: String) {
        state.query = query
    }

    func selectCategory(_ category: GearCategory?) async {
        state.selectedCategory = category
        await load()
    }

    func selectSort(_ sort: GearAtlasSort) async {
        state.sort = sort
        await load()
    }

    func loadMore() async {
        guard let cursor = state.nextCursor, !state.loadingMore else { return }
        state.loadingMore = true
        defer { state.loadingMore = false }
        do {
            let response = try await repository.list(request(cursor: cursor))
            state.items.append(contentsOf: response.items)
            state.nextCursor = response.nextCursor
        } catch {
            state.error = error.localizedDescription
        }
    }

    private func request(cursor: String?) -> ListGearAtlasRequest {
        ListGearAtlasRequest(category: state.selectedCategory, q: cursor == nil ? state.query : state.activeQuery, sort: state.sort, limit: 20, cursor: cursor)
    }
}

@MainActor
final class GearAtlasDetailViewModel: ObservableObject {
    @Published private(set) var item: GearAtlasPublicItem?
    @Published private(set) var loading = false
    @Published private(set) var error: String?

    private let id: String
    private let repository: any GearAtlasRepositorying

    init(id: String, repository: any GearAtlasRepositorying) {
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
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class GearAtlasSubmitViewModel: ObservableObject {
    @Published var draft = GearFormDraft.blank
    @Published private(set) var specFields = GearOptions.specFieldViews(for: GearFormDraft.blank.category, specs: [:])
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var completed = false

    private let repository: any GearAtlasRepositorying

    init(repository: any GearAtlasRepositorying) {
        self.repository = repository
    }

    func selectCategory(_ category: GearCategory) {
        draft.category = category
        draft.specs = GearOptions.normalizeSpecs(category: category, specs: draft.specs) ?? [:]
        refreshSpecFields()
    }

    func updateSpecValue(key: String, value: String, unit: String) {
        draft.specs[key] = GearOptions.combineSpecValue(value, unit: unit)
        refreshSpecFields()
    }

    func updateSpecUnit(key: String, fieldIndex: Int, unitIndex: Int) {
        guard specFields.indices.contains(fieldIndex) else { return }
        let field = specFields[fieldIndex]
        let unit = field.units.indices.contains(unitIndex) ? field.units[unitIndex] : ""
        draft.specs[key] = GearOptions.combineSpecValue(field.valueText, unit: unit)
        refreshSpecFields()
    }

    func submit() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            _ = try await repository.createSubmission(draft.buildAtlasPayload())
            completed = true
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func refreshSpecFields() {
        specFields = GearOptions.specFieldViews(for: draft.category, specs: draft.specs)
    }
}
