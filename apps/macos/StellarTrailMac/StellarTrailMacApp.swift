import SwiftUI

@main
struct StellarTrailMacApp: App {
    @StateObject private var environment = MacAppEnvironment.makeDefault()

    var body: some Scene {
        WindowGroup {
            MacRootView(environment: environment)
                .environmentObject(environment)
                .preferredColorScheme(environment.settingsStore.preferredColorScheme)
                .frame(minWidth: 1080, minHeight: 720)
        }
        .windowStyle(.titleBar)
        .commands {
            CommandGroup(replacing: .newItem) {}
            SidebarCommands()
        }
    }
}
