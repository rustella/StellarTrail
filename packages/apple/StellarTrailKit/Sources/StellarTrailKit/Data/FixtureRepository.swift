import Foundation

@MainActor
final class FixtureRepository: AuthRepositorying, GearRepositorying, GearAtlasRepositorying, SkillRepositorying, ContentRepositorying, GearPackingRepositorying, TripRepositorying, ProfileRepositorying, RoadmapRepositorying, FeedbackRepositorying, ClientVersionRepositorying {
    private var gearItems: [GearItem] = FixtureData.gearItems
    private var submissions: [GearAtlasSubmission] = FixtureData.atlasSubmissions
    private var packingLists: [GearPackingListDetail] = FixtureData.packingLists
    private var tripDetails: [TripDetail] = FixtureData.tripDetails
    private var outdoorProfile: OutdoorProfile = FixtureData.outdoorProfile
    private var outdoorExperiences: [OutdoorExperience] = FixtureData.outdoorExperiences
    private var roadmapItems: [RoadmapItem] = FixtureData.roadmapItems
    private var acceptedKnotDisclaimer = true

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

    func delete(id: String) async throws {
        if let index = gearItems.firstIndex(where: { $0.id == id }) {
            gearItems[index] = FixtureData.copy(gearItems[index], archivedAt: gearItems[index].archivedAt, isDeleted: true)
            return
        }
        if packingLists.contains(where: { $0.id == id }) {
            packingLists.removeAll { $0.id == id }
            return
        }
        if tripDetails.contains(where: { $0.trip.id == id }) {
            tripDetails.removeAll { $0.trip.id == id }
            return
        }
        throw AppError.server("没有找到要删除的记录")
    }

    func undelete(id: String) async throws -> GearItem {
        guard let index = gearItems.firstIndex(where: { $0.id == id }) else { throw AppError.server("没有找到这件装备") }
        let item = FixtureData.copy(gearItems[index], archivedAt: gearItems[index].archivedAt, isDeleted: false)
        gearItems[index] = item
        return item
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

    func knotDisclaimer() async throws -> KnotDisclaimerResponse {
        KnotDisclaimerResponse(
            key: "knots-safety",
            version: "2026-05",
            title: "绳结安全免责声明",
            content: "绳结教学仅用于辅助学习。实际户外活动前请在安全环境中练习，并结合专业培训、装备状态和现场风险判断。",
            accepted: acceptedKnotDisclaimer,
            acceptedAt: acceptedKnotDisclaimer ? "2026-05-17T09:00:00Z" : nil
        )
    }

    func acceptKnotDisclaimer(_ request: AcceptKnotDisclaimerRequest) async throws -> KnotDisclaimerResponse {
        acceptedKnotDisclaimer = true
        return try await knotDisclaimer()
    }

    func gearTemplates() async throws -> GearTemplatesResponse {
        GearTemplatesResponse(items: FixtureData.gearTemplates)
    }

    func list(_ request: ListGearPackingListsRequest) async throws -> ListGearPackingListsResponse {
        let slice = Array(packingLists.prefix(request.limit))
        return ListGearPackingListsResponse(items: slice.map(Self.summary), nextCursor: nil)
    }

    func create(_ request: CreateGearPackingListRequest) async throws -> GearPackingListDetail {
        let detail = GearPackingListDetail(
            id: "packing-\(packingLists.count + 1)",
            name: request.name.nilIfBlank ?? "新的打包清单",
            routeName: request.routeName?.nilIfBlank,
            durationLabel: request.durationLabel?.nilIfBlank,
            stats: .empty,
            items: [],
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
        packingLists.insert(detail, at: 0)
        return detail
    }

    func get(id: String) async throws -> GearPackingListDetail {
        guard let detail = packingLists.first(where: { $0.id == id }) else {
            throw AppError.server("没有找到这份打包清单")
        }
        return detail
    }

    func update(id: String, request: UpdateGearPackingListRequest) async throws -> GearPackingListDetail {
        guard let index = packingLists.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到这份打包清单")
        }
        let current = packingLists[index]
        let updated = GearPackingListDetail(
            id: current.id,
            name: request.name.nilIfBlank ?? current.name,
            routeName: request.routeName?.nilIfBlank,
            durationLabel: request.durationLabel?.nilIfBlank,
            stats: current.stats,
            items: current.items,
            createdAt: current.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        packingLists[index] = updated
        return updated
    }

    func addItems(id: String, request: AddGearPackingItemsRequest) async throws -> GearPackingListDetail {
        guard let index = packingLists.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到这份打包清单")
        }
        var items = packingLists[index].items
        for gearId in request.gearIds where !items.contains(where: { $0.gearId == gearId }) {
            if let gear = gearItems.first(where: { $0.id == gearId }) {
                items.append(Self.packingItem(from: gear, number: items.count + 1))
            }
        }
        let updated = Self.copy(packingLists[index], items: items)
        packingLists[index] = updated
        return updated
    }

    func updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest) async throws -> GearPackingListDetail {
        guard let listIndex = packingLists.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到这份打包清单")
        }
        let current = packingLists[listIndex]
        let items = current.items.map { item in
            guard item.id == itemId else { return item }
            return GearPackingListItem(
                id: item.id,
                gearId: item.gearId,
                plannedQuantity: request.plannedQuantity ?? item.plannedQuantity,
                packedQuantity: request.packedQuantity ?? (request.packed == true ? item.plannedQuantity : item.packedQuantity),
                packed: request.packed ?? item.packed,
                unavailable: item.unavailable,
                unavailableReason: item.unavailableReason,
                gear: item.gear,
                createdAt: item.createdAt,
                updatedAt: "2026-05-17T09:30:00Z"
            )
        }
        let updated = Self.copy(current, items: items)
        packingLists[listIndex] = updated
        return updated
    }

    func deleteItem(id: String, itemId: String) async throws -> GearPackingListDetail {
        guard let listIndex = packingLists.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到这份打包清单")
        }
        let updated = Self.copy(packingLists[listIndex], items: packingLists[listIndex].items.filter { $0.id != itemId })
        packingLists[listIndex] = updated
        return updated
    }

    func list(_ request: ListTripsRequest) async throws -> ListTripsResponse {
        let today = request.today ?? Formatters.localDateString(Date())
        var summaries = tripDetails.map { FixtureData.tripSummary(from: $0, today: today) }
        if request.bucket != .all {
            summaries = summaries.filter { $0.timeBucket == request.bucket }
        }
        if let tripType = request.tripType {
            summaries = summaries.filter { $0.tripType == tripType }
        }
        return ListTripsResponse(items: Array(summaries.prefix(request.limit)), nextCursor: nil)
    }

    func homeHighlight(today: String) async throws -> TripHomeHighlightResponse {
        let summaries = tripDetails.map { FixtureData.tripSummary(from: $0, today: today) }
        if let ongoing = summaries.first(where: { $0.timeBucket == .ongoing }) {
            return TripHomeHighlightResponse(item: TripHomeHighlightItem(trip: ongoing, status: .ongoing, daysUntilStart: ongoing.daysUntilStart ?? 0, daysUntilEnd: ongoing.daysUntilEnd ?? 0))
        }
        if let upcoming = summaries.first(where: { $0.timeBucket == .upcoming }) {
            return TripHomeHighlightResponse(item: TripHomeHighlightItem(trip: upcoming, status: .upcoming, daysUntilStart: upcoming.daysUntilStart ?? 0, daysUntilEnd: upcoming.daysUntilEnd ?? 0))
        }
        return TripHomeHighlightResponse(item: nil)
    }

    func create(_ request: CreateTripRequest) async throws -> TripDetail {
        let trip = Trip(
            id: "trip-\(tripDetails.count + 1)",
            ownerUserId: Session.fixture.user.id,
            tripType: request.tripType,
            title: request.title.nilIfBlank ?? "新的行程",
            description: request.description?.nilIfBlank,
            startDate: request.startDate?.nilIfBlank,
            endDate: request.endDate?.nilIfBlank,
            enabledSections: TripSectionKey.allowed(for: request.tripType),
            routeUseSlopeAdjustment: request.routeUseSlopeAdjustment ?? true,
            routeUseHighAltitudeAdjustment: request.routeUseHighAltitudeAdjustment ?? false,
            routeStartAltitudeM: request.routeStartAltitudeM,
            dayCount: 1,
            fieldVersions: [:],
            isDeleted: false,
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
        let detail = FixtureData.tripDetail(for: trip)
        tripDetails.insert(detail, at: 0)
        return detail
    }

    func get(id: String) async throws -> TripDetail {
        guard let detail = tripDetails.first(where: { $0.trip.id == id }) else {
            throw AppError.server("没有找到这条行程")
        }
        return detail
    }

    func update(id: String, request: UpdateTripRequest) async throws -> TripDetail {
        guard let index = tripDetails.firstIndex(where: { $0.trip.id == id }) else {
            throw AppError.server("没有找到这条行程")
        }
        let current = tripDetails[index].trip
        let trip = Trip(
            id: current.id,
            ownerUserId: current.ownerUserId,
            tripType: current.tripType,
            title: request.title?.nilIfBlank ?? current.title,
            description: request.description ?? current.description,
            startDate: request.startDate ?? current.startDate,
            endDate: request.endDate ?? current.endDate,
            enabledSections: current.enabledSections,
            routeUseSlopeAdjustment: request.routeUseSlopeAdjustment ?? current.routeUseSlopeAdjustment,
            routeUseHighAltitudeAdjustment: request.routeUseHighAltitudeAdjustment ?? current.routeUseHighAltitudeAdjustment,
            routeStartAltitudeM: request.routeStartAltitudeM ?? current.routeStartAltitudeM,
            dayCount: current.dayCount,
            fieldVersions: current.fieldVersions,
            isDeleted: current.isDeleted,
            createdAt: current.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        let updated = Self.copy(tripDetails[index], trip: trip, sections: trip.enabledSections)
        tripDetails[index] = updated
        return updated
    }

    func updateSections(id: String, request: UpdateTripSectionsRequest) async throws -> TripDetail {
        guard let index = tripDetails.firstIndex(where: { $0.trip.id == id }) else {
            throw AppError.server("没有找到这条行程")
        }
        let current = tripDetails[index].trip
        let allowed = Set(TripSectionKey.allowed(for: current.tripType))
        let sections = request.enabledSections.filter { allowed.contains($0) }
        let trip = Trip(
            id: current.id,
            ownerUserId: current.ownerUserId,
            tripType: current.tripType,
            title: current.title,
            description: current.description,
            startDate: current.startDate,
            endDate: current.endDate,
            enabledSections: sections,
            routeUseSlopeAdjustment: current.routeUseSlopeAdjustment,
            routeUseHighAltitudeAdjustment: current.routeUseHighAltitudeAdjustment,
            routeStartAltitudeM: current.routeStartAltitudeM,
            dayCount: current.dayCount,
            fieldVersions: current.fieldVersions,
            isDeleted: current.isDeleted,
            createdAt: current.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        let updated = Self.copy(tripDetails[index], trip: trip, sections: sections)
        tripDetails[index] = updated
        return updated
    }

    func createInvitation(id: String) async throws -> CreateTripInvitationResponse {
        CreateTripInvitationResponse(invitation: TripInvitation(id: "invite-\(id)", planId: id, token: "fixture-token-\(id)", createdByUserId: Session.fixture.user.id, revokedAt: nil, createdAt: "2026-05-17T09:00:00Z"))
    }

    func acceptInvitation(token: String) async throws -> TripDetail {
        guard let teamTrip = tripDetails.first(where: { $0.trip.tripType == .team }) else {
            throw AppError.server("邀请已失效")
        }
        return teamTrip
    }

    func importPackingList(id: String, request: ImportTripPackingListRequest) async throws -> TripDetail {
        guard let index = tripDetails.firstIndex(where: { $0.trip.id == id }) else {
            throw AppError.server("没有找到这条行程")
        }
        let list = packingLists.first { $0.id == request.packingListId }
        let imported = list?.items.map {
            TripPersonalGearItem(
                id: "trip-gear-\($0.id)",
                category: $0.gear.category,
                categoryLabel: $0.gear.categoryLabel,
                name: $0.gear.name,
                brand: $0.gear.brand,
                model: $0.gear.model,
                plannedQuantity: $0.plannedQuantity,
                packedQuantity: $0.packedQuantity,
                unitWeightG: $0.gear.weightG,
                notes: nil,
                fieldVersions: [:],
                createdAt: "2026-05-17T09:00:00Z",
                updatedAt: "2026-05-17T09:00:00Z",
                memberId: tripDetails[index].myMemberId,
                sourcePackingListId: request.packingListId,
                sourcePackingItemId: $0.id,
                sourceGearId: $0.gearId
            )
        } ?? []
        let updated = Self.copy(tripDetails[index], personalGear: imported)
        tripDetails[index] = updated
        return updated
    }

    func createRecord(id: String, collectionPath: String, request: TripRecordCreateRequest) async throws -> TripDetail {
        let detail: TripDetail = try await get(id: id)
        return detail
    }

    func updateRecord(id: String, collectionPath: String, recordId: String, request: TripRecordPatchRequest) async throws -> TripDetail {
        let detail: TripDetail = try await get(id: id)
        return detail
    }

    func deleteRecord(id: String, collectionPath: String, recordId: String) async throws -> TripDetail {
        let detail: TripDetail = try await get(id: id)
        return detail
    }

    func convertToOutdoorExperience(id: String, today: String) async throws -> OutdoorExperience {
        let detail: TripDetail = try await get(id: id)
        let experience = OutdoorExperience(
            id: "experience-\(outdoorExperiences.count + 1)",
            userId: Session.fixture.user.id,
            sourceTripId: id,
            tripType: detail.trip.tripType,
            title: detail.trip.title,
            startDate: detail.trip.startDate,
            endDate: detail.trip.endDate,
            dayCount: detail.trip.dayCount,
            companionCount: max(detail.members.count - 1, 0),
            routeSummary: detail.routeSegments.map { $0.name }.joined(separator: "、").nilIfBlank,
            gearSummary: detail.personalGear.map { $0.name }.joined(separator: "、").nilIfBlank,
            foodSummary: detail.foodMeals.map { $0.dishName ?? $0.mealKey }.joined(separator: "、").nilIfBlank,
            budgetSummary: nil,
            notes: "由行程转为户外经历。",
            createdAt: "2026-05-17T09:30:00Z",
            updatedAt: "2026-05-17T09:30:00Z"
        )
        outdoorExperiences.insert(experience, at: 0)
        return experience
    }

    func listOutdoorExperiences() async throws -> ListOutdoorExperiencesResponse {
        ListOutdoorExperiencesResponse(items: outdoorExperiences)
    }

    func createOutdoorExperience(_ request: OutdoorExperienceRequest) async throws -> OutdoorExperience {
        let experience = OutdoorExperience(
            id: "experience-\(outdoorExperiences.count + 1)",
            userId: Session.fixture.user.id,
            sourceTripId: nil,
            tripType: .team,
            title: request.title.nilIfBlank ?? "新的户外经历",
            startDate: request.startDate?.nilIfBlank,
            endDate: request.endDate?.nilIfBlank,
            dayCount: request.dayCount,
            companionCount: request.companionCount,
            routeSummary: request.routeSummary?.nilIfBlank,
            gearSummary: request.gearSummary?.nilIfBlank,
            foodSummary: request.foodSummary?.nilIfBlank,
            budgetSummary: request.budgetSummary?.nilIfBlank,
            notes: request.notes?.nilIfBlank,
            createdAt: "2026-05-17T09:30:00Z",
            updatedAt: "2026-05-17T09:30:00Z"
        )
        outdoorExperiences.insert(experience, at: 0)
        return experience
    }

    func updateOutdoorExperience(id: String, request: OutdoorExperienceRequest) async throws -> OutdoorExperience {
        guard let index = outdoorExperiences.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到这条户外经历")
        }
        let current = outdoorExperiences[index]
        let updated = OutdoorExperience(
            id: current.id,
            userId: current.userId,
            sourceTripId: current.sourceTripId,
            tripType: current.tripType,
            title: request.title.nilIfBlank ?? current.title,
            startDate: request.startDate,
            endDate: request.endDate,
            dayCount: request.dayCount,
            companionCount: request.companionCount,
            routeSummary: request.routeSummary,
            gearSummary: request.gearSummary,
            foodSummary: request.foodSummary,
            budgetSummary: request.budgetSummary,
            notes: request.notes,
            createdAt: current.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        outdoorExperiences[index] = updated
        return updated
    }

    func deleteOutdoorExperience(id: String) async throws {
        outdoorExperiences.removeAll { $0.id == id }
    }

    func outdoorProfile() async throws -> OutdoorProfileResponse {
        OutdoorProfileResponse(profile: outdoorProfile)
    }

    func updateOutdoorProfile(_ request: UpdateOutdoorProfileRequest) async throws -> OutdoorProfileResponse {
        outdoorProfile = OutdoorProfile(
            userId: outdoorProfile.userId,
            outdoorId: request.outdoorId ?? outdoorProfile.outdoorId,
            realName: request.realName ?? outdoorProfile.realName,
            gender: request.gender ?? outdoorProfile.gender,
            birthDate: request.birthDate ?? outdoorProfile.birthDate,
            heightCm: request.heightCm ?? outdoorProfile.heightCm,
            phone: request.phone ?? outdoorProfile.phone,
            emergencyContact: request.emergencyContact ?? outdoorProfile.emergencyContact,
            emergencyContactRelationship: request.emergencyContactRelationship ?? outdoorProfile.emergencyContactRelationship,
            emergencyPhone: request.emergencyPhone ?? outdoorProfile.emergencyPhone,
            bloodType: request.bloodType ?? outdoorProfile.bloodType,
            medicalHistory: request.medicalHistory ?? outdoorProfile.medicalHistory,
            allergyHistory: request.allergyHistory ?? outdoorProfile.allergyHistory,
            medicalResponseNote: request.medicalResponseNote ?? outdoorProfile.medicalResponseNote,
            dietPreference: request.dietPreference ?? outdoorProfile.dietPreference,
            insurancePolicyNo: request.insurancePolicyNo ?? outdoorProfile.insurancePolicyNo,
            insuranceCompanyPhone: request.insuranceCompanyPhone ?? outdoorProfile.insuranceCompanyPhone,
            experienceNote: request.experienceNote ?? outdoorProfile.experienceNote,
            createdAt: outdoorProfile.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        return OutdoorProfileResponse(profile: outdoorProfile)
    }

    func list(_ request: ListRoadmapRequest, includeUserState: Bool) async throws -> ListRoadmapResponse {
        let filtered = roadmapItems
            .filter { $0.clientKey == request.clientKey }
            .filter { request.status == nil || $0.status == request.status?.rawValue }
            .sorted { $0.sortOrder < $1.sortOrder }
        return ListRoadmapResponse(items: Array(filtered.prefix(request.limit)), nextCursor: nil)
    }

    func vote(id: String) async throws -> RoadmapItem {
        try updateRoadmap(id: id, voted: true)
    }

    func unvote(id: String) async throws -> RoadmapItem {
        try updateRoadmap(id: id, voted: false)
    }

    func subscribe(id: String) async throws -> RoadmapItem {
        try updateRoadmap(id: id, subscribed: true)
    }

    func unsubscribe(id: String) async throws -> RoadmapItem {
        try updateRoadmap(id: id, subscribed: false)
    }

    func uploadImage(data: Data, fileName: String, mimeType: String) async throws -> UploadImageInfo {
        UploadImageInfo(id: "upload-\(fileName)", purpose: "feedback", originalFilename: fileName, imageType: "feedback", contentType: mimeType, sizeBytes: data.count, sha256: "fixture", downloadUrl: "https://cdn.example.invalid/uploads/\(fileName)", isDeleted: false, createdAt: "2026-05-17T09:00:00Z")
    }

    func create(_ request: CreateFeedbackRequest) async throws -> FeedbackResponse {
        FeedbackResponse(id: "feedback-1", category: request.category, content: request.content, contact: request.contact, page: request.page, clientPlatform: request.clientPlatform, clientVersion: request.clientVersion, deviceModel: request.deviceModel, status: "open", images: [], isDeleted: false, createdAt: "2026-05-17T09:00:00Z", updatedAt: "2026-05-17T09:00:00Z")
    }

    func list(_ request: ListClientVersionsRequest) async throws -> ListClientVersionsResponse {
        let items = FixtureData.clientVersions.filter { $0.clientKey == request.clientKey }
        return ListClientVersionsResponse(items: Array(items.prefix(request.limit)), nextCursor: nil)
    }

    func current(clientKey: ClientKey) async throws -> ClientVersion {
        FixtureData.clientVersions.first { $0.clientKey == clientKey } ?? ClientVersion(id: "ios-empty", clientKey: clientKey, version: "0.1.0", title: "暂无 iOS 版本公告", releaseNotes: [], releaseNoteSections: [], status: .published, createdAt: "2026-05-17T09:00:00Z", updatedAt: "2026-05-17T09:00:00Z")
    }

    private static func summary(_ detail: GearPackingListDetail) -> GearPackingListSummary {
        GearPackingListSummary(
            id: detail.id,
            name: detail.name,
            routeName: detail.routeName,
            durationLabel: detail.durationLabel,
            itemCount: detail.stats.itemCount,
            packedCount: detail.stats.packedCount,
            totalWeightG: detail.stats.totalWeightG,
            createdAt: detail.createdAt,
            updatedAt: detail.updatedAt
        )
    }

    private static func packingItem(from gear: GearItem, number: Int) -> GearPackingListItem {
        GearPackingListItem(id: "packing-item-\(number)", gearId: gear.id, plannedQuantity: 1, packedQuantity: 0, packed: false, unavailable: gear.isArchived || gear.isDeleted, unavailableReason: gear.isArchived ? "archived" : (gear.isDeleted ? "deleted" : nil), gear: gear.summary(), createdAt: "2026-05-17T09:00:00Z", updatedAt: "2026-05-17T09:00:00Z")
    }

    private static func copy(_ detail: GearPackingListDetail, items: [GearPackingListItem]) -> GearPackingListDetail {
        let stats = GearPackingListStats(
            itemCount: items.reduce(0) { $0 + $1.plannedQuantity },
            packedCount: items.reduce(0) { $0 + $1.packedQuantity },
            totalWeightG: items.reduce(0) { $0 + (($1.gear.weightG ?? 0) * $1.plannedQuantity) }
        )
        return GearPackingListDetail(id: detail.id, name: detail.name, routeName: detail.routeName, durationLabel: detail.durationLabel, stats: stats, items: items, createdAt: detail.createdAt, updatedAt: "2026-05-17T09:30:00Z")
    }

    private static func copy(
        _ detail: TripDetail,
        trip: Trip? = nil,
        sections: [TripSectionKey]? = nil,
        personalGear: [TripPersonalGearItem]? = nil
    ) -> TripDetail {
        TripDetail(
            trip: trip ?? detail.trip,
            sections: sections ?? detail.sections,
            myMemberId: detail.myMemberId,
            members: detail.members,
            personalGear: personalGear ?? detail.personalGear,
            sharedGearDemands: detail.trip.tripType == .solo ? [] : detail.sharedGearDemands,
            itineraryDays: detail.itineraryDays,
            routeSegments: detail.routeSegments,
            foodMeals: detail.foodMeals,
            foodSupplies: detail.foodSupplies,
            medicalItems: detail.medicalItems,
            segmentAssignments: detail.segmentAssignments,
            safetyRisks: detail.safetyRisks,
            rescueContacts: detail.rescueContacts,
            budgetItems: detail.budgetItems,
            goals: detail.goals,
            weightSummaries: detail.weightSummaries,
            memberGearViews: detail.memberGearViews
        )
    }

    private func updateRoadmap(id: String, voted: Bool? = nil, subscribed: Bool? = nil) throws -> RoadmapItem {
        guard let index = roadmapItems.firstIndex(where: { $0.id == id }) else {
            throw AppError.server("没有找到路线图条目")
        }
        let current = roadmapItems[index]
        let next = RoadmapItem(
            id: current.id,
            clientKey: current.clientKey,
            title: current.title,
            summary: current.summary,
            details: current.details,
            category: current.category,
            status: current.status,
            priority: current.priority,
            sortOrder: current.sortOrder,
            isPublished: current.isPublished,
            voteCount: current.voteCount + ((voted == true && !current.isVoted) ? 1 : (voted == false && current.isVoted ? -1 : 0)),
            subscriptionCount: current.subscriptionCount + ((subscribed == true && !current.isSubscribed) ? 1 : (subscribed == false && current.isSubscribed ? -1 : 0)),
            isVoted: voted ?? current.isVoted,
            isSubscribed: subscribed ?? current.isSubscribed,
            publishedAt: current.publishedAt,
            createdAt: current.createdAt,
            updatedAt: "2026-05-17T09:30:00Z"
        )
        roadmapItems[index] = next
        return next
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
        SkillCategorySummary(id: "knots", slug: "knots", title: "绳结", summary: "常用绳结步骤与用途。", itemCount: 225, href: "/api/v1/skills/knots/list"),
        SkillCategorySummary(id: "camp", slug: "camp", title: "营地", summary: "营地搭建与安全检查。", itemCount: 12, href: nil)
    ]

    static let mediaAssets: [KnotMediaAsset] = [
        KnotMediaAsset(id: "thumbnail", mediaType: "thumbnail", url: "https://cdn.example.invalid/knots/bowline.png", mimeType: "image/png", width: 600, height: 400, sizeBytes: 1024, attribution: "Knots 3D", licenseNote: "fixture"),
        KnotMediaAsset(id: "draw_gif", mediaType: "draw_gif", url: "https://cdn.example.invalid/knots/bowline.gif", mimeType: "image/gif", width: nil, height: nil, sizeBytes: 4096, attribution: "Knots 3D", licenseNote: "fixture")
    ]

    static let knots: [KnotSummary] = [
        KnotSummary(id: "bowline", slug: "bowline", title: "单套结", summary: "可靠固定绳圈，适合临时连接。", categories: [KnotTaxonomyItem(id: "rescue", slug: "rescue", title: "救援")], types: [KnotTaxonomyItem(id: "loop", slug: "loop", title: "绳圈")], media: mediaAssets, href: "/api/v1/skills/knots/detail/bowline"),
        KnotSummary(id: "clove-hitch", slug: "clove-hitch", title: "丁香结", summary: "快速固定在柱体上，方便调整。", categories: [KnotTaxonomyItem(id: "camp", slug: "camp", title: "营地")], types: [], media: mediaAssets, href: "/api/v1/skills/knots/detail/clove-hitch")
    ]

    static let knotDetails: [String: KnotDetail] = [
        "bowline": KnotDetail(id: "bowline", slug: "bowline", title: "单套结", summary: "可靠固定绳圈，适合临时连接。", categories: [KnotTaxonomyItem(id: "rescue", slug: "rescue", title: "救援")], types: [KnotTaxonomyItem(id: "loop", slug: "loop", title: "绳圈")], media: mediaAssets, href: "/api/v1/skills/knots/detail/bowline", description: "单套结受力后仍相对容易解开，是户外常用基础绳结。", steps: ["在主绳上绕出一个小圈。", "将绳头从小圈下方穿出。", "绕过主绳后再回到小圈。", "整理绳股并逐步收紧。"], locale: "zh-CN"),
        "clove-hitch": KnotDetail(id: "clove-hitch", slug: "clove-hitch", title: "丁香结", summary: "快速固定在柱体上，方便调整。", categories: [KnotTaxonomyItem(id: "camp", slug: "camp", title: "营地")], types: [], media: mediaAssets, href: "/api/v1/skills/knots/detail/clove-hitch", description: "适合临时固定营绳或整理物品。", steps: ["绕柱体一圈。", "再交叉绕第二圈。", "将绳头压入交叉处并拉紧。"], locale: "zh-CN")
    ]

    static let gearItems: [GearItem] = [
        GearItem(id: "gear-1", userId: "user-fixture", category: .backpackSystem, name: "轻量背包", brand: "山野", model: "45L", color: "松石绿", material: "尼龙", capacity: "45L", size: "M", description: "周末和一晚露营都能用。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", warmthIndex: nil, waterproofIndex: "防泼水", purchaseDate: "2026-05-01", purchasePriceCents: 89900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: "2028-05-01", purchaseLocation: "品牌官网", status: .available, storageLocation: "装备柜 A", specs: ["capacity": "45 L", "recommended_load": "12 kg", "back_length": "48 cm", "backpack_size": "M", "waterproof_rating": "防泼水"], tags: ["轻量", "三季"], tagColors: ["轻量": "teal", "三季": "green"], shareEnabled: true, shareStatus: .approved, notes: "常用背包，肩带已调好。", archivedAt: nil, isDeleted: false, createdAt: "2026-05-01T10:00:00Z", updatedAt: "2026-05-02T10:00:00Z"),
        GearItem(id: "gear-2", userId: "user-fixture", category: .lightingSystem, name: "头灯", brand: "星火", model: "HL200", color: "黑色", material: nil, capacity: nil, size: nil, description: "备用电池放在顶包。", weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", warmthIndex: nil, waterproofIndex: "IPX4", purchaseDate: "2026-04-12", purchasePriceCents: 15900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: nil, purchaseLocation: "京东", status: .inUse, storageLocation: "顶包", specs: ["max_brightness": "450 lm", "runtime": "8 h", "battery_type": "AAA", "waterproof_rating": "IPX4"], tags: ["夜行", "备用"], tagColors: ["夜行": "blue", "备用": "amber"], shareEnabled: false, shareStatus: .notShared, notes: nil, archivedAt: nil, isDeleted: false, createdAt: "2026-04-12T10:00:00Z", updatedAt: "2026-05-01T08:00:00Z"),
        GearItem(id: "gear-archived", userId: "user-fixture", category: .sleepSystem, name: "旧睡袋", brand: "北山", model: "Warm 400", color: "蓝色", material: "羽绒", capacity: nil, size: "L", description: "已换新，保留记录。", weightG: 760, officialPriceCents: 89900, officialPriceCurrency: "CNY", warmthIndex: "5℃", waterproofIndex: nil, purchaseDate: "2023-02-01", purchasePriceCents: 69900, purchasePriceCurrency: "CNY", expiryOrWarrantyDate: nil, purchaseLocation: "线下户外店", status: .retired, storageLocation: "储藏箱", specs: ["type": "睡袋", "temperature_or_r_value": "5 ℃", "filling": "羽绒"], tags: ["历史"], tagColors: ["历史": "slate"], shareEnabled: false, shareStatus: .notShared, notes: "拉链磨损。", archivedAt: "2026-01-01T10:00:00Z", isDeleted: false, createdAt: "2023-02-01T10:00:00Z", updatedAt: "2026-01-01T10:00:00Z")
    ]

    static let atlasItems: [GearAtlasPublicItem] = [
        GearAtlasPublicItem(id: "atlas-1", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, name: "山野 45L 轻量背包", brand: "山野", model: "45L", description: "适合周末和一晚轻量露营的背负系统。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", specs: ["capacity": "45 L", "recommended_load": "12 kg", "back_length": "48 cm"], approvedAt: "2026-05-03T10:00:00Z", isDeleted: false, createdAt: "2026-05-01T10:00:00Z", updatedAt: "2026-05-03T10:00:00Z"),
        GearAtlasPublicItem(id: "atlas-2", category: .lightingSystem, categoryLabel: GearCategory.lightingSystem.label, name: "星火 HL200 头灯", brand: "星火", model: "HL200", description: "轻量备用头灯，夜行和营地都够用。", weightG: 86, officialPriceCents: 19900, officialPriceCurrency: "CNY", specs: ["max_brightness": "450 lm", "runtime": "8 h", "waterproof_rating": "IPX4"], approvedAt: "2026-04-18T10:00:00Z", isDeleted: false, createdAt: "2026-04-15T10:00:00Z", updatedAt: "2026-04-18T10:00:00Z")
    ]

    static let atlasSubmissions: [GearAtlasSubmission] = [
        GearAtlasSubmission(id: "submission-1", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, name: "轻量背包", brand: "山野", model: "45L", description: "用户装备公开字段投稿。", weightG: 980, officialPriceCents: 109900, officialPriceCurrency: "CNY", specs: ["capacity": "45 L", "recommended_load": "12 kg"], approvedAt: nil, isDeleted: false, createdAt: "2026-05-02T10:00:00Z", updatedAt: "2026-05-02T10:00:00Z", sourceType: .userGear, sourceUserGearId: "gear-1", status: .pending, rejectionReason: nil, reviewedAt: nil)
    ]

    static let packingLists: [GearPackingListDetail] = {
        let items = [
            packingListItem(id: "packing-item-1", gear: gearItems[0], packed: true),
            packingListItem(id: "packing-item-2", gear: gearItems[1], packed: false)
        ]
        return [
            GearPackingListDetail(
                id: "packing-1",
                name: "武功山两日穿越",
                routeName: "沈子村 - 金顶 - 龙山村",
                durationLabel: "2 天 1 夜",
                stats: GearPackingListStats(itemCount: 2, packedCount: 1, totalWeightG: items.reduce(0) { $0 + ($1.gear.weightG ?? 0) }),
                items: items,
                createdAt: "2026-05-15T09:00:00Z",
                updatedAt: "2026-05-16T09:00:00Z"
            )
        ]
    }()

    static let tripDetails: [TripDetail] = [
        tripDetail(for: Trip(
            id: "trip-team-1",
            ownerUserId: Session.fixture.user.id,
            tripType: .team,
            title: "武功山两日穿越",
            description: "多人协作行程，重点确认公共装备和安全预案。",
            startDate: "2026-06-06",
            endDate: "2026-06-07",
            enabledSections: TripSectionKey.allowed(for: .team),
            routeUseSlopeAdjustment: true,
            routeUseHighAltitudeAdjustment: false,
            routeStartAltitudeM: 720,
            dayCount: 2,
            fieldVersions: [:],
            isDeleted: false,
            createdAt: "2026-05-15T09:00:00Z",
            updatedAt: "2026-05-16T09:00:00Z"
        )),
        tripDetail(for: Trip(
            id: "trip-solo-1",
            ownerUserId: Session.fixture.user.id,
            tripType: .solo,
            title: "近郊夜行复盘",
            description: "单人行程只保留个人准备板块。",
            startDate: "2026-05-10",
            endDate: "2026-05-10",
            enabledSections: TripSectionKey.allowed(for: .solo),
            routeUseSlopeAdjustment: true,
            routeUseHighAltitudeAdjustment: false,
            routeStartAltitudeM: 120,
            dayCount: 1,
            fieldVersions: [:],
            isDeleted: false,
            createdAt: "2026-05-09T09:00:00Z",
            updatedAt: "2026-05-10T20:00:00Z"
        ))
    ]

    static let outdoorProfile = OutdoorProfile(
        userId: Session.fixture.user.id,
        outdoorId: "ST-ALICE",
        realName: "Alice",
        gender: "female",
        birthDate: "1995-05-01",
        heightCm: 168,
        phone: "13800000000",
        emergencyContact: "Bob",
        emergencyContactRelationship: "朋友",
        emergencyPhone: "13900000000",
        bloodType: "O",
        medicalHistory: "无重大病史",
        allergyHistory: "花粉轻微过敏",
        medicalResponseNote: "必要时联系紧急联系人。",
        dietPreference: "少辣",
        insurancePolicyNo: "POLICY-2026",
        insuranceCompanyPhone: "95500",
        experienceNote: "一年十次轻徒步，熟悉夜行基础。",
        createdAt: "2026-05-01T09:00:00Z",
        updatedAt: "2026-05-16T09:00:00Z"
    )

    static let outdoorExperiences: [OutdoorExperience] = [
        OutdoorExperience(
            id: "experience-1",
            userId: Session.fixture.user.id,
            sourceTripId: "trip-solo-1",
            tripType: .solo,
            title: "近郊夜行复盘",
            startDate: "2026-05-10",
            endDate: "2026-05-10",
            dayCount: 1,
            companionCount: 0,
            routeSummary: "公园环线 9 km，夜间返回。",
            gearSummary: "头灯、轻量背包、急救包。",
            foodSummary: "能量胶和热水。",
            budgetSummary: "交通 28 元。",
            notes: "头灯电池应提前替换。",
            createdAt: "2026-05-11T09:00:00Z",
            updatedAt: "2026-05-11T09:00:00Z"
        )
    ]

    static let roadmapItems: [RoadmapItem] = [
        RoadmapItem(
            id: "roadmap-ios-1",
            clientKey: .ios,
            title: "iOS 行程详情协作",
            summary: "补齐行程成员、装备、食品和安全预案的原生编辑体验。",
            details: "优先与小程序端保持信息架构一致。",
            category: "routes",
            status: "building",
            priority: 1,
            sortOrder: 10,
            isPublished: true,
            voteCount: 8,
            subscriptionCount: 5,
            isVoted: false,
            isSubscribed: true,
            publishedAt: "2026-05-16T09:00:00Z",
            createdAt: "2026-05-15T09:00:00Z",
            updatedAt: "2026-05-16T09:00:00Z"
        ),
        RoadmapItem(
            id: "roadmap-ios-2",
            clientKey: .ios,
            title: "离线装备与绳结缓存",
            summary: "在弱网环境中保留关键装备、清单和绳结媒体。",
            details: nil,
            category: "offline",
            status: "planned",
            priority: 2,
            sortOrder: 20,
            isPublished: true,
            voteCount: 3,
            subscriptionCount: 2,
            isVoted: false,
            isSubscribed: false,
            publishedAt: "2026-05-16T09:00:00Z",
            createdAt: "2026-05-15T09:00:00Z",
            updatedAt: "2026-05-16T09:00:00Z"
        )
    ]

    static let clientVersions: [ClientVersion] = [
        ClientVersion(
            id: "ios-version-1",
            clientKey: .ios,
            version: "0.1.0",
            title: "iOS 原生预览版",
            releaseNotes: ["五个 Tab 对齐小程序端。", "新增行程、打包清单、路线图和户外资料入口。"],
            releaseNoteSections: [
                ClientVersionReleaseNoteSection(key: "new", title: "新增", items: ["行程 Tab", "打包清单", "户外经历"])
            ],
            status: .published,
            publishedAt: "2026-05-17T09:00:00Z",
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
    ]

    static func packingListItem(id: String, gear: GearItem, packed: Bool) -> GearPackingListItem {
        GearPackingListItem(
            id: id,
            gearId: gear.id,
            plannedQuantity: 1,
            packedQuantity: packed ? 1 : 0,
            packed: packed,
            unavailable: false,
            unavailableReason: nil,
            gear: gear.summary(),
            createdAt: "2026-05-15T09:00:00Z",
            updatedAt: "2026-05-16T09:00:00Z"
        )
    }

    static func tripDetail(for trip: Trip) -> TripDetail {
        let owner = TripMember(
            id: "member-owner-\(trip.id)",
            planId: trip.id,
            userId: Session.fixture.user.id,
            isOwner: true,
            profile: TripMemberProfile(displayName: "Alice", outdoorId: "ST-ALICE", realName: "Alice", gender: "female", age: 31, heightCm: 168, phone: "13800000000", emergencyContact: "Bob", emergencyContactRelationship: "朋友", emergencyPhone: "13900000000", bloodType: "O", medicalHistory: nil, allergyHistory: "花粉轻微过敏", medicalResponseNote: nil, dietPreference: "少辣", insurancePolicyNo: "POLICY-2026", insuranceCompanyPhone: "95500", experienceNote: "轻徒步经验丰富", roleLabel: "队长"),
            fieldVersions: [:],
            isDeleted: false,
            createdAt: trip.createdAt,
            updatedAt: trip.updatedAt
        )
        let teammate = TripMember(
            id: "member-teammate-\(trip.id)",
            planId: trip.id,
            userId: "user-fixture-2",
            isOwner: false,
            profile: TripMemberProfile(displayName: "Chen", outdoorId: nil, realName: "Chen", gender: nil, age: nil, heightCm: nil, phone: "13700000000", emergencyContact: nil, emergencyContactRelationship: nil, emergencyPhone: nil, bloodType: nil, medicalHistory: nil, allergyHistory: nil, medicalResponseNote: nil, dietPreference: nil, insurancePolicyNo: nil, insuranceCompanyPhone: nil, experienceNote: "负责导航", roleLabel: "导航"),
            fieldVersions: [:],
            isDeleted: false,
            createdAt: trip.createdAt,
            updatedAt: trip.updatedAt
        )
        let members = trip.tripType == .solo ? [owner] : [owner, teammate]
        let personalGear = [
            TripPersonalGearItem(id: "personal-gear-\(trip.id)-1", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, name: "轻量背包", brand: "山野", model: "45L", plannedQuantity: 1, packedQuantity: 1, unitWeightG: 980, notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt, memberId: owner.id, sourcePackingListId: "packing-1", sourcePackingItemId: "packing-item-1", sourceGearId: "gear-1"),
            TripPersonalGearItem(id: "personal-gear-\(trip.id)-2", category: .lightingSystem, categoryLabel: GearCategory.lightingSystem.label, name: "头灯", brand: "星火", model: "HL200", plannedQuantity: 1, packedQuantity: 0, unitWeightG: 86, notes: "备用电池在顶包。", fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt, memberId: owner.id, sourcePackingListId: "packing-1", sourcePackingItemId: "packing-item-2", sourceGearId: "gear-2")
        ]
        let sharedGear = trip.tripType == .solo ? [] : [
            TripSharedGearDemand(id: "shared-\(trip.id)-1", category: .kitchenSystem, categoryLabel: GearCategory.kitchenSystem.label, name: "炉头", brand: nil, model: nil, plannedQuantity: 1, packedQuantity: 0, unitWeightG: 180, notes: "公共晚餐使用。", fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt, sourceMemberId: nil, sourceGearId: nil, responsibleMemberId: teammate.id, createdByUserId: Session.fixture.user.id, templateKey: "stove", demandName: "炉头", concreteName: nil)
        ]
        let routeSegment = TripRouteSegment(id: "segment-\(trip.id)-1", name: "沈子村至金顶", startPoint: "沈子村", endPoint: "金顶", checkpoint: "发云界", leaderMemberId: owner.id, bailoutRoute: "天气恶化从景区车道下撤", trailCondition: "山脊草甸，雨后湿滑", distanceKm: trip.tripType == .solo ? 9.0 : 13.5, ascentM: trip.tripType == .solo ? 260 : 1180, descentM: trip.tripType == .solo ? 260 : 620, descentProfile: "mixed", technicalFactor: 1.0, restFactor: 1.15, packFactor: 1.1, formulaEstimateMinutes: 330, finalEstimateMinutes: 390, manualEstimateMinutes: nil, estimatedStartAltitudeM: trip.routeStartAltitudeM, estimatedEndAltitudeM: 1700, estimatedHighestAltitudeM: 1918, highAltitudeFactor: nil, notes: "午后风大，注意保暖。", fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let timeSlot = TripItineraryTimeSlot(id: "slot-\(trip.id)-1", dayId: "day-\(trip.id)-1", slotKey: "morning", routeSegmentId: routeSegment.id, routeDescription: "上午完成主要爬升。", notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let day = TripItineraryDay(id: "day-\(trip.id)-1", dayIndex: 1, dateLabel: trip.startDate, title: "第 1 天", notes: "控制节奏，下午到营地。", weather: "多云", highTemperatureC: 22, lowTemperatureC: 12, weatherSummary: "山顶风大", weatherNotes: nil, campName: trip.tripType == .solo ? nil : "金顶营地", campAltitudeM: trip.tripType == .solo ? nil : 1700, campTerrain: "草甸", campSlope: "缓坡", campArea: "开阔", campWaterSource: "营地补水", campNotes: nil, estimateMinutes: 390, timeSlots: [timeSlot], fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let foodItem = TripFoodItem(id: "food-item-\(trip.id)-1", foodMealId: "meal-\(trip.id)-1", name: "米饭与牛肉", amountG: 350, perPersonAmountG: 175, totalPriceCents: 3800, responsibleMemberId: owner.id, notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let foodMeal = TripFoodMeal(id: "meal-\(trip.id)-1", itineraryDayId: day.id, mealKey: "dinner", mealType: "晚餐", skipped: false, dishName: "牛肉饭", responsibleMemberId: owner.id, notes: nil, items: [foodItem], fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let foodSupply = TripFoodSupply(id: "supply-\(trip.id)-1", name: "饮用水", supplyType: "water", amountG: 2000, perPersonAmountG: 1000, totalPriceCents: 1200, responsibleMemberId: owner.id, notes: "出发前补满。", fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let medical = TripMedicalItem(id: "medical-\(trip.id)-1", name: "弹性绷带", itemType: "bandage", scope: "shared", suggestedQuantity: 1, requiredQuantity: 1, packedQuantity: 1, responsibleMemberId: owner.id, notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let assignment = TripSegmentAssignment(id: "assignment-\(trip.id)-1", routeSegmentId: routeSegment.id, checkpoint: "发云界", leaderRecordMemberId: owner.id, navigatorSafetyMemberId: trip.tripType == .solo ? owner.id : teammate.id, collaboratorMemberId: nil, photographerMemberId: nil, safetyMemberId: owner.id, environmentMemberId: nil, sweeperMemberId: nil, notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let risk = TripSafetyRisk(id: "risk-\(trip.id)-1", riskType: "天气突变", prevention: "午后前通过山脊，携带雨具和保温层。", response: "能见度下降时原路撤回。", responsibleMemberId: owner.id, itineraryDayId: day.id, routeSegmentId: routeSegment.id, notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let rescue = TripRescueContact(id: "rescue-\(trip.id)-1", organization: "景区救援站", address: "金顶游客中心", phone: "0799-123456", notes: "无信号区提前告知队员。", fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let budget = TripBudgetItem(id: "budget-\(trip.id)-1", category: "transport", name: "往返交通", quantity: 1, unitPriceCents: 8800, totalPriceCents: 8800, splitMemberCount: max(members.count, 1), notes: nil, linkedSharedGearId: nil, linkedSharedGearDeleted: false, linkedSharedGearName: nil, linkedSharedGearResponsibleMemberId: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let goal = TripGoalItem(id: "goal-\(trip.id)-1", scope: "team", memberId: nil, content: trip.tripType == .solo ? "安全完成夜行复盘" : "全员按计划完成穿越", notes: nil, fieldVersions: [:], createdAt: trip.createdAt, updatedAt: trip.updatedAt)
        let memberGearItem = TripMemberGearViewItem(id: "view-\(trip.id)-1", source: "personal", name: "轻量背包", category: .backpackSystem, categoryLabel: GearCategory.backpackSystem.label, plannedQuantity: 1, packedQuantity: 1, unitWeightG: 980, labels: ["个人", "已打包"], countsWeight: true)
        return TripDetail(
            trip: trip,
            sections: trip.enabledSections,
            myMemberId: owner.id,
            members: members,
            personalGear: personalGear,
            sharedGearDemands: sharedGear,
            itineraryDays: [day],
            routeSegments: [routeSegment],
            foodMeals: [foodMeal],
            foodSupplies: [foodSupply],
            medicalItems: [medical],
            segmentAssignments: trip.tripType == .solo ? [] : [assignment],
            safetyRisks: [risk],
            rescueContacts: [rescue],
            budgetItems: [budget],
            goals: [goal],
            weightSummaries: [TripMemberGearWeightSummary(memberId: owner.id, allWeightG: 1066, actualWeightG: 980)],
            memberGearViews: [TripMemberGearView(memberId: owner.id, allWeightG: 1066, actualWeightG: 980, items: [memberGearItem])]
        )
    }

    static func tripSummary(from detail: TripDetail, today: String) -> TripSummary {
        let trip = detail.trip
        let bucket: TripTimeBucket = trip.id.contains("solo") ? .past : .upcoming
        return TripSummary(
            id: trip.id,
            ownerUserId: trip.ownerUserId,
            tripType: trip.tripType,
            title: trip.title,
            description: trip.description,
            startDate: trip.startDate,
            endDate: trip.endDate,
            enabledSections: trip.enabledSections,
            routeUseSlopeAdjustment: trip.routeUseSlopeAdjustment,
            routeUseHighAltitudeAdjustment: trip.routeUseHighAltitudeAdjustment,
            routeStartAltitudeM: trip.routeStartAltitudeM,
            dayCount: trip.dayCount,
            fieldVersions: trip.fieldVersions,
            isDeleted: trip.isDeleted,
            createdAt: trip.createdAt,
            updatedAt: trip.updatedAt,
            timeBucket: bucket,
            daysUntilStart: bucket == .upcoming ? 8 : nil,
            daysUntilEnd: bucket == .ongoing ? 1 : nil,
            memberCount: detail.members.count,
            readiness: TripReadiness(missingCount: detail.personalGear.filter { $0.packedQuantity < $0.plannedQuantity }.count, missingLabels: detail.personalGear.filter { $0.packedQuantity < $0.plannedQuantity }.map(\.name), completionPercent: 70),
            outdoorExperienceId: trip.id.contains("solo") ? "experience-1" : nil
        )
    }

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
            isDeleted: false,
            createdAt: "2026-05-17T09:00:00Z",
            updatedAt: "2026-05-17T09:00:00Z"
        )
    }

    static func copy(_ item: GearItem, archivedAt: String?, isDeleted: Bool? = nil) -> GearItem {
        GearItem(id: item.id, userId: item.userId, category: item.category, name: item.name, brand: item.brand, model: item.model, color: item.color, material: item.material, capacity: item.capacity, size: item.size, description: item.description, weightG: item.weightG, officialPriceCents: item.officialPriceCents, officialPriceCurrency: item.officialPriceCurrency, warmthIndex: item.warmthIndex, waterproofIndex: item.waterproofIndex, purchaseDate: item.purchaseDate, purchasePriceCents: item.purchasePriceCents, purchasePriceCurrency: item.purchasePriceCurrency, expiryOrWarrantyDate: item.expiryOrWarrantyDate, purchaseLocation: item.purchaseLocation, status: item.status, storageLocation: item.storageLocation, specs: item.specs, tags: item.tags, tagColors: item.tagColors, shareEnabled: item.shareEnabled, shareStatus: item.shareStatus, notes: item.notes, archivedAt: archivedAt, isDeleted: isDeleted ?? item.isDeleted, createdAt: item.createdAt, updatedAt: "2026-05-17T09:00:00Z")
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
            isDeleted: false,
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
