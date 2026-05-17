import SwiftUI

private struct TrailPaletteKey: EnvironmentKey {
    static let defaultValue = TrailColors.light
}

extension EnvironmentValues {
    var trailPalette: TrailPalette {
        get { self[TrailPaletteKey.self] }
        set { self[TrailPaletteKey.self] = newValue }
    }
}

struct TrailThemeModifier: ViewModifier {
    @ObservedObject var settingsStore: AppSettingsStore
    @Environment(\.colorScheme) private var colorScheme

    var palette: TrailPalette {
        switch settingsStore.themeMode {
        case .light:
            return TrailColors.light
        case .dark:
            return TrailColors.dark
        case .system:
            return colorScheme == .dark ? TrailColors.dark : TrailColors.light
        }
    }

    func body(content: Content) -> some View {
        content
            .environment(\.trailPalette, palette)
            .background(TrailPageBackground().ignoresSafeArea())
    }
}

extension View {
    func trailTheme(settingsStore: AppSettingsStore) -> some View {
        modifier(TrailThemeModifier(settingsStore: settingsStore))
    }
}

struct TrailPageBackground: View {
    @Environment(\.trailPalette) private var palette

    var body: some View {
        ZStack {
            palette.pageBackground
            LinearGradient(
                colors: [palette.pageBackground, palette.heroStart.opacity(0.20), palette.pageBackground],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .opacity(!palette.isDark ? 0.45 : 0.90)
        }
    }
}
