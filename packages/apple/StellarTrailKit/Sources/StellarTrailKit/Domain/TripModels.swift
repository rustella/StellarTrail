import Foundation

typealias FieldVersions = [String: Int]

enum JSONValue: Codable, Equatable {
    case string(String)
    case int(Int)
    case double(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Int.self) {
            self = .int(value)
        } else if let value = try? container.decode(Double.self) {
            self = .double(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([String: JSONValue].self) {
            self = .object(value)
        } else if let value = try? container.decode([JSONValue].self) {
            self = .array(value)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unsupported JSON value")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value): try container.encode(value)
        case .int(let value): try container.encode(value)
        case .double(let value): try container.encode(value)
        case .bool(let value): try container.encode(value)
        case .object(let value): try container.encode(value)
        case .array(let value): try container.encode(value)
        case .null: try container.encodeNil()
        }
    }
}

enum TripSectionKey: String, Codable, CaseIterable, Identifiable {
    case members
    case personalGear = "personal_gear"
    case itinerary
    case sharedGear = "shared_gear"
    case foodPlan = "food_plan"
    case medicalKit = "medical_kit"
    case safetyPlan = "safety_plan"
    case rescueInfo = "rescue_info"
    case budget
    case goals

    var id: String { rawValue }

    var label: String {
        switch self {
        case .members: return "成员"
        case .personalGear: return "个人装备"
        case .itinerary: return "行程安排"
        case .sharedGear: return "公共装备"
        case .foodPlan: return "食品计划"
        case .medicalKit: return "医药包"
        case .safetyPlan: return "安全预案"
        case .rescueInfo: return "救援信息"
        case .budget: return "预算"
        case .goals: return "目标"
        }
    }

    var systemImage: String {
        switch self {
        case .members: return "person.2.fill"
        case .personalGear: return "backpack.fill"
        case .itinerary: return "map.fill"
        case .sharedGear: return "shippingbox.fill"
        case .foodPlan: return "fork.knife"
        case .medicalKit: return "cross.case.fill"
        case .safetyPlan: return "shield.lefthalf.filled"
        case .rescueInfo: return "phone.fill"
        case .budget: return "yensign.circle.fill"
        case .goals: return "target"
        }
    }

    static func allowed(for tripType: TripType) -> [TripSectionKey] {
        switch tripType {
        case .solo:
            return allCases.filter { $0 != .members && $0 != .sharedGear }
        case .team:
            return allCases
        }
    }
}

enum TripType: String, Codable, CaseIterable, Identifiable {
    case solo
    case team

    var id: String { rawValue }
    var label: String { self == .solo ? "单人" : "多人" }
}

enum TripTimeBucket: String, Codable, CaseIterable, Identifiable {
    case ongoing
    case upcoming
    case past
    case undated
    case all

    var id: String { rawValue }

    var label: String {
        switch self {
        case .ongoing: return "进行中"
        case .upcoming: return "未来行程"
        case .past: return "历史行程"
        case .undated: return "未定日期"
        case .all: return "全部"
        }
    }
}

struct TripReadiness: Codable, Equatable {
    let missingCount: Int
    let missingLabels: [String]
    let completionPercent: Int
}

struct Trip: Codable, Equatable, Identifiable {
    let id: String
    let ownerUserId: String
    let tripType: TripType
    let title: String
    let description: String?
    let startDate: String?
    let endDate: String?
    let enabledSections: [TripSectionKey]
    let routeUseSlopeAdjustment: Bool
    let routeUseHighAltitudeAdjustment: Bool
    let routeStartAltitudeM: Int?
    let dayCount: Int
    let fieldVersions: FieldVersions
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String
}

struct TripSummary: Codable, Equatable, Identifiable {
    let id: String
    let ownerUserId: String
    let tripType: TripType
    let title: String
    let description: String?
    let startDate: String?
    let endDate: String?
    let enabledSections: [TripSectionKey]
    let routeUseSlopeAdjustment: Bool
    let routeUseHighAltitudeAdjustment: Bool
    let routeStartAltitudeM: Int?
    let dayCount: Int
    let fieldVersions: FieldVersions
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String
    let timeBucket: TripTimeBucket
    let daysUntilStart: Int?
    let daysUntilEnd: Int?
    let memberCount: Int
    let readiness: TripReadiness
    let outdoorExperienceId: String?

    var dateText: String {
        Formatters.dateRange(startDate: startDate, endDate: endDate)
    }

    var durationText: String {
        dayCount > 0 ? "\(dayCount)天" : "未设置"
    }

    var readinessText: String {
        readiness.missingCount > 0 ? "还差 \(readiness.missingCount) 项准备" : "准备较完整"
    }

    var asTrip: Trip {
        Trip(
            id: id,
            ownerUserId: ownerUserId,
            tripType: tripType,
            title: title,
            description: description,
            startDate: startDate,
            endDate: endDate,
            enabledSections: enabledSections,
            routeUseSlopeAdjustment: routeUseSlopeAdjustment,
            routeUseHighAltitudeAdjustment: routeUseHighAltitudeAdjustment,
            routeStartAltitudeM: routeStartAltitudeM,
            dayCount: dayCount,
            fieldVersions: fieldVersions,
            isDeleted: isDeleted,
            createdAt: createdAt,
            updatedAt: updatedAt
        )
    }
}

struct ListTripsRequest: Equatable {
    var limit: Int = 20
    var cursor: String?
    var bucket: TripTimeBucket = .all
    var tripType: TripType?
    var today: String?

    var queryItems: [URLQueryItem] {
        var items = [URLQueryItem(name: "limit", value: String(limit))]
        if bucket != .all { items.append(URLQueryItem(name: "bucket", value: bucket.rawValue)) }
        if let tripType { items.append(URLQueryItem(name: "trip_type", value: tripType.rawValue)) }
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        if let today { items.append(URLQueryItem(name: "today", value: today)) }
        return items
    }
}

struct ListTripsResponse: Codable, Equatable {
    let items: [TripSummary]
    let nextCursor: String?
}

enum TripHomeHighlightStatus: String, Codable, Equatable {
    case ongoing
    case upcoming

    var label: String { self == .ongoing ? "正在进行" : "即将出发" }
}

struct TripHomeHighlightItem: Codable, Equatable {
    let trip: TripSummary
    let status: TripHomeHighlightStatus
    let daysUntilStart: Int
    let daysUntilEnd: Int
}

struct TripHomeHighlightResponse: Codable, Equatable {
    let item: TripHomeHighlightItem?
}

struct CreateTripRequest: Codable, Equatable {
    var tripType: TripType
    var title: String
    var description: String?
    var startDate: String?
    var endDate: String?
    var routeUseSlopeAdjustment: Bool?
    var routeUseHighAltitudeAdjustment: Bool?
    var routeStartAltitudeM: Int?
}

struct UpdateTripRequest: Codable, Equatable {
    var title: String?
    var description: String?
    var startDate: String?
    var endDate: String?
    var routeUseSlopeAdjustment: Bool?
    var routeUseHighAltitudeAdjustment: Bool?
    var routeStartAltitudeM: Int?
    var baseFieldVersions: FieldVersions?
    var forceFields: [String]?
}

struct UpdateTripSectionsRequest: Codable, Equatable {
    var enabledSections: [TripSectionKey]
    var baseFieldVersions: FieldVersions?
    var forceFields: [String]?
}

struct TripMemberProfile: Codable, Equatable {
    var displayName: String
    var outdoorId: String?
    var realName: String?
    var gender: String?
    var age: Int?
    var heightCm: Int?
    var phone: String?
    var emergencyContact: String?
    var emergencyContactRelationship: String?
    var emergencyPhone: String?
    var bloodType: String?
    var medicalHistory: String?
    var allergyHistory: String?
    var medicalResponseNote: String?
    var dietPreference: String?
    var insurancePolicyNo: String?
    var insuranceCompanyPhone: String?
    var experienceNote: String?
    var roleLabel: String?
}

struct TripMember: Codable, Equatable, Identifiable {
    let id: String
    let planId: String
    let userId: String
    let isOwner: Bool
    let profile: TripMemberProfile
    let fieldVersions: FieldVersions
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String
}

struct TripGearSnapshotBase: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let plannedQuantity: Int
    let packedQuantity: Int
    let unitWeightG: Int?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripPersonalGearItem: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let plannedQuantity: Int
    let packedQuantity: Int
    let unitWeightG: Int?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
    let memberId: String
    let sourcePackingListId: String?
    let sourcePackingItemId: String?
    let sourceGearId: String?

    var brandModelText: String { Formatters.brandModel(brand: brand, model: model).nilIfBlank ?? "未填写品牌型号" }
}

struct TripSharedGearDemand: Codable, Equatable, Identifiable {
    let id: String
    let category: GearCategory
    let categoryLabel: String
    let name: String
    let brand: String?
    let model: String?
    let plannedQuantity: Int
    let packedQuantity: Int
    let unitWeightG: Int?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
    let sourceMemberId: String?
    let sourceGearId: String?
    let responsibleMemberId: String
    let createdByUserId: String?
    let templateKey: String?
    let demandName: String?
    let concreteName: String?
}

struct TripRouteSegment: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let startPoint: String?
    let endPoint: String?
    let checkpoint: String?
    let leaderMemberId: String?
    let bailoutRoute: String?
    let trailCondition: String?
    let distanceKm: Double
    let ascentM: Int
    let descentM: Int
    let descentProfile: String
    let technicalFactor: Double
    let restFactor: Double
    let packFactor: Double
    let formulaEstimateMinutes: Int
    let finalEstimateMinutes: Int
    let manualEstimateMinutes: Int?
    let estimatedStartAltitudeM: Int?
    let estimatedEndAltitudeM: Int?
    let estimatedHighestAltitudeM: Int?
    let highAltitudeFactor: Double?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripItineraryTimeSlot: Codable, Equatable, Identifiable {
    let id: String
    let dayId: String
    let slotKey: String
    let routeSegmentId: String?
    let routeDescription: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripItineraryDay: Codable, Equatable, Identifiable {
    let id: String
    let dayIndex: Int
    let dateLabel: String?
    let title: String?
    let notes: String?
    let weather: String?
    let highTemperatureC: Int?
    let lowTemperatureC: Int?
    let weatherSummary: String?
    let weatherNotes: String?
    let campName: String?
    let campAltitudeM: Int?
    let campTerrain: String?
    let campSlope: String?
    let campArea: String?
    let campWaterSource: String?
    let campNotes: String?
    let estimateMinutes: Int
    let timeSlots: [TripItineraryTimeSlot]
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripFoodItem: Codable, Equatable, Identifiable {
    let id: String
    let foodMealId: String
    let name: String
    let amountG: Int?
    let perPersonAmountG: Int?
    let totalPriceCents: Int?
    let responsibleMemberId: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripFoodMeal: Codable, Equatable, Identifiable {
    let id: String
    let itineraryDayId: String
    let mealKey: String
    let mealType: String?
    let skipped: Bool
    let dishName: String?
    let responsibleMemberId: String?
    let notes: String?
    let items: [TripFoodItem]
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripFoodSupply: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let supplyType: String?
    let amountG: Int?
    let perPersonAmountG: Int?
    let totalPriceCents: Int?
    let responsibleMemberId: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripMedicalItem: Codable, Equatable, Identifiable {
    let id: String
    let name: String
    let itemType: String?
    let scope: String?
    let suggestedQuantity: Int?
    let requiredQuantity: Int
    let packedQuantity: Int
    let responsibleMemberId: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripSegmentAssignment: Codable, Equatable, Identifiable {
    let id: String
    let routeSegmentId: String?
    let checkpoint: String?
    let leaderRecordMemberId: String?
    let navigatorSafetyMemberId: String?
    let collaboratorMemberId: String?
    let photographerMemberId: String?
    let safetyMemberId: String?
    let environmentMemberId: String?
    let sweeperMemberId: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripSafetyRisk: Codable, Equatable, Identifiable {
    let id: String
    let riskType: String
    let prevention: String?
    let response: String?
    let responsibleMemberId: String?
    let itineraryDayId: String?
    let routeSegmentId: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripRescueContact: Codable, Equatable, Identifiable {
    let id: String
    let organization: String
    let address: String?
    let phone: String?
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripBudgetItem: Codable, Equatable, Identifiable {
    let id: String
    let category: String?
    let name: String
    let quantity: Int
    let unitPriceCents: Int?
    let totalPriceCents: Int?
    let splitMemberCount: Int?
    let notes: String?
    let linkedSharedGearId: String?
    let linkedSharedGearDeleted: Bool
    let linkedSharedGearName: String?
    let linkedSharedGearResponsibleMemberId: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripGoalItem: Codable, Equatable, Identifiable {
    let id: String
    let scope: String
    let memberId: String?
    let content: String
    let notes: String?
    let fieldVersions: FieldVersions
    let createdAt: String
    let updatedAt: String
}

struct TripMemberGearWeightSummary: Codable, Equatable {
    let memberId: String
    let allWeightG: Int
    let actualWeightG: Int
}

struct TripMemberGearViewItem: Codable, Equatable, Identifiable {
    let id: String
    let source: String
    let name: String
    let category: GearCategory
    let categoryLabel: String
    let plannedQuantity: Int
    let packedQuantity: Int
    let unitWeightG: Int?
    let labels: [String]
    let countsWeight: Bool
}

struct TripMemberGearView: Codable, Equatable, Identifiable {
    let memberId: String
    let allWeightG: Int
    let actualWeightG: Int
    let items: [TripMemberGearViewItem]

    var id: String { memberId }
}

struct TripDetail: Codable, Equatable {
    let trip: Trip
    let sections: [TripSectionKey]
    let myMemberId: String
    let members: [TripMember]
    let personalGear: [TripPersonalGearItem]
    let sharedGearDemands: [TripSharedGearDemand]
    let itineraryDays: [TripItineraryDay]
    let routeSegments: [TripRouteSegment]
    let foodMeals: [TripFoodMeal]
    let foodSupplies: [TripFoodSupply]
    let medicalItems: [TripMedicalItem]
    let segmentAssignments: [TripSegmentAssignment]
    let safetyRisks: [TripSafetyRisk]
    let rescueContacts: [TripRescueContact]
    let budgetItems: [TripBudgetItem]
    let goals: [TripGoalItem]
    let weightSummaries: [TripMemberGearWeightSummary]
    let memberGearViews: [TripMemberGearView]

    var visibleSections: [TripSectionKey] {
        sections.filter { TripSectionKey.allowed(for: trip.tripType).contains($0) }
    }
}

struct TripInvitation: Codable, Equatable, Identifiable {
    let id: String
    let planId: String
    let token: String
    let createdByUserId: String
    let revokedAt: String?
    let createdAt: String
}

struct CreateTripInvitationResponse: Codable, Equatable {
    let invitation: TripInvitation
}

struct ImportTripPackingListRequest: Codable, Equatable {
    var packingListId: String
}

struct TripRecordCreateRequest: Encodable, Equatable {
    var parentId: String?
    var sortOrder: Int?
    var payload: [String: JSONValue]

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        if let parentId { try container.encode(parentId, forKey: DynamicCodingKey("parent_id")) }
        if let sortOrder { try container.encode(sortOrder, forKey: DynamicCodingKey("sort_order")) }
        for (key, value) in payload {
            try container.encode(value, forKey: DynamicCodingKey(key))
        }
    }
}

struct TripRecordPatchRequest: Encodable, Equatable {
    var baseFieldVersions: FieldVersions?
    var forceFields: [String]?
    var fields: [String: JSONValue]

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        if let baseFieldVersions { try container.encode(baseFieldVersions, forKey: DynamicCodingKey("base_field_versions")) }
        if let forceFields { try container.encode(forceFields, forKey: DynamicCodingKey("force_fields")) }
        for (key, value) in fields {
            try container.encode(value, forKey: DynamicCodingKey(key))
        }
    }
}

struct TripFieldConflict: Codable, Equatable {
    let field: String
    let clientValue: JSONValue?
    let serverValue: JSONValue?
    let serverVersion: Int
}

struct TripConflictResponse: Codable, Equatable {
    let code: String
    let message: String
    let conflicts: [TripFieldConflict]
}

struct OutdoorExperience: Codable, Equatable, Identifiable {
    let id: String
    let userId: String
    let sourceTripId: String?
    let tripType: TripType
    let title: String
    let startDate: String?
    let endDate: String?
    let dayCount: Int?
    let companionCount: Int?
    let routeSummary: String?
    let gearSummary: String?
    let foodSummary: String?
    let budgetSummary: String?
    let notes: String?
    let createdAt: String
    let updatedAt: String

    var dateText: String {
        Formatters.dateRange(startDate: startDate, endDate: endDate)
    }
}

struct OutdoorExperienceRequest: Codable, Equatable {
    var title: String
    var startDate: String? = nil
    var endDate: String? = nil
    var dayCount: Int? = nil
    var companionCount: Int? = nil
    var routeSummary: String? = nil
    var gearSummary: String? = nil
    var foodSummary: String? = nil
    var budgetSummary: String? = nil
    var notes: String? = nil
}

struct ListOutdoorExperiencesResponse: Codable, Equatable {
    let items: [OutdoorExperience]
}

struct OutdoorProfile: Codable, Equatable {
    let userId: String
    var outdoorId: String?
    var realName: String?
    var gender: String?
    var birthDate: String?
    var heightCm: Int?
    var phone: String?
    var emergencyContact: String?
    var emergencyContactRelationship: String?
    var emergencyPhone: String?
    var bloodType: String?
    var medicalHistory: String?
    var allergyHistory: String?
    var medicalResponseNote: String?
    var dietPreference: String?
    var insurancePolicyNo: String?
    var insuranceCompanyPhone: String?
    var experienceNote: String?
    var createdAt: String?
    var updatedAt: String?

    static let empty = OutdoorProfile(userId: "", outdoorId: nil, realName: nil, gender: nil, birthDate: nil, heightCm: nil, phone: nil, emergencyContact: nil, emergencyContactRelationship: nil, emergencyPhone: nil, bloodType: nil, medicalHistory: nil, allergyHistory: nil, medicalResponseNote: nil, dietPreference: nil, insurancePolicyNo: nil, insuranceCompanyPhone: nil, experienceNote: nil, createdAt: nil, updatedAt: nil)
}

struct UpdateOutdoorProfileRequest: Codable, Equatable {
    var outdoorId: String?
    var realName: String?
    var gender: String?
    var birthDate: String?
    var heightCm: Int?
    var phone: String?
    var emergencyContact: String?
    var emergencyContactRelationship: String?
    var emergencyPhone: String?
    var bloodType: String?
    var medicalHistory: String?
    var allergyHistory: String?
    var medicalResponseNote: String?
    var dietPreference: String?
    var insurancePolicyNo: String?
    var insuranceCompanyPhone: String?
    var experienceNote: String?
}

struct OutdoorProfileResponse: Codable, Equatable {
    let profile: OutdoorProfile
}

private struct DynamicCodingKey: CodingKey {
    let stringValue: String
    let intValue: Int?

    init(_ stringValue: String) {
        self.stringValue = stringValue
        self.intValue = nil
    }

    init?(stringValue: String) {
        self.stringValue = stringValue
        self.intValue = nil
    }

    init?(intValue: Int) {
        self.stringValue = "\(intValue)"
        self.intValue = intValue
    }
}
