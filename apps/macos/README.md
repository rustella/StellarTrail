# StellarTrail macOS

Native SwiftUI macOS client for StellarTrail.

Architecture:

- macOS app shell in `apps/macos/StellarTrailMac`
- Shared Apple source in `packages/apple/StellarTrailKit/Sources/StellarTrailKit`
- Desktop navigation with sidebar and detail panes
- Final native-window screenshots captured on macOS with `scripts/capture-macos-screenshots.sh`

Linux can validate YAML, plist, JSON, and text-level migration checks only. XcodeGen, `xcodebuild`, XCTest, and runtime screenshots must run on macOS.
