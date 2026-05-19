import Foundation

@MainActor
final class FixtureRepository: AuthRepositorying, GearRepositorying, GearAtlasRepositorying, SkillRepositorying, ContentRepositorying {
    private var gearItems: [GearItem] = FixtureData.gearItems
    private var submissions: [GearAtlasSubmission] = FixtureData.atlasSubmissions

    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse {
        EmailVerificationCodeResponse(email: email, expiresAt: "2026-05-16T10:00:00Z", debugCode: "123456")
    }

    func sendEmailLoginCode(email: String) async throws -> EmailVerificationCodeResponse {
        EmailVerificationCodeResponse(email: email, expiresAt: "2026-05-16T10:00:00Z", debugCode: "654321")
    }

    func sendPasswordResetCode(email: String) async throws -> EmailVerificationCodeResponse {
        EmailVerificationCodeResponse(email: email, expiresAt: "2026-05-16T10:00:00Z", debugCode: "888888")
    }

    func register(_ request: RegisterRequest) async throws -> LoginResponse { .fixture }

    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse { .fixture }

    func loginWithEmailCode(email: String, code: String) async throws -> LoginResponse { .fixture }

    func resetPassword(_ request: PasswordResetRequest) async throws -> LoginResponse { .fixture }

    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse { .fixture }

    func captcha(account: String) async throws -> CaptchaChallengeResponse {
        CaptchaChallengeResponse(captchaTicket: "fixture-ticket", captchaType: "image", imageSvg: "<svg></svg>", expiresAt: "2026-05-16T10:05:00Z", debugAnswer: "A7K2")
    }

    func currentUser() async throws -> UserProfile { Session.fixture.user }

    func sendBindEmailCode(email: String) async throws -> EmailVerificationCodeResponse {
        EmailVerificationCodeResponse(email: email, expiresAt: "2026-05-16T10:00:00Z", debugCode: "135790")
    }

    func bindEmail(email: String, code: String) async throws -> UserProfile {
        UserProfile(id: Session.fixture.user.id, username: Session.fixture.user.username, email: email, nickname: Session.fixture.user.nickname, avatarUrl: Session.fixture.user.avatarUrl)
    }

    func uploadAvatar(data: Data, fileName: String, mimeType: String) async throws -> UserProfile {
        UserProfile(id: Session.fixture.user.id, username: Session.fixture.user.username, email: Session.fixture.user.email, nickname: Session.fixture.user.nickname, avatarUrl: "https://cdn.example.invalid/avatar/\(fileName)")
    }

    func stats(tab: GearTab) async throws -> GearStatsResponse {
        let filtered = gearItems.filter { tab == .history ? $0.isArchived : !$0.isArchived }
        let archived = gearItems.filter(\.isArchived)
        let categories = Dictionary(grouping: filtered, by: \.category).map { category, items in
            GearCategoryCount(category: category, label: category.label, count: items.count)
        }.sorted { $0.label < $1.label }
        let statuses = Dictionary(grouping: filtered, by: \.status).map { status, items in
            GearStatusCount(status: status, label: status.label, count: items.count)
        }.sorted { $0.label < $1.label }
        return GearStatsResponse(
            currentCount: gearItems.filter { !$0.isArchived }.count,
            archivedCount: archived.count,
            totalValueCents: filtered.compactMap(\.purchasePriceCents).reduce(0, +),
            totalWeightG: filtered.compactMap(\.weightG).reduce(0, +),
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

    func specKeyRankings(category: GearCategory) async throws -> GearSpecKeyRankingsResponse {
        var counts: [String: Int] = [:]
        for item in gearItems where item.category == category {
            for key in (item.specs ?? [:]).keys {
                counts[key, default: 0] += 1
            }
        }
        let ranked = counts
            .sorted { left, right in
                left.value == right.value ? left.key < right.key : left.value > right.value
            }
            .map(\.key)
        return GearSpecKeyRankingsResponse(keys: ranked)
    }

    func tagSuggestions(limit: Int) async throws -> GearTagSuggestionsResponse {
        let suggestions = gearItems
            .flatMap { item in item.tagViews.map { GearTagSuggestion(tag: $0.name, color: $0.color.rawValue) } }
            .reduce(into: [String: GearTagSuggestion]()) { partialResult, suggestion in
                partialResult[suggestion.tag] = suggestion
            }
            .values
            .sorted { $0.tag < $1.tag }
            .prefix(limit)
        return GearTagSuggestionsResponse(items: Array(suggestions))
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
                [item.name, item.brand, item.model, item.description, item.tags.joined(separator: " ")]
                    .compactMap { $0 }
                    .joined(separator: " ")
                    .localizedCaseInsensitiveContains(q)
            }
        }
        switch request.sort {
        case .nameAsc:
            filtered.sort { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
        case .weightDesc:
            filtered.sort { ($0.weightG ?? 0) > ($1.weightG ?? 0) }
        case .priceDesc:
            filtered.sort { ($0.purchasePriceCents ?? 0) > ($1.purchasePriceCents ?? 0) }
        case .createdAtAsc:
            filtered.sort { $0.createdAt < $1.createdAt }
        case .purchaseDateDesc:
            filtered.sort { ($0.purchaseDate ?? "") > ($1.purchaseDate ?? "") }
        case .createdAtDesc:
            filtered.sort { $0.createdAt > $1.createdAt }
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
        gearItems[index] = FixtureData.copy(gearItems[index], archivedAt: "2026-05-17T09:00:00Z")
    }

    func restore(id: String) async throws -> GearItem {
        guard let index = gearItems.firstIndex(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let item = FixtureData.copy(gearItems[index], archivedAt: nil)
        gearItems[index] = item
        return item
    }

    func list(_ request: ListGearAtlasRequest) async throws -> ListGearAtlasResponse {
        var items = FixtureData.atlasItems
        if let category = request.category {
            items = items.filter { $0.category == category }
        }
        if let q = request.q?.nilIfBlank {
            items = items.filter { [$0.name, $0.brand, $0.model, $0.description].compactMap { $0 }.joined(separator: " ").localizedCaseInsensitiveContains(q) }
        }
        return ListGearAtlasResponse(items: items, nextCursor: nil)
    }

    func get(id: String) async throws -> GearAtlasPublicItem {
        guard let item = FixtureData.atlasItems.first(where: { $0.id == id }) else { throw AppError.server("没有找到这条图鉴") }
        return item
    }

    func createSubmission(_ request: CreateGearAtlasSubmissionRequest) async throws -> GearAtlasSubmission {
        let item = FixtureData.atlasSubmission(from: request, id: "atlas-submission-\(submissions.count + 1)", sourceGearID: nil)
        submissions.insert(item, at: 0)
        return item
    }

    func submitGear(id: String) async throws -> GearAtlasSubmission {
        guard let gear = gearItems.first(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let request = CreateGearAtlasSubmissionRequest(
            category: gear.category,
            name: gear.name,
            brand: gear.brand,
            model: gear.model,
            description: gear.description,
            weightG: gear.weightG,
            officialPriceCents: gear.officialPriceCents,
            officialPriceCurrency: gear.officialPriceCurrency,
            specs: gear.specs
        )
        let item = FixtureData.atlasSubmission(from: request, id: "atlas-submission-\(submissions.count + 1)", sourceGearID: id)
        submissions.insert(item, at: 0)
        return item
    }

    func mySubmissions(_ request: ListGearAtlasSubmissionsRequest) async throws -> ListGearAtlasSubmissionsResponse {
        var items = submissions
        if let status = request.status { items = items.filter { $0.status == status } }
        if let category = request.category { items = items.filter { $0.category == category } }
        if let q = request.q?.nilIfBlank {
            items = items.filter { [$0.name, $0.brand, $0.model, $0.description].compactMap { $0 }.joined(separator: " ").localizedCaseInsensitiveContains(q) }
        }
        return ListGearAtlasSubmissionsResponse(items: items, nextCursor: nil)
    }

    func categories() async throws -> SkillCategoriesResponse {
        SkillCategoriesResponse(items: FixtureData.skillCategories)
    }

    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse {
        var all = FixtureData.knots
        if let category = request.category {
            all = all.filter { $0.categories.contains { $0.id == category || $0.slug == category } }
        }
        if let q = request.q?.nilIfBlank {
            all = all.filter { "\($0.title) \($0.summary)".localizedCaseInsensitiveContains(q) }
        }
        let slice = Array(all.dropFirst(request.offset).prefix(request.limit))
        let next = request.offset + slice.count < all.count ? request.offset + slice.count : nil
        return KnotListResponse(locale: "zh-CN", items: slice, page: PageInfo(limit: request.limit, offset: request.offset, nextOffset: next))
    }

    func knotDetail(id: String) async throws -> KnotDetail {
        guard let detail = FixtureData.knotDetails[id] else { throw AppError.server("没有找到这个绳结") }
        return detail
    }

    func offlineManifest() async throws -> KnotOfflineManifestResponse {
        KnotOfflineManifestResponse(locale: "zh-CN", itemCount: FixtureData.knotDetails.count, mediaCount: FixtureData.mediaAssets.count * FixtureData.knotDetails.count, estimatedBytes: 120_000, items: Array(FixtureData.knotDetails.values))
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
        GearItem(id: "gear-1", userId: "user-fixture", category: .backpackSystem, name: "轻量背包", brand: "山野", model: "45L", color: "松石绿", material: "尼龙", capacity: "45L", size: "M", description: "周末和一晚露营都能用。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", warmthIndex: nil, waterproofIndex: "防泼水", purchaseDate: "2026-05-01", purchasePriceCents: 89900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: "2028-05-01", purchaseLocation: "品牌官网", status: .available, storageLocation: "装备柜 A", specs: ["capacity": "45 L", "recommended_load": "12 kg", "back_length": "48 cm", "backpack_size": "M", "waterproof_rating": "防泼水"], tags: ["轻量", "三季"], tagColors: ["轻量": "teal", "三季": "green"], shareEnabled: true, shareStatus: .approved, notes: "常用背包，肩带已调好。", archivedAt: nil, createdAt: "2026-05-01T10:00:00Z", updatedAt: "2026-05-02T10:00:00Z"),
        GearItem(id: "gear-2", userId: "user-fixture", category: .lightingSystem, name: "头灯", brand: "星火", model: "HL200", color: "黑色", material: nil, capacity: nil, size: nil, description: "备用电池放在顶包。", weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", warmthIndex: nil, waterproofIndex: "IPX4", purchaseDate: "2026-04-12", purchasePriceCents: 15900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: nil, purchaseLocation: "京东", status: .inUse, storageLocation: "顶包", specs: ["max_brightness": "450 lm", "runtime": "8 h", "battery_type": "AAA", "waterproof_rating": "IPX4"], tags: ["夜行", "备用"], tagColors: ["夜行": "blue", "备用": "amber"], shareEnabled: false, shareStatus: .notShared, notes: nil, archivedAt: nil, createdAt: "2026-04-12T10:00:00Z", updatedAt: "2026-05-01T08:00:00Z"),
        GearItem(id: "gear-archived", userId: "user-fixture", category: .sleepSystem, name: "旧睡袋", brand: "北山", model: "Warm 400", color: "蓝色", material: "羽绒", capacity: nil, size: "L", description: "已换新，保留记录。", weightG: 760, officialPriceCents: 89900, officialPriceCurrency: "CNY", warmthIndex: "5℃", waterproofIndex: nil, purchaseDate: "2023-02-01", purchasePriceCents: 69900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: nil, purchaseLocation: "线下户外店", status: .retired, storageLocation: "储藏箱", specs: ["type": "睡袋", "temperature_or_r_value": "5 ℃", "filling": "羽绒"], tags: ["历史"], tagColors: ["历史": "slate"], shareEnabled: false, shareStatus: .notShared, notes: "拉链磨损。", archivedAt: "2026-01-01T10:00:00Z", createdAt: "2023-02-01T10:00:00Z", updatedAt: "2026-01-01T10:00:00Z")
    ]

    static let atlasItems: [GearAtlasPublicItem] = [
        GearAtlasPublicItem(id: "atlas-1", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, name: "山野 45L 轻量背包", brand: "山野", model: "45L", description: "适合周末和一晚轻量露营的背负系统。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", specs: ["capacity": "45 L", "recommended_load": "12 kg", "back_length": "48 cm"], approvedAt: "2026-05-03T10:00:00Z", createdAt: "2026-05-01T10:00:00Z", updatedAt: "2026-05-03T10:00:00Z"),
        GearAtlasPublicItem(id: "atlas-2", category: .lightingSystem, categoryLabel: GearCategory.lightingSystem.label, name: "星火 HL200 头灯", brand: "星火", model: "HL200", description: "轻量备用头灯，夜行和营地都够用。", weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", specs: ["max_brightness": "450 lm", "runtime": "8 h", "waterproof_rating": "IPX4"], approvedAt: "2026-04-18T10:00:00Z", createdAt: "2026-04-15T10:00:00Z", updatedAt: "2026-04-18T10:00:00Z")
    ]

    static let atlasSubmissions: [GearAtlasSubmission] = [
        GearAtlasSubmission(id: "submission-1", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, name: "轻量背包", brand: "山野", model: "45L", description: "用户装备公开字段投稿。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", specs: ["capacity": "45 L", "recommended_load": "12 kg"], approvedAt: nil, createdAt: "2026-05-02T10:00:00Z", updatedAt: "2026-05-02T10:00:00Z", sourceType: .userGear, sourceUserGearId: "gear-1", status: .pending, rejectionReason: nil, reviewedAt: nil)
    ]

    static func item(from request: CreateGearRequest, id: String, archivedAt: String? = nil) -> GearItem {
        GearItem(
            id: id,
            userId: Session.fixture.user.id,
            category: request.category,
            name: request.name.nilIfBlank ?? "未命名装备",
            brand: request.brand?.nilIfBlank,
            model: request.model?.nilIfBlank,
            color: nil,
            material: nil,
            capacity: nil,
            size: nil,
            description: request.description?.nilIfBlank,
            weightG: request.weightG,
            officialPriceCents: request.officialPriceCents,
            officialPriceCurrency: request.officialPriceCurrency,
            warmthIndex: nil,
            waterproofIndex: request.specs?["waterproof_rating"],
            purchaseDate: request.purchaseDate?.nilIfBlank,
            purchasePriceCents: request.purchasePriceCents,
            purchasePriceCurrency: request.purchasePriceCurrency,
            expiryOrWarrantyDate: request.specs?["expiry_date"] ?? request.specs?["retirement_date"],
            purchaseLocation: request.purchaseLocation?.nilIfBlank,
            status: request.status ?? .available,
            storageLocation: request.storageLocation?.nilIfBlank,
            specs: request.specs,
            tags: request.tags ?? [],
            tagColors: request.tagColors,
            shareEnabled: request.shareEnabled ?? false,
            shareStatus: request.shareEnabled == true ? .pending : .notShared,
            notes: request.notes?.nilIfBlank,
            archivedAt: archivedAt,
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
    }

    static func copy(_ item: GearItem, archivedAt: String?) -> GearItem {
        GearItem(id: item.id, userId: item.userId, category: item.category, name: item.name, brand: item.brand, model: item.model, color: item.color, material: item.material, capacity: item.capacity, size: item.size, description: item.description, weightG: item.weightG, officialPriceCents: item.officialPriceCents, officialPriceCurrency: item.officialPriceCurrency, warmthIndex: item.warmthIndex, waterproofIndex: item.waterproofIndex, purchaseDate: item.purchaseDate, purchasePriceCents: item.purchasePriceCents, purchasePriceCurrency: item.purchasePriceCurrency, expiryOrWarrantyDate: item.expiryOrWarrantyDate, purchaseLocation: item.purchaseLocation, status: item.status, storageLocation: item.storageLocation, specs: item.specs, tags: item.tags, tagColors: item.tagColors, shareEnabled: item.shareEnabled, shareStatus: item.shareStatus, notes: item.notes, archivedAt: archivedAt, createdAt: item.createdAt, updatedAt: "2026-05-17T09:00:00Z")
    }

    static func atlasSubmission(from request: CreateGearAtlasSubmissionRequest, id: String, sourceGearID: String?) -> GearAtlasSubmission {
        GearAtlasSubmission(
            id: id,
            category: request.category,
            categoryLabel: request.category.label,
            name: request.name,
            brand: request.brand,
            model: request.model,
            description: request.description,
            weightG: request.weightG,
            officialPriceCents: request.officialPriceCents,
            officialPriceCurrency: request.officialPriceCurrency,
            specs: request.specs,
            approvedAt: nil,
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z",
            sourceType: sourceGearID == nil ? .manual : .userGear,
            sourceUserGearId: sourceGearID,
            status: .pending,
            rejectionReason: nil,
            reviewedAt: nil
        )
    }
}
