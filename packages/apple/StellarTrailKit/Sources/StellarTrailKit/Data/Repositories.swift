import Foundation

@MainActor
protocol AuthRepositorying {
    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse
    func sendEmailLoginCode(email: String) async throws -> EmailVerificationCodeResponse
    func sendPasswordResetCode(email: String) async throws -> EmailVerificationCodeResponse
    func register(_ request: RegisterRequest) async throws -> LoginResponse
    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse
    func loginWithEmailCode(email: String, code: String) async throws -> LoginResponse
    func resetPassword(_ request: PasswordResetRequest) async throws -> LoginResponse
    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse
    func captcha(account: String) async throws -> CaptchaChallengeResponse
    func currentUser() async throws -> UserProfile
    func sendBindEmailCode(email: String) async throws -> EmailVerificationCodeResponse
    func bindEmail(email: String, code: String) async throws -> UserProfile
    func uploadAvatar(data: Data, fileName: String, mimeType: String) async throws -> UserProfile
}

@MainActor
protocol GearRepositorying {
    func stats(tab: GearTab) async throws -> GearStatsResponse
    func categories(tab: GearTab) async throws -> GearCategoriesResponse
    func specKeyRankings(category: GearCategory) async throws -> GearSpecKeyRankingsResponse
    func tagSuggestions(limit: Int) async throws -> GearTagSuggestionsResponse
    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse
    func get(id: String) async throws -> GearItem
    func create(_ request: CreateGearRequest) async throws -> GearItem
    func update(id: String, request: UpdateGearRequest) async throws -> GearItem
    func archive(id: String) async throws
    func delete(id: String) async throws
    func undelete(id: String) async throws -> GearItem
    func restore(id: String) async throws -> GearItem
}

@MainActor
protocol GearAtlasRepositorying {
    func list(_ request: ListGearAtlasRequest) async throws -> ListGearAtlasResponse
    func get(id: String) async throws -> GearAtlasPublicItem
    func createSubmission(_ request: CreateGearAtlasSubmissionRequest) async throws -> GearAtlasSubmission
    func submitGear(id: String) async throws -> GearAtlasSubmission
    func mySubmissions(_ request: ListGearAtlasSubmissionsRequest) async throws -> ListGearAtlasSubmissionsResponse
}

@MainActor
protocol SkillRepositorying {
    func categories() async throws -> SkillCategoriesResponse
    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse
    func knotDetail(id: String) async throws -> KnotDetail
    func offlineManifest() async throws -> KnotOfflineManifestResponse
    func knotDisclaimer() async throws -> KnotDisclaimerResponse
    func acceptKnotDisclaimer(_ request: AcceptKnotDisclaimerRequest) async throws -> KnotDisclaimerResponse
}

@MainActor
protocol ContentRepositorying {
    func gearTemplates() async throws -> GearTemplatesResponse
}

@MainActor
protocol GearPackingRepositorying {
    func list(_ request: ListGearPackingListsRequest) async throws -> ListGearPackingListsResponse
    func create(_ request: CreateGearPackingListRequest) async throws -> GearPackingListDetail
    func get(id: String) async throws -> GearPackingListDetail
    func update(id: String, request: UpdateGearPackingListRequest) async throws -> GearPackingListDetail
    func delete(id: String) async throws
    func addItems(id: String, request: AddGearPackingItemsRequest) async throws -> GearPackingListDetail
    func updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest) async throws -> GearPackingListDetail
    func deleteItem(id: String, itemId: String) async throws -> GearPackingListDetail
}

@MainActor
protocol TripRepositorying {
    func list(_ request: ListTripsRequest) async throws -> ListTripsResponse
    func homeHighlight(today: String) async throws -> TripHomeHighlightResponse
    func create(_ request: CreateTripRequest) async throws -> TripDetail
    func get(id: String) async throws -> TripDetail
    func update(id: String, request: UpdateTripRequest) async throws -> TripDetail
    func delete(id: String) async throws
    func updateSections(id: String, request: UpdateTripSectionsRequest) async throws -> TripDetail
    func createInvitation(id: String) async throws -> CreateTripInvitationResponse
    func acceptInvitation(token: String) async throws -> TripDetail
    func importPackingList(id: String, request: ImportTripPackingListRequest) async throws -> TripDetail
    func createRecord(id: String, collectionPath: String, request: TripRecordCreateRequest) async throws -> TripDetail
    func updateRecord(id: String, collectionPath: String, recordId: String, request: TripRecordPatchRequest) async throws -> TripDetail
    func deleteRecord(id: String, collectionPath: String, recordId: String) async throws -> TripDetail
    func convertToOutdoorExperience(id: String, today: String) async throws -> OutdoorExperience
    func listOutdoorExperiences() async throws -> ListOutdoorExperiencesResponse
    func createOutdoorExperience(_ request: OutdoorExperienceRequest) async throws -> OutdoorExperience
    func updateOutdoorExperience(id: String, request: OutdoorExperienceRequest) async throws -> OutdoorExperience
    func deleteOutdoorExperience(id: String) async throws
}

@MainActor
protocol ProfileRepositorying {
    func outdoorProfile() async throws -> OutdoorProfileResponse
    func updateOutdoorProfile(_ request: UpdateOutdoorProfileRequest) async throws -> OutdoorProfileResponse
}

@MainActor
protocol RoadmapRepositorying {
    func list(_ request: ListRoadmapRequest, includeUserState: Bool) async throws -> ListRoadmapResponse
    func vote(id: String) async throws -> RoadmapItem
    func unvote(id: String) async throws -> RoadmapItem
    func subscribe(id: String) async throws -> RoadmapItem
    func unsubscribe(id: String) async throws -> RoadmapItem
}

@MainActor
protocol FeedbackRepositorying {
    func uploadImage(data: Data, fileName: String, mimeType: String) async throws -> UploadImageInfo
    func create(_ request: CreateFeedbackRequest) async throws -> FeedbackResponse
}

@MainActor
protocol ClientVersionRepositorying {
    func list(_ request: ListClientVersionsRequest) async throws -> ListClientVersionsResponse
    func current(clientKey: ClientKey) async throws -> ClientVersion
}

@MainActor
final class AuthRepository: AuthRepositorying {
    private let client: APIClient
    private let sessionStore: SessionStore

    init(client: APIClient, sessionStore: SessionStore) {
        self.client = client
        self.sessionStore = sessionStore
    }

    func sendEmailVerificationCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/email-verification-code", body: EmailVerificationCodeRequest(email: email)), requiresAuth: false)
    }

    func sendEmailLoginCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/email-login-code", body: EmailLoginCodeRequest(email: email)), requiresAuth: false)
    }

    func sendPasswordResetCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/auth/password-reset-code", body: PasswordResetCodeRequest(email: email)), requiresAuth: false)
    }

    func register(_ request: RegisterRequest) async throws -> LoginResponse {
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/register", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?) async throws -> LoginResponse {
        let request = PasswordLoginRequest(account: account, password: password, captchaTicket: captchaTicket, captchaAnswer: captchaAnswer)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func loginWithEmailCode(email: String, code: String) async throws -> LoginResponse {
        let request = EmailLoginRequest(email: email, emailVerificationCode: code)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/email-login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func resetPassword(_ request: PasswordResetRequest) async throws -> LoginResponse {
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/password-reset", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func wechatLogin(code: String, profile: WechatLoginProfile?) async throws -> LoginResponse {
        let request = WechatLoginRequest(code: code, profile: profile)
        let response: LoginResponse = try await client.send(try APIRequest.post("/auth/wechat-login", body: request), requiresAuth: false)
        sessionStore.replace(with: response)
        return response
    }

    func captcha(account: String) async throws -> CaptchaChallengeResponse {
        try await client.send(try APIRequest.post("/auth/captcha", body: CaptchaChallengeRequest(account: account)), requiresAuth: false)
    }

    func currentUser() async throws -> UserProfile {
        let response: ProfileUserResponse = try await client.send(.get("/me/profile"), requiresAuth: true)
        replaceCurrentUser(response.user)
        return response.user
    }

    func sendBindEmailCode(email: String) async throws -> EmailVerificationCodeResponse {
        try await client.send(try APIRequest.post("/me/email-binding-code", body: BindEmailCodeRequest(email: email)), requiresAuth: true)
    }

    func bindEmail(email: String, code: String) async throws -> UserProfile {
        let response: BindEmailResponse = try await client.send(try APIRequest.post("/me/email-binding", body: BindEmailRequest(email: email, emailVerificationCode: code)), requiresAuth: true)
        replaceCurrentUser(response.user)
        return response.user
    }

    func uploadAvatar(data: Data, fileName: String, mimeType: String) async throws -> UserProfile {
        let response = try await client.uploadAvatar(data: data, fileName: fileName, mimeType: mimeType)
        replaceCurrentUser(response.user)
        return response.user
    }

    private func replaceCurrentUser(_ user: UserProfile) {
        guard let current = sessionStore.currentSession else { return }
        sessionStore.replace(with: Session(
            accessToken: current.accessToken,
            expiresAt: current.expiresAt,
            refreshToken: current.refreshToken,
            refreshExpiresAt: current.refreshExpiresAt,
            user: user
        ))
    }
}

@MainActor
final class GearRepository: GearRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func stats(tab: GearTab) async throws -> GearStatsResponse {
        try await client.send(.get("/me/gears/stats", queryItems: [URLQueryItem(name: "tab", value: tab.rawValue)]), requiresAuth: true)
    }

    func categories(tab: GearTab) async throws -> GearCategoriesResponse {
        try await client.send(.get("/me/gears/categories", queryItems: [URLQueryItem(name: "tab", value: tab.rawValue)]), requiresAuth: true)
    }

    func specKeyRankings(category: GearCategory) async throws -> GearSpecKeyRankingsResponse {
        try await client.send(.get("/me/gears/spec-key-rankings", queryItems: [URLQueryItem(name: "category", value: category.rawValue)]), requiresAuth: true)
    }

    func tagSuggestions(limit: Int) async throws -> GearTagSuggestionsResponse {
        try await client.send(.get("/me/gears/tag-suggestions", queryItems: [URLQueryItem(name: "limit", value: String(limit))]), requiresAuth: true)
    }

    func list(_ request: ListGearsRequest) async throws -> ListGearsResponse {
        try await client.send(.get("/me/gears", queryItems: request.queryItems), requiresAuth: true)
    }

    func get(id: String) async throws -> GearItem {
        try await client.send(.get("/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func create(_ request: CreateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.post("/me/gears", body: request), requiresAuth: true)
    }

    func update(id: String, request: UpdateGearRequest) async throws -> GearItem {
        try await client.send(try APIRequest.patch("/me/gears/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func archive(id: String) async throws {
        let _: EmptyResponse = try await client.sendEmpty(.delete("/me/gears/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func delete(id: String) async throws {
        let _: EmptyResponse = try await client.sendEmpty(.post("/me/gears/\(id.urlPathEscaped)/delete"), requiresAuth: true)
    }

    func undelete(id: String) async throws -> GearItem {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/undelete"), requiresAuth: true)
    }

    func restore(id: String) async throws -> GearItem {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/restore"), requiresAuth: true)
    }
}

@MainActor
final class GearAtlasRepository: GearAtlasRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListGearAtlasRequest) async throws -> ListGearAtlasResponse {
        try await client.send(.get("/gear-atlas", queryItems: request.queryItems), requiresAuth: false)
    }

    func get(id: String) async throws -> GearAtlasPublicItem {
        try await client.send(.get("/gear-atlas/\(id.urlPathEscaped)"), requiresAuth: false)
    }

    func createSubmission(_ request: CreateGearAtlasSubmissionRequest) async throws -> GearAtlasSubmission {
        try await client.send(try APIRequest.post("/me/gear-atlas-submissions", body: request), requiresAuth: true)
    }

    func submitGear(id: String) async throws -> GearAtlasSubmission {
        try await client.send(.post("/me/gears/\(id.urlPathEscaped)/atlas-submission"), requiresAuth: true)
    }

    func mySubmissions(_ request: ListGearAtlasSubmissionsRequest) async throws -> ListGearAtlasSubmissionsResponse {
        try await client.send(.get("/me/gear-atlas-submissions", queryItems: request.queryItems), requiresAuth: true)
    }
}

@MainActor
final class SkillRepository: SkillRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func categories() async throws -> SkillCategoriesResponse {
        try await client.send(.get("/skills"), requiresAuth: false)
    }

    func knots(_ request: ListKnotsRequest) async throws -> KnotListResponse {
        try await client.send(.get("/skills/knots/list", queryItems: request.queryItems), requiresAuth: false)
    }

    func knotDetail(id: String) async throws -> KnotDetail {
        try await client.send(.get("/skills/knots/detail/\(id.urlPathEscaped)"), requiresAuth: false)
    }

    func offlineManifest() async throws -> KnotOfflineManifestResponse {
        try await client.send(.get("/skills/knots/offline-manifest"), requiresAuth: false)
    }

    func knotDisclaimer() async throws -> KnotDisclaimerResponse {
        try await client.send(.get("/me/skills/knots/disclaimer"), requiresAuth: true)
    }

    func acceptKnotDisclaimer(_ request: AcceptKnotDisclaimerRequest) async throws -> KnotDisclaimerResponse {
        try await client.send(try APIRequest.post("/me/skills/knots/disclaimer/acceptance", body: request), requiresAuth: true)
    }
}

@MainActor
final class ContentRepository: ContentRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func gearTemplates() async throws -> GearTemplatesResponse {
        try await client.send(.get("/gear-templates"), requiresAuth: false)
    }
}

@MainActor
final class GearPackingRepository: GearPackingRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListGearPackingListsRequest) async throws -> ListGearPackingListsResponse {
        try await client.send(.get("/me/packing-lists", queryItems: request.queryItems), requiresAuth: true)
    }

    func create(_ request: CreateGearPackingListRequest) async throws -> GearPackingListDetail {
        try await client.send(try APIRequest.post("/me/packing-lists", body: request), requiresAuth: true)
    }

    func get(id: String) async throws -> GearPackingListDetail {
        try await client.send(.get("/me/packing-lists/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func update(id: String, request: UpdateGearPackingListRequest) async throws -> GearPackingListDetail {
        try await client.send(try APIRequest.patch("/me/packing-lists/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func delete(id: String) async throws {
        try await client.sendEmpty(.delete("/me/packing-lists/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func addItems(id: String, request: AddGearPackingItemsRequest) async throws -> GearPackingListDetail {
        try await client.send(try APIRequest.post("/me/packing-lists/\(id.urlPathEscaped)/items", body: request), requiresAuth: true)
    }

    func updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest) async throws -> GearPackingListDetail {
        try await client.send(try APIRequest.patch("/me/packing-lists/\(id.urlPathEscaped)/items/\(itemId.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func deleteItem(id: String, itemId: String) async throws -> GearPackingListDetail {
        try await client.send(.delete("/me/packing-lists/\(id.urlPathEscaped)/items/\(itemId.urlPathEscaped)"), requiresAuth: true)
    }
}

@MainActor
final class TripRepository: TripRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListTripsRequest) async throws -> ListTripsResponse {
        try await client.send(.get("/me/trips", queryItems: request.queryItems), requiresAuth: true)
    }

    func homeHighlight(today: String) async throws -> TripHomeHighlightResponse {
        try await client.send(.get("/me/trips/home-highlight", queryItems: [URLQueryItem(name: "today", value: today)]), requiresAuth: true)
    }

    func create(_ request: CreateTripRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.post("/me/trips", body: request), requiresAuth: true)
    }

    func get(id: String) async throws -> TripDetail {
        try await client.send(.get("/me/trips/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func update(id: String, request: UpdateTripRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.patch("/me/trips/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func delete(id: String) async throws {
        try await client.sendEmpty(.delete("/me/trips/\(id.urlPathEscaped)"), requiresAuth: true)
    }

    func updateSections(id: String, request: UpdateTripSectionsRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.patch("/me/trips/\(id.urlPathEscaped)/sections", body: request), requiresAuth: true)
    }

    func createInvitation(id: String) async throws -> CreateTripInvitationResponse {
        try await client.send(.post("/me/trips/\(id.urlPathEscaped)/invitations"), requiresAuth: true)
    }

    func acceptInvitation(token: String) async throws -> TripDetail {
        try await client.send(.post("/me/trip-invitations/\(token.urlPathEscaped)/accept"), requiresAuth: true)
    }

    func importPackingList(id: String, request: ImportTripPackingListRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.post("/me/trips/\(id.urlPathEscaped)/personal-gear/import-packing-list", body: request), requiresAuth: true)
    }

    func createRecord(id: String, collectionPath: String, request: TripRecordCreateRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.post("/me/trips/\(id.urlPathEscaped)/\(collectionPath)", body: request), requiresAuth: true)
    }

    func updateRecord(id: String, collectionPath: String, recordId: String, request: TripRecordPatchRequest) async throws -> TripDetail {
        try await client.send(try APIRequest.patch("/me/trips/\(id.urlPathEscaped)/\(collectionPath)/\(recordId.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func deleteRecord(id: String, collectionPath: String, recordId: String) async throws -> TripDetail {
        try await client.send(.delete("/me/trips/\(id.urlPathEscaped)/\(collectionPath)/\(recordId.urlPathEscaped)"), requiresAuth: true)
    }

    func convertToOutdoorExperience(id: String, today: String) async throws -> OutdoorExperience {
        try await client.send(.post("/me/trips/\(id.urlPathEscaped)/convert-to-outdoor-experience", queryItems: [URLQueryItem(name: "today", value: today)]), requiresAuth: true)
    }

    func listOutdoorExperiences() async throws -> ListOutdoorExperiencesResponse {
        try await client.send(.get("/me/outdoor-experiences"), requiresAuth: true)
    }

    func createOutdoorExperience(_ request: OutdoorExperienceRequest) async throws -> OutdoorExperience {
        try await client.send(try APIRequest.post("/me/outdoor-experiences", body: request), requiresAuth: true)
    }

    func updateOutdoorExperience(id: String, request: OutdoorExperienceRequest) async throws -> OutdoorExperience {
        try await client.send(try APIRequest.patch("/me/outdoor-experiences/\(id.urlPathEscaped)", body: request), requiresAuth: true)
    }

    func deleteOutdoorExperience(id: String) async throws {
        try await client.sendEmpty(.delete("/me/outdoor-experiences/\(id.urlPathEscaped)"), requiresAuth: true)
    }
}

@MainActor
final class ProfileRepository: ProfileRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func outdoorProfile() async throws -> OutdoorProfileResponse {
        try await client.send(.get("/me/profile/outdoor"), requiresAuth: true)
    }

    func updateOutdoorProfile(_ request: UpdateOutdoorProfileRequest) async throws -> OutdoorProfileResponse {
        try await client.send(try APIRequest.patch("/me/profile/outdoor", body: request), requiresAuth: true)
    }
}

@MainActor
final class RoadmapRepository: RoadmapRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListRoadmapRequest, includeUserState: Bool) async throws -> ListRoadmapResponse {
        try await client.send(.get(includeUserState ? "/me/roadmap" : "/roadmap", queryItems: request.queryItems), requiresAuth: includeUserState)
    }

    func vote(id: String) async throws -> RoadmapItem {
        try await client.send(.put("/me/roadmap/\(id.urlPathEscaped)/vote"), requiresAuth: true)
    }

    func unvote(id: String) async throws -> RoadmapItem {
        try await client.send(.delete("/me/roadmap/\(id.urlPathEscaped)/vote"), requiresAuth: true)
    }

    func subscribe(id: String) async throws -> RoadmapItem {
        try await client.send(.put("/me/roadmap/\(id.urlPathEscaped)/subscription"), requiresAuth: true)
    }

    func unsubscribe(id: String) async throws -> RoadmapItem {
        try await client.send(.delete("/me/roadmap/\(id.urlPathEscaped)/subscription"), requiresAuth: true)
    }
}

@MainActor
final class FeedbackRepository: FeedbackRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func uploadImage(data: Data, fileName: String, mimeType: String) async throws -> UploadImageInfo {
        try await client.uploadFeedbackImage(data: data, fileName: fileName, mimeType: mimeType)
    }

    func create(_ request: CreateFeedbackRequest) async throws -> FeedbackResponse {
        try await client.send(try APIRequest.post("/me/feedback", body: request), requiresAuth: true)
    }
}

@MainActor
final class ClientVersionRepository: ClientVersionRepositorying {
    private let client: APIClient

    init(client: APIClient) { self.client = client }

    func list(_ request: ListClientVersionsRequest) async throws -> ListClientVersionsResponse {
        try await client.send(.get("/client-versions", queryItems: request.queryItems), requiresAuth: false)
    }

    func current(clientKey: ClientKey) async throws -> ClientVersion {
        try await client.send(.get("/client-versions/current", queryItems: [URLQueryItem(name: "client_key", value: clientKey.rawValue)]), requiresAuth: false)
    }
}

private extension String {
    var urlPathEscaped: String {
        addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? self
    }
}
