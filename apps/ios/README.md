# StellarTrail iOS

StellarTrail iOS is a native SwiftUI client for the same phase-one mobile experience as the WeChat Mini Program and Android app.

## Stack

- Swift + SwiftUI
- MVVM with repository boundaries
- URLSession + Codable for JSON
- Keychain Services for session tokens
- XCTest / XCUITest for behavior and screenshot checks
- XcodeGen for a versioned project definition

## Local setup

Requirements:

- macOS with Xcode 16 or newer
- iOS 17 simulator runtime
- XcodeGen 2.42 or newer

```bash
cd apps/ios
xcodegen generate
xcodebuild -project StellarTrail.xcodeproj -scheme StellarTrail -destination 'platform=iOS Simulator,name=iPhone 15 Pro' build
xcodebuild -project StellarTrail.xcodeproj -scheme StellarTrail -destination 'platform=iOS Simulator,name=iPhone 15 Pro' test
```

## Screenshot review

The UI test target supports deterministic fixture data. It does not require a real account or a running service.

```bash
cd apps/ios
xcodegen generate
STELLARTRAIL_SCREENSHOT_DIR="$PWD/../../.hermes/local/ios-screenshots/$(date +%Y-%m-%d_%H%M%S)" \
  xcodebuild -project StellarTrail.xcodeproj \
  -scheme StellarTrail \
  -destination 'platform=iOS Simulator,name=iPhone 15 Pro' \
  -only-testing:StellarTrailUITests/ScreenshotFlowUITests test
```

The screenshot flow writes the page review set into `STELLARTRAIL_SCREENSHOT_DIR`.

## Runtime notes

Debug builds default to `http://127.0.0.1:8080` for the simulator. Users can change the local connection address from the Profile page in debug builds.
