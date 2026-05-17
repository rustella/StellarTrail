import SwiftUI

enum RootTab: String, CaseIterable, Identifiable {
    case home
    case gear
    case skills
    case profile

    var id: String { rawValue }

    var title: String {
        switch self {
        case .home: return "首页"
        case .gear: return "装备"
        case .skills: return "技能"
        case .profile: return "我的"
        }
    }

    var systemImage: String {
        switch self {
        case .home: return "house.fill"
        case .gear: return "backpack.fill"
        case .skills: return "figure.hiking"
        case .profile: return "person.crop.circle.fill"
        }
    }
}

struct RootTabView: View {
    @ObservedObject var environment: AppEnvironment
    @State private var selectedTab: RootTab = .home

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                HomeView(environment: environment)
            }
            .tabItem { Label(RootTab.home.title, systemImage: RootTab.home.systemImage) }
            .tag(RootTab.home)

            NavigationStack {
                GearListView(environment: environment)
            }
            .tabItem { Label(RootTab.gear.title, systemImage: RootTab.gear.systemImage) }
            .tag(RootTab.gear)

            NavigationStack {
                SkillsView(environment: environment)
            }
            .tabItem { Label(RootTab.skills.title, systemImage: RootTab.skills.systemImage) }
            .tag(RootTab.skills)

            NavigationStack {
                ProfileView(environment: environment)
            }
            .tabItem { Label(RootTab.profile.title, systemImage: RootTab.profile.systemImage) }
            .tag(RootTab.profile)
        }
        .tint(TrailColors.light.brand)
        .trailTheme(settingsStore: environment.settingsStore)
    }
}
