import XCTest
import Foundation
@testable import StellarTrailKit

@MainActor
final class APIClientTests: XCTestCase {
    override func tearDown() {
        MockURLProtocol.requestHandler = nil
        super.tearDown()
    }

    func testPublicRequestDoesNotAttachAuthorizationHeader() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)

        MockURLProtocol.requestHandler = { request in
            XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
            XCTAssertEqual(request.value(forHTTPHeaderField: "X-StellarTrail-Locale"), "zh-CN")
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, #"{"items":[]}"#.data(using: .utf8)!)
        }

        let _: GearTemplatesResponse = try await client.send(.get("/api/gear-templates"), requiresAuth: false)
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

        let _: GearStatsResponse = try await client.send(.get("/api/me/gears/stats"), requiresAuth: true)
    }

    func testPrivateRequestRefreshesTokenAndRetriesOnce() async throws {
        let settings = AppSettingsStore(defaults: .testSuite())
        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session(accessToken: "old", expiresAt: "2026-05-16T12:00:00Z", refreshToken: "refresh", refreshExpiresAt: "2026-06-15T10:00:00Z", user: Session.fixture.user))
        let client = APIClient(settingsStore: settings, sessionStore: sessionStore, session: .mocked)
        var statsAttempts = 0

        MockURLProtocol.requestHandler = { request in
            if request.url?.path == "/api/auth/refresh" {
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

        let stats: GearStatsResponse = try await client.send(.get("/api/me/gears/stats"), requiresAuth: true)

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
            XCTAssertEqual(request.url?.path, "/api/auth/wechat-login")
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

    func testSettingsMigratesMisspelledAPIBaseURL() {
        let defaults = UserDefaults.testSuite()
        defaults.set("https://api.example.invalid/", forKey: "stellartrail.baseURLString")

        let settings = AppSettingsStore(defaults: defaults)

        XCTAssertEqual(settings.baseURLString, "https://api.example.invalid")
        XCTAssertEqual(defaults.string(forKey: "stellartrail.baseURLString"), "https://api.example.invalid")
    }

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
