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

    var preferredColorScheme: ColorScheme? { themeMode.preferredColorScheme }
    var baseURL: URL { URL(string: baseURLString) ?? URL(string: Self.defaultBaseURLString)! }

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        let storedTheme = defaults.string(forKey: Keys.themeMode).flatMap(AppThemeMode.init(rawValue:)) ?? .system
        self.themeMode = storedTheme
        self.baseURLString = defaults.string(forKey: Keys.baseURLString) ?? Self.defaultBaseURLString
    }

    func resetBaseURL() {
        baseURLString = Self.defaultBaseURLString
    }

    private enum Keys {
        static let themeMode = "stellartrail.themeMode"
        static let baseURLString = "stellartrail.baseURLString"
    }

    static let defaultBaseURLString = "http://127.0.0.1:8080"
}
