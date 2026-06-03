import Foundation
import Combine

@MainActor
final class AppEnvironment: ObservableObject {
    let settingsStore: AppSettingsStore
    let sessionStore: SessionStore
    let authRepository: any AuthRepositorying
    let gearRepository: any GearRepositorying
    let gearAtlasRepository: any GearAtlasRepositorying
    let skillRepository: any SkillRepositorying
    let contentRepository: any ContentRepositorying
    let mediaCache: any MediaCacheManaging
    let screenshotMode: Bool

    init(
        settingsStore: AppSettingsStore,
        sessionStore: SessionStore,
        authRepository: any AuthRepositorying,
        gearRepository: any GearRepositorying,
        gearAtlasRepository: any GearAtlasRepositorying,
        skillRepository: any SkillRepositorying,
        contentRepository: any ContentRepositorying,
        mediaCache: any MediaCacheManaging,
        screenshotMode: Bool = false
    ) {
        self.settingsStore = settingsStore
        self.sessionStore = sessionStore
        self.authRepository = authRepository
        self.gearRepository = gearRepository
        self.gearAtlasRepository = gearAtlasRepository
        self.skillRepository = skillRepository
        self.contentRepository = contentRepository
        self.mediaCache = mediaCache
        self.screenshotMode = screenshotMode
    }

    static func makeDefault() -> AppEnvironment {
        let arguments = ProcessInfo.processInfo.arguments
        let screenshotMode = arguments.contains("--stellartrail-screenshot-fixtures")
        let clientConfig = ClientConfig.load(client: "ios", version: "0.1.0")

        if screenshotMode {
            let defaultsSuiteName = "com.rustella.stellartrail.screenshots"
            let defaults = UserDefaults(suiteName: defaultsSuiteName) ?? .standard
            defaults.removePersistentDomain(forName: defaultsSuiteName)
            let settingsStore = AppSettingsStore(defaults: defaults, clientConfig: clientConfig)
            let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
            let fixture = FixtureRepository()
            return AppEnvironment(
                settingsStore: settingsStore,
                sessionStore: sessionStore,
                authRepository: fixture,
                gearRepository: fixture,
                gearAtlasRepository: fixture,
                skillRepository: fixture,
                contentRepository: fixture,
                mediaCache: FixtureMediaCacheManager(),
                screenshotMode: true
            )
        }

        let settingsStore = AppSettingsStore(clientConfig: clientConfig)
        let keychainStore = KeychainStore(service: "com.rustella.stellartrail.session")
        let sessionStore = SessionStore(keychainStore: keychainStore)
        let client = APIClient(settingsStore: settingsStore, sessionStore: sessionStore)
        return AppEnvironment(
            settingsStore: settingsStore,
            sessionStore: sessionStore,
            authRepository: AuthRepository(client: client, sessionStore: sessionStore),
            gearRepository: GearRepository(client: client),
            gearAtlasRepository: GearAtlasRepository(client: client),
            skillRepository: SkillRepository(client: client),
            contentRepository: ContentRepository(client: client),
            mediaCache: MediaCacheManager(),
            screenshotMode: false
        )
    }
}
