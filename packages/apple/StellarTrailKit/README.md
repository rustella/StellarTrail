# StellarTrailKit

Shared Apple source for StellarTrail iOS and macOS clients.

The iOS and macOS XcodeGen projects include `Sources/StellarTrailKit` directly so the first shared-code migration can preserve the existing internal Swift API. The Swift package remains the canonical home for shared unit tests and can become a true imported module after the API surface is stabilized and access control is reviewed.

Shared here:

- Core coding, errors, logging, settings
- Domain DTOs and formatters
- Data repositories, API client, session, Keychain abstraction, fixtures
- Design tokens and reusable SwiftUI components
- Cross-platform view models

Platform-specific code stays in `apps/ios/StellarTrail` and `apps/macos/StellarTrailMac`.
