import Foundation

enum GearCategory: String, Codable, CaseIterable, Identifiable {
    case backpackSystem = "backpack_system"
    case sleepSystem = "sleep_system"
    case kitchenSystem = "kitchen_system"
    case walkingSystem = "walking_system"
    case clothingSystem = "clothing_system"
    case lightingSystem = "lighting_system"
    case firstAidSystem = "first_aid_system"
    case electronicsSystem = "electronics_system"
    case technicalGear = "technical_gear"
    case otherGear = "other_gear"
    case consumable

    var id: String { rawValue }

    var label: String {
        switch self {
        case .backpackSystem: return "背负系统"
        case .sleepSystem: return "睡眠系统"
        case .kitchenSystem: return "炊具餐饮"
        case .walkingSystem: return "行走辅助"
        case .clothingSystem: return "服装鞋袜"
        case .lightingSystem: return "照明"
        case .firstAidSystem: return "急救安全"
        case .electronicsSystem: return "电子设备"
        case .technicalGear: return "技术装备"
        case .otherGear: return "其他装备"
        case .consumable: return "消耗品"
        }
    }
}

enum GearStatus: String, Codable, CaseIterable, Identifiable {
    case available
    case inUse = "in_use"
    case maintenance
    case damaged
    case lost
    case retired
    case sold
    case idle

    var id: String { rawValue }

    var label: String {
        switch self {
        case .available: return "可用"
        case .inUse: return "使用中"
        case .maintenance: return "保养中"
        case .damaged: return "待维修"
        case .lost: return "已遗失"
        case .retired: return "已退役"
        case .sold: return "已转让"
        case .idle: return "闲置"
        }
    }

    var badgeTone: TrailBadgeTone {
        switch self {
        case .available, .inUse: return .success
        case .maintenance, .idle: return .warning
        case .damaged, .lost: return .danger
        case .retired, .sold: return .neutral
        }
    }
}

enum GearShareStatus: String, Codable, CaseIterable, Identifiable {
    case notShared = "not_shared"
    case pending
    case approved
    case rejected
    case withdrawn

    var id: String { rawValue }

    var label: String {
        switch self {
        case .notShared: return "仅自己可见"
        case .pending: return "待审核"
        case .approved: return "已公开"
        case .rejected: return "未通过"
        case .withdrawn: return "已撤回"
        }
    }
}

enum GearTab: String, Codable, CaseIterable, Identifiable {
    case available
    case history

    var id: String { rawValue }
    var label: String { self == .available ? "可用" : "历史" }
}

enum GearSort: String, Codable, CaseIterable, Identifiable {
    case createdAtDesc = "created_at_desc"
    case createdAtAsc = "created_at_asc"
    case purchaseDateDesc = "purchase_date_desc"
    case nameAsc = "name_asc"
    case weightDesc = "weight_desc"
    case priceDesc = "price_desc"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .createdAtDesc: return "最近添加"
        case .createdAtAsc: return "最早添加"
        case .purchaseDateDesc: return "最近购买"
        case .nameAsc: return "名称排序"
        case .weightDesc: return "重量优先"
        case .priceDesc: return "价格优先"
        }
    }
}

struct GearItem: Codable, Equatable, Identifiable {
    let id: String
    let userId: String
    let category: GearCategory
    let name: String
    let brand: String?
    let model: String?
    let color: String?
    let material: String?
    let capacity: String?
    let size: String?
    let description: String?
    let weightG: Int?
    let warmthIndex: String?
    let waterproofIndex: String?
    let purchaseDate: String?
    let purchasePriceCents: Int?
    let expiryOrWarrantyDate: String?
    let purchaseLocation: String?
    let status: GearStatus
    let storageLocation: String?
    let tags: [String]
    let shareEnabled: Bool
    let shareStatus: GearShareStatus
    let notes: String?
    let archivedAt: String?
    let createdAt: String
    let updatedAt: String

    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedPrice: String { Formatters.price(purchasePriceCents) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
    var isArchived: Bool { archivedAt != nil }
}

struct GearSummary: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let status: GearStatus
    let statusLabel: String
    let weightG: Int?
    let purchasePriceCents: Int?
    let purchaseDate: String?
    let createdAt: String
    let updatedAt: String

    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedPrice: String { Formatters.price(purchasePriceCents) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
}

struct CreateGearRequest: Codable, Equatable {
    var category: GearCategory
    var name: String
    var brand: String?
    var model: String?
    var color: String?
    var material: String?
    var capacity: String?
    var size: String?
    var description: String?
    var weightG: Int?
    var warmthIndex: String?
    var waterproofIndex: String?
    var purchaseDate: String?
    var purchasePriceCents: Int?
    var expiryOrWarrantyDate: String?
    var purchaseLocation: String?
    var status: GearStatus?
    var storageLocation: String?
    var tags: [String]?
    var shareEnabled: Bool?
    var notes: String?
}

typealias UpdateGearRequest = CreateGearRequest

struct GearCategoryCount: Codable, Equatable, Identifiable {
    let category: GearCategory
    let label: String
    let count: Int
    var id: String { category.rawValue }
}

struct GearStatusCount: Codable, Equatable, Identifiable {
    let status: GearStatus
    let label: String
    let count: Int
    var id: String { status.rawValue }
}

struct GearStatsResponse: Codable, Equatable {
    let currentCount: Int
    let archivedCount: Int
    let totalValueCents: Int
    let totalWeightG: Int
    let byCategory: [GearCategoryCount]
    let byStatus: [GearStatusCount]

    static let empty = GearStatsResponse(currentCount: 0, archivedCount: 0, totalValueCents: 0, totalWeightG: 0, byCategory: [], byStatus: [])
}

struct GearCategoryFilter: Codable, Equatable, Identifiable {
    let id: String
    let label: String
    let count: Int
}

struct GearCategoriesResponse: Codable, Equatable {
    let items: [GearCategoryFilter]
}

struct ListGearsRequest: Equatable {
    var tab: GearTab = .available
    var category: GearCategory?
    var status: GearStatus?
    var q: String?
    var sort: GearSort = .createdAtDesc
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "tab", value: tab.rawValue),
            URLQueryItem(name: "sort", value: sort.rawValue),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let category { items.append(URLQueryItem(name: "category", value: category.rawValue)) }
        if let status { items.append(URLQueryItem(name: "status", value: status.rawValue)) }
        if let q = q?.nilIfBlank { items.append(URLQueryItem(name: "q", value: q)) }
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListGearsResponse: Codable, Equatable {
    let items: [GearSummary]
    let nextCursor: String?
}

struct GearTemplateCategory: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let items: [String]
}

struct GearTemplate: Codable, Equatable, Identifiable {
    let id: String
    let title: String
    let categories: [GearTemplateCategory]
}

struct GearTemplatesResponse: Codable, Equatable {
    let items: [GearTemplate]
}

extension GearItem {
    func summary() -> GearSummary {
        GearSummary(
            id: id,
            category: category,
            categoryLabel: category.label,
            name: name,
            brand: brand,
            model: model,
            status: status,
            statusLabel: status.label,
            weightG: weightG,
            purchasePriceCents: purchasePriceCents,
            purchaseDate: purchaseDate,
            createdAt: createdAt,
            updatedAt: updatedAt
        )
    }
}

extension CreateGearRequest {
    static let blank = CreateGearRequest(
        category: .backpackSystem,
        name: "",
        brand: nil,
        model: nil,
        color: nil,
        material: nil,
        capacity: nil,
        size: nil,
        description: nil,
        weightG: nil,
        warmthIndex: nil,
        waterproofIndex: nil,
        purchaseDate: nil,
        purchasePriceCents: nil,
        expiryOrWarrantyDate: nil,
        purchaseLocation: nil,
        status: .available,
        storageLocation: nil,
        tags: [],
        shareEnabled: false,
        notes: nil
    )

    init(item: GearItem) {
        self.init(
            category: item.category,
            name: item.name,
            brand: item.brand,
            model: item.model,
            color: item.color,
            material: item.material,
            capacity: item.capacity,
            size: item.size,
            description: item.description,
            weightG: item.weightG,
            warmthIndex: item.warmthIndex,
            waterproofIndex: item.waterproofIndex,
            purchaseDate: item.purchaseDate,
            purchasePriceCents: item.purchasePriceCents,
            expiryOrWarrantyDate: item.expiryOrWarrantyDate,
            purchaseLocation: item.purchaseLocation,
            status: item.status,
            storageLocation: item.storageLocation,
            tags: item.tags,
            shareEnabled: item.shareEnabled,
            notes: item.notes
        )
    }
}
