import SwiftUI

enum AppThemeMode: String, CaseIterable, Identifiable, Codable {
    case system
    case light
    case dark

    var id: String { rawValue }

    var label: String {
        switch self {
        case .system: return "跟随系统"
        case .light: return "浅色"
        case .dark: return "深色"
        }
    }

    var preferredColorScheme: ColorScheme? {
        switch self {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}

@MainActor
final class AppSettingsStore: ObservableObject {
    @Published var themeMode: AppThemeMode {
        didSet { defaults.set(themeMode.rawValue, forKey: Keys.themeMode) }
    }

    @Published var baseURLString: String {
        didSet { defaults.set(baseURLString, forKey: Keys.baseURLString) }
    }

    @Published var assetsBaseURLString: String {
        didSet { defaults.set(assetsBaseURLString, forKey: Keys.assetsBaseURLString) }
    }

    var preferredColorScheme: ColorScheme? { themeMode.preferredColorScheme }
    var baseURL: URL { URL(string: baseURLString) ?? URL(string: defaultClientConfig.apiBaseURLString)! }
    var assetsBaseURL: URL { URL(string: assetsBaseURLString) ?? URL(string: defaultClientConfig.assetsBaseURLString)! }
    var clientIdentity: String { defaultClientConfig.clientIdentity }
    var domainCandidates: [ClientDomainCandidate] { defaultClientConfig.domainCandidates }

    private let defaults: UserDefaults
    private let defaultClientConfig: ClientConfig

    init(defaults: UserDefaults = .standard, clientConfig: ClientConfig = .load()) {
        self.defaults = defaults
        self.defaultClientConfig = clientConfig
        let storedBaseURLString = defaults.string(forKey: Keys.baseURLString)
        let normalizedBaseURLString = ClientConfig.sanitizeAPIBaseURL(
            storedBaseURLString,
            fallback: clientConfig.apiBaseURLString
        )
        let storedAssetsBaseURLString = defaults.string(forKey: Keys.assetsBaseURLString)
        let storedTheme = defaults.string(forKey: Keys.themeMode).flatMap(AppThemeMode.init(rawValue:)) ?? .system
        self.themeMode = storedTheme
        self.baseURLString = normalizedBaseURLString
        self.assetsBaseURLString = ClientConfig.sanitizeBaseURL(
            storedAssetsBaseURLString,
            fallback: clientConfig.assetsBaseURLString
        )
        if let storedBaseURLString, storedBaseURLString != normalizedBaseURLString {
            defaults.set(normalizedBaseURLString, forKey: Keys.baseURLString)
        }
    }

    func resetBaseURL() {
        baseURLString = defaultClientConfig.apiBaseURLString
        assetsBaseURLString = defaultClientConfig.assetsBaseURLString
    }

    private enum Keys {
        static let themeMode = "stellartrail.themeMode"
        static let baseURLString = "stellartrail.baseURLString"
        static let assetsBaseURLString = "stellartrail.assetsBaseURLString"
    }
}
