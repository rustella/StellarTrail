import Foundation

@MainActor
protocol AuthRepositorying {
    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse
    func sendEmailLoginCode(email: String) async throws -> EmailVerificationCodeResponse
    func sendPasswordResetCode(email: String) async throws -> EmailVerificationCodeResponse
    func register(_ request: RegisterRequest) async throws -> LoginResponse
    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse
    func loginWithEmailCode(email: String, code: String) async throws -> LoginResponse
    func resetPassword(_ request: PasswordResetRequest) async throws -> LoginResponse
    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse
    func captcha(account: String) async throws -> CaptchaChallengeResponse
    func currentUser() async throws -> UserProfile
    func sendBindEmailCode(email: String) async throws -> EmailVerificationCodeResponse
    func bindEmail(email: String, code: String) async throws -> UserProfile
    func uploadAvatar(data: Data, fileName: String, mimeType: String) async throws -> UserProfile
}

@MainActor
protocol GearRepositorying {
    func stats(tab: GearTab) async throws -> GearStatsResponse
    func categories(tab: GearTab) async throws -> GearCategoriesResponse
    func specKeyRankings(category: GearCategory) async throws -> GearSpecKeyRankingsResponse
    func tagSuggestions(limit: Int) async throws -> GearTagSuggestionsResponse
    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse
    func get(id: String) async throws -> GearItem
    func create(_ request: CreateGearRequest) async throws -> GearItem
    func update(id: String, request: UpdateGearRequest) async throws -> GearItem
    func archive(id: String) async throws
    func delete(id: String) async throws
    func undelete(id: String) async throws -> GearItem
    func restore(id: String) async throws -> GearItem
}

@MainActor
protocol GearAtlasRepositorying {
    func list(_ request: ListGearAtlasRequest) async throws -> ListGearAtlasResponse
    func get(id: String) async throws -> GearAtlasPublicItem
    func createSubmission(_ request: CreateGearAtlasSubmissionRequest) async throws -> GearAtlasSubmission
    func submitGear(id: String) async throws -> GearAtlasSubmission
    func mySubmissions(_ request: ListGearAtlasSubmissionsRequest) async throws -> ListGearAtlasSubmissionsResponse
}

@MainActor
protocol SkillRepositorying {
    func categories() async throws -> SkillCategoriesResponse
    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse
    func knotDetail(id: String) async throws -> KnotDetail
    func offlineManifest() async throws -> KnotOfflineManifestResponse
}

@MainActor
protocol ContentRepositorying {
    func gearTemplates() async throws -> GearTemplatesResponse
}

@MainActor
final class AuthRepository: AuthRepositorying {
    private let client: APIClient
    private let sessionStore: SessionStore

    init(client: APIClient, sessionStore: SessionStore) {
        self.client = client
        self.sessionStore = sessionStore
    }

    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/email-verification-code", body: EmailVerificationCodeRequest(email: email)), requiresAuth: false)
    }

    func sendEmailLoginCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/email-login-code", body: EmailLoginCodeRequest(email: email)), requiresAuth: false)
    }

    func sendPasswordResetCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/password-reset-code", body: PasswordResetCodeRequest(email: email)), requiresAuth: false)
    }

    func register(_ request: RegisterRequest) async throws -> LoginResponse {
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/register", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse {
        let request = PasswordLoginRequest(account: account, password: password, captchaTicket: captchaTicket, captchaAnswer: captchaAnswer)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func loginWithEmailCode(email: String, code: String) async throws -> LoginResponse {
        let request = EmailLoginRequest(email: email, emailVerificationCode: code)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/email-login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func resetPassword(_ request: PasswordResetRequest) async throws -> LoginResponse {
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/password-reset", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse {
        let request = WechatLoginRequest(code: code, profile: profile)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/wechat-login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func captcha(account: String) async throws -> CaptchaChallengeResponse {
        try await client.send(try APIRequest.post("/auth/captcha", body: CaptchaChallengeRequest(account: account)), requiresAuth: false)
    }

    func currentUser() async throws -> UserProfile {
        let response: ProfileUserResponse = try await client.send(.get("/me/profile"), requiresAuth: true)
        replaceCurrentUser(response.user)
        return response.user
    }

    func sendBindEmailCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/me/email-binding-code", body: BindEmailCodeRequest(email: email)), requiresAuth: true)
    }

    func bindEmail(email: String, code: String) async throws -> UserProfile {
        let response: BindEmailResponse = try await client.send(try APIRequest.post("/me/email-binding", body: BindEmailRequest(email: email, emailVerificationCode: code)), requiresAuth: true)
        replaceCurrentUser(response.user)
        return response.user
    }

    func uploadAvatar(data: Data, fileName: String, mimeType: String) async throws -> UserProfile {
        let response = try await client.uploadAvatar(data: data, fileName: fileName, mimeType: mimeType)
        replaceCurrentUser(response.user)
        return response.user
    }

    private func replaceCurrentUser(_ user: UserProfile) {
        guard let current = sessionStore.currentSession else { return }
        sessionStore.replace(with: Session(
            accessToken: current.accessToken,
            expiresAt: current.expiresAt,
            refreshToken: current.refreshToken,
            refreshExpiresAt: current.refreshExpiresAt,
            user: user
        ))
    }
}

@MainActor
final class GearRepository: GearRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func stats(tab: GearTab) async throws -> GearStatsResponse {
        try await client.send(.get("/me/gears/stats", queryItems: [URLQueryItem(name: "tab", value: tab.rawValue)]), requiresAuth: true)
    }

    func categories(tab: GearTab) async throws -> GearCategoriesResponse {
        try await client.send(.get("/me/gears/categories", queryItems: [URLQueryItem(name: "tab", value: tab.rawValue)]), requiresAuth: true)
    }

    func specKeyRankings(category: GearCategory) async throws -> GearSpecKeyRankingsResponse {
        try await client.send(.get("/me/gears/spec-key-rankings", queryItems: [URLQueryItem(name: "category", value: category.rawValue)]), requiresAuth: true)
    }

    func tagSuggestions(limit: Int) async throws -> GearTagSuggestionsResponse {
        try await client.send(.get("/me/gears/tag-suggestions", queryItems: [URLQueryItem(name: "limit", value: String(limit))]), requiresAuth: true)
    }

    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse {
        try await client.send(.get("/me/gears", queryItems: request.queryItems), requiresAuth: true)
    }

    func get(id: String) async throws -> GearItem {
        try await client.send(.get("/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func create(_ request: CreateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.post("/me/gears", body: request), requiresAuth: true)
    }

    func update(id: String, request: UpdateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.patch("/me/gears/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func archive(id: String) async throws {
        let _: EmptyResponse = try await client.sendEmpty(.delete("/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func delete(id: String) async throws {
        let _: EmptyResponse = try await client.sendEmpty(.post("/me/gears/\(id.urlPathEscaped)/delete"), requiresAuth: true)
    }

    func undelete(id: String) async throws -> GearItem {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/undelete"), requiresAuth: true)
    }

    func restore(id: String) async throws -> GearItem {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/restore"), requiresAuth: true)
    }
}

@MainActor
final class GearAtlasRepository: GearAtlasRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListGearAtlasRequest) async throws -> ListGearAtlasResponse {
        try await client.send(.get("/gear-atlas", queryItems: request.queryItems), requiresAuth: false)
    }

    func get(id: String) async throws -> GearAtlasPublicItem {
        try await client.send(.get("/gear-atlas/\(id.urlPathEscaped)"), requiresAuth: false)
    }

    func createSubmission(_ request: CreateGearAtlasSubmissionRequest) async throws -> GearAtlasSubmission {
        try await client.send(try APIRequest.post("/me/gear-atlas-submissions", body: request), requiresAuth: true)
    }

    func submitGear(id: String) async throws -> GearAtlasSubmission {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/atlas-submission"), requiresAuth: true)
    }

    func mySubmissions(_ request: ListGearAtlasSubmissionsRequest) async throws -> ListGearAtlasSubmissionsResponse {
        try await client.send(.get("/me/gear-atlas-submissions", queryItems: request.queryItems), requiresAuth: true)
    }
}

@MainActor
final class SkillRepository: SkillRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func categories() async throws -> SkillCategoriesResponse {
        try await client.send(.get("/skills"), requiresAuth: false)
    }

    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse {
        try await client.send(.get("/skills/knots/list", queryItems: request.queryItems), requiresAuth: false)
    }

    func knotDetail(id: String) async throws -> KnotDetail {
        try await client.send(.get("/skills/knots/detail/\(id.urlPathEscaped)"), requiresAuth: false)
    }

    func offlineManifest() async throws -> KnotOfflineManifestResponse {
        try await client.send(.get("/skills/knots/offline-manifest"), requiresAuth: false)
    }
}

@MainActor
final class ContentRepository: ContentRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func gearTemplates() async throws -> GearTemplatesResponse {
        try await client.send(.get("/gear-templates"), requiresAuth: false)
    }
}

private extension String {
    var urlPathEscaped: String {
        addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? self
    }
}
