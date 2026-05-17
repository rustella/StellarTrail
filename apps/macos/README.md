# StellarTrail macOS

Native SwiftUI macOS client for StellarTrail.

Architecture:

- macOS app shell in `apps/macos/StellarTrailMac`
- Shared Apple source in `packages/apple/StellarTrailKit/Sources/StellarTrailKit`
- Desktop navigation with sidebar and detail panes
- Final native-window screenshots captured on macOS with `scripts/capture-macos-screenshots.sh`

Linux can validate YAML, plist, JSON, and text-level migration checks only. XcodeGen, `xcodebuild`, XCTest, and runtime screenshots must run on macOS.

## Runtime config

The macOS client shares the Apple runtime config loader with iOS. It defaults to:

- API base URL: `https://api.example.invalid`
- Image asset / CORS asset origin: `https://assets.example.invalid`

Copy `StellarTrailMac/Resources/ClientConfig.example.plist` to the Git-ignored `StellarTrailMac/Resources/ClientConfig.plist` when a build needs different endpoints.
