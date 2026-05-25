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
            .preferredColorScheme(settingsStore.themeMode.preferredColorScheme)
    }
}

extension View {
    func trailTheme(settingsStore: AppSettingsStore) -> some View {
        modifier(TrailThemeModifier(settingsStore: settingsStore))
    }

    func trailScreenBackground() -> some View {
        modifier(TrailScreenBackgroundModifier())
    }
}

private struct TrailScreenBackgroundModifier: ViewModifier {
    @Environment(\.trailPalette) private var palette

    func body(content: Content) -> some View {
#if os(iOS)
        content
            .scrollContentBackground(.hidden)
            .background(TrailPageBackground().ignoresSafeArea())
            .toolbarBackground(palette.pageBackground, for: .navigationBar)
            .toolbarBackground(.visible, for: .navigationBar)
            .toolbarColorScheme(palette.isDark ? .dark : .light, for: .navigationBar)
            .navigationBarTitleDisplayMode(.inline)
#else
        content
            .scrollContentBackground(.hidden)
            .background(TrailPageBackground().ignoresSafeArea())
#endif
    }
}

struct TrailPageBackground: View {
    @Environment(\.trailPalette) private var palette

    var body: some View {
        ZStack {
            if palette.isDark {
                darkBackground
            } else {
                palette.pageBackground
                LinearGradient(
                    colors: [palette.pageBackground, palette.heroStart.opacity(0.20), palette.pageBackground],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .opacity(0.45)
            }
        }
    }

    private var darkBackground: some View {
        GeometryReader { proxy in
            ZStack {
                LinearGradient(
                    colors: [
                        Color(hex: 0x07051A),
                        Color(hex: 0x12082E),
                        Color(hex: 0x21104D)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )

                LinearGradient(
                    stops: [
                        .init(color: .clear, location: 0.00),
                        .init(color: .clear, location: 0.34),
                        .init(color: palette.brand.opacity(0.08), location: 0.45),
                        .init(color: Color(hex: 0x2DD4BF).opacity(0.055), location: 0.52),
                        .init(color: .clear, location: 0.66),
                        .init(color: .clear, location: 1.00)
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )

                TrailStarField(size: proxy.size)
                    .opacity(0.82)
            }
        }
        .background(palette.pageBackground)
    }
}

private struct TrailStarField: View {
    let size: CGSize

    private let stars: [(x: CGFloat, y: CGFloat, opacity: Double)] = [
        (0.16, 0.22, 0.36),
        (0.62, 0.18, 0.28),
        (0.88, 0.42, 0.24),
        (0.28, 0.08, 0.18),
        (0.72, 0.32, 0.20),
        (0.10, 0.60, 0.16),
        (0.46, 0.72, 0.14),
        (0.84, 0.82, 0.16)
    ]

    var body: some View {
        ZStack {
            ForEach(Array(stars.enumerated()), id: \.offset) { _, star in
                Circle()
                    .fill(Color.white.opacity(star.opacity))
                    .frame(width: 2, height: 2)
                    .position(x: size.width * star.x, y: size.height * star.y)
            }
        }
    }
}
