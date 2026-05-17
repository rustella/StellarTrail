import SwiftUI

enum MacSidebarItem: String, CaseIterable, Identifiable {
    case home
    case gear
    case skills
    case profile
    case authLogin = "auth-login"
    case authRegister = "auth-register"

    var id: String { rawValue }

    var title: String {
        switch self {
        case .home: return "首页"
        case .gear: return "装备"
        case .skills: return "技能"
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
        case .profile: return "person.crop.circle.fill"
        case .authLogin: return "person.badge.key.fill"
        case .authRegister: return "person.badge.plus.fill"
        }
    }
}

struct MacRootView: View {
    @ObservedObject var environment: MacAppEnvironment
    @State private var selection: MacSidebarItem? = MacRootView.initialSelection()

    var body: some View {
        NavigationSplitView {
            List(selection: $selection) {
                Section("寻径星野") {
                    sidebarLink(.home)
                    sidebarLink(.gear)
                    sidebarLink(.skills)
                }
                Section("账号") {
                    sidebarLink(.profile)
                    sidebarLink(.authLogin)
                    sidebarLink(.authRegister)
                }
            }
            .navigationTitle("StellarTrail")
            .listStyle(.sidebar)
        } detail: {
            detailView
                .trailTheme(settingsStore: environment.settingsStore)
        }
    }

    private func sidebarLink(_ item: MacSidebarItem) -> some View {
        Label(item.title, systemImage: item.systemImage).tag(item)
    }

    @ViewBuilder
    private var detailView: some View {
        switch selection ?? .home {
        case .home:
            MacHomeView(environment: environment)
        case .gear:
            MacGearView(environment: environment)
        case .skills:
            MacSkillsView(environment: environment)
        case .profile:
            MacProfileView(environment: environment)
        case .authLogin:
            MacAuthPageView(environment: environment, mode: .login)
        case .authRegister:
            MacAuthPageView(environment: environment, mode: .register)
        }
    }

    private static func initialSelection() -> MacSidebarItem {
        let arguments = ProcessInfo.processInfo.arguments
        guard let index = arguments.firstIndex(of: "--stellartrail-screenshot-page"), arguments.indices.contains(index + 1) else {
            return .home
        }
        return MacSidebarItem(rawValue: arguments[index + 1]) ?? .home
    }
}
