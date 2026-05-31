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
    let gearPackingRepository: any GearPackingRepositorying
    let tripRepository: any TripRepositorying
    let profileRepository: any ProfileRepositorying
    let roadmapRepository: any RoadmapRepositorying
    let feedbackRepository: any FeedbackRepositorying
    let clientVersionRepository: any ClientVersionRepositorying
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
        gearPackingRepository: any GearPackingRepositorying,
        tripRepository: any TripRepositorying,
        profileRepository: any ProfileRepositorying,
        roadmapRepository: any RoadmapRepositorying,
        feedbackRepository: any FeedbackRepositorying,
        clientVersionRepository: any ClientVersionRepositorying,
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
        self.gearPackingRepository = gearPackingRepository
        self.tripRepository = tripRepository
        self.profileRepository = profileRepository
        self.roadmapRepository = roadmapRepository
        self.feedbackRepository = feedbackRepository
        self.clientVersionRepository = clientVersionRepository
        self.mediaCache = mediaCache
        self.screenshotMode = screenshotMode
    }

    static func makeDefault() -> AppEnvironment {
        let arguments = ProcessInfo.processInfo.arguments
        let screenshotMode = arguments.contains("--stellartrail-screenshot-fixtures")

        if screenshotMode {
            let defaultsSuiteName = "com.rustella.stellartrail.screenshots"
            let defaults = UserDefaults(suiteName: defaultsSuiteName) ?? .standard
            defaults.removePersistentDomain(forName: defaultsSuiteName)
            let settingsStore = AppSettingsStore(defaults: defaults)
            if arguments.contains("--stellartrail-screenshot-dark") {
                settingsStore.themeMode = .dark
            } else if arguments.contains("--stellartrail-screenshot-light") {
                settingsStore.themeMode = .light
            }
            let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
            if arguments.contains("--stellartrail-screenshot-signed-in") {
                sessionStore.replace(with: Session.fixture)
            }
            let fixture = FixtureRepository()
            return AppEnvironment(
                settingsStore: settingsStore,
                sessionStore: sessionStore,
                authRepository: fixture,
                gearRepository: fixture,
                gearAtlasRepository: fixture,
                skillRepository: fixture,
                contentRepository: fixture,
                gearPackingRepository: fixture,
                tripRepository: fixture,
                profileRepository: fixture,
                roadmapRepository: fixture,
                feedbackRepository: fixture,
                clientVersionRepository: fixture,
                mediaCache: FixtureMediaCacheManager(),
                screenshotMode: true
            )
        }

        let settingsStore = AppSettingsStore()
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
            gearPackingRepository: GearPackingRepository(client: client),
            tripRepository: TripRepository(client: client),
            profileRepository: ProfileRepository(client: client),
            roadmapRepository: RoadmapRepository(client: client),
            feedbackRepository: FeedbackRepository(client: client),
            clientVersionRepository: ClientVersionRepository(client: client),
            mediaCache: MediaCacheManager(),
            screenshotMode: false
        )
    }
}
