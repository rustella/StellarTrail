import Foundation
import Combine

@MainActor
final class HomeViewModel: ObservableObject {
    struct State: Equatable {
        var loading = false
        var error: String?
        var isLoggedIn = false
        var stats = GearStatsResponse.empty
        var recentGears: [GearSummary] = []
        var templates: [GearTemplate] = []
        var skills: [SkillCategorySummary] = []
    }

    @Published private(set) var state = State()

    private let sessionStore: SessionStore
    private let gearRepository: any GearRepositorying
    private let skillRepository: any SkillRepositorying
    private let contentRepository: any ContentRepositorying

    init(sessionStore: SessionStore, gearRepository: any GearRepositorying, skillRepository: any SkillRepositorying, contentRepository: any ContentRepositorying) {
        self.sessionStore = sessionStore
        self.gearRepository = gearRepository
        self.skillRepository = skillRepository
        self.contentRepository = contentRepository
    }

    func load() async {
        state.loading = true
        state.error = nil
        state.isLoggedIn = sessionStore.isLoggedIn
        do {
            let templates = try await contentRepository.gearTemplates().items
            let skills = try await skillRepository.categories().items
            var stats = GearStatsResponse.empty
            var recent: [GearSummary] = []
            if sessionStore.isLoggedIn {
                stats = try await gearRepository.stats()
                recent = try await gearRepository.list(ListGearsRequest(limit: 3)).items
            }
            state = State(loading: false, error: nil, isLoggedIn: sessionStore.isLoggedIn, stats: stats, recentGears: recent, templates: templates, skills: skills)
        } catch {
            state.loading = false
            state.error = error.localizedDescription
        }
    }

    func setFixtureSignedIn() {
        sessionStore.replace(with: Session.fixture)
        state.isLoggedIn = true
    }
}
