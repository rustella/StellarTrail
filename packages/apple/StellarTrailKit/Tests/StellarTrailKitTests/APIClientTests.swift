import XCTest
import Foundation
@testable import StellarTrailKit

@MainActor
final class APIClientTests: XCTestCase {
    override func tearDown() {
        MockURLProtocol.requestHandler = nil
        super.tearDown()
    }

    func testDefaultSettingsUseProductionAPIAndAssetDomains() {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)

        XCTAssertEqual(settings.baseURLString, "https://api.example.invalid")
        XCTAssertEqual(settings.assetsBaseURLString, "https://assets.example.invalid")
        XCTAssertEqual(urlHost(settings.baseURL), "api.example.invalid")
        XCTAssertEqual(urlHost(settings.assetsBaseURL), "assets.example.invalid")
        XCTAssertEqual(client.resolveAssetURL("knots/bowline.png")?.absoluteString, "https://assets.example.invalid/knots/bowline.png")
        XCTAssertEqual(client.resolveAssetURL("/knots/bowline.png")?.absoluteString, "https://assets.example.invalid/knots/bowline.png")
        XCTAssertEqual(client.resolveAssetURL("https://cdn.example.invalid/knots/bowline.png")?.absoluteString, "https://cdn.example.invalid/knots/bowline.png")
    }

    func testPublicRequestDoesNotAttachAuthorizationHeader() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)

        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.url?.scheme, "https")
            XCTAssertEqual(urlHost(request.url), "api.example.invalid")
            XCTAssertEqual(request.url?.path, "/api/v1/gear-templates")
            XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
            XCTAssertEqual(request.value(forHTTPHeaderField: "X-StellarTrail-Locale"), "zh-CN")
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, #"{"items":[]}"#.data(using: .utf8)!)
        }

        let _: GearTemplatesResponse = try await client.send(.get("/api/v1/gear-templates"), requiresAuth: false)
    }

    func testPrivateRequestAttachesBearerToken() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)

        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer x")
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, #"{"current_count":0,"archived_count":0,"total_value_cents":0,"total_weight_g":0,"by_category":[],"by_status":[]}"#.data(using: .utf8)!)
        }

        let _: GearStatsResponse = try await client.send(.get("/api/v1/me/gears/stats"), requiresAuth: true)
    }

    func testPrivateRequestRefreshesTokenAndRetriesOnce() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session(accessToken: "old", expiresAt: "2026-05-16T12:00:00Z", refreshToken: "refresh", refreshExpiresAt: "2026-06-15T10:00:00Z", user: Session.fixture.user))
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        var statsAttempts = 0

        MockURLProtocol.requestHandler = { request in
            if request.url?.path == "/api/v1/auth/refresh" {
                XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
                let body = requestBodyString(from: request)
                XCTAssertEqual(body, #"{"refresh_token":"refresh"}"#)
                let payload = #"{"access_token":"new","expires_at":"2026-05-16T12:30:00Z","refresh_token":"next","refresh_expires_at":"2026-06-15T10:30:00Z","user":{"id":"user-fixture","username":"trail_alice","email":"alice@example.com","nickname":"星野 Alice","avatar_url":null}}"#.data(using: .utf8)!
                return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, payload)
            }
            statsAttempts += 1
            if statsAttempts == 1 {
                XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer old")
                return (HTTPURLResponse(url: request.url!, statusCode: 401, httpVersion: nil, headerFields: nil)!, #"{"message":"expired"}"#.data(using: .utf8)!)
            }
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer new")
            let payload = #"{"current_count":1,"archived_count":0,"total_value_cents":0,"total_weight_g":0,"by_category":[],"by_status":[]}"#.data(using: .utf8)!
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, payload)
        }

        let stats: GearStatsResponse = try await client.send(.get("/api/v1/me/gears/stats"), requiresAuth: true)

        XCTAssertEqual(stats.currentCount, 1)
        XCTAssertEqual(sessionStore.currentSession?.accessToken, "new")
        XCTAssertEqual(sessionStore.currentSession?.refreshToken, "next")
    }

    func testWechatLoginPostsCodeAndStoresSession() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        let repository = AuthRepository(client: client, sessionStore: sessionStore)

        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.url?.path, "/api/v1/auth/wechat-login")
            XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
            let bodyData = try XCTUnwrap(requestBodyData(from: request))
            let body = try XCTUnwrap(JSONSerialization.jsonObject(with: bodyData) as? [String: Any])
            XCTAssertEqual(body["code"] as? String, "macos-local-user")
            let profile = try XCTUnwrap(body["profile"] as? [String: Any])
            XCTAssertEqual(profile["nickname"] as? String, "macOS 本地用户")
            XCTAssertNil(profile["avatar_url"])
            let payload = #"{"access_token":"wechat-token","expires_at":"2026-05-16T12:30:00Z","refresh_token":"wechat-refresh","refresh_expires_at":"2026-06-15T10:30:00Z","user":{"id":"wechat-user","username":null,"email":null,"nickname":"macOS 本地用户","avatar_url":null}}"#.data(using: .utf8)!
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, payload)
        }

        let response = try await repository.wechatLogin(
            code: "macos-local-user",
            profile: WechatLoginProfile(nickname: "macOS 本地用户", avatarUrl: nil)
        )

        XCTAssertEqual(response.accessToken, "wechat-token")
        XCTAssertEqual(sessionStore.currentSession?.accessToken, "wechat-token")
        XCTAssertEqual(sessionStore.currentSession?.user.displayName, "macOS 本地用户")
    }

    func testEmailLoginPostsCodeAndStoresSession() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        let repository = AuthRepository(client: client, sessionStore: sessionStore)

        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.url?.path, "/api/v1/auth/email-login")
            XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
            let bodyData = try XCTUnwrap(requestBodyData(from: request))
            let body = try XCTUnwrap(JSONSerialization.jsonObject(with: bodyData) as? [String: Any])
            XCTAssertEqual(body["email"] as? String, "alice@example.com")
            XCTAssertEqual(body["email_verification_code"] as? String, "654321")
            let payload = #"{"access_token":"email-token","expires_at":"2026-05-16T12:30:00Z","refresh_token":"email-refresh","refresh_expires_at":"2026-06-15T10:30:00Z","user":{"id":"email-user","username":"trail_alice","email":"alice@example.com","nickname":null,"avatar_url":null}}"#.data(using: .utf8)!
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, payload)
        }

        let response = try await repository.loginWithEmailCode(email: "alice@example.com", code: "654321")

        XCTAssertEqual(response.accessToken, "email-token")
        XCTAssertEqual(sessionStore.currentSession?.user.email, "alice@example.com")
    }

    func testGearAtlasRepositoryUsesPublicAndPrivateRoutes() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        let repository = GearAtlasRepository(client: client)

        MockURLProtocol.requestHandler = { request in
            if request.url?.path == "/api/v1/gear-atlas" {
                XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
                XCTAssertEqual(request.url?.query?.contains("category=lighting_system"), true)
                return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, #"{"items":[],"next_cursor":null}"#.data(using: .utf8)!)
            }
            XCTAssertEqual(request.url?.path, "/api/v1/me/gear-atlas-submissions")
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer x")
            let bodyData = try XCTUnwrap(requestBodyData(from: request))
            let body = try XCTUnwrap(JSONSerialization.jsonObject(with: bodyData) as? [String: Any])
            XCTAssertEqual(body["category"] as? String, "lighting_system")
            let payload = #"{"id":"submission-1","category":"lighting_system","category_label":"照明系统","name":"头灯","brand":null,"model":null,"description":null,"weight_g":86,"official_price_cents":19900,"official_price_currency":"CNY","specs":{"max_brightness":"450 lm"},"approved_at":null,"created_at":"2026-05-01T10:00:00Z","updated_at":"2026-05-01T10:00:00Z","source_type":"manual","source_user_gear_id":null,"status":"pending","rejection_reason":null,"reviewed_at":null}"#.data(using: .utf8)!
            return (HTTPURLResponse(url: request.url!, statusCode: 201, httpVersion: nil, headerFields: nil)!, payload)
        }

        let list = try await repository.list(ListGearAtlasRequest(category: .lightingSystem, q: "头灯"))
        XCTAssertTrue(list.items.isEmpty)

        let submission = try await repository.createSubmission(CreateGearAtlasSubmissionRequest(category: .lightingSystem, name: "头灯", brand: nil, model: nil, description: nil, weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", specs: ["max_brightness": "450 lm"]))
        XCTAssertEqual(submission.status, .pending)
    }

    func testPageAndButtonRepositoriesRequestProductionAPIHost() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        let authRepository = AuthRepository(client: client, sessionStore: sessionStore)
        let contentRepository = ContentRepository(client: client)
        let gearRepository = GearRepository(client: client)
        let atlasRepository = GearAtlasRepository(client: client)
        let skillRepository = SkillRepository(client: client)
        var seenRequests: [String] = []

        MockURLProtocol.requestHandler = { request in
            let url = try XCTUnwrap(request.url)
            let key = "\(request.httpMethod ?? "GET") \(url.path)"
            seenRequests.append(key)

            XCTAssertEqual(url.scheme, "https")
            XCTAssertEqual(urlHost(url), "api.example.invalid")
            XCTAssertEqual(request.value(forHTTPHeaderField: "X-StellarTrail-Locale"), "zh-CN")
            if url.path.hasPrefix("/api/v1/me/") {
                XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer x")
            } else {
                XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
            }

            switch key {
            case "GET /api/v1/gear-templates":
                return jsonResponse(url: url, #"{"items":[]}"#)
            case "GET /api/v1/me/gears/stats":
                return jsonResponse(url: url, #"{"current_count":0,"archived_count":0,"total_value_cents":0,"total_weight_g":0,"by_category":[],"by_status":[]}"#)
            case "GET /api/v1/me/gears/categories":
                return jsonResponse(url: url, #"{"items":[]}"#)
            case "GET /api/v1/me/gears/spec-key-rankings":
                return jsonResponse(url: url, #"{"keys":["max_brightness"]}"#)
            case "GET /api/v1/me/gears/tag-suggestions":
                return jsonResponse(url: url, #"{"items":[{"tag":"夜行","color":"violet"}]}"#)
            case "GET /api/v1/me/gears":
                return jsonResponse(url: url, #"{"items":[],"next_cursor":null}"#)
            case "GET /api/v1/me/gears/gear-1", "POST /api/v1/me/gears", "PATCH /api/v1/me/gears/gear-1", "POST /api/v1/me/gears/gear-1/restore":
                return jsonResponse(url: url, gearItemPayload)
            case "DELETE /api/v1/me/gears/gear-1":
                return emptyResponse(url: url)
            case "GET /api/v1/gear-atlas":
                return jsonResponse(url: url, #"{"items":[],"next_cursor":null}"#)
            case "GET /api/v1/gear-atlas/atlas-1":
                return jsonResponse(url: url, gearAtlasItemPayload)
            case "POST /api/v1/me/gear-atlas-submissions", "POST /api/v1/me/gears/gear-1/atlas-submission":
                return jsonResponse(url: url, gearAtlasSubmissionPayload, statusCode: 201)
            case "GET /api/v1/me/gear-atlas-submissions":
                return jsonResponse(url: url, #"{"items":[],"next_cursor":null}"#)
            case "GET /api/v1/skills":
                return jsonResponse(url: url, #"{"items":[]}"#)
            case "GET /api/v1/skills/knots/list":
                return jsonResponse(url: url, #"{"locale":"zh-CN","items":[],"page":{"limit":20,"offset":0,"next_offset":null}}"#)
            case "GET /api/v1/skills/knots/detail/bowline":
                return jsonResponse(url: url, knotDetailPayload)
            case "GET /api/v1/skills/knots/offline-manifest":
                return jsonResponse(url: url, #"{"locale":"zh-CN","item_count":0,"media_count":0,"estimated_bytes":0,"items":[]}"#)
            case "GET /api/v1/me/profile", "POST /api/v1/me/email-binding", "PUT /api/v1/me/profile/avatar":
                return jsonResponse(url: url, profilePayload)
            case "POST /api/v1/me/email-binding-code",
                 "POST /api/v1/auth/email-verification-code",
                 "POST /api/v1/auth/email-login-code",
                 "POST /api/v1/auth/password-reset-code":
                return jsonResponse(url: url, #"{"email":"alice@example.com","expires_at":"2026-05-16T12:30:00Z","debug_code":"654321"}"#)
            case "POST /api/v1/auth/captcha":
                return jsonResponse(url: url, #"{"captcha_ticket":"ticket-1","captcha_type":"svg","image_svg":"<svg></svg>","expires_at":"2026-05-16T12:30:00Z","debug_answer":"1234"}"#)
            case "POST /api/v1/auth/register",
                 "POST /api/v1/auth/login",
                 "POST /api/v1/auth/email-login",
                 "POST /api/v1/auth/password-reset",
                 "POST /api/v1/auth/wechat-login":
                return jsonResponse(url: url, loginPayload)
            default:
                XCTFail("Unexpected request: \(key)")
                return jsonResponse(url: url, #"{}"#, statusCode: 500)
            }
        }

        let gearPayload = CreateGearRequest(
            category: .lightingSystem,
            name: "头灯",
            brand: "Nitecore",
            model: "NU25",
            description: "夜行备用",
            weightG: 86,
            officialPriceCents: 19900,
            officialPriceCurrency: "CNY",
            purchaseDate: "2026-05-01",
            purchasePriceCents: 15900,
            purchasePriceCurrency: "CNY",
            purchaseLocation: "旗舰店",
            status: .available,
            storageLocation: "玄关",
            specs: ["max_brightness": "450 lm"],
            tags: ["夜行"],
            tagColors: ["夜行": "violet"],
            shareEnabled: true,
            notes: "备用电池已检查"
        )

        let _: GearTemplatesResponse = try await contentRepository.gearTemplates()
        let _: GearStatsResponse = try await gearRepository.stats(tab: .available)
        let _: GearCategoriesResponse = try await gearRepository.categories(tab: .available)
        let _: GearSpecKeyRankingsResponse = try await gearRepository.specKeyRankings(category: .lightingSystem)
        let _: GearTagSuggestionsResponse = try await gearRepository.tagSuggestions(limit: 8)
        let _: ListGearsResponse = try await gearRepository.list(ListGearsRequest(tab: .available, category: .lightingSystem, q: "头灯"))
        let _: GearItem = try await gearRepository.get(id: "gear-1")
        let _: GearItem = try await gearRepository.create(gearPayload)
        let _: GearItem = try await gearRepository.update(id: "gear-1", request: gearPayload)
        try await gearRepository.archive(id: "gear-1")
        let _: GearItem = try await gearRepository.restore(id: "gear-1")
        let _: ListGearAtlasResponse = try await atlasRepository.list(ListGearAtlasRequest(category: .lightingSystem, q: "头灯"))
        let _: GearAtlasPublicItem = try await atlasRepository.get(id: "atlas-1")
        let _: GearAtlasSubmission = try await atlasRepository.createSubmission(CreateGearAtlasSubmissionRequest(category: .lightingSystem, name: "头灯", brand: "Nitecore", model: "NU25", description: "公开字段", weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", specs: ["max_brightness": "450 lm"]))
        let _: GearAtlasSubmission = try await atlasRepository.submitGear(id: "gear-1")
        let _: ListGearAtlasSubmissionsResponse = try await atlasRepository.mySubmissions(ListGearAtlasSubmissionsRequest(status: .pending))
        let _: SkillCategoriesResponse = try await skillRepository.categories()
        let _: KnotListResponse = try await skillRepository.knots(ListKnotsRequest(q: "称人结"))
        let _: KnotDetail = try await skillRepository.knotDetail(id: "bowline")
        let _: KnotOfflineManifestResponse = try await skillRepository.offlineManifest()
        let _: UserProfile = try await authRepository.currentUser()
        let _: EmailVerificationCodeResponse = try await authRepository.sendBindEmailCode(email: "alice@example.com")
        let _: UserProfile = try await authRepository.bindEmail(email: "alice@example.com", code: "654321")
        let _: UserProfile = try await authRepository.uploadAvatar(data: Data([0x01, 0x02]), fileName: "avatar.jpg", mimeType: "image/jpeg")
        let _: EmailVerificationCodeResponse = try await authRepository.sendEmailVerificationCode(email: "alice@example.com")
        let _: EmailVerificationCodeResponse = try await authRepository.sendEmailLoginCode(email: "alice@example.com")
        let _: EmailVerificationCodeResponse = try await authRepository.sendPasswordResetCode(email: "alice@example.com")
        let _: CaptchaChallengeResponse = try await authRepository.captcha(account: "alice@example.com")
        let _: LoginResponse = try await authRepository.register(RegisterRequest(username: "trail_alice", email: "alice@example.com", password: "Password1", confirmPassword: "Password1", emailVerificationCode: "654321"))
        let _: LoginResponse = try await authRepository.login(account: "alice@example.com", password: "Password1", captchaTicket: nil, captchaAnswer: nil)
        let _: LoginResponse = try await authRepository.loginWithEmailCode(email: "alice@example.com", code: "654321")
        let _: LoginResponse = try await authRepository.resetPassword(PasswordResetRequest(email: "alice@example.com", emailVerificationCode: "654321", password: "Password1", confirmPassword: "Password1"))
        let _: LoginResponse = try await authRepository.wechatLogin(code: "macos-local-user", profile: WechatLoginProfile(nickname: "macOS 本地用户", avatarUrl: nil))

        XCTAssertEqual(Set(seenRequests), Set([
            "GET /api/v1/gear-templates",
            "GET /api/v1/me/gears/stats",
            "GET /api/v1/me/gears/categories",
            "GET /api/v1/me/gears/spec-key-rankings",
            "GET /api/v1/me/gears/tag-suggestions",
            "GET /api/v1/me/gears",
            "GET /api/v1/me/gears/gear-1",
            "POST /api/v1/me/gears",
            "PATCH /api/v1/me/gears/gear-1",
            "DELETE /api/v1/me/gears/gear-1",
            "POST /api/v1/me/gears/gear-1/restore",
            "GET /api/v1/gear-atlas",
            "GET /api/v1/gear-atlas/atlas-1",
            "POST /api/v1/me/gear-atlas-submissions",
            "POST /api/v1/me/gears/gear-1/atlas-submission",
            "GET /api/v1/me/gear-atlas-submissions",
            "GET /api/v1/skills",
            "GET /api/v1/skills/knots/list",
            "GET /api/v1/skills/knots/detail/bowline",
            "GET /api/v1/skills/knots/offline-manifest",
            "GET /api/v1/me/profile",
            "POST /api/v1/me/email-binding-code",
            "POST /api/v1/me/email-binding",
            "PUT /api/v1/me/profile/avatar",
            "POST /api/v1/auth/email-verification-code",
            "POST /api/v1/auth/email-login-code",
            "POST /api/v1/auth/password-reset-code",
            "POST /api/v1/auth/captcha",
            "POST /api/v1/auth/register",
            "POST /api/v1/auth/login",
            "POST /api/v1/auth/email-login",
            "POST /api/v1/auth/password-reset",
            "POST /api/v1/auth/wechat-login"
        ]))
    }

    func testSettingsMigratesMisspelledAPIBaseURL() {
        let defaults = UserDefaults.testSuite()
        defaults.set("https://api.example.invalid/", forKey: "stellartrail.baseURLString")

        let settings = AppSettingsStore(defaults: defaults)

        XCTAssertEqual(settings.baseURLString, "https://api.example.invalid")
        XCTAssertEqual(defaults.string(forKey: "stellartrail.baseURLString"), "https://api.example.invalid")
    }

}

private let gearItemPayload = #"{"id":"gear-1","user_id":"user-fixture","category":"lighting_system","name":"头灯","brand":"Nitecore","model":"NU25","color":null,"material":null,"capacity":null,"size":null,"description":"夜行备用","weight_g":86,"official_price_cents":19900,"official_price_currency":"CNY","warmth_index":null,"waterproof_index":null,"purchase_date":"2026-05-01","purchase_price_cents":15900,"purchase_price_currency":"CNY","expiry_or_warranty_date":null,"purchase_location":"旗舰店","status":"available","storage_location":"玄关","specs":{"max_brightness":"450 lm"},"tags":["夜行"],"tag_colors":{"夜行":"violet"},"share_enabled":true,"share_status":"pending","notes":"备用电池已检查","archived_at":null,"created_at":"2026-05-01T10:00:00Z","updated_at":"2026-05-01T10:00:00Z"}"#

private let gearAtlasItemPayload = #"{"id":"atlas-1","category":"lighting_system","category_label":"照明系统","name":"头灯","brand":"Nitecore","model":"NU25","description":"公开字段","weight_g":86,"official_price_cents":19900,"official_price_currency":"CNY","specs":{"max_brightness":"450 lm"},"approved_at":"2026-05-01T10:00:00Z","created_at":"2026-05-01T10:00:00Z","updated_at":"2026-05-01T10:00:00Z"}"#

private let gearAtlasSubmissionPayload = #"{"id":"submission-1","category":"lighting_system","category_label":"照明系统","name":"头灯","brand":"Nitecore","model":"NU25","description":"公开字段","weight_g":86,"official_price_cents":19900,"official_price_currency":"CNY","specs":{"max_brightness":"450 lm"},"approved_at":null,"created_at":"2026-05-01T10:00:00Z","updated_at":"2026-05-01T10:00:00Z","source_type":"manual","source_user_gear_id":null,"status":"pending","rejection_reason":null,"reviewed_at":null}"#

private let knotDetailPayload = #"{"id":"bowline","slug":"bowline","title":"称人结","summary":"常用固定绳结","categories":[],"types":[],"media":[],"href":null,"description":"适合快速形成固定绳圈。","steps":["绕圈","穿入","收紧"],"locale":"zh-CN"}"#

private let profilePayload = #"{"user":{"id":"user-fixture","username":"trail_alice","email":"alice@example.com","nickname":"星野 Alice","avatar_url":"https://assets.example.invalid/avatars/alice.jpg"}}"#

private let loginPayload = #"{"access_token":"login-token","expires_at":"2026-05-16T12:30:00Z","refresh_token":"login-refresh","refresh_expires_at":"2026-06-15T10:30:00Z","user":{"id":"user-fixture","username":"trail_alice","email":"alice@example.com","nickname":"星野 Alice","avatar_url":null}}"#

private func jsonResponse(url: URL, _ json: String, statusCode: Int = 200) -> (HTTPURLResponse, Data) {
    (
        HTTPURLResponse(url: url, statusCode: statusCode, httpVersion: nil, headerFields: nil)!,
        Data(json.utf8)
    )
}

private func emptyResponse(url: URL, statusCode: Int = 204) -> (HTTPURLResponse, Data) {
    (
        HTTPURLResponse(url: url, statusCode: statusCode, httpVersion: nil, headerFields: nil)!,
        Data()
    )
}

private func urlHost(_ url: URL?) -> String? {
    guard let url else { return nil }
    return URLComponents(url: url, resolvingAgainstBaseURL: false)?.host
}

private func requestBodyString(from request: URLRequest) -> String? {
    requestBodyData(from: request).flatMap { String(data: $0, encoding: .utf8) }
}

private func requestBodyData(from request: URLRequest) -> Data? {
    if let body = request.httpBody { return body }
    guard let stream = request.httpBodyStream else { return nil }
    stream.open()
    defer { stream.close() }
    var data = Data()
    let bufferSize = 1024
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: bufferSize)
    defer { buffer.deallocate() }
    while stream.hasBytesAvailable {
        let count = stream.read(buffer, maxLength: bufferSize)
        if count <= 0 { break }
        data.append(buffer, count: count)
    }
    return data
}

private extension URLSession {
    static var mocked: URLSession {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [MockURLProtocol.self]
        return URLSession(configuration: configuration)
    }
}

private final class MockURLProtocol: URLProtocol {
    static var requestHandler: ((URLRequest) throws -> (HTTPURLResponse, Data))?

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        guard let handler = Self.requestHandler else {
            client?.urlProtocol(self, didFailWithError: URLError(.badServerResponse))
            return
        }
        do {
            let (response, data) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }

    override func stopLoading() {}
}

private extension UserDefaults {
    static func testSuite() -> UserDefaults {
        let suiteName = "stellartrail.tests.\(UUID().uuidString)"
        return UserDefaults(suiteName: suiteName)!
    }
}
