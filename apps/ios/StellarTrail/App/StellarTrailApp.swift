import SwiftUI

@main
struct StellarTrailApp: App {
    @StateObject private var environment = AppEnvironment.makeDefault()

    var body: some Scene {
        WindowGroup {
            RootTabView(environment: environment)
                .environmentObject(environment)
                .preferredColorScheme(environment.settingsStore.preferredColorScheme)
        }
    }
}
