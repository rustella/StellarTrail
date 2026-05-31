import XCTest
@testable import StellarTrail

final class IOSParityDataTests: XCTestCase {
    func testDTOsDecodeSnakeCasePayloads() throws {
        let roadmapJSON = Data("""
        {
          "items": [
            {
              "id": "roadmap-ios-1",
              "client_key": "ios",
              "title": "iOS 行程详情",
              "summary": "补齐行程详情",
              "details": null,
              "category": "routes",
              "status": "building",
              "priority": 1,
              "sort_order": 10,
              "is_published": true,
              "vote_count": 3,
              "subscription_count": 2,
              "is_voted": false,
              "is_subscribed": true,
              "published_at": "2026-05-17T09:00:00Z",
              "created_at": "2026-05-17T09:00:00Z",
              "updated_at": "2026-05-17T09:00:00Z"
            }
          ],
          "next_cursor": null
        }
        """.utf8)
        let roadmap = try JSONDecoder.stellarTrail.decode(ListRoadmapResponse.self, from: roadmapJSON)
        XCTAssertEqual(roadmap.items.first?.clientKey, .ios)
        XCTAssertEqual(roadmap.items.first?.statusLabel, "开发中")

        let tripsJSON = Data("""
        {
          "items": [
            {
              "id": "trip-1",
              "owner_user_id": "user-1",
              "trip_type": "solo",
              "title": "近郊夜行",
              "description": null,
              "start_date": "2026-05-10",
              "end_date": "2026-05-10",
              "enabled_sections": ["personal_gear", "itinerary"],
              "route_use_slope_adjustment": true,
              "route_use_high_altitude_adjustment": false,
              "route_start_altitude_m": 120,
              "day_count": 1,
              "field_versions": {},
              "is_deleted": false,
              "created_at": "2026-05-10T09:00:00Z",
              "updated_at": "2026-05-10T09:00:00Z",
              "time_bucket": "past",
              "days_until_start": null,
              "days_until_end": null,
              "member_count": 1,
              "readiness": {
                "missing_count": 0,
                "missing_labels": [],
                "completion_percent": 100
              },
              "outdoor_experience_id": "experience-1"
            }
          ],
          "next_cursor": null
        }
        """.utf8)
        let trips = try JSONDecoder.stellarTrail.decode(ListTripsResponse.self, from: tripsJSON)
        XCTAssertEqual(trips.items.first?.tripType, .solo)
        XCTAssertEqual(trips.items.first?.readiness.completionPercent, 100)
    }

    func testTripSectionFilteringMatchesSoloTeamRules() {
        XCTAssertFalse(TripSectionKey.allowed(for: .solo).contains(.members))
        XCTAssertFalse(TripSectionKey.allowed(for: .solo).contains(.sharedGear))
        XCTAssertTrue(TripSectionKey.allowed(for: .solo).contains(.personalGear))
        XCTAssertTrue(TripSectionKey.allowed(for: .team).contains(.members))
        XCTAssertTrue(TripSectionKey.allowed(for: .team).contains(.sharedGear))
    }

    @MainActor
    func testRepositoriesUseExpectedRoutesAndAuthBoundaries() async throws {
        URLProtocolStub.reset()
        URLProtocolStub.handler = { request in
            URLProtocolStub.requests.append(request)
            let path = request.url?.path ?? ""
            let body: String
            switch path {
            case "/api/v1/roadmap":
                body = #"{"items":[],"next_cursor":null}"#
            case "/api/v1/me/trips":
                body = #"{"items":[],"next_cursor":null}"#
            default:
                body = #"{"message":"unexpected route"}"#
            }
            let status = path == "/api/v1/roadmap" || path == "/api/v1/me/trips" ? 200 : 404
            return (HTTPURLResponse(url: request.url!, statusCode: status, httpVersion: nil, headerFields: ["Content-Type": "application/json"])!, Data(body.utf8))
        }
        defer { URLProtocolStub.reset() }

        let sessionStore = SessionStore(keychainStore: InMemoryKeychainStore())
        sessionStore.replace(with: Session.fixture)
        let client = makeClient(sessionStore: sessionStore)

        _ = try await RoadmapRepository(client: client).list(ListRoadmapRequest(clientKey: .ios, status: .building, limit: 10), includeUserState: false)
        let roadmapRequest = try XCTUnwrap(URLProtocolStub.requests.first)
        XCTAssertEqual(roadmapRequest.url?.path, "/api/v1/roadmap")
        XCTAssertEqual(URLComponents(url: roadmapRequest.url!, resolvingAgainstBaseURL: false)?.queryItems?.first(where: { $0.name == "client_key" })?.value, "ios")
        XCTAssertNil(roadmapRequest.value(forHTTPHeaderField: "Authorization"))

        _ = try await TripRepository(client: client).list(ListTripsRequest(limit: 7, bucket: .upcoming, today: "2026-05-29"))
        let tripRequest = try XCTUnwrap(URLProtocolStub.requests.last)
        XCTAssertEqual(tripRequest.url?.path, "/api/v1/me/trips")
        XCTAssertEqual(URLComponents(url: tripRequest.url!, resolvingAgainstBaseURL: false)?.queryItems?.first(where: { $0.name == "bucket" })?.value, "upcoming")
        XCTAssertEqual(tripRequest.value(forHTTPHeaderField: "Authorization"), "Bearer \(Session.fixture.accessToken)")
    }

    @MainActor
    func testPrivateRepositoriesRequireSession() async {
        let client = makeClient(sessionStore: SessionStore(keychainStore: InMemoryKeychainStore()))
        do {
            _ = try await TripRepository(client: client).list(ListTripsRequest())
            XCTFail("Expected private trip repository to require a session")
        } catch {
            XCTAssertEqual(error as? AppError, .missingSession)
        }
    }

    @MainActor
    private func makeClient(sessionStore: SessionStore) -> APIClient {
        let config = URLSessionConfiguration.ephemeral
        config.protocolClasses = [URLProtocolStub.self]
        let session = URLSession(configuration: config)
        let defaults = UserDefaults(suiteName: "ios-parity-data-tests-\(UUID().uuidString)")!
        let settings = AppSettingsStore(
            defaults: defaults,
            clientConfig: ClientConfig(apiBaseURLString: "https://unit.test", assetsBaseURLString: "https://assets.unit.test")
        )
        return APIClient(settingsStore: settings, sessionStore: sessionStore, session: session)
    }
}

private final class URLProtocolStub: URLProtocol {
    static var requests: [URLRequest] = []
    static var handler: ((URLRequest) throws -> (HTTPURLResponse, Data))?

    static func reset() {
        requests = []
        handler = nil
    }

    override class func canInit(with request: URLRequest) -> Bool {
        true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        guard let handler = Self.handler else {
            client?.urlProtocol(self, didFailWithError: AppError.network("missing test handler"))
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
