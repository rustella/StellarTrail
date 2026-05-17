import Foundation

@MainActor
protocol AuthRepositorying {
    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse
    func register(_ request: RegisterRequest) async throws -> LoginResponse
    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse
    func captcha(account: String) async throws -> CaptchaChallengeResponse
}

@MainActor
protocol GearRepositorying {
    func stats() async throws -> GearStatsResponse
    func categories(tab: GearTab) async throws -> GearCategoriesResponse
    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse
    func get(id: String) async throws -> GearItem
    func create(_ request: CreateGearRequest) async throws -> GearItem
    func update(id: String, request: UpdateGearRequest) async throws -> GearItem
    func archive(id: String) async throws
    func restore(id: String) async throws -> GearItem
}

@MainActor
protocol SkillRepositorying {
    func categories() async throws -> SkillCategoriesResponse
    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse
    func knotDetail(id: String) async throws -> KnotDetail
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
        try await client.send(try APIRequest.post("/api/auth/email-verification-code", body: EmailVerificationCodeRequest(email: email)), requiresAuth: false)
    }

    func register(_ request: RegisterRequest) async throws -> LoginResponse {
        let response: LoginResponse = try await client.send(try APIRequest.post("/api/auth/register", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse {
        let request = PasswordLoginRequest(account: account, password: password, captchaTicket: captchaTicket, captchaAnswer: captchaAnswer)
        let response: LoginResponse = try await client.send(try APIRequest.post("/api/auth/login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func captcha(account: String) async throws -> CaptchaChallengeResponse {
        try await client.send(try APIRequest.post("/api/auth/captcha", body: CaptchaChallengeRequest(account: account)), requiresAuth: false)
    }
}

@MainActor
final class GearRepository: GearRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func stats() async throws -> GearStatsResponse {
        try await client.send(.get("/api/me/gears/stats"), requiresAuth: true)
    }

    func categories(tab: GearTab) async throws -> GearCategoriesResponse {
        try await client.send(.get("/api/me/gears/categories", queryItems: [URLQueryItem(name: "tab", value: tab.rawValue)]), requiresAuth: true)
    }

    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse {
        try await client.send(.get("/api/me/gears", queryItems: request.queryItems), requiresAuth: true)
    }

    func get(id: String) async throws -> GearItem {
        try await client.send(.get("/api/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func create(_ request: CreateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.post("/api/me/gears", body: request), requiresAuth: true)
    }

    func update(id: String, request: UpdateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.patch("/api/me/gears/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func archive(id: String) async throws {
        let _: EmptyResponse = try await client.sendEmpty(.delete("/api/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func restore(id: String) async throws -> GearItem {
        try await client.send(.post("/api/me/gears/\(id.urlPathEscaped)/restore"), requiresAuth: true)
    }
}

@MainActor
final class SkillRepository: SkillRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func categories() async throws -> SkillCategoriesResponse {
        try await client.send(.get("/api/skills"), requiresAuth: false)
    }

    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse {
        try await client.send(.get("/api/skills/knots/list", queryItems: request.queryItems), requiresAuth: false)
    }

    func knotDetail(id: String) async throws -> KnotDetail {
        try await client.send(.get("/api/skills/knots/detail/\(id.urlPathEscaped)"), requiresAuth: false)
    }
}

@MainActor
final class ContentRepository: ContentRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func gearTemplates() async throws -> GearTemplatesResponse {
        try await client.send(.get("/api/gear-templates"), requiresAuth: false)
    }
}

private extension String {
    var urlPathEscaped: String {
        addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? self
    }
}
