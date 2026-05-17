import SwiftUI

struct TrailPalette {
    let isDark: Bool
    let pageBackground: Color
    let surface: Color
    let surfaceStrong: Color
    let controlBackground: Color
    let border: Color
    let softBorder: Color
    let textPrimary: Color
    let textMuted: Color
    let headingMuted: Color
    let accent: Color
    let brand: Color
    let brandText: Color
    let brandSoft: Color
    let brandSoftText: Color
    let softControlBackground: Color
    let softControlText: Color
    let chipBackground: Color
    let footerBackground: Color
    let heroStart: Color
    let heroMid: Color
    let heroEnd: Color
    let heroHill: Color
    let heroSun: Color
    let successText: Color
    let successBackground: Color
    let warningText: Color
    let warningBackground: Color
    let dangerText: Color
    let dangerBackground: Color
    let infoText: Color
    let infoBackground: Color
}

enum TrailColors {
    static let light = TrailPalette(
        isDark: false,
        pageBackground: Color(hex: 0xF8FAFC),
        surface: .white,
        surfaceStrong: .white,
        controlBackground: Color(hex: 0xF8FAFC),
        border: Color(hex: 0xE2E8F0),
        softBorder: Color(hex: 0xF1F5F9),
        textPrimary: Color(hex: 0x0F172A),
        textMuted: Color(hex: 0x64748B),
        headingMuted: Color(hex: 0x334155),
        accent: Color(hex: 0x0F766E),
        brand: Color(hex: 0x0F766E),
        brandText: .white,
        brandSoft: Color(hex: 0xCCFBF1),
        brandSoftText: Color(hex: 0x0F766E),
        softControlBackground: Color(hex: 0xE2E8F0),
        softControlText: Color(hex: 0x475569),
        chipBackground: Color(hex: 0xECFDF5),
        footerBackground: .white.opacity(0.96),
        heroStart: Color(hex: 0xFFF7ED),
        heroMid: Color(hex: 0xECFEFF),
        heroEnd: Color(hex: 0xEEF2FF),
        heroHill: Color(hex: 0xD8F1F6),
        heroSun: Color(hex: 0xFBBF24),
        successText: Color(hex: 0x047857),
        successBackground: Color(hex: 0xD1FAE5),
        warningText: Color(hex: 0xB45309),
        warningBackground: Color(hex: 0xFEF3C7),
        dangerText: Color(hex: 0xDC2626),
        dangerBackground: Color(hex: 0xFFF1F2),
        infoText: Color(hex: 0x2563EB),
        infoBackground: Color(hex: 0xEFF6FF)
    )

    static let dark = TrailPalette(
        isDark: true,
        pageBackground: Color(hex: 0x07051A),
        surface: Color(hex: 0x181234).opacity(0.90),
        surfaceStrong: Color(hex: 0x17112F),
        controlBackground: Color(hex: 0x120D2C),
        border: Color(hex: 0x3D2D63),
        softBorder: Color(hex: 0x332555),
        textPrimary: Color(hex: 0xF6F1FF),
        textMuted: Color(hex: 0xC7B9F4),
        headingMuted: Color(hex: 0xDDD6FE),
        accent: Color(hex: 0xE879F9),
        brand: Color(hex: 0xA78BFA),
        brandText: Color(hex: 0x12071F),
        brandSoft: Color(hex: 0x2A1F4F),
        brandSoftText: Color(hex: 0xEDE7FF),
        softControlBackground: Color(hex: 0x2A1F4F),
        softControlText: Color(hex: 0xEDE7FF),
        chipBackground: Color(hex: 0x2A1F4F),
        footerBackground: Color(hex: 0x0E0A22).opacity(0.94),
        heroStart: Color(hex: 0x12082E),
        heroMid: Color(hex: 0x4C1D95),
        heroEnd: Color(hex: 0x0F766E),
        heroHill: Color(hex: 0x1F3F4A),
        heroSun: Color(hex: 0xFDE68A),
        successText: Color(hex: 0xBBF7D0),
        successBackground: Color(hex: 0x123522),
        warningText: Color(hex: 0xFDE68A),
        warningBackground: Color(hex: 0x3B2A11),
        dangerText: Color(hex: 0xFECDD3),
        dangerBackground: Color(hex: 0x3B1520),
        infoText: Color(hex: 0xBFDBFE),
        infoBackground: Color(hex: 0x1A274D)
    )
}

extension Color {
    init(hex: UInt32) {
        let red = Double((hex >> 16) & 0xFF) / 255.0
        let green = Double((hex >> 8) & 0xFF) / 255.0
        let blue = Double(hex & 0xFF) / 255.0
        self.init(red: red, green: green, blue: blue)
    }
}
