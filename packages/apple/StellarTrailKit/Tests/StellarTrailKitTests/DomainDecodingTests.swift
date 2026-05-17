import XCTest
import Foundation
@testable import StellarTrailKit

final class DomainDecodingTests: XCTestCase {
    func testLoginResponseDecodesSnakeCaseSessionPayload() throws {
        let json = """
        {
          "access_token":"x",
          "expires_at":"2026-05-16T12:00:00Z",
          "refresh_token":"r",
          "refresh_expires_at":"2026-06-15T10:00:00Z",
          "user":{"id":"u1","username":"trail_alice","email":"alice@example.com","nickname":null,"avatar_url":null}
        }
        """.data(using: .utf8)!

        let response = try JSONDecoder.stellarTrail.decode(LoginResponse.self, from: json)

        XCTAssertEqual(response.accessToken, "x")
        XCTAssertEqual(response.refreshToken, "r")
        XCTAssertEqual(response.user.username, "trail_alice")
    }

    func testGearItemDecodesOptionalFieldsAndLabels() throws {
        let json = """
        {
          "id":"gear-1",
          "user_id":"user-1",
          "category":"backpack_system",
          "name":"轻量背包",
          "brand":"山野",
          "model":"45L",
          "color":null,
          "material":null,
          "capacity":"45L",
          "size":null,
          "description":"周末线路",
          "weight_g":980,
          "warmth_index":null,
          "waterproof_index":null,
          "purchase_date":"2026-05-01",
          "purchase_price_cents":89900,
          "expiry_or_warranty_date":null,
          "purchase_location":"杭州",
          "status":"available",
          "storage_location":"装备柜",
          "tags":["轻量","三季"],
          "share_enabled":true,
          "share_status":"approved",
          "notes":"常用",
          "archived_at":null,
          "created_at":"2026-05-01T10:00:00Z",
          "updated_at":"2026-05-02T10:00:00Z"
        }
        """.data(using: .utf8)!

        let item = try JSONDecoder.stellarTrail.decode(GearItem.self, from: json)

        XCTAssertEqual(item.category.label, "背负系统")
        XCTAssertEqual(item.status.label, "可用")
        XCTAssertEqual(item.formattedWeight, "980 g")
        XCTAssertEqual(item.formattedPrice, "¥899")
    }

    func testKnotListUsesOffsetPagination() throws {
        let json = """
        {
          "items":[{"id":"bowline","title":"单套结","summary":"可靠绳圈","difficulty":"入门","categories":[{"id":"rescue","title":"救援"}],"types":[],"media_count":6}],
          "next_offset":20
        }
        """.data(using: .utf8)!

        let response = try JSONDecoder.stellarTrail.decode(ListKnotsResponse.self, from: json)

        XCTAssertEqual(response.items.first?.title, "单套结")
        XCTAssertEqual(response.nextOffset, 20)
    }
}
