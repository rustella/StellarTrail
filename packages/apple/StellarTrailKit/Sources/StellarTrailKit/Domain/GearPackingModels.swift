import Foundation

struct GearPackingListStats: Codable, Equatable {
    let itemCount: Int
    let packedCount: Int
    let totalWeightG: Int

    static let empty = GearPackingListStats(itemCount: 0, packedCount: 0, totalWeightG: 0)
}

struct GearPackingListSummary: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let routeName: String?
    let durationLabel: String?
    let itemCount: Int
    let packedCount: Int
    let totalWeightG: Int
    let createdAt: String
    let updatedAt: String

    var metaText: String {
        [routeName?.nilIfBlank, durationLabel?.nilIfBlank].compactMap { $0 }.joined(separator: " · ")
    }

    var progressText: String {
        "\(packedCount)/\(itemCount)"
    }

    var weightText: String {
        Formatters.weight(totalWeightG)
    }
}

struct ListGearPackingListsRequest: Equatable {
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [URLQueryItem(name: "limit", value: String(limit))]
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListGearPackingListsResponse: Codable, Equatable {
    let items: [GearPackingListSummary]
    let nextCursor: String?
}

struct CreateGearPackingListRequest: Codable, Equatable {
    var name: String
    var routeName: String?
    var durationLabel: String?
}

typealias UpdateGearPackingListRequest = CreateGearPackingListRequest

struct AddGearPackingItemsRequest: Codable, Equatable {
    var gearIds: [String]
}

struct UpdateGearPackingItemRequest: Codable, Equatable {
    var packed: Bool?
    var plannedQuantity: Int?
    var packedQuantity: Int?
}

struct GearPackingListItem: Codable, Equatable, Identifiable {
    let id: String
    let gearId: String
    let plannedQuantity: Int
    let packedQuantity: Int
    let packed: Bool
    let unavailable: Bool
    let unavailableReason: String?
    let gear: GearSummary
    let createdAt: String
    let updatedAt: String

    var plannedText: String {
        "计划 \(plannedQuantity)"
    }

    var packedText: String {
        "已装 \(packedQuantity)"
    }

    var unavailableText: String {
        switch unavailableReason {
        case "archived": return "这件装备已移入历史"
        case "deleted": return "这件装备已删除"
        default: return "这件装备当前不可用"
        }
    }
}

struct GearPackingListDetail: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let routeName: String?
    let durationLabel: String?
    let stats: GearPackingListStats
    let items: [GearPackingListItem]
    let createdAt: String
    let updatedAt: String

    var metaText: String {
        [routeName?.nilIfBlank, durationLabel?.nilIfBlank].compactMap { $0 }.joined(separator: " · ")
    }

    var progressText: String {
        "\(stats.packedCount)/\(stats.itemCount)"
    }

    var weightText: String {
        Formatters.weight(stats.totalWeightG)
    }
}
