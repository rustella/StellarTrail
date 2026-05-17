import SwiftUI

struct TrailSurfaceCard<Content: View>: View {
    @Environment(\.trailPalette) private var palette
    var padding: CGFloat
    let content: Content

    init(padding: CGFloat = 18, @ViewBuilder content: () -> Content) {
        self.padding = padding
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(padding)
        .background(palette.surface)
        .overlay(
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .stroke(palette.softBorder, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
        .shadow(color: !palette.isDark ? Color.black.opacity(0.05) : Color.clear, radius: 14, x: 0, y: 8)
    }
}

struct TrailHeroCard<Actions: View>: View {
    @Environment(\.trailPalette) private var palette
    let eyebrow: String
    let title: String
    let subtitle: String
    let chips: [String]
    let actions: Actions

    init(
        eyebrow: String,
        title: String,
        subtitle: String,
        chips: [String] = [],
        @ViewBuilder actions: () -> Actions = { EmptyView() }
    ) {
        self.eyebrow = eyebrow
        self.title = title
        self.subtitle = subtitle
        self.chips = chips
        self.actions = actions()
    }

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            LinearGradient(colors: [palette.heroStart, palette.heroMid, palette.heroEnd], startPoint: .topLeading, endPoint: .bottomTrailing)
            TrailHeroDecoration()
            VStack(alignment: .leading, spacing: 12) {
                Text(eyebrow)
                    .font(.caption.weight(.heavy))
                    .foregroundStyle(palette.brandSoftText)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(.white.opacity(!palette.isDark ? 0.65 : 0.10))
                    .clipShape(Capsule())
                Text(title)
                    .font(.title.weight(.heavy))
                    .foregroundStyle(!palette.isDark ? palette.textPrimary : .white)
                    .fixedSize(horizontal: false, vertical: true)
                Text(subtitle)
                    .font(.body)
                    .foregroundStyle(!palette.isDark ? palette.textMuted : .white.opacity(0.84))
                    .fixedSize(horizontal: false, vertical: true)
                if !chips.isEmpty {
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(chips, id: \.self) { chip in
                                TrailHeroChip(text: chip)
                            }
                        }
                    }
                }
                actions
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(20)
        }
        .clipShape(RoundedRectangle(cornerRadius: 26, style: .continuous))
        .accessibilityElement(children: .contain)
    }
}

private struct TrailHeroDecoration: View {
    @Environment(\.trailPalette) private var palette

    var body: some View {
        GeometryReader { proxy in
            let width = proxy.size.width
            let height = proxy.size.height
            ZStack {
                Circle()
                    .fill(palette.heroSun.opacity(0.86))
                    .frame(width: 58, height: 58)
                    .position(x: width - 52, y: 48)
                Circle()
                    .fill(palette.heroSun.opacity(0.22))
                    .frame(width: 108, height: 108)
                    .position(x: width - 52, y: 48)
                Path { path in
                    path.move(to: CGPoint(x: width * 0.25, y: height))
                    path.addQuadCurve(to: CGPoint(x: width, y: height * 0.62), control: CGPoint(x: width * 0.70, y: height * 0.30))
                    path.addLine(to: CGPoint(x: width, y: height))
                    path.closeSubpath()
                }
                .fill(palette.heroHill.opacity(0.48))
                Path { path in
                    path.move(to: CGPoint(x: 0, y: height))
                    path.addQuadCurve(to: CGPoint(x: width, y: height * 0.76), control: CGPoint(x: width * 0.55, y: height * 0.45))
                    path.addLine(to: CGPoint(x: width, y: height))
                    path.closeSubpath()
                }
                .fill(palette.brandSoft.opacity(0.42))
            }
        }
        .allowsHitTesting(false)
    }
}

private struct TrailHeroChip: View {
    @Environment(\.trailPalette) private var palette
    let text: String

    var body: some View {
        Text(text)
            .font(.caption.weight(.bold))
            .foregroundStyle(palette.brandSoftText)
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            .background(palette.brandSoft.opacity(!palette.isDark ? 0.95 : 0.78))
            .clipShape(Capsule())
    }
}

enum TrailBadgeTone {
    case brand
    case neutral
    case success
    case warning
    case danger
    case info
}

struct TrailBadge: View {
    @Environment(\.trailPalette) private var palette
    let text: String
    var tone: TrailBadgeTone = .neutral

    var body: some View {
        Text(text)
            .font(.caption.weight(.bold))
            .foregroundStyle(colors.text)
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(colors.background)
            .clipShape(Capsule())
    }

    private var colors: (text: Color, background: Color) {
        switch tone {
        case .brand: return (palette.brandSoftText, palette.brandSoft)
        case .neutral: return (palette.textMuted, palette.controlBackground)
        case .success: return (palette.successText, palette.successBackground)
        case .warning: return (palette.warningText, palette.warningBackground)
        case .danger: return (palette.dangerText, palette.dangerBackground)
        case .info: return (palette.infoText, palette.infoBackground)
        }
    }
}

struct TrailMetricTile: View {
    @Environment(\.trailPalette) private var palette
    let value: String
    let label: String
    var hint: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(value)
                .font(.title3.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
                .minimumScaleFactor(0.75)
            Text(label)
                .font(.caption.weight(.bold))
                .foregroundStyle(palette.textMuted)
            if let hint, !hint.isEmpty {
                Text(hint)
                    .font(.caption2)
                    .foregroundStyle(palette.textMuted)
                    .lineLimit(2)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(palette.controlBackground)
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
    }
}

struct TrailSectionTitle: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    var subtitle: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            if let subtitle {
                Text(subtitle)
                    .font(.subheadline)
                    .foregroundStyle(palette.textMuted)
            }
        }
    }
}

struct TrailPrimaryButton: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let action: () -> Void
    var isDisabled = false

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.headline.weight(.bold))
                .frame(maxWidth: .infinity)
                .padding(.vertical, 13)
        }
        .buttonStyle(.plain)
        .foregroundStyle(palette.brandText)
        .background(isDisabled ? palette.textMuted.opacity(0.35) : palette.brand)
        .clipShape(Capsule())
        .disabled(isDisabled)
    }
}

struct TrailSoftButton: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.headline.weight(.bold))
                .frame(maxWidth: .infinity)
                .padding(.vertical, 13)
        }
        .buttonStyle(.plain)
        .foregroundStyle(palette.brandSoftText)
        .background(palette.brandSoft)
        .clipShape(Capsule())
    }
}

struct TrailLoadingState: View {
    @Environment(\.trailPalette) private var palette

    var body: some View {
        TrailSurfaceCard {
            HStack(spacing: 12) {
                ProgressView()
                Text("正在加载…")
                    .foregroundStyle(palette.textMuted)
            }
        }
    }
}

struct TrailErrorState: View {
    @Environment(\.trailPalette) private var palette
    let message: String
    var retry: (() -> Void)? = nil

    var body: some View {
        TrailSurfaceCard {
            Text(message)
                .foregroundStyle(palette.dangerText)
            if let retry {
                TrailPrimaryButton(title: "重试", action: retry)
            }
        }
        .background(palette.dangerBackground.opacity(0.20))
    }
}

struct TrailEmptyState: View {
    @Environment(\.trailPalette) private var palette
    let title: String
    let subtitle: String

    var body: some View {
        TrailSurfaceCard {
            Text(title)
                .font(.headline.weight(.heavy))
                .foregroundStyle(palette.textPrimary)
            Text(subtitle)
                .foregroundStyle(palette.textMuted)
        }
    }
}

struct TrailInfoRow: View {
    @Environment(\.trailPalette) private var palette
    let label: String
    let value: String

    var body: some View {
        HStack(alignment: .top) {
            Text(label)
                .foregroundStyle(palette.textMuted)
            Spacer(minLength: 12)
            Text(value)
                .foregroundStyle(palette.textPrimary)
                .multilineTextAlignment(.trailing)
        }
        .font(.subheadline)
    }
}

struct TrailTagRow: View {
    let tags: [String]

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(tags, id: \.self) { tag in
                    TrailBadge(text: tag, tone: .brand)
                }
            }
        }
    }
}
