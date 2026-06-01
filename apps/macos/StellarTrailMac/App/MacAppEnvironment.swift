import Combine
import Foundation

@MainActor
final class MacAppEnvironment: ObservableObject {
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

    static func makeDefault() -> MacAppEnvironment {
        let arguments = ProcessInfo.processInfo.arguments
        let screenshotMode = arguments.contains("--stellartrail-screenshot-fixtures")
        let settingsStore = AppSettingsStore()

        if screenshotMode {
            let fixture = FixtureRepository()
            let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
            sessionStore.replace(with: Session.fixture)
            return MacAppEnvironment(
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

        let sessionStore = SessionStore(keychainStore: KeychainStore(service: "com.rustella.stellartrail.macos.session"))
        let client = APIClient(settingsStore: settingsStore, sessionStore: sessionStore)
        return MacAppEnvironment(
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
