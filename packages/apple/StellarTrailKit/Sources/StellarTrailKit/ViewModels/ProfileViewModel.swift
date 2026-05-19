import Foundation
import Combine

@MainActor
final class ProfileViewModel: ObservableObject {
    @Published private(set) var session: Session?
    @Published var baseURLString: String
    @Published var nicknameDraft = ""
    @Published var emailDraft = ""
    @Published var emailCode = ""
    @Published private(set) var loading = false
    @Published private(set) var message: String?
    @Published private(set) var debugCode: String?

    let settingsStore: AppSettingsStore
    private let sessionStore: SessionStore
    private let authRepository: any AuthRepositorying

    init(settingsStore: AppSettingsStore, sessionStore: SessionStore, authRepository: any AuthRepositorying) {
        self.settingsStore = settingsStore
        self.sessionStore = sessionStore
        self.authRepository = authRepository
        self.session = sessionStore.currentSession
        self.baseURLString = settingsStore.baseURLString
        self.nicknameDraft = sessionStore.currentSession?.user.displayName ?? ""
        self.emailDraft = sessionStore.currentSession?.user.email ?? ""
        sessionStore.$currentSession.assign(to: &$session)
    }

    var canEditBaseURL: Bool {
        #if DEBUG
        true
        #else
        false
        #endif
    }

    var themeMode: AppThemeMode {
        get { settingsStore.themeMode }
        set { settingsStore.themeMode = newValue }
    }

    func refreshProfile() async {
        guard sessionStore.isLoggedIn else { return }
        loading = true
        defer { loading = false }
        do {
            let user = try await authRepository.currentUser()
            nicknameDraft = user.displayName
            emailDraft = user.email ?? ""
            message = nil
        } catch {
            message = error.localizedDescription
        }
    }

    func updateBaseURL() {
        settingsStore.baseURLString = baseURLString
    }

    func resetBaseURL() {
        settingsStore.resetBaseURL()
        baseURLString = settingsStore.baseURLString
    }

    func saveNicknameLocally() {
        guard let current = sessionStore.currentSession else { return }
        let user = UserProfile(
            id: current.user.id,
            username: current.user.username,
            email: current.user.email,
            nickname: nicknameDraft.nilIfBlank,
            avatarUrl: current.user.avatarUrl
        )
        replaceUser(user)
        message = "昵称已更新到本机登录态"
    }

    func sendBindEmailCode() async {
        guard let email = emailDraft.nilIfBlank else {
            message = "请先填写邮箱"
            return
        }
        loading = true
        defer { loading = false }
        do {
            let response = try await authRepository.sendBindEmailCode(email: email)
            debugCode = response.debugCode
            message = response.debugCode.map { "验证码已发送，本地调试码：\($0)" } ?? "验证码已发送，请查看邮箱"
        } catch {
            message = error.localizedDescription
        }
    }

    func bindEmail() async {
        guard let email = emailDraft.nilIfBlank, let code = emailCode.nilIfBlank else {
            message = "请填写邮箱和验证码"
            return
        }
        loading = true
        defer { loading = false }
        do {
            let user = try await authRepository.bindEmail(email: email, code: code)
            replaceUser(user)
            message = "邮箱已绑定"
        } catch {
            message = error.localizedDescription
        }
    }

    func uploadAvatar(data: Data, fileName: String = "avatar.jpg", mimeType: String = "image/jpeg") async {
        loading = true
        defer { loading = false }
        do {
            let user = try await authRepository.uploadAvatar(data: data, fileName: fileName, mimeType: mimeType)
            replaceUser(user)
            message = "头像已更新"
        } catch {
            message = error.localizedDescription
        }
    }

    func logout() {
        sessionStore.clear()
    }

    func setFixtureSignedIn() {
        sessionStore.replace(with: Session.fixture)
    }

    private func replaceUser(_ user: UserProfile) {
        guard let current = sessionStore.currentSession else { return }
        sessionStore.replace(with: Session(accessToken: current.accessToken, expiresAt: current.expiresAt, refreshToken: current.refreshToken, refreshExpiresAt: current.refreshExpiresAt, user: user))
        nicknameDraft = user.displayName
        emailDraft = user.email ?? ""
    }
}
