import Foundation

struct UserProfile: Codable, Equatable, Identifiable {
    let id: String
    let username: String?
    let email: String?
    let nickname: String?
    let avatarUrl: String?

    var displayName: String {
        nickname?.nilIfBlank ?? username?.nilIfBlank ?? email?.nilIfBlank ?? "已登录用户"
    }
}

struct LoginResponse: Codable, Equatable {
    let accessToken: String
    let expiresAt: String
    let refreshToken: String
    let refreshExpiresAt: String
    let user: UserProfile
}

typealias RefreshTokenResponse = LoginResponse

struct Session: Codable, Equatable {
    let accessToken: String
    let expiresAt: String
    let refreshToken: String
    let refreshExpiresAt: String
    let user: UserProfile

    init(response: LoginResponse) {
        self.accessToken = response.accessToken
        self.expiresAt = response.expiresAt
        self.refreshToken = response.refreshToken
        self.refreshExpiresAt = response.refreshExpiresAt
        self.user = response.user
    }

    init(accessToken: String, expiresAt: String, refreshToken: String, refreshExpiresAt: String, user: UserProfile) {
        self.accessToken = accessToken
        self.expiresAt = expiresAt
        self.refreshToken = refreshToken
        self.refreshExpiresAt = refreshExpiresAt
        self.user = user
    }

    static let fixture = Session(
        accessToken: "x",
        expiresAt: "2026-05-16T12:00:00Z",
        refreshToken: "r",
        refreshExpiresAt: "2026-06-15T10:00:00Z",
        user: UserProfile(id: "user-fixture", username: "trail_alice", email: "alice@example.com", nickname: "星野 Alice", avatarUrl: nil)
    )
}

extension LoginResponse {
    static let fixture = LoginResponse(
        accessToken: Session.fixture.accessToken,
        expiresAt: Session.fixture.expiresAt,
        refreshToken: Session.fixture.refreshToken,
        refreshExpiresAt: Session.fixture.refreshExpiresAt,
        user: Session.fixture.user
    )
}

struct EmailVerificationCodeRequest: Encodable, Equatable {
    let email: String
}

struct EmailVerificationCodeResponse: Decodable, Equatable {
    let email: String
    let expiresAt: String
    let debugCode: String?
}

struct RegisterRequest: Encodable, Equatable {
    let username: String
    let email: String
    let password: String
    let confirmPassword: String
    let emailVerificationCode: String
}

struct PasswordLoginRequest: Encodable, Equatable {
    let account: String
    let password: String
    let captchaTicket: String?
    let captchaAnswer: String?
}

struct RefreshTokenRequest: Encodable, Equatable {
    let refreshToken: String
}

struct CaptchaChallengeRequest: Encodable, Equatable {
    let account: String
}

struct CaptchaChallengeResponse: Decodable, Equatable {
    let captchaTicket: String
    let captchaType: String
    let imageSvg: String
    let expiresAt: String
    let debugAnswer: String?
}
