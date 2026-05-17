import Foundation
import Combine

@MainActor
final class SkillsViewModel: ObservableObject {
    struct State: Equatable {
        var loading = false
        var loadingMore = false
        var error: String?
        var categories: [SkillCategorySummary] = []
        var knots: [KnotSummary] = []
        var nextOffset: Int?
        var selectedKnot: KnotDetail?
    }

    @Published private(set) var state = State()
    private let repository: any SkillRepositorying

    init(repository: any SkillRepositorying) {
        self.repository = repository
    }

    func load() async {
        state.loading = true
        state.error = nil
        do {
            let categories = try await repository.categories().items
            let knots = try await repository.knots(ListKnotsRequest(offset: 0, limit: 20))
            state.categories = categories
            state.knots = knots.items
            state.nextOffset = knots.page.nextOffset
            state.loading = false
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func loadMoreKnots() async {
        guard let offset = state.nextOffset, !state.loadingMore else { return }
        state.loadingMore = true
        defer { state.loadingMore = false }
        do {
            let response = try await repository.knots(ListKnotsRequest(offset: offset, limit: 20))
            state.knots.append(contentsOf: response.items)
            state.nextOffset = response.page.nextOffset
        } catch {
            state.error = error.localizedDescription
        }
    }

    func openKnot(_ id: String) async {
        do {
            state.selectedKnot = try await repository.knotDetail(id: id)
        } catch {
            state.error = error.localizedDescription
        }
    }

    func closeKnot() {
        state.selectedKnot = nil
    }
}
