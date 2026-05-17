import Foundation

@MainActor
final class FixtureRepository: AuthRepositorying, GearRepositorying, SkillRepositorying, ContentRepositorying {
    private var gearItems: [GearItem] = FixtureData.gearItems

    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse {
        EmailVerificationCodeResponse(email: email, expiresAt: "2026-05-16T10:00:00Z", debugCode: "123456")
    }

    func register(_ request: RegisterRequest) async throws -> LoginResponse { .fixture }

    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse { .fixture }

    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse { .fixture }

    func captcha(account: String) async throws -> CaptchaChallengeResponse {
        CaptchaChallengeResponse(captchaTicket: "fixture-ticket", captchaType: "image", imageSvg: "<svg></svg>", expiresAt: "2026-05-16T10:05:00Z", debugAnswer: "A7K2")
    }

    func stats() async throws -> GearStatsResponse {
        let available = gearItems.filter { !$0.isArchived }
        let archived = gearItems.filter { $0.isArchived }
        let categories = Dictionary(grouping: available, by: \.category).map { category, items in
            GearCategoryCount(category: category, label: category.label, count: items.count)
        }.sorted { $0.label < $1.label }
        let statuses = Dictionary(grouping: available, by: \.status).map { status, items in
            GearStatusCount(status: status, label: status.label, count: items.count)
        }.sorted { $0.label < $1.label }
        return GearStatsResponse(
            currentCount: available.count,
            archivedCount: archived.count,
            totalValueCents: available.compactMap(\.purchasePriceCents).reduce(0, +),
            totalWeightG: available.compactMap(\.weightG).reduce(0, +),
            byCategory: categories,
            byStatus: statuses
        )
    }

    func categories(tab: GearTab) async throws -> GearCategoriesResponse {
        let filtered = gearItems.filter { tab == .history ? $0.isArchived : !$0.isArchived }
        var items = [GearCategoryFilter(id: "all", label: "全部装备", count: filtered.count)]
        items += Dictionary(grouping: filtered, by: \.category).map { category, values in
            GearCategoryFilter(id: category.rawValue, label: category.label, count: values.count)
        }.sorted { $0.label < $1.label }
        return GearCategoriesResponse(items: items)
    }

    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse {
        var filtered = gearItems.filter { request.tab == .history ? $0.isArchived : !$0.isArchived }
        if let category = request.category {
            filtered = filtered.filter { $0.category == category }
        }
        if let status = request.status {
            filtered = filtered.filter { $0.status == status }
        }
        if let q = request.q?.nilIfBlank {
            filtered = filtered.filter { item in
                [item.name, item.brand, item.model, item.description].compactMap { $0 }.joined(separator: " ").localizedCaseInsensitiveContains(q)
            }
        }
        return ListGearsResponse(items: filtered.map { $0.summary() }, nextCursor: nil)
    }

    func get(id: String) async throws -> GearItem {
        guard let item = gearItems.first(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        return item
    }

    func create(_ request: CreateGearRequest) async throws -> GearItem {
        let item = FixtureData.item(from: request, id: "gear-\(gearItems.count + 1)")
        gearItems.insert(item, at: 0)
        return item
    }

    func update(id: String, request: UpdateGearRequest) async throws -> GearItem {
        guard let index = gearItems.firstIndex(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let item = FixtureData.item(from: request, id: id, archivedAt: gearItems[index].archivedAt)
        gearItems[index] = item
        return item
    }

    func archive(id: String) async throws {
        guard let index = gearItems.firstIndex(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let current = gearItems[index]
        gearItems[index] = FixtureData.copy(current, archivedAt: "2026-05-17T09:00:00Z")
    }

    func restore(id: String) async throws -> GearItem {
        guard let index = gearItems.firstIndex(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let item = FixtureData.copy(gearItems[index], archivedAt: nil)
        gearItems[index] = item
        return item
    }

    func categories() async throws -> SkillCategoriesResponse {
        SkillCategoriesResponse(items: FixtureData.skillCategories)
    }

    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse {
        let all = FixtureData.knots
        let slice = Array(all.dropFirst(request.offset).prefix(request.limit))
        let next = request.offset + slice.count < all.count ? request.offset + slice.count : nil
        return KnotListResponse(locale: "zh-CN", items: slice, page: PageInfo(limit: request.limit, offset: request.offset, nextOffset: next))
    }

    func knotDetail(id: String) async throws -> KnotDetail {
        guard let detail = FixtureData.knotDetails[id] else { throw AppError.server("没有找到这个绳结") }
        return detail
    }

    func gearTemplates() async throws -> GearTemplatesResponse {
        GearTemplatesResponse(items: FixtureData.gearTemplates)
    }
}

enum FixtureData {
    static let gearTemplates: [GearTemplate] = [
        GearTemplate(
            id: "weekend-hiking",
            title: "周末轻徒步",
            categories: [
                GearTemplateCategory(id: "backpack", name: "背包与收纳", items: ["轻量背包", "防水袋", "登山杖"]),
                GearTemplateCategory(id: "safety", name: "安全与照明", items: ["头灯", "急救包", "保温毯"])
            ]
        ),
        GearTemplate(
            id: "overnight-camp",
            title: "一晚露营",
            categories: [
                GearTemplateCategory(id: "sleep", name: "睡眠", items: ["帐篷", "睡袋", "防潮垫"]),
                GearTemplateCategory(id: "cook", name: "餐饮", items: ["炉头", "气罐", "套锅"])
            ]
        )
    ]

    static let skillCategories: [SkillCategorySummary] = [
        SkillCategorySummary(id: "knots", slug: "knots", title: "绳结", summary: "常用绳结步骤与用途。", itemCount: 225, href: "/api/skills/knots/list"),
        SkillCategorySummary(id: "camp", slug: "camp", title: "营地", summary: "营地搭建与安全检查。", itemCount: 12, href: nil)
    ]

    static let mediaAssets: [KnotMediaAsset] = [
        KnotMediaAsset(id: "thumbnail", mediaType: "thumbnail", url: "https://cdn.example.invalid/knots/bowline.png", mimeType: "image/png", width: 600, height: 400, sizeBytes: 1024, attribution: "Knots 3D", licenseNote: "fixture"),
        KnotMediaAsset(id: "draw_gif", mediaType: "draw_gif", url: "https://cdn.example.invalid/knots/bowline.gif", mimeType: "image/gif", width: nil, height: nil, sizeBytes: 4096, attribution: "Knots 3D", licenseNote: "fixture")
    ]

    static let knots: [KnotSummary] = [
        KnotSummary(id: "bowline", slug: "bowline", title: "单套结", summary: "可靠固定绳圈，适合临时连接。", difficulty: "入门", categories: [KnotTaxonomyItem(id: "rescue", slug: "rescue", title: "救援")], types: [KnotTaxonomyItem(id: "loop", slug: "loop", title: "绳圈")], media: mediaAssets, href: "/api/skills/knots/detail/bowline"),
        KnotSummary(id: "clove-hitch", slug: "clove-hitch", title: "丁香结", summary: "快速固定在柱体上，方便调整。", difficulty: "入门", categories: [KnotTaxonomyItem(id: "camp", slug: "camp", title: "营地")], types: [], media: mediaAssets, href: "/api/skills/knots/detail/clove-hitch")
    ]

    static let knotDetails: [String: KnotDetail] = [
        "bowline": KnotDetail(id: "bowline", slug: "bowline", title: "单套结", summary: "可靠固定绳圈，适合临时连接。", difficulty: "入门", categories: [KnotTaxonomyItem(id: "rescue", slug: "rescue", title: "救援")], types: [KnotTaxonomyItem(id: "loop", slug: "loop", title: "绳圈")], media: mediaAssets, href: "/api/skills/knots/detail/bowline", description: "单套结受力后仍相对容易解开，是户外常用基础绳结。", steps: ["在主绳上绕出一个小圈。", "将绳头从小圈下方穿出。", "绕过主绳后再回到小圈。", "整理绳股并逐步收紧。"], locale: "zh-CN"),
        "clove-hitch": KnotDetail(id: "clove-hitch", slug: "clove-hitch", title: "丁香结", summary: "快速固定在柱体上，方便调整。", difficulty: "入门", categories: [KnotTaxonomyItem(id: "camp", slug: "camp", title: "营地")], types: [], media: mediaAssets, href: "/api/skills/knots/detail/clove-hitch", description: "适合临时固定营绳或整理物品。", steps: ["绕柱体一圈。", "再交叉绕第二圈。", "将绳头压入交叉处并拉紧。"], locale: "zh-CN")
    ]

    static let gearItems: [GearItem] = [
        GearItem(id: "gear-1", userId: "user-fixture", category: .backpackSystem, name: "轻量背包", brand: "山野", model: "45L", color: "松石绿", material: "尼龙", capacity: "45L", size: "M", description: "周末和一晚露营都能用。", weightG: 980, warmthIndex: nil, waterproofIndex: "防泼水", purchaseDate: "2026-05-01", purchasePriceCents: 89900, expiryOrWarrantyDate: "2028-05-01", purchaseLocation: "杭州", status: .available, storageLocation: "装备柜 A", tags: ["轻量", "三季"], shareEnabled: true, shareStatus: .approved, notes: "常用背包，肩带已调好。", archivedAt: nil, createdAt: "2026-05-01T10:00:00Z", updatedAt: "2026-05-02T10:00:00Z"),
        GearItem(id: "gear-2", userId: "user-fixture", category: .lightingSystem, name: "头灯", brand: "星火", model: "HL200", color: "黑色", material: nil, capacity: nil, size: nil, description: "备用电池放在顶包。", weightG: 86, warmthIndex: nil, waterproofIndex: "IPX4", purchaseDate: "2026-04-12", purchasePriceCents: 15900, expiryOrWarrantyDate: nil, purchaseLocation: "上海", status: .inUse, storageLocation: "顶包", tags: ["夜行", "备用"], shareEnabled: false, shareStatus: .notShared, notes: nil, archivedAt: nil, createdAt: "2026-04-12T10:00:00Z", updatedAt: "2026-05-01T08:00:00Z"),
        GearItem(id: "gear-archived", userId: "user-fixture", category: .sleepSystem, name: "旧睡袋", brand: "北山", model: "Warm 400", color: "蓝色", material: "羽绒", capacity: nil, size: "L", description: "已换新，保留记录。", weightG: 760, warmthIndex: "5℃", waterproofIndex: nil, purchaseDate: "2023-02-01", purchasePriceCents: 69900, expiryOrWarrantyDate: nil, purchaseLocation: "南京", status: .retired, storageLocation: "储藏箱", tags: ["历史"], shareEnabled: false, shareStatus: .notShared, notes: "拉链磨损。", archivedAt: "2026-01-01T10:00:00Z", createdAt: "2023-02-01T10:00:00Z", updatedAt: "2026-01-01T10:00:00Z")
    ]

    static func item(from request: CreateGearRequest, id: String, archivedAt: String? = nil) -> GearItem {
        GearItem(
            id: id,
            userId: Session.fixture.user.id,
            category: request.category,
            name: request.name.nilIfBlank ?? "未命名装备",
            brand: request.brand?.nilIfBlank,
            model: request.model?.nilIfBlank,
            color: request.color?.nilIfBlank,
            material: request.material?.nilIfBlank,
            capacity: request.capacity?.nilIfBlank,
            size: request.size?.nilIfBlank,
            description: request.description?.nilIfBlank,
            weightG: request.weightG,
            warmthIndex: request.warmthIndex?.nilIfBlank,
            waterproofIndex: request.waterproofIndex?.nilIfBlank,
            purchaseDate: request.purchaseDate?.nilIfBlank,
            purchasePriceCents: request.purchasePriceCents,
            expiryOrWarrantyDate: request.expiryOrWarrantyDate?.nilIfBlank,
            purchaseLocation: request.purchaseLocation?.nilIfBlank,
            status: request.status ?? .available,
            storageLocation: request.storageLocation?.nilIfBlank,
            tags: request.tags ?? [],
            shareEnabled: request.shareEnabled ?? false,
            shareStatus: request.shareEnabled == true ? .pending : .notShared,
            notes: request.notes?.nilIfBlank,
            archivedAt: archivedAt,
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
    }

    static func copy(_ item: GearItem, archivedAt: String?) -> GearItem {
        GearItem(id: item.id, userId: item.userId, category: item.category, name: item.name, brand: item.brand, model: item.model, color: item.color, material: item.material, capacity: item.capacity, size: item.size, description: item.description, weightG: item.weightG, warmthIndex: item.warmthIndex, waterproofIndex: item.waterproofIndex, purchaseDate: item.purchaseDate, purchasePriceCents: item.purchasePriceCents, expiryOrWarrantyDate: item.expiryOrWarrantyDate, purchaseLocation: item.purchaseLocation, status: item.status, storageLocation: item.storageLocation, tags: item.tags, shareEnabled: item.shareEnabled, shareStatus: item.shareStatus, notes: item.notes, archivedAt: archivedAt, createdAt: item.createdAt, updatedAt: "2026-05-17T09:00:00Z")
    }
}
