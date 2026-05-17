import SwiftUI

enum MacSidebarItem: String, CaseIterable, Identifiable {
    case home
    case gear
    case skills
    case skillKnots = "skill-knots"
    case profile
    case authLogin = "auth-login"
    case authRegister = "auth-register"

    var id: String { rawValue }

    var title: String {
        switch self {
        case .home: return "首页"
        case .gear: return "装备"
        case .skills: return "技能"
        case .skillKnots: return "绳结"
        case .profile: return "我的"
        case .authLogin: return "登录"
        case .authRegister: return "注册"
        }
    }

    var systemImage: String {
        switch self {
        case .home: return "house.fill"
        case .gear: return "backpack.fill"
        case .skills: return "figure.hiking"
        case .skillKnots: return "point.3.connected.trianglepath.dotted"
        case .profile: return "person.crop.circle.fill"
        case .authLogin: return "person.badge.key.fill"
        case .authRegister: return "person.badge.plus.fill"
        }
    }
}

struct MacRootView: View {
    @ObservedObject var environment: MacAppEnvironment
    @ObservedObject private var sessionStore: SessionStore
    @State private var selection: MacSidebarItem?
    @State private var skillsExpanded = true
    @State private var guestBrowsing = false

    init(environment: MacAppEnvironment) {
        self.environment = environment
        _sessionStore = ObservedObject(wrappedValue: environment.sessionStore)
        _selection = State(initialValue: MacRootView.initialSelection())
    }

    var body: some View {
        Group {
            if shouldShowAuthGate {
                authView(mode: .login)
            } else if let authMode = selectedAuthMode {
                authView(mode: authMode)
            } else {
                navigationShell
            }
        }
        .trailTheme(settingsStore: environment.settingsStore)
        .onAppear(perform: normalizeSelection)
        .onChange(of: sessionStore.currentSession != nil) { _, isLoggedIn in
            if isLoggedIn {
                guestBrowsing = false
                if selection == nil || selection?.isAuthItem == true {
                    selection = .home
                }
            } else {
                guestBrowsing = false
                selection = nil
            }
        }
    }

    private var navigationShell: some View {
        NavigationSplitView {
            List(selection: $selection) {
                Section("寻径星野") {
                    sidebarLink(.home)
                    sidebarLink(.gear)
                    DisclosureGroup(isExpanded: $skillsExpanded) {
                        sidebarLink(.skillKnots)
                            .padding(.leading, 8)
                    } label: {
                        Label(MacSidebarItem.skills.title, systemImage: MacSidebarItem.skills.systemImage)
                    }
                }
                Section("账号") {
                    sidebarLink(.profile)
                }
            }
            .navigationTitle("StellarTrail")
            .listStyle(.sidebar)
        } detail: {
            selectedDetailView
        }
    }

    private func sidebarLink(_ item: MacSidebarItem) -> some View {
        Label(item.title, systemImage: item.systemImage).tag(item)
    }

    @ViewBuilder
    private var selectedDetailView: some View {
        switch selection ?? .home {
        case .home:
            MacHomeView(environment: environment)
        case .gear:
            MacGearView(environment: environment)
        case .skills:
            MacSkillsView(environment: environment)
        case .skillKnots:
            MacSkillsView(environment: environment)
        case .profile:
            MacProfileView(
                environment: environment,
                onRequestLogin: {
                    guestBrowsing = false
                    selection = .authLogin
                },
                onRequestRegister: {
                    guestBrowsing = false
                    selection = .authRegister
                }
            )
        case .authLogin:
            MacAuthPageView(environment: environment, mode: .login) {
                guestBrowsing = true
                selection = .home
            } onAuthenticated: {
                guestBrowsing = false
                selection = .home
            }
        case .authRegister:
            MacAuthPageView(environment: environment, mode: .register) {
                guestBrowsing = true
                selection = .home
            } onAuthenticated: {
                guestBrowsing = false
                selection = .home
            }
        }
    }

    private func authView(mode: AuthMode) -> some View {
        MacAuthPageView(
            environment: environment,
            mode: mode,
            onContinueAsGuest: {
                guestBrowsing = true
                selection = .home
            },
            onAuthenticated: {
                guestBrowsing = false
                selection = .home
            }
        )
    }

    private var shouldShowAuthGate: Bool {
        sessionStore.currentSession == nil && !guestBrowsing && selection?.isAuthItem != true
    }

    private var selectedAuthMode: AuthMode? {
        switch selection {
        case .authLogin:
            return .login
        case .authRegister:
            return .register
        default:
            return nil
        }
    }

    private func normalizeSelection() {
        if sessionStore.currentSession == nil && selection?.isAuthItem != true {
            selection = nil
        } else if sessionStore.currentSession != nil && selection == nil {
            selection = .home
        }
    }

    private static func initialSelection() -> MacSidebarItem {
        let arguments = ProcessInfo.processInfo.arguments
        guard let index = arguments.firstIndex(of: "--stellartrail-screenshot-page"), arguments.indices.contains(index + 1) else {
            return .home
        }
        let rawValue = arguments[index + 1]
        if rawValue == MacSidebarItem.skills.rawValue {
            return .skillKnots
        }
        return MacSidebarItem(rawValue: rawValue) ?? .home
    }
}

private extension MacSidebarItem {
    var isAuthItem: Bool {
        self == .authLogin || self == .authRegister
    }
}
