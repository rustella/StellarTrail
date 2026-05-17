import Foundation
import Combine

@MainActor
final class ProfileViewModel: ObservableObject {
    @Published private(set) var session: Session?
    @Published var baseURLString: String

    let settingsStore: AppSettingsStore
    private let sessionStore: SessionStore
    private var cancellables: Set<AnyCancellable> = []

    init(settingsStore: AppSettingsStore, sessionStore: SessionStore) {
        self.settingsStore = settingsStore
        self.sessionStore = sessionStore
        self.session = sessionStore.currentSession
        self.baseURLString = settingsStore.baseURLString
        sessionStore.$currentSession.assign(to: &$session)
    }

    var canEditBaseURL: Bool {
        #if DEBUG
        true
        #else
        false
        #endif
    }

    var themeMode: AppThemeMode {
        get { settingsStore.themeMode }
        set { settingsStore.themeMode = newValue }
    }

    func updateBaseURL() {
        settingsStore.baseURLString = baseURLString
    }

    func resetBaseURL() {
        settingsStore.resetBaseURL()
        baseURLString = settingsStore.baseURLString
    }

    func logout() {
        sessionStore.clear()
    }

    func setFixtureSignedIn() {
        sessionStore.replace(with: Session.fixture)
    }
}
