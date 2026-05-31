import Foundation
import Combine

@MainActor
final class GearStatsViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var stats = GearStatsResponse.empty

    private let repository: any GearRepositorying

    init(repository: any GearRepositorying) {
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            stats = try await repository.stats(tab: .available)
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class PackingListViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var isLoggedIn = false
    @Published private(set) var lists: [GearPackingListSummary] = []
    @Published var nameDraft = ""
    @Published var routeNameDraft = ""
    @Published var durationDraft = ""

    private let sessionStore: SessionStore
    private let repository: any GearPackingRepositorying

    init(sessionStore: SessionStore, repository: any GearPackingRepositorying) {
        self.sessionStore = sessionStore
        self.repository = repository
    }

    func load() async {
        isLoggedIn = sessionStore.isLoggedIn
        guard isLoggedIn else {
            lists = []
            return
        }
        loading = true
        error = nil
        defer { loading = false }
        do {
            lists = try await repository.list(ListGearPackingListsRequest()).items
        } catch {
            self.error = error.localizedDescription
        }
    }

    func create() async -> GearPackingListDetail? {
        guard nameDraft.nilIfBlank != nil else {
            error = "请填写清单名称"
            return nil
        }
        do {
            let detail = try await repository.create(CreateGearPackingListRequest(name: nameDraft, routeName: routeNameDraft.nilIfBlank, durationLabel: durationDraft.nilIfBlank))
            nameDraft = ""
            routeNameDraft = ""
            durationDraft = ""
            await load()
            return detail
        } catch {
            self.error = error.localizedDescription
            return nil
        }
    }

    func delete(id: String) async {
        do {
            try await repository.delete(id: id)
            await load()
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class PackingDetailViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var detail: GearPackingListDetail?

    private let id: String
    private let repository: any GearPackingRepositorying

    init(id: String, repository: any GearPackingRepositorying) {
        self.id = id
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            detail = try await repository.get(id: id)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func toggle(_ item: GearPackingListItem) async {
        do {
            detail = try await repository.updateItem(
                id: id,
                itemId: item.id,
                request: UpdateGearPackingItemRequest(packed: !item.packed, plannedQuantity: nil, packedQuantity: !item.packed ? item.plannedQuantity : 0)
            )
        } catch {
            self.error = error.localizedDescription
        }
    }

    func remove(_ item: GearPackingListItem) async {
        do {
            detail = try await repository.deleteItem(id: id, itemId: item.id)
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class PackingSelectGearViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var gears: [GearSummary] = []
    @Published var selectedIDs = Set<String>()

    private let packingListID: String
    private let gearRepository: any GearRepositorying
    private let packingRepository: any GearPackingRepositorying

    init(packingListID: String, gearRepository: any GearRepositorying, packingRepository: any GearPackingRepositorying) {
        self.packingListID = packingListID
        self.gearRepository = gearRepository
        self.packingRepository = packingRepository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            gears = try await gearRepository.list(ListGearsRequest(limit: 50)).items
        } catch {
            self.error = error.localizedDescription
        }
    }

    func toggle(id: String) {
        if selectedIDs.contains(id) {
            selectedIDs.remove(id)
        } else {
            selectedIDs.insert(id)
        }
    }

    func addSelected() async -> GearPackingListDetail? {
        guard !selectedIDs.isEmpty else {
            error = "请先选择装备"
            return nil
        }
        do {
            return try await packingRepository.addItems(id: packingListID, request: AddGearPackingItemsRequest(gearIds: Array(selectedIDs)))
        } catch {
            self.error = error.localizedDescription
            return nil
        }
    }
}

@MainActor
final class TripsViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var isLoggedIn = false
    @Published private(set) var bucket: TripTimeBucket = .all
    @Published private(set) var trips: [TripSummary] = []

    private let sessionStore: SessionStore
    private let repository: any TripRepositorying

    init(sessionStore: SessionStore, repository: any TripRepositorying) {
        self.sessionStore = sessionStore
        self.repository = repository
    }

    func load() async {
        isLoggedIn = sessionStore.isLoggedIn
        guard isLoggedIn else {
            trips = []
            return
        }
        loading = true
        error = nil
        defer { loading = false }
        do {
            trips = try await repository.list(ListTripsRequest(bucket: bucket, today: Formatters.localDateString(Date()))).items
        } catch {
            self.error = error.localizedDescription
        }
    }

    func selectBucket(_ next: TripTimeBucket) async {
        bucket = next
        await load()
    }

    func delete(id: String) async {
        do {
            try await repository.delete(id: id)
            await load()
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class TripFormViewModel: ObservableObject {
    @Published var tripType: TripType
    @Published var title = ""
    @Published var description = ""
    @Published var startDate = ""
    @Published var endDate = ""
    @Published var useSlopeAdjustment = true
    @Published var useHighAltitudeAdjustment = false
    @Published var startAltitude = ""
    @Published private(set) var error: String?

    private let repository: any TripRepositorying

    init(tripType: TripType, repository: any TripRepositorying) {
        self.tripType = tripType
        self.repository = repository
    }

    func submit() async -> TripDetail? {
        guard title.nilIfBlank != nil else {
            error = "请填写行程名称"
            return nil
        }
        do {
            return try await repository.create(CreateTripRequest(
                tripType: tripType,
                title: title,
                description: description.nilIfBlank,
                startDate: startDate.nilIfBlank,
                endDate: endDate.nilIfBlank,
                routeUseSlopeAdjustment: useSlopeAdjustment,
                routeUseHighAltitudeAdjustment: useHighAltitudeAdjustment,
                routeStartAltitudeM: Int(startAltitude)
            ))
        } catch {
            self.error = error.localizedDescription
            return nil
        }
    }
}

@MainActor
final class TripJoinViewModel: ObservableObject {
    @Published var token = ""
    @Published private(set) var error: String?

    private let repository: any TripRepositorying

    init(repository: any TripRepositorying) {
        self.repository = repository
    }

    func join() async -> TripDetail? {
        guard let token = token.nilIfBlank else {
            error = "请粘贴邀请口令"
            return nil
        }
        do {
            return try await repository.acceptInvitation(token: token)
        } catch {
            self.error = error.localizedDescription
            return nil
        }
    }
}

@MainActor
final class TripDetailViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var message: String?
    @Published private(set) var detail: TripDetail?
    @Published var selectedSection: TripSectionKey = .personalGear

    private let id: String
    private let repository: any TripRepositorying

    init(id: String, repository: any TripRepositorying) {
        self.id = id
        self.repository = repository
    }

    var visibleSections: [TripSectionKey] {
        detail?.visibleSections ?? []
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            let loaded = try await repository.get(id: id)
            detail = loaded
            if !loaded.visibleSections.contains(selectedSection) {
                selectedSection = loaded.visibleSections.first ?? .personalGear
            }
        } catch {
            self.error = error.localizedDescription
        }
    }

    func toggleSection(_ section: TripSectionKey) async {
        guard let detail else { return }
        var sections = Set(detail.sections)
        if sections.contains(section) {
            sections.remove(section)
        } else {
            sections.insert(section)
        }
        let ordered = TripSectionKey.allowed(for: detail.trip.tripType).filter { sections.contains($0) }
        do {
            self.detail = try await repository.updateSections(id: id, request: UpdateTripSectionsRequest(enabledSections: ordered, baseFieldVersions: detail.trip.fieldVersions, forceFields: nil))
        } catch {
            self.error = error.localizedDescription
        }
    }

    func createInvitation() async {
        do {
            let response = try await repository.createInvitation(id: id)
            message = "邀请口令：\(response.invitation.token)"
        } catch {
            self.error = error.localizedDescription
        }
    }

    func convertToExperience() async {
        do {
            let experience = try await repository.convertToOutdoorExperience(id: id, today: Formatters.localDateString(Date()))
            message = "已转为户外经历：\(experience.title)"
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class RoadmapViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var selectedStatus: RoadmapStatus?
    @Published private(set) var items: [RoadmapItem] = []

    private let sessionStore: SessionStore
    private let repository: any RoadmapRepositorying

    init(sessionStore: SessionStore, repository: any RoadmapRepositorying) {
        self.sessionStore = sessionStore
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            items = try await repository.list(ListRoadmapRequest(status: selectedStatus), includeUserState: sessionStore.isLoggedIn).items
        } catch {
            self.error = error.localizedDescription
        }
    }

    func selectStatus(_ status: RoadmapStatus?) async {
        selectedStatus = status
        await load()
    }

    func toggleVote(_ item: RoadmapItem) async {
        do {
            let updated = item.isVoted ? try await repository.unvote(id: item.id) : try await repository.vote(id: item.id)
            replace(updated)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func toggleSubscription(_ item: RoadmapItem) async {
        do {
            let updated = item.isSubscribed ? try await repository.unsubscribe(id: item.id) : try await repository.subscribe(id: item.id)
            replace(updated)
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func replace(_ item: RoadmapItem) {
        if let index = items.firstIndex(where: { $0.id == item.id }) {
            items[index] = item
        }
    }
}

@MainActor
final class OutdoorProfileViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var message: String?
    @Published var profile = OutdoorProfile.empty

    private let repository: any ProfileRepositorying

    init(repository: any ProfileRepositorying) {
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            profile = try await repository.outdoorProfile().profile
        } catch {
            self.error = error.localizedDescription
        }
    }

    func save() async {
        do {
            profile = try await repository.updateOutdoorProfile(UpdateOutdoorProfileRequest(
                outdoorId: profile.outdoorId,
                realName: profile.realName,
                gender: profile.gender,
                birthDate: profile.birthDate,
                heightCm: profile.heightCm,
                phone: profile.phone,
                emergencyContact: profile.emergencyContact,
                emergencyContactRelationship: profile.emergencyContactRelationship,
                emergencyPhone: profile.emergencyPhone,
                bloodType: profile.bloodType,
                medicalHistory: profile.medicalHistory,
                allergyHistory: profile.allergyHistory,
                medicalResponseNote: profile.medicalResponseNote,
                dietPreference: profile.dietPreference,
                insurancePolicyNo: profile.insurancePolicyNo,
                insuranceCompanyPhone: profile.insuranceCompanyPhone,
                experienceNote: profile.experienceNote
            )).profile
            message = "户外资料已保存"
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class OutdoorExperiencesViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var items: [OutdoorExperience] = []
    @Published var titleDraft = ""
    @Published var routeDraft = ""
    @Published var notesDraft = ""

    private let repository: any TripRepositorying

    init(repository: any TripRepositorying) {
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            items = try await repository.listOutdoorExperiences().items
        } catch {
            self.error = error.localizedDescription
        }
    }

    func create() async {
        guard titleDraft.nilIfBlank != nil else {
            error = "请填写经历标题"
            return
        }
        do {
            _ = try await repository.createOutdoorExperience(OutdoorExperienceRequest(title: titleDraft, routeSummary: routeDraft.nilIfBlank, notes: notesDraft.nilIfBlank))
            titleDraft = ""
            routeDraft = ""
            notesDraft = ""
            await load()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func delete(id: String) async {
        do {
            try await repository.deleteOutdoorExperience(id: id)
            await load()
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class FeedbackViewModel: ObservableObject {
    @Published var category: FeedbackCategory = .bug
    @Published var content = ""
    @Published var contact = ""
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var message: String?

    private let repository: any FeedbackRepositorying

    init(repository: any FeedbackRepositorying) {
        self.repository = repository
    }

    func submit(page: String = "ios/profile") async {
        guard content.nilIfBlank != nil else {
            error = "请填写反馈内容"
            return
        }
        loading = true
        error = nil
        defer { loading = false }
        do {
            let response = try await repository.create(CreateFeedbackRequest(category: category.rawValue, content: content, contact: contact.nilIfBlank, page: page, clientPlatform: "ios", clientVersion: nil, deviceModel: nil, imageIds: []))
            content = ""
            message = "反馈已提交：\(response.status)"
        } catch {
            self.error = error.localizedDescription
        }
    }
}

@MainActor
final class ClientVersionViewModel: ObservableObject {
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var current: ClientVersion?
    @Published private(set) var versions: [ClientVersion] = []

    private let repository: any ClientVersionRepositorying

    init(repository: any ClientVersionRepositorying) {
        self.repository = repository
    }

    func load() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            current = try await repository.current(clientKey: .ios)
            versions = try await repository.list(ListClientVersionsRequest(clientKey: .ios)).items
        } catch {
            self.error = error.localizedDescription
        }
    }
}
