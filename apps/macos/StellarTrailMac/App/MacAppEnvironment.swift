import Combine
import Foundation

@MainActor
final class MacAppEnvironment: ObservableObject {
    let settingsStore: AppSettingsStore
    let sessionStore: SessionStore
    let authRepository: any AuthRepositorying
    let gearRepository: any GearRepositorying
    let skillRepository: any SkillRepositorying
    let contentRepository: any ContentRepositorying
    let screenshotMode: Bool

    init(
        settingsStore: AppSettingsStore,
        sessionStore: SessionStore,
        authRepository: any AuthRepositorying,
        gearRepository: any GearRepositorying,
        skillRepository: any SkillRepositorying,
        contentRepository: any ContentRepositorying,
        screenshotMode: Bool = false
    ) {
        self.settingsStore = settingsStore
        self.sessionStore = sessionStore
        self.authRepository = authRepository
        self.gearRepository = gearRepository
        self.skillRepository = skillRepository
        self.contentRepository = contentRepository
        self.screenshotMode = screenshotMode
    }

    static func makeDefault() -> MacAppEnvironment {
        let arguments = ProcessInfo.processInfo.arguments
        let screenshotMode = arguments.contains("--stellartrail-screenshot-fixtures")
        let settingsStore = AppSettingsStore(clientConfig: ClientConfig.load(client: "mac", version: "0.1.0"))

        if screenshotMode {
            let fixture = FixtureRepository()
            let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
            sessionStore.replace(with: Session.fixture)
            return MacAppEnvironment(
                settingsStore: settingsStore,
                sessionStore: sessionStore,
                authRepository: fixture,
                gearRepository: fixture,
                skillRepository: fixture,
                contentRepository: fixture,
                screenshotMode: true
            )
        }

        let sessionStore = SessionStore(keychainStore: KeychainStore(service: "com.rustella.stellartrail.macos.session"))
        let client = APIClient(settingsStore: settingsStore, sessionStore: sessionStore)
        return MacAppEnvironment(
            settingsStore: settingsStore,
            sessionStore: sessionStore,
            authRepository: AuthRepository(client: client, sessionStore: sessionStore),
            gearRepository: GearRepository(client: client),
            skillRepository: SkillRepository(client: client),
            contentRepository: ContentRepository(client: client),
            screenshotMode: false
        )
    }
}
