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
        case .kitchenSystem: return "炊具系统"
        case .walkingSystem: return "行走系统"
        case .clothingSystem: return "服装系统"
        case .lightingSystem: return "照明系统"
        case .firstAidSystem: return "急救系统"
        case .electronicsSystem: return "电子系统"
        case .technicalGear: return "技术装备"
        case .otherGear: return "其他装备"
        case .consumable: return "消耗品"
        }
    }

    var hint: String {
        switch self {
        case .backpackSystem: return "背包、外挂、收纳"
        case .sleepSystem: return "帐篷、睡袋、防潮垫"
        case .kitchenSystem: return "炉具、锅具、餐具"
        case .walkingSystem: return "登山杖、鞋袜、护具"
        case .clothingSystem: return "冲锋衣、保暖、换洗"
        case .lightingSystem: return "头灯、营灯、电池"
        case .firstAidSystem: return "药包、绷带、应急毯"
        case .electronicsSystem: return "电源、导航、通信"
        case .technicalGear: return "冰雪、攀登、安全"
        case .otherGear: return "杂项与个性化装备"
        case .consumable: return "气罐、食品、一次性用品"
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
        case .damaged: return "损坏"
        case .lost: return "丢失"
        case .retired: return "退役"
        case .sold: return "已售出"
        case .idle: return "闲置"
        }
    }

    var badgeTone: TrailBadgeTone {
        switch self {
        case .available: return .success
        case .inUse, .maintenance, .idle: return .warning
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
        case .notShared: return "未共享"
        case .pending: return "审核中"
        case .approved: return "已共享"
        case .rejected: return "已拒绝"
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

enum DeletedFilter: String, Codable, CaseIterable, Identifiable {
    case active
    case deleted
    case all

    var id: String { rawValue }
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
        case .nameAsc: return "名称 A-Z"
        case .weightDesc: return "重量优先"
        case .priceDesc: return "价格优先"
        }
    }
}

enum GearCurrency: String, Codable, CaseIterable, Identifiable {
    case cny = "CNY"
    case usd = "USD"
    case eur = "EUR"
    case jpy = "JPY"
    case hkd = "HKD"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .cny: return "¥ CNY"
        case .usd: return "USD"
        case .eur: return "EUR"
        case .jpy: return "JPY"
        case .hkd: return "HKD"
        }
    }

    static func normalized(_ value: String?) -> GearCurrency {
        guard let value = value?.nilIfBlank?.uppercased(),
              let currency = GearCurrency(rawValue: value)
        else { return .cny }
        return currency
    }
}

enum GearWeightUnit: String, CaseIterable, Identifiable {
    case kg
    case g
    case lb
    case oz

    var id: String { rawValue }
    var label: String { rawValue }
}

enum GearAtlasStatus: String, Codable, CaseIterable, Identifiable {
    case pending
    case approved
    case rejected

    var id: String { rawValue }

    var label: String {
        switch self {
        case .pending: return "审核中"
        case .approved: return "已收录"
        case .rejected: return "已拒绝"
        }
    }

    var badgeTone: TrailBadgeTone {
        switch self {
        case .pending: return .warning
        case .approved: return .success
        case .rejected: return .danger
        }
    }
}

enum GearAtlasSourceType: String, Codable, CaseIterable, Identifiable {
    case manual
    case userGear = "user_gear"
    case externalImport = "external_import"

    var id: String { rawValue }
    var label: String {
        switch self {
        case .manual: return "手动投稿"
        case .userGear: return "来自我的装备"
        case .externalImport: return "外部导入"
        }
    }
}

enum GearAtlasSort: String, Codable, CaseIterable, Identifiable {
    case approvedAtDesc = "approved_at_desc"
    case nameAsc = "name_asc"
    case weightDesc = "weight_desc"
    case officialPriceDesc = "official_price_desc"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .approvedAtDesc: return "最近收录"
        case .nameAsc: return "名称 A-Z"
        case .weightDesc: return "重量优先"
        case .officialPriceDesc: return "价格优先"
        }
    }
}

enum GearTagColor: String, Codable, CaseIterable, Identifiable {
    case teal
    case blue
    case violet
    case rose
    case orange
    case amber
    case green
    case slate

    var id: String { rawValue }

    var label: String {
        switch self {
        case .teal: return "青绿"
        case .blue: return "蓝"
        case .violet: return "紫"
        case .rose: return "粉"
        case .orange: return "橙"
        case .amber: return "黄"
        case .green: return "绿"
        case .slate: return "灰"
        }
    }

    static func normalized(_ value: String?) -> GearTagColor? {
        guard let value = value?.nilIfBlank else { return nil }
        return GearTagColor(rawValue: value)
    }

    static func fallback(for tag: String) -> GearTagColor {
        let options = GearTagColor.allCases
        let hash = tag.unicodeScalars.reduce(UInt32(0)) { partialResult, scalar in
            partialResult &* 31 &+ scalar.value
        }
        return options[Int(hash % UInt32(options.count))]
    }
}

typealias GearSpecs = [String: String]
typealias GearTagColorMap = [String: String]

struct GearSpecField: Equatable, Identifiable {
    let key: String
    let label: String
    let placeholder: String
    let inputType: String
    let units: [String]
    let unitLabels: [String]
    let choiceOnly: Bool

    var id: String { key }
}

struct GearSpecFieldView: Equatable, Identifiable {
    let field: GearSpecField
    var valueText: String
    var unitIndex: Int

    var id: String { field.key }
    var key: String { field.key }
    var label: String { field.label }
    var placeholder: String { field.placeholder }
    var inputType: String { field.inputType }
    var units: [String] { field.units }
    var unitLabels: [String] { field.unitLabels.isEmpty ? field.units : field.unitLabels }
    var choiceOnly: Bool { field.choiceOnly }
    var unitLabel: String { field.units.indices.contains(unitIndex) ? field.units[unitIndex] : "" }
}

struct GearTagView: Equatable, Identifiable {
    var name: String
    var color: GearTagColor

    var id: String { name }
}

struct GearTagSuggestion: Codable, Equatable, Identifiable {
    let tag: String
    let color: String?

    var id: String { tag }
}

struct GearTagSuggestionView: Equatable, Identifiable {
    let name: String
    let color: GearTagColor

    var id: String { name }
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
    let officialPriceCents: Int?
    let officialPriceCurrency: String?
    let warmthIndex: String?
    let waterproofIndex: String?
    let purchaseDate: String?
    let purchasePriceCents: Int?
    let purchasePriceCurrency: String?
    let expiryOrWarrantyDate: String?
    let purchaseLocation: String?
    let status: GearStatus
    let storageLocation: String?
    let specs: GearSpecs?
    let tags: [String]
    let tagColors: GearTagColorMap?
    let shareEnabled: Bool
    let shareStatus: GearShareStatus
    let notes: String?
    let archivedAt: String?
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String

    var categoryLabel: String { category.label }
    var statusLabel: String { status.label }
    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedOfficialPrice: String { Formatters.price(officialPriceCents, currency: officialPriceCurrency) }
    var formattedPrice: String { Formatters.price(purchasePriceCents, currency: purchasePriceCurrency) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
    var isArchived: Bool { archivedAt != nil }
    var tagViews: [GearTagView] { GearOptions.createTagViews(tags: tags, colors: tagColors ?? [:]) }
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
    let officialPriceCents: Int?
    let officialPriceCurrency: String?
    let purchasePriceCents: Int?
    let purchasePriceCurrency: String?
    let purchaseDate: String?
    let specs: GearSpecs?
    let tags: [String]
    let tagColors: GearTagColorMap?
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String

    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedOfficialPrice: String { Formatters.price(officialPriceCents, currency: officialPriceCurrency) }
    var formattedPrice: String { Formatters.price(purchasePriceCents, currency: purchasePriceCurrency) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
    var tagViews: [GearTagView] { GearOptions.createTagViews(tags: tags, colors: tagColors ?? [:]) }

    init(
        id: String,
        category: GearCategory,
        categoryLabel: String? = nil,
        name: String,
        brand: String?,
        model: String?,
        status: GearStatus,
        statusLabel: String? = nil,
        weightG: Int?,
        officialPriceCents: Int? = nil,
        officialPriceCurrency: String? = nil,
        purchasePriceCents: Int?,
        purchasePriceCurrency: String? = nil,
        purchaseDate: String?,
        specs: GearSpecs? = nil,
        tags: [String] = [],
        tagColors: GearTagColorMap? = nil,
        isDeleted: Bool = false,
        createdAt: String,
        updatedAt: String
    ) {
        self.id = id
        self.category = category
        self.categoryLabel = categoryLabel ?? category.label
        self.name = name
        self.brand = brand
        self.model = model
        self.status = status
        self.statusLabel = statusLabel ?? status.label
        self.weightG = weightG
        self.officialPriceCents = officialPriceCents
        self.officialPriceCurrency = officialPriceCurrency
        self.purchasePriceCents = purchasePriceCents
        self.purchasePriceCurrency = purchasePriceCurrency
        self.purchaseDate = purchaseDate
        self.specs = specs
        self.tags = tags
        self.tagColors = tagColors
        self.isDeleted = isDeleted
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }

    enum CodingKeys: String, CodingKey {
        case id, category, categoryLabel, name, brand, model, status, statusLabel, weightG, officialPriceCents, officialPriceCurrency, purchasePriceCents, purchasePriceCurrency, purchaseDate, specs, tags, tagColors, isDeleted, createdAt, updatedAt
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let category = try container.decode(GearCategory.self, forKey: .category)
        let status = try container.decode(GearStatus.self, forKey: .status)
        self.init(
            id: try container.decode(String.self, forKey: .id),
            category: category,
            categoryLabel: try container.decodeIfPresent(String.self, forKey: .categoryLabel),
            name: try container.decode(String.self, forKey: .name),
            brand: try container.decodeIfPresent(String.self, forKey: .brand),
            model: try container.decodeIfPresent(String.self, forKey: .model),
            status: status,
            statusLabel: try container.decodeIfPresent(String.self, forKey: .statusLabel),
            weightG: try container.decodeIfPresent(Int.self, forKey: .weightG),
            officialPriceCents: try container.decodeIfPresent(Int.self, forKey: .officialPriceCents),
            officialPriceCurrency: try container.decodeIfPresent(String.self, forKey: .officialPriceCurrency),
            purchasePriceCents: try container.decodeIfPresent(Int.self, forKey: .purchasePriceCents),
            purchasePriceCurrency: try container.decodeIfPresent(String.self, forKey: .purchasePriceCurrency),
            purchaseDate: try container.decodeIfPresent(String.self, forKey: .purchaseDate),
            specs: try container.decodeIfPresent(GearSpecs.self, forKey: .specs),
            tags: try container.decodeIfPresent([String].self, forKey: .tags) ?? [],
            tagColors: try container.decodeIfPresent(GearTagColorMap.self, forKey: .tagColors),
            isDeleted: try container.decodeIfPresent(Bool.self, forKey: .isDeleted) ?? false,
            createdAt: try container.decode(String.self, forKey: .createdAt),
            updatedAt: try container.decode(String.self, forKey: .updatedAt)
        )
    }
}

struct CreateGearRequest: Codable, Equatable {
    var category: GearCategory
    var name: String
    var brand: String?
    var model: String?
    var description: String?
    var weightG: Int?
    var officialPriceCents: Int?
    var officialPriceCurrency: String?
    var purchaseDate: String?
    var purchasePriceCents: Int?
    var purchasePriceCurrency: String?
    var purchaseLocation: String?
    var status: GearStatus?
    var storageLocation: String?
    var specs: GearSpecs?
    var tags: [String]?
    var tagColors: GearTagColorMap?
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

    var isAll: Bool { id == "all" }
    var category: GearCategory? { GearCategory(rawValue: id) }
}

struct GearCategoriesResponse: Codable, Equatable {
    let items: [GearCategoryFilter]
}

struct GearSpecKeyRankingsResponse: Codable, Equatable {
    let keys: [String]
}

struct GearTagSuggestionsResponse: Codable, Equatable {
    let items: [GearTagSuggestion]
}

struct ListGearsRequest: Equatable {
    var tab: GearTab = .available
    var category: GearCategory?
    var status: GearStatus?
    var deleted: DeletedFilter = .active
    var q: String?
    var sort: GearSort = .createdAtDesc
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "tab", value: tab.rawValue),
            URLQueryItem(name: "deleted", value: deleted.rawValue),
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

struct GearAtlasPublicItem: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let description: String?
    let weightG: Int?
    let officialPriceCents: Int?
    let officialPriceCurrency: String?
    let specs: GearSpecs?
    let approvedAt: String?
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String

    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedOfficialPrice: String { Formatters.price(officialPriceCents, currency: officialPriceCurrency) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
}

struct GearAtlasSubmission: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let description: String?
    let weightG: Int?
    let officialPriceCents: Int?
    let officialPriceCurrency: String?
    let specs: GearSpecs?
    let approvedAt: String?
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String
    let sourceType: GearAtlasSourceType
    let sourceUserGearId: String?
    let status: GearAtlasStatus
    let rejectionReason: String?
    let reviewedAt: String?

    var formattedWeight: String { Formatters.weight(weightG) }
    var formattedOfficialPrice: String { Formatters.price(officialPriceCents, currency: officialPriceCurrency) }
    var brandModel: String { Formatters.brandModel(brand: brand, model: model) }
}

struct CreateGearAtlasSubmissionRequest: Codable, Equatable {
    var category: GearCategory
    var name: String
    var brand: String?
    var model: String?
    var description: String?
    var weightG: Int?
    var officialPriceCents: Int?
    var officialPriceCurrency: String?
    var specs: GearSpecs?
}

struct ListGearAtlasRequest: Equatable {
    var category: GearCategory?
    var q: String?
    var sort: GearAtlasSort = .approvedAtDesc
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "sort", value: sort.rawValue),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let category { items.append(URLQueryItem(name: "category", value: category.rawValue)) }
        if let q = q?.nilIfBlank { items.append(URLQueryItem(name: "q", value: q)) }
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListGearAtlasResponse: Codable, Equatable {
    let items: [GearAtlasPublicItem]
    let nextCursor: String?
}

struct ListGearAtlasSubmissionsRequest: Equatable {
    var status: GearAtlasStatus?
    var category: GearCategory?
    var deleted: DeletedFilter = .active
    var q: String?
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [URLQueryItem(name: "limit", value: String(limit))]
        if let status { items.append(URLQueryItem(name: "status", value: status.rawValue)) }
        if let category { items.append(URLQueryItem(name: "category", value: category.rawValue)) }
        items.append(URLQueryItem(name: "deleted", value: deleted.rawValue))
        if let q = q?.nilIfBlank { items.append(URLQueryItem(name: "q", value: q)) }
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListGearAtlasSubmissionsResponse: Codable, Equatable {
    let items: [GearAtlasSubmission]
    let nextCursor: String?
}

struct GearFormDraft: Equatable {
    var category: GearCategory
    var name: String
    var brand: String
    var model: String
    var description: String
    var weightText: String
    var weightUnit: GearWeightUnit
    var purchaseDate: String
    var officialPriceText: String
    var officialPriceCurrency: GearCurrency
    var purchasePriceText: String
    var purchasePriceCurrency: GearCurrency
    var purchaseLocation: String
    var status: GearStatus
    var storageLocation: String
    var specs: GearSpecs
    var tags: [GearTagView]
    var shareEnabled: Bool
    var notes: String

    static let blank = GearFormDraft(
        category: .backpackSystem,
        name: "",
        brand: "",
        model: "",
        description: "",
        weightText: "",
        weightUnit: .kg,
        purchaseDate: "",
        officialPriceText: "",
        officialPriceCurrency: .cny,
        purchasePriceText: "",
        purchasePriceCurrency: .cny,
        purchaseLocation: "",
        status: .available,
        storageLocation: "",
        specs: [:],
        tags: [],
        shareEnabled: false,
        notes: ""
    )

    init(
        category: GearCategory,
        name: String,
        brand: String,
        model: String,
        description: String,
        weightText: String,
        weightUnit: GearWeightUnit,
        purchaseDate: String,
        officialPriceText: String,
        officialPriceCurrency: GearCurrency,
        purchasePriceText: String,
        purchasePriceCurrency: GearCurrency,
        purchaseLocation: String,
        status: GearStatus,
        storageLocation: String,
        specs: GearSpecs,
        tags: [GearTagView],
        shareEnabled: Bool,
        notes: String
    ) {
        self.category = category
        self.name = name
        self.brand = brand
        self.model = model
        self.description = description
        self.weightText = weightText
        self.weightUnit = weightUnit
        self.purchaseDate = purchaseDate
        self.officialPriceText = officialPriceText
        self.officialPriceCurrency = officialPriceCurrency
        self.purchasePriceText = purchasePriceText
        self.purchasePriceCurrency = purchasePriceCurrency
        self.purchaseLocation = purchaseLocation
        self.status = status
        self.storageLocation = storageLocation
        self.specs = specs
        self.tags = tags
        self.shareEnabled = shareEnabled
        self.notes = notes
    }

    init(item: GearItem) {
        self.init(
            category: item.category,
            name: item.name,
            brand: item.brand ?? "",
            model: item.model ?? "",
            description: item.description ?? "",
            weightText: Formatters.weightInputText(item.weightG, unit: .kg),
            weightUnit: .kg,
            purchaseDate: item.purchaseDate ?? "",
            officialPriceText: Formatters.priceInputText(item.officialPriceCents, currency: item.officialPriceCurrency),
            officialPriceCurrency: GearCurrency.normalized(item.officialPriceCurrency),
            purchasePriceText: Formatters.priceInputText(item.purchasePriceCents, currency: item.purchasePriceCurrency),
            purchasePriceCurrency: GearCurrency.normalized(item.purchasePriceCurrency),
            purchaseLocation: item.purchaseLocation ?? "",
            status: item.status,
            storageLocation: item.storageLocation ?? "",
            specs: item.specs ?? GearOptions.legacySpecs(from: item),
            tags: item.tagViews,
            shareEnabled: item.shareEnabled,
            notes: item.notes ?? ""
        )
    }

    func buildGearPayload() throws -> CreateGearRequest {
        let name = try requiredText(self.name, message: "装备名称不能为空")
        return CreateGearRequest(
            category: category,
            name: name,
            brand: brand.nilIfBlank,
            model: model.nilIfBlank,
            description: description.nilIfBlank,
            weightG: try GearOptions.weightToGrams(weightText, unit: weightUnit),
            officialPriceCents: try GearOptions.priceToMinorUnits(officialPriceText, currency: officialPriceCurrency),
            officialPriceCurrency: officialPriceText.nilIfBlank == nil ? nil : officialPriceCurrency.rawValue,
            purchaseDate: purchaseDate.nilIfBlank,
            purchasePriceCents: try GearOptions.priceToMinorUnits(purchasePriceText, currency: purchasePriceCurrency),
            purchasePriceCurrency: purchasePriceText.nilIfBlank == nil ? nil : purchasePriceCurrency.rawValue,
            purchaseLocation: purchaseLocation.nilIfBlank,
            status: status,
            storageLocation: storageLocation.nilIfBlank,
            specs: GearOptions.normalizeSpecs(category: category, specs: specs),
            tags: GearOptions.normalizeTagViews(tags).map(\.name),
            tagColors: GearOptions.tagColorPayload(tags),
            shareEnabled: shareEnabled,
            notes: notes.nilIfBlank
        )
    }

    func buildAtlasPayload() throws -> CreateGearAtlasSubmissionRequest {
        let name = try requiredText(self.name, message: "装备名称不能为空")
        return CreateGearAtlasSubmissionRequest(
            category: category,
            name: name,
            brand: brand.nilIfBlank,
            model: model.nilIfBlank,
            description: description.nilIfBlank,
            weightG: try GearOptions.weightToGrams(weightText, unit: weightUnit),
            officialPriceCents: try GearOptions.priceToMinorUnits(officialPriceText, currency: officialPriceCurrency),
            officialPriceCurrency: officialPriceText.nilIfBlank == nil ? nil : officialPriceCurrency.rawValue,
            specs: GearOptions.normalizeSpecs(category: category, specs: specs)
        )
    }

    private func requiredText(_ value: String, message: String) throws -> String {
        guard let text = value.nilIfBlank else { throw AppError.server(message) }
        return text
    }
}

enum GearOptions {
    static let purchaseLocations = [
        "京东", "淘宝", "天猫", "拼多多", "亚马逊", "闲鱼", "迪卡侬", "三夫户外",
        "REI", "Backcountry", "Moosejaw", "Campsaver", "品牌官网", "品牌门店",
        "线下户外店", "朋友赠送", "其他"
    ]

    static func specFields(for category: GearCategory, rankedKeys: [String] = []) -> [GearSpecField] {
        let fields = allSpecFields[category] ?? []
        guard !rankedKeys.isEmpty else { return fields }
        let byKey = Dictionary(uniqueKeysWithValues: fields.map { ($0.key, $0) })
        var used = Set<String>()
        var ranked: [GearSpecField] = []
        for key in rankedKeys where !used.contains(key) {
            if let field = byKey[key] {
                ranked.append(field)
                used.insert(key)
            }
        }
        ranked.append(contentsOf: fields.filter { !used.contains($0.key) })
        return ranked
    }

    static func specFieldViews(for category: GearCategory, specs: GearSpecs, rankedKeys: [String] = []) -> [GearSpecFieldView] {
        specFields(for: category, rankedKeys: rankedKeys).map { field in
            let parsed = splitSpecValue(specs[field.key] ?? "", units: field.units)
            return GearSpecFieldView(field: field, valueText: parsed.valueText, unitIndex: parsed.unitIndex)
        }
    }

    static func combineSpecValue(_ value: String, unit: String) -> String {
        let text = value.trimmingCharacters(in: .whitespacesAndNewlines)
        let unitText = unit.trimmingCharacters(in: .whitespacesAndNewlines)
        if text.isEmpty { return unitText }
        if unitText.isEmpty { return text }
        return "\(text) \(unitText)"
    }

    static func normalizeSpecs(category: GearCategory, specs: GearSpecs) -> GearSpecs? {
        let allowed = Set(specFields(for: category).map(\.key))
        let normalized = specs.reduce(into: GearSpecs()) { partialResult, element in
            guard allowed.contains(element.key), let value = element.value.nilIfBlank else { return }
            partialResult[element.key] = value
        }
        return normalized.isEmpty ? nil : normalized
    }

    static func createTagViews(tags: [String], colors: GearTagColorMap = [:]) -> [GearTagView] {
        var seen = Set<String>()
        var views: [GearTagView] = []
        for raw in tags {
            guard let name = raw.nilIfBlank, !seen.contains(name) else { continue }
            seen.insert(name)
            let color = GearTagColor.normalized(colors[name]) ?? GearTagColor.fallback(for: name)
            views.append(GearTagView(name: name, color: color))
            if views.count == 20 { break }
        }
        return views
    }

    static func createTagSuggestionViews(_ suggestions: [GearTagSuggestion]) -> [GearTagSuggestionView] {
        suggestions.compactMap { suggestion in
            guard let name = suggestion.tag.nilIfBlank else { return nil }
            return GearTagSuggestionView(name: name, color: GearTagColor.normalized(suggestion.color) ?? GearTagColor.fallback(for: name))
        }
    }

    static func addTagViews(current: [GearTagView], input: String, color: GearTagColor? = nil) -> [GearTagView] {
        var existing = normalizeTagViews(current)
        var seen = Set(existing.map(\.name))
        for name in parseTags(input) where !seen.contains(name) && existing.count < 20 {
            let resolvedColor = color ?? GearTagColor.fallback(for: name)
            existing.append(GearTagView(name: name, color: resolvedColor))
            seen.insert(name)
        }
        return existing
    }

    static func normalizeTagViews(_ tags: [GearTagView]) -> [GearTagView] {
        var seen = Set<String>()
        var normalized: [GearTagView] = []
        for tag in tags {
            guard let name = tag.name.nilIfBlank, !seen.contains(name) else { continue }
            normalized.append(GearTagView(name: name, color: tag.color))
            seen.insert(name)
            if normalized.count == 20 { break }
        }
        return normalized
    }

    static func parseTags(_ input: String) -> [String] {
        var seen = Set<String>()
        return input
            .split { character in
                character == "," || character == "，" || character == ";" || character == "；" || character == "\n" || character == " "
            }
            .compactMap { String($0).nilIfBlank }
            .filter { seen.insert($0).inserted }
    }

    static func tagColorPayload(_ tags: [GearTagView]) -> GearTagColorMap? {
        let colors = normalizeTagViews(tags).reduce(into: GearTagColorMap()) { partialResult, tag in
            partialResult[tag.name] = tag.color.rawValue
        }
        return colors.isEmpty ? nil : colors
    }

    static func weightToGrams(_ value: String, unit: GearWeightUnit) throws -> Int? {
        guard let text = value.nilIfBlank else { return nil }
        guard let number = Double(text), number >= 0 else { throw AppError.server("重量必须是非负数字") }
        switch unit {
        case .g: return Int(number.rounded())
        case .kg: return Int((number * 1000).rounded())
        case .lb: return Int((number * 453.59237).rounded())
        case .oz: return Int((number * 28.349523125).rounded())
        }
    }

    static func priceToMinorUnits(_ value: String, currency: GearCurrency) throws -> Int? {
        guard let text = value.nilIfBlank else { return nil }
        guard let number = Double(text), number >= 0 else { throw AppError.server("价格必须是非负数字") }
        if currency == .jpy { return Int(number.rounded()) }
        return Int((number * 100).rounded())
    }

    static func valueOrUnset(_ value: String?) -> String {
        value?.nilIfBlank ?? "未记录"
    }

    static func legacySpecs(from item: GearItem) -> GearSpecs {
        var specs: GearSpecs = [:]
        if let color = item.color?.nilIfBlank { specs["color"] = color }
        if let material = item.material?.nilIfBlank { specs["material"] = material }
        if let capacity = item.capacity?.nilIfBlank { specs["capacity"] = capacity }
        if let size = item.size?.nilIfBlank { specs["size"] = size }
        if let warmth = item.warmthIndex?.nilIfBlank { specs["warmth_rating"] = warmth }
        if let waterproof = item.waterproofIndex?.nilIfBlank { specs["waterproof_rating"] = waterproof }
        if let expiry = item.expiryOrWarrantyDate?.nilIfBlank { specs["expiry_date"] = expiry }
        return specs
    }

    private static func splitSpecValue(_ value: String, units: [String]) -> (valueText: String, unitIndex: Int) {
        let text = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !units.isEmpty else { return (text, 0) }
        let candidates = units.filter { !$0.isEmpty }.sorted { $0.count > $1.count }
        guard let matched = candidates.first(where: { text == $0 || text.hasSuffix(" \($0)") }),
              let index = units.firstIndex(of: matched)
        else { return (text, 0) }
        return (text == matched ? "" : String(text.dropLast(matched.count)).trimmingCharacters(in: .whitespacesAndNewlines), index)
    }

    private static func spec(
        _ key: String,
        _ label: String,
        _ placeholder: String,
        _ inputType: String = "text",
        _ units: [String] = [],
        unitLabels: [String]? = nil,
        choiceOnly: Bool = false
    ) -> GearSpecField {
        GearSpecField(key: key, label: label, placeholder: placeholder, inputType: inputType, units: units, unitLabels: unitLabels ?? units, choiceOnly: choiceOnly)
    }

    private static let capacityUnits = ["L", "ml", "fl oz"]
    private static let consumableContentUnits = ["g", "ml", "kg", "L", "oz"]
    private static let lengthUnits = ["cm", "m", "mm", "in"]
    private static let backLengthUnits = ["cm", "in"]
    private static let backpackSizeUnits = ["", "XS", "S", "M", "L", "XL", "XXL", "均码"]
    private static let backpackSizeLabels = ["选择尺码", "XS", "S", "M", "L", "XL", "XXL", "均码"]
    private static let shoeSizeOrLengthUnits = ["cm", "EU", "US", "UK", "in"]
    private static let loadUnits = ["kg", "g", "lb"]
    private static let temperatureUnits = ["℃", "℉"]
    private static let timeUnits = ["h", "min"]
    private static let distanceUnits = ["m", "km"]
    private static let commonWaterproofUnits = ["", "IPX4", "IPX5", "IPX6", "IPX7", "IPX8", "mm"]

    private static let allSpecFields: [GearCategory: [GearSpecField]] = [
        .backpackSystem: [
            spec("capacity", "容量", "例如 45", "number", capacityUnits),
            spec("recommended_load", "推荐负重", "例如 12", "number", loadUnits),
            spec("back_length", "背长", "例如 48", "number", backLengthUnits),
            spec("backpack_size", "尺码", "选择 XS / S / M / L", "text", backpackSizeUnits, unitLabels: backpackSizeLabels, choiceOnly: true),
            spec("waterproof_rating", "防水等级", "例如 防泼水", "text", commonWaterproofUnits)
        ],
        .sleepSystem: [
            spec("type", "类型", "例如 睡袋 / 帐篷"),
            spec("people_count", "适用人数", "例如 2", "number", ["人"]),
            spec("temperature_or_r_value", "温标/R 值", "例如 -5 或 4.2", "text", ["℃", "℉", "R"]),
            spec("filling", "填充物", "例如 800FP 羽绒"),
            spec("packed_size", "收纳尺寸", "例如 18 x 30", "text", lengthUnits),
            spec("waterproof_rating", "防水等级", "例如 外帐 3000", "text", commonWaterproofUnits)
        ],
        .kitchenSystem: [
            spec("fuel_type", "燃料类型", "例如 气罐"),
            spec("capacity", "容量", "例如 1.2", "number", capacityUnits),
            spec("power", "功率", "例如 2600", "number", ["W"]),
            spec("people_count", "适用人数", "例如 2", "number", ["人"]),
            spec("packed_size", "收纳尺寸", "例如 12 x 8", "text", lengthUnits)
        ],
        .walkingSystem: [
            spec("size_or_length", "尺码/长度", "例如 42 或 120", "text", shoeSizeOrLengthUnits),
            spec("terrain", "适用地形", "例如 山地 / 泥地"),
            spec("waterproof_rating", "防水等级", "例如 GTX", "text", commonWaterproofUnits),
            spec("material", "材质", "例如 铝合金"),
            spec("support", "缓震/支撑", "例如 中等支撑")
        ],
        .clothingSystem: [
            spec("size", "尺码", "例如 M"),
            spec("layer", "适用层级", "例如 中间层"),
            spec("warmth_rating", "保暖等级", "例如 200", "text", ["g/m²"]),
            spec("waterproof_rating", "防水指数", "例如 20000", "number", ["mm"]),
            spec("breathability_rating", "透湿指数", "例如 15000", "number", ["g/m²/24h"]),
            spec("season", "适用季节", "例如 三季")
        ],
        .lightingSystem: [
            spec("max_brightness", "最大亮度", "例如 450", "number", ["lm"]),
            spec("runtime", "续航时间", "例如 8", "number", timeUnits),
            spec("battery_type", "电池类型", "例如 18650"),
            spec("charging_port", "充电接口", "例如 USB-C"),
            spec("waterproof_rating", "防水等级", "例如 IPX4", "text", commonWaterproofUnits),
            spec("beam_distance", "照射距离", "例如 120", "number", distanceUnits)
        ],
        .firstAidSystem: [
            spec("kit_size", "套装规格", "例如 轻量 12 件"),
            spec("expiry_date", "有效期", "例如 2027-05"),
            spec("people_count", "适用人数", "例如 2", "number", ["人"]),
            spec("days", "适用天数", "例如 3", "number", ["天"]),
            spec("waterproof_packaging", "防水包装", "例如 自封袋")
        ],
        .electronicsSystem: [
            spec("battery_capacity", "电池容量", "例如 20000", "number", ["mAh", "Wh"]),
            spec("rated_energy", "额定能量", "例如 74", "number", ["Wh"]),
            spec("output_power", "输出功率", "例如 65", "number", ["W"]),
            spec("ports", "接口类型", "例如 USB-C x2"),
            spec("waterproof_rating", "防水等级", "例如 IPX4", "text", commonWaterproofUnits),
            spec("working_temperature", "工作温度", "例如 -10 - 45", "text", temperatureUnits)
        ],
        .technicalGear: [
            spec("certification", "认证标准", "例如 CE / UIAA"),
            spec("strength", "承重/强度", "例如 22", "number", ["kN", "kg"]),
            spec("specification", "规格", "例如 HMS"),
            spec("length", "长度", "例如 60", "number", lengthUnits),
            spec("material", "材质", "例如 尼龙"),
            spec("retirement_date", "报废期限", "例如 2030-05")
        ],
        .otherGear: [
            spec("use_case", "用途", "例如 营地收纳"),
            spec("specification", "规格", "例如 大号"),
            spec("capacity", "容量", "例如 10", "number", capacityUnits),
            spec("waterproof_rating", "防水等级", "例如 防泼水", "text", commonWaterproofUnits),
            spec("accessories", "附件", "例如 收纳袋")
        ],
        .consumable: [
            spec("type", "类型", "例如 气罐 / 食品"),
            spec("net_content", "净含量", "例如 230", "number", consumableContentUnits),
            spec("quantity", "数量", "例如 2", "number", ["件", "个", "包"]),
            spec("expiry_date", "有效期", "例如 2027-05"),
            spec("storage_condition", "储存条件", "例如 阴凉干燥"),
            spec("restock_threshold", "补货阈值", "例如 1", "number", ["件", "个", "包"])
        ]
    ]
}

extension GearItem {
    func summary() -> GearSummary {
        GearSummary(
            id: id,
            category: category,
            categoryLabel: categoryLabel,
            name: name,
            brand: brand,
            model: model,
            status: status,
            statusLabel: statusLabel,
            weightG: weightG,
            officialPriceCents: officialPriceCents,
            officialPriceCurrency: officialPriceCurrency,
            purchasePriceCents: purchasePriceCents,
            purchasePriceCurrency: purchasePriceCurrency,
            purchaseDate: purchaseDate,
            specs: specs,
            tags: tags,
            tagColors: tagColors,
            isDeleted: isDeleted,
            createdAt: createdAt,
            updatedAt: updatedAt
        )
    }
}
