package com.rustella.stellartrail.screenshot

import android.content.Context
import android.content.Intent
import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.config.InMemoryAppConfigStore
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.core.network.InMemoryOfflineHttpCacheStore
import com.rustella.stellartrail.core.network.OfflineHttpCacheStore
import com.rustella.stellartrail.core.session.InMemorySessionStore
import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.core.theme.InMemoryThemeRepository
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.packing.PackingRepositoryContract
import com.rustella.stellartrail.data.profile.ProfileRepositoryContract
import com.rustella.stellartrail.data.skills.KnotCacheStatus
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
import com.rustella.stellartrail.data.trip.TripRepositoryContract
import com.rustella.stellartrail.di.AppContainer
import com.rustella.stellartrail.domain.atlas.CreateGearAtlasSubmissionRequest
import com.rustella.stellartrail.domain.atlas.GearAtlasPublicItem
import com.rustella.stellartrail.domain.atlas.GearAtlasSourceType
import com.rustella.stellartrail.domain.atlas.GearAtlasStatus
import com.rustella.stellartrail.domain.atlas.GearAtlasSubmission
import com.rustella.stellartrail.domain.atlas.ListGearAtlasRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasResponse
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsRequest
import com.rustella.stellartrail.domain.atlas.ListGearAtlasSubmissionsResponse
import com.rustella.stellartrail.domain.auth.CaptchaChallengeResponse
import com.rustella.stellartrail.domain.auth.EmailVerificationCodeResponse
import com.rustella.stellartrail.domain.auth.LoginResponse
import com.rustella.stellartrail.domain.auth.LoginUser
import com.rustella.stellartrail.domain.auth.RegisterRequest
import com.rustella.stellartrail.domain.auth.SmsCodeResponse
import com.rustella.stellartrail.domain.auth.SmsRegisterRequest
import com.rustella.stellartrail.domain.auth.UserSession
import com.rustella.stellartrail.domain.gear.CreateGearRequest
import com.rustella.stellartrail.domain.gear.GearCategoriesResponse
import com.rustella.stellartrail.domain.gear.GearCategory
import com.rustella.stellartrail.domain.gear.GearCategoryFilter
import com.rustella.stellartrail.domain.gear.GearItem
import com.rustella.stellartrail.domain.gear.GearShareStatus
import com.rustella.stellartrail.domain.gear.GearStatsResponse
import com.rustella.stellartrail.domain.gear.GearStatus
import com.rustella.stellartrail.domain.gear.GearTab
import com.rustella.stellartrail.domain.gear.GearTemplate
import com.rustella.stellartrail.domain.gear.GearTemplateCategory
import com.rustella.stellartrail.domain.gear.ListGearTemplatesResponse
import com.rustella.stellartrail.domain.gear.ListGearsRequest
import com.rustella.stellartrail.domain.gear.ListGearsResponse
import com.rustella.stellartrail.domain.gear.UpdateGearRequest
import com.rustella.stellartrail.domain.packing.AddGearPackingItemsRequest
import com.rustella.stellartrail.domain.packing.CreateGearPackingListRequest
import com.rustella.stellartrail.domain.packing.GearPackingListDetail
import com.rustella.stellartrail.domain.packing.GearPackingListItem
import com.rustella.stellartrail.domain.packing.GearPackingListStats
import com.rustella.stellartrail.domain.packing.GearPackingListSummary
import com.rustella.stellartrail.domain.packing.ListGearPackingListsRequest
import com.rustella.stellartrail.domain.packing.ListGearPackingListsResponse
import com.rustella.stellartrail.domain.packing.UpdateGearPackingItemRequest
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.domain.profile.ListOutdoorExperiencesResponse
import com.rustella.stellartrail.domain.profile.ListRoadmapResponse
import com.rustella.stellartrail.domain.profile.OutdoorExperienceRequest
import com.rustella.stellartrail.domain.profile.OutdoorProfile
import com.rustella.stellartrail.domain.profile.OutdoorProfileResponse
import com.rustella.stellartrail.domain.profile.ProfileUserResponse
import com.rustella.stellartrail.domain.profile.RoadmapItem
import com.rustella.stellartrail.domain.profile.RoadmapStatusFilter
import com.rustella.stellartrail.domain.skills.FavoriteKnotItem
import com.rustella.stellartrail.domain.skills.FavoriteKnotStatusResponse
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotMediaAsset
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.KnotTaxonomyItem
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsRequest
import com.rustella.stellartrail.domain.skills.ListFavoriteSkillsResponse
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
import com.rustella.stellartrail.domain.trip.CreateTripInvitationResponse
import com.rustella.stellartrail.domain.trip.CreateTripRequest
import com.rustella.stellartrail.domain.trip.ImportTripPackingListRequest
import com.rustella.stellartrail.domain.trip.ListTripsRequest
import com.rustella.stellartrail.domain.trip.ListTripsResponse
import com.rustella.stellartrail.domain.trip.OutdoorExperience
import com.rustella.stellartrail.domain.trip.TripBudgetItem
import com.rustella.stellartrail.domain.trip.TripDetail
import com.rustella.stellartrail.domain.trip.TripFoodMeal
import com.rustella.stellartrail.domain.trip.TripGoalItem
import com.rustella.stellartrail.domain.trip.TripHomeHighlightItem
import com.rustella.stellartrail.domain.trip.TripHomeHighlightResponse
import com.rustella.stellartrail.domain.trip.TripHomeHighlightStatus
import com.rustella.stellartrail.domain.trip.TripItineraryDay
import com.rustella.stellartrail.domain.trip.TripInvitation
import com.rustella.stellartrail.domain.trip.TripMedicalItem
import com.rustella.stellartrail.domain.trip.TripMember
import com.rustella.stellartrail.domain.trip.TripMemberGearWeightSummary
import com.rustella.stellartrail.domain.trip.TripMemberProfile
import com.rustella.stellartrail.domain.trip.TripPersonalGearItem
import com.rustella.stellartrail.domain.trip.TripRecordKind
import com.rustella.stellartrail.domain.trip.TripRescueContact
import com.rustella.stellartrail.domain.trip.TripRouteSegment
import com.rustella.stellartrail.domain.trip.TripSafetyRisk
import com.rustella.stellartrail.domain.trip.TripSectionKey
import com.rustella.stellartrail.domain.trip.TripSegmentAssignment
import com.rustella.stellartrail.domain.trip.TripSharedGearDemand
import com.rustella.stellartrail.domain.trip.TripSummary
import com.rustella.stellartrail.domain.trip.TripTimeBucket
import com.rustella.stellartrail.domain.trip.TripType
import com.rustella.stellartrail.domain.trip.UpdateTripRequest
import com.rustella.stellartrail.domain.trip.UpdateTripSectionsRequest
import com.rustella.stellartrail.domain.trip.emptyFieldVersions
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.json.JsonObject

object ScreenshotFixtures {
    private const val EXTRA_ENABLED = "stellartrail.screenshot"
    private const val EXTRA_STATE = "stellartrail.screenshot.state"
    private const val EXTRA_ROUTE = "stellartrail.route"

    data class FixtureLaunch(
        val container: AppContainer,
        val startDestination: String,
    )

    fun createContainer(context: Context, intent: Intent): FixtureLaunch? {
        if (!intent.getBooleanExtra(EXTRA_ENABLED, false)) return null
        val loggedIn = intent.getStringExtra(EXTRA_STATE) == "logged-in"
        val sessionStore = InMemorySessionStore(if (loggedIn) fixtureSession() else null)
        return FixtureLaunch(
            container = FixtureAppContainer(context.applicationContext, sessionStore),
            startDestination = startDestination(intent),
        )
    }

    fun startDestination(intent: Intent): String = intent.getStringExtra(EXTRA_ROUTE)?.takeIf { it.isNotBlank() } ?: "home"
}

private class FixtureAppContainer(
    context: Context,
    override val sessionStore: SessionStore,
) : AppContainer {
    override val configStore: AppConfigStore = InMemoryAppConfigStore(
        AppConfig(baseUrl = "https://fixture.stellartrail.local", assetsBaseUrl = "https://assets.stellartrail.local"),
    )
    override val themeRepository: ThemeRepository = InMemoryThemeRepository(ThemeMode.LIGHT)
    override val offlineHttpCacheStore: OfflineHttpCacheStore = InMemoryOfflineHttpCacheStore()
    override val apiClient: ApiClient = ApiClient(configProvider = { configStore.config.value })
    override val authRepository: AuthRepositoryContract = FixtureAuthRepository(sessionStore)
    override val gearRepository: GearRepositoryContract = FixtureGearRepository()
    override val gearAtlasRepository: GearAtlasRepositoryContract = FixtureGearAtlasRepository()
    override val packingRepository: PackingRepositoryContract = FixturePackingRepository()
    override val skillRepository: SkillRepositoryContract = FixtureSkillRepository()
    override val tripRepository: TripRepositoryContract = FixtureTripRepository()
    override val profileRepository: ProfileRepositoryContract = FixtureProfileRepository(sessionStore)
}

private fun fixtureSession(): UserSession = UserSession(
    accessToken = "fixture-access-token",
    expiresAt = "2099-01-01T00:00:00Z",
    refreshToken = "fixture-refresh-token",
    refreshExpiresAt = "2099-01-02T00:00:00Z",
    user = LoginUser(
        id = "fixture-user",
        username = "trail_user",
        email = "trail@example.test",
        phone = "13800000000",
        nickname = "星野徒步者",
    ),
)

private class FixtureAuthRepository(private val sessionStore: SessionStore) : AuthRepositoryContract {
    override val session: StateFlow<UserSession?> = sessionStore.session
    override suspend fun sendEmailCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
    override suspend fun sendEmailLoginCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
    override suspend fun loginWithEmailCode(email: String, emailCode: String): LoginResponse = login(email)
    override suspend fun sendPasswordResetCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
    override suspend fun resetPassword(email: String, emailCode: String, password: String, confirmPassword: String): LoginResponse = login(email)
    override suspend fun sendBindEmailCode(email: String): EmailVerificationCodeResponse = codeResponse(email)
    override suspend fun bindEmail(email: String, emailCode: String): LoginUser {
        val user = sessionStore.session.value?.user?.copy(email = email) ?: LoginUser(
            id = "fixture-user",
            username = "trail_user",
            email = email,
            phone = "13800000000",
            nickname = "星野徒步者",
        )
        sessionStore.session.value?.copy(user = user)?.let(sessionStore::save)
        return user
    }
    override suspend fun sendSmsRegistrationCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-register-ticket")
    override suspend fun smsRegister(request: SmsRegisterRequest): LoginResponse = login(request.phone)
    override suspend fun sendSmsLoginCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-login-ticket")
    override suspend fun smsLogin(phone: String, smsTicket: String, smsCode: String): LoginResponse = login(phone)
    override suspend fun sendSmsPasswordResetCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-reset-ticket")
    override suspend fun smsResetPassword(
        phone: String,
        smsTicket: String,
        smsCode: String,
        password: String,
        confirmPassword: String,
    ): LoginResponse = login(phone)
    override suspend fun sendBindPhoneCode(phone: String): SmsCodeResponse = smsCodeResponse(phone, "sms-bind-ticket")
    override suspend fun sendRebindCurrentPhoneCode(): SmsCodeResponse = smsCodeResponse("13800000000", "sms-current-ticket")
    override suspend fun bindPhone(
        phone: String,
        smsTicket: String,
        smsCode: String,
        currentSmsTicket: String?,
        currentSmsCode: String?,
    ): LoginUser {
        val user = sessionStore.session.value?.user?.copy(phone = phone) ?: LoginUser(
            id = "fixture-user",
            username = "trail_user",
            email = "trail@example.test",
            phone = phone,
            nickname = "星野徒步者",
        )
        sessionStore.session.value?.copy(user = user)?.let(sessionStore::save)
        return user
    }
    override suspend fun createCaptcha(account: String): CaptchaChallengeResponse =
        CaptchaChallengeResponse("fixture-ticket", "image", "<svg />", "2099-01-01T00:00:00Z", "1234")
    override suspend fun login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?): LoginResponse = login(account)
    override suspend fun register(request: RegisterRequest): LoginResponse = login(request.email)
    override fun logout() = sessionStore.clear()

    private fun codeResponse(email: String) = EmailVerificationCodeResponse(email, "2099-01-01T00:10:00Z", "246810")
    private fun smsCodeResponse(phone: String, ticket: String) = SmsCodeResponse(phone, ticket, "2099-01-01T00:10:00Z", "246810")
    private fun login(account: String): LoginResponse = LoginResponse(
        accessToken = "fixture-access-token",
        expiresAt = "2099-01-01T00:00:00Z",
        refreshToken = "fixture-refresh-token",
        refreshExpiresAt = "2099-01-02T00:00:00Z",
        user = LoginUser(id = "fixture-user", username = account, email = "trail@example.test", phone = "13800000000", nickname = "星野徒步者"),
    ).also(sessionStore::save)
}

private class FixtureProfileRepository(private val sessionStore: SessionStore) : ProfileRepositoryContract {
    private val experiences = listOf(
        OutdoorExperience(
            id = "experience-1",
            userId = "fixture-user",
            sourceTripId = null,
            tripType = TripType.SOLO,
            title = "罗浮山三天两夜重装",
            startDate = "2026-05-01",
            endDate = "2026-05-03",
            dayCount = 3,
            companionCount = 2,
            routeSummary = "罗浮山环线，夜宿拨云寺附近。",
            gearSummary = "轻量雨衣够用，睡袋夜间略冷。",
            foodSummary = "早餐偏少，电解质很有帮助。",
            budgetSummary = "包车 300，人均约 120。",
            notes = "第二天下午注意补水。",
            createdAt = "2026-05-04T00:00:00Z",
            updatedAt = "2026-05-04T00:00:00Z",
        ),
    )

    private val roadmapItems = listOf(
        RoadmapItem(
            id = "roadmap-android-trip",
            title = "Android 行程协作完善",
            summary = "补齐成员、装备、食品、医药、安全预案与预算协作。",
            details = "优先对齐小程序端的信息架构和状态反馈。",
            category = "routes",
            status = "building",
            priority = 1,
            voteCount = 36,
            subscriptionCount = 12,
            isVoted = true,
            isSubscribed = false,
            createdAt = "2026-05-01T00:00:00Z",
            updatedAt = "2026-05-20T00:00:00Z",
        ),
        RoadmapItem(
            id = "roadmap-offline",
            title = "离线路线资料包",
            summary = "把路线资料、绳结教程和装备清单缓存到本地。",
            category = "offline",
            status = "planned",
            priority = 2,
            voteCount = 24,
            subscriptionCount = 8,
            createdAt = "2026-05-01T00:00:00Z",
            updatedAt = "2026-05-20T00:00:00Z",
        ),
    )

    override suspend fun currentProfile(): ProfileUserResponse = ProfileUserResponse(
        sessionStore.session.value?.user ?: LoginUser(id = "fixture-user", username = "trail_user", email = "trail@example.test", nickname = "星野徒步者"),
    )

    override suspend fun outdoorProfile(): OutdoorProfileResponse = OutdoorProfileResponse(
        OutdoorProfile(
            userId = "fixture-user",
            outdoorId = "星星",
            realName = "星野",
            gender = "女",
            birthDate = "1996-03-21",
            heightCm = 168,
            phone = "13800000000",
            emergencyContact = "家属",
            emergencyContactRelationship = "家人",
            emergencyPhone = "13900000000",
            bloodType = "O",
            medicalHistory = "无",
            allergyHistory = "无",
            medicalResponseNote = "随身携带常用药。",
            dietPreference = "不吃辛辣",
            insurancePolicyNo = "TEST-OUTDOOR-001",
            insuranceCompanyPhone = "4000000000",
            createdAt = "2026-05-01T00:00:00Z",
            updatedAt = "2026-05-20T00:00:00Z",
        ),
    )

    override suspend fun updateOutdoorProfile(request: JsonObject): OutdoorProfileResponse = outdoorProfile()
    override suspend fun listOutdoorExperiences(): ListOutdoorExperiencesResponse = ListOutdoorExperiencesResponse(experiences)
    override suspend fun createOutdoorExperience(request: OutdoorExperienceRequest): OutdoorExperience =
        experiences.first().copy(id = "experience-created", title = request.title)
    override suspend fun updateOutdoorExperience(id: String, request: OutdoorExperienceRequest): OutdoorExperience =
        experiences.first().copy(id = id, title = request.title)
    override suspend fun deleteOutdoorExperience(id: String) = Unit
    override suspend fun listRoadmap(isLoggedIn: Boolean, status: RoadmapStatusFilter): ListRoadmapResponse =
        ListRoadmapResponse(
            roadmapItems.filter { status.apiValue == null || it.status == status.apiValue },
        )
    override suspend fun voteRoadmapItem(id: String): RoadmapItem =
        roadmapItems.first { it.id == id }.copy(isVoted = true, voteCount = itVoteCount(id) + 1)
    override suspend fun unvoteRoadmapItem(id: String): RoadmapItem =
        roadmapItems.first { it.id == id }.copy(isVoted = false, voteCount = itVoteCount(id).coerceAtLeast(1) - 1)
    override suspend fun subscribeRoadmapItem(id: String): RoadmapItem =
        roadmapItems.first { it.id == id }.copy(isSubscribed = true)
    override suspend fun unsubscribeRoadmapItem(id: String): RoadmapItem =
        roadmapItems.first { it.id == id }.copy(isSubscribed = false)

    private fun itVoteCount(id: String): Int = roadmapItems.first { it.id == id }.voteCount
}

private class FixtureGearRepository : GearRepositoryContract {
    private val gears = fixtureGears()

    override suspend fun listTemplates(): ListGearTemplatesResponse = ListGearTemplatesResponse(fixtureTemplates())
    override suspend fun listCategories(tab: GearTab): GearCategoriesResponse = GearCategoriesResponse(
        listOf(
            GearCategoryFilter("all", "全部装备", gears.size),
            GearCategoryFilter(GearCategory.BACKPACK_SYSTEM.name.lowercase(), GearCategory.BACKPACK_SYSTEM.label, 1),
            GearCategoryFilter(GearCategory.LIGHTING_SYSTEM.name.lowercase(), GearCategory.LIGHTING_SYSTEM.label, 1),
        ),
    )
    override suspend fun stats(tab: GearTab): GearStatsResponse = GearStatsResponse(
        currentCount = 35,
        archivedCount = 2,
        totalValueCents = 3_106_442,
        totalWeightG = 16_090,
    )
    override suspend fun list(request: ListGearsRequest): ListGearsResponse = ListGearsResponse(
        items = gears.map { item ->
            com.rustella.stellartrail.domain.gear.GearSummary(
                id = item.id,
                category = item.category,
                categoryLabel = item.category.label,
                name = item.name,
                brand = item.brand,
                model = item.model,
                status = item.status,
                statusLabel = item.status.label,
                weightG = item.weightG,
                officialPriceCents = item.officialPriceCents,
                officialPriceCurrency = item.officialPriceCurrency,
                purchasePriceCents = item.purchasePriceCents,
                purchasePriceCurrency = item.purchasePriceCurrency,
                purchaseDate = item.purchaseDate,
                specs = item.specs,
                tags = item.tags,
                tagColors = item.tagColors,
                createdAt = item.createdAt,
                updatedAt = item.updatedAt,
            )
        },
    )
    override suspend fun get(id: String): GearItem = gears.firstOrNull { it.id == id } ?: gears.first()
    override suspend fun create(request: CreateGearRequest): GearItem = gears.first().copy(id = "fixture-created", name = request.name)
    override suspend fun update(id: String, request: UpdateGearRequest): GearItem = get(id).copy(name = request.name ?: get(id).name)
    override suspend fun archive(id: String) = Unit
    override suspend fun delete(id: String) = Unit
    override suspend fun undelete(id: String): GearItem = get(id).copy(isDeleted = false)
    override suspend fun restore(id: String): GearItem = get(id).copy(status = GearStatus.AVAILABLE, archivedAt = null)
}

private class FixtureGearAtlasRepository : GearAtlasRepositoryContract {
    private val items = fixtureAtlas()
    override suspend fun list(request: ListGearAtlasRequest): ListGearAtlasResponse = ListGearAtlasResponse(
        items = items.filter { request.category == null || it.category == request.category }.filter {
            request.query.isNullOrBlank() || it.name.contains(request.query, ignoreCase = true) ||
                it.brand.orEmpty().contains(request.query, ignoreCase = true)
        },
    )
    override suspend fun get(id: String): GearAtlasPublicItem = items.firstOrNull { it.id == id } ?: items.first()
    override suspend fun createSubmission(request: CreateGearAtlasSubmissionRequest): GearAtlasSubmission = submission(request.name, null)
    override suspend fun createSubmissionFromGear(id: String): GearAtlasSubmission = submission("NITECORE NU25 UL", id)
    override suspend fun listMySubmissions(request: ListGearAtlasSubmissionsRequest): ListGearAtlasSubmissionsResponse =
        ListGearAtlasSubmissionsResponse(listOf(submission("NITECORE NU25 UL", "gear-headlamp")))

    private fun submission(name: String, gearId: String?) = GearAtlasSubmission(
        id = "atlas-submission-1",
        category = GearCategory.LIGHTING_SYSTEM,
        categoryLabel = GearCategory.LIGHTING_SYSTEM.label,
        name = name,
        brand = "NITECORE",
        model = "NU25 UL",
        description = "轻量头灯，适合夜间行走与营地照明。",
        weightG = 45,
        officialPriceCents = 29800,
        officialPriceCurrency = "CNY",
        specs = mapOf("续航" to "45h", "防水" to "IP66"),
        approvedAt = null,
        createdAt = "2026-05-01T08:00:00Z",
        updatedAt = "2026-05-01T08:00:00Z",
        sourceType = GearAtlasSourceType.USER_GEAR,
        sourceUserGearId = gearId,
        status = GearAtlasStatus.PENDING,
    )
}

private class FixturePackingRepository : PackingRepositoryContract {
    private var detail = fixturePackingList()
    override suspend fun list(request: ListGearPackingListsRequest): ListGearPackingListsResponse = ListGearPackingListsResponse(
        listOf(
            GearPackingListSummary(
                id = detail.id,
                title = detail.title,
                description = detail.description,
                targetDate = detail.targetDate,
                totalItems = detail.stats.totalItems,
                packedItems = detail.stats.packedItems,
                totalWeightG = detail.stats.totalWeightG,
                packedWeightG = detail.stats.packedWeightG,
            ),
        ),
    )
    override suspend fun create(request: CreateGearPackingListRequest): GearPackingListDetail {
        detail = detail.copy(id = "packing-created", title = request.title, description = request.description)
        return detail
    }
    override suspend fun get(id: String): GearPackingListDetail = detail
    override suspend fun update(id: String, request: CreateGearPackingListRequest): GearPackingListDetail {
        detail = detail.copy(title = request.title, description = request.description, targetDate = request.targetDate)
        return detail
    }
    override suspend fun delete(id: String) = Unit
    override suspend fun addItems(id: String, request: AddGearPackingItemsRequest): GearPackingListDetail = detail
    override suspend fun updateItem(id: String, itemId: String, request: UpdateGearPackingItemRequest): GearPackingListDetail {
        detail = detail.copy(items = detail.items.map { if (it.id == itemId) it.copy(packedQuantity = request.packedQuantity) else it })
        return detail
    }
    override suspend fun removeItem(id: String, itemId: String): GearPackingListDetail = detail
}

private class FixtureSkillRepository : SkillRepositoryContract {
    override val knotCacheStatus: StateFlow<KnotCacheStatus> = MutableStateFlow(KnotCacheStatus(cachedKnotCount = 1, lastUpdatedAtMillis = 1000L))
    private val categories = listOf(
        SkillCategorySummary("knots", "knots", "绳结", "常用露营、钓鱼、连接和固定绳结，按场景快速复习。", 3, "/api/v1/skills/knots"),
    )
    private val knots = fixtureKnots()
    private var favoriteStatus = FavoriteKnotStatusResponse(
        skillCategory = "knots",
        knotId = "taut-line",
        isFavorited = false,
    )

    override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = SkillCategoriesResponse(categories)
    override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse =
        KnotListResponse(locale, knots, PageInfo(limit = request.limit, offset = request.offset))
    override suspend fun listFavoriteSkills(locale: SkillLocale, request: ListFavoriteSkillsRequest): ListFavoriteSkillsResponse =
        ListFavoriteSkillsResponse(
            locale = locale,
            items = listOf(FavoriteKnotItem("knots", "2026-05-01T00:00:00Z", knots.first())),
            page = PageInfo(limit = request.limit, offset = request.offset),
        )
    override suspend fun getFavoriteKnotStatus(id: String): FavoriteKnotStatusResponse =
        favoriteStatus.copy(knotId = id)
    override suspend fun favoriteKnot(id: String): FavoriteKnotStatusResponse {
        favoriteStatus = FavoriteKnotStatusResponse("knots", id, true, "2026-05-01T00:00:00Z")
        return favoriteStatus
    }
    override suspend fun unfavoriteKnot(id: String): FavoriteKnotStatusResponse {
        favoriteStatus = FavoriteKnotStatusResponse("knots", id, false, null)
        return favoriteStatus
    }
    override suspend fun knotDetail(id: String, locale: SkillLocale): KnotDetail = knots.firstOrNull { it.id == id }?.let {
        KnotDetail(
            id = it.id,
            slug = it.slug,
            title = it.title,
            summary = it.summary,
            categories = it.categories,
            types = it.types,
            media = it.media,
            href = it.href,
            description = "常用于制作固定绳圈，受力后稳定，适合帐篷拉线、临时固定和救援场景。",
            steps = listOf("在主绳上绕出一个小圈。", "将绳头从圈中穿出。", "绕过主绳后再穿回小圈。", "整理绳圈并缓慢收紧。"),
            locale = locale,
        )
    } ?: knotDetail("bowline", locale)
    override suspend fun cacheAllKnots(locale: SkillLocale): KnotCacheStatus = KnotCacheStatus(knots.size, 1000L)
    override suspend fun clearKnotCache(): KnotCacheStatus = KnotCacheStatus()
    override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
}

private class FixtureTripRepository : TripRepositoryContract {
    private var detail = fixtureTripDetail()

    override suspend fun list(request: ListTripsRequest): ListTripsResponse = ListTripsResponse(listOf(detail.trip))

    override suspend fun homeHighlight(today: String): TripHomeHighlightResponse = TripHomeHighlightResponse(
        TripHomeHighlightItem(
            trip = detail.trip,
            status = TripHomeHighlightStatus.UPCOMING,
            daysUntilStart = 7,
            daysUntilEnd = 9,
        ),
    )

    override suspend fun create(request: CreateTripRequest): TripDetail {
        detail = fixtureTripDetail().copy(
            trip = fixtureTripDetail().trip.copy(
                id = "trip-created",
                tripType = request.tripType,
                title = request.title,
                startDate = request.startDate,
                endDate = request.endDate,
                description = request.description,
            ),
        )
        return detail
    }

    override suspend fun get(id: String): TripDetail = detail
    override suspend fun update(id: String, request: UpdateTripRequest): TripDetail {
        detail = detail.copy(
            trip = detail.trip.copy(
                title = request.title ?: detail.trip.title,
                startDate = request.startDate ?: detail.trip.startDate,
                endDate = request.endDate ?: detail.trip.endDate,
                description = request.description ?: detail.trip.description,
            ),
        )
        return detail
    }

    override suspend fun delete(id: String) = Unit
    override suspend fun updateSections(id: String, request: UpdateTripSectionsRequest): TripDetail {
        detail = detail.copy(sections = request.enabledSections, trip = detail.trip.copy(enabledSections = request.enabledSections))
        return detail
    }

    override suspend fun createInvitation(id: String): CreateTripInvitationResponse = CreateTripInvitationResponse(
        TripInvitation(
            id = "invite-1",
            planId = id,
            token = "11111111-2222-3333-4444-555555555555",
            createdByUserId = "fixture-user",
        ),
    )

    override suspend fun acceptInvitation(token: String): TripDetail = detail
    override suspend fun convertToOutdoorExperience(id: String): OutdoorExperience = OutdoorExperience(
        id = "experience-1",
        userId = "fixture-user",
        sourceTripId = id,
        tripType = detail.trip.tripType,
        title = detail.trip.title,
    )
    override suspend fun updateMember(id: String, memberId: String, request: JsonObject): TripDetail = detail
    override suspend fun removeMember(id: String, memberId: String): TripDetail = detail
    override suspend fun importPackingList(id: String, request: ImportTripPackingListRequest): TripDetail = detail
    override suspend fun createRecord(id: String, collectionPath: String, request: JsonObject): TripDetail = detail
    override suspend fun updateRecord(id: String, collectionPath: String, recordId: String, request: JsonObject): TripDetail = detail
    override suspend fun deleteRecord(id: String, collectionPath: String, recordId: String): TripDetail = detail
    override suspend fun bindSharedGearDemandMyGear(id: String, itemId: String, request: JsonObject): TripDetail = detail
    override suspend fun fillSharedGearDemandConcreteGear(id: String, itemId: String, request: JsonObject): TripDetail = detail
}

private fun fixtureTemplates() = listOf(
    GearTemplate(
        id = "weekend-hike",
        title = "周末轻徒步清单",
        categories = listOf(
            GearTemplateCategory("carry", "背负与收纳", listOf("20L 背包", "防水收纳袋", "垃圾袋")),
            GearTemplateCategory("safety", "安全与照明", listOf("头灯", "急救包", "备用电池")),
        ),
    ),
    GearTemplate(
        id = "overnight",
        title = "一晚露营清单",
        categories = listOf(
            GearTemplateCategory("sleep", "睡眠系统", listOf("三季帐篷", "睡袋", "防潮垫")),
            GearTemplateCategory("kitchen", "餐厨系统", listOf("炉头", "气罐", "钛锅")),
        ),
    ),
)

private fun fixtureGears() = listOf(
    GearItem(
        id = "gear-headlamp",
        userId = "fixture-user",
        category = GearCategory.LIGHTING_SYSTEM,
        name = "NITECORE NU25 UL",
        brand = "NITECORE",
        model = "NU25 UL",
        description = "夜间行走和营地照明备用。",
        weightG = 45,
        officialPriceCents = 29800,
        officialPriceCurrency = "CNY",
        purchaseDate = "2026-04-02",
        purchasePriceCents = 26800,
        purchasePriceCurrency = "CNY",
        purchaseLocation = "山野仓库",
        status = GearStatus.AVAILABLE,
        storageLocation = "装备柜 A1",
        specs = mapOf("续航" to "45h", "防水" to "IP66"),
        tags = listOf("轻量", "夜行"),
        shareEnabled = false,
        shareStatus = GearShareStatus.NOT_SHARED,
        notes = "出发前充电。",
        createdAt = "2026-04-02T12:00:00Z",
        updatedAt = "2026-04-10T12:00:00Z",
    ),
    GearItem(
        id = "gear-pack",
        userId = "fixture-user",
        category = GearCategory.BACKPACK_SYSTEM,
        name = "Osprey Talon 22",
        brand = "Osprey",
        model = "Talon 22",
        description = "轻徒步一日包。",
        weightG = 900,
        officialPriceCents = 129900,
        officialPriceCurrency = "CNY",
        purchaseDate = "2026-03-12",
        purchasePriceCents = 118000,
        purchasePriceCurrency = "CNY",
        purchaseLocation = "旗舰店",
        status = GearStatus.IN_USE,
        storageLocation = "玄关挂架",
        specs = mapOf("容量" to "22L", "背长" to "S/M"),
        tags = listOf("一日线", "通勤"),
        shareEnabled = false,
        shareStatus = GearShareStatus.NOT_SHARED,
        createdAt = "2026-03-12T12:00:00Z",
        updatedAt = "2026-04-05T12:00:00Z",
    ),
)

private fun fixtureTripDetail(): TripDetail {
    val sections = listOf(
        TripSectionKey.MEMBERS,
        TripSectionKey.PERSONAL_GEAR,
        TripSectionKey.ITINERARY,
        TripSectionKey.SHARED_GEAR,
        TripSectionKey.FOOD_PLAN,
        TripSectionKey.MEDICAL_KIT,
        TripSectionKey.SAFETY_PLAN,
        TripSectionKey.RESCUE_INFO,
        TripSectionKey.BUDGET,
        TripSectionKey.GOALS,
    )
    val trip = TripSummary(
        id = "trip-wugong",
        ownerUserId = "fixture-user",
        tripType = TripType.TEAM,
        title = "端午武功山重装",
        description = "两天一夜，重点检查天气、营地和公共装备。",
        startDate = "2026-06-19",
        endDate = "2026-06-21",
        enabledSections = sections,
        dayCount = 3,
        itineraryDayCount = 3,
        timeBucket = TripTimeBucket.UPCOMING,
        daysUntilStart = 21,
        memberCount = 3,
        fieldVersions = emptyFieldVersions(),
        createdAt = "2026-05-20T12:00:00Z",
        updatedAt = "2026-05-22T12:00:00Z",
    )
    val owner = TripMember(
        id = "member-owner",
        tripId = trip.id,
        userId = "fixture-user",
        isOwner = true,
        profile = TripMemberProfile(displayName = "星野徒步者", phone = "13800000000", roleLabel = "队长"),
    )
    val teammate = TripMember(
        id = "member-navigator",
        tripId = trip.id,
        userId = "fixture-user-2",
        profile = TripMemberProfile(displayName = "山脊领航", roleLabel = "导航"),
    )
    return TripDetail(
        trip = trip,
        sections = sections,
        myMemberId = owner.id,
        members = listOf(owner, teammate),
        personalGear = listOf(
            TripPersonalGearItem(
                id = "trip-gear-pack",
                memberId = owner.id,
                category = GearCategory.BACKPACK_SYSTEM,
                categoryLabel = GearCategory.BACKPACK_SYSTEM.label,
                name = "Osprey Talon 22",
                plannedQuantity = 1,
                packedQuantity = 1,
                unitWeightG = 900,
            ),
        ),
        sharedGearDemands = listOf(
            TripSharedGearDemand(
                id = "shared-stove",
                category = GearCategory.KITCHEN_SYSTEM,
                categoryLabel = GearCategory.KITCHEN_SYSTEM.label,
                name = "炉头",
                responsibleMemberId = owner.id,
                demandName = "炉头",
                slotName = "炉头",
                concreteName = "BRS-3000T",
                plannedQuantity = 1,
                unitWeightG = 25,
            ),
        ),
        itineraryDays = listOf(
            TripItineraryDay(
                id = "day-1",
                dayIndex = 1,
                dateLabel = "2026-06-19",
                title = "集合进山",
                estimateMinutes = 320,
            ),
        ),
        routeSegments = listOf(
            TripRouteSegment(
                id = "segment-1",
                name = "龙山村到金顶",
                distanceKm = 10.8,
                ascentM = 1200,
                descentM = 100,
                formulaEstimateMinutes = 320,
                finalEstimateMinutes = 340,
            ),
        ),
        foodMeals = listOf(
            TripFoodMeal(
                id = "meal-1",
                itineraryDayId = "day-1",
                mealKey = "dinner",
                dishName = "营地热食",
            ),
        ),
        medicalItems = listOf(
            TripMedicalItem(id = "medical-1", name = "弹性绷带", requiredQuantity = 2, packedQuantity = 1),
        ),
        segmentAssignments = listOf(
            TripSegmentAssignment(id = "assignment-1", checkpoint = "金顶前最后补水点", leaderRecordMemberId = owner.id),
        ),
        safetyRisks = listOf(
            TripSafetyRisk(id = "risk-weather", riskType = "雷雨", prevention = "午后雷雨前通过暴露山脊"),
        ),
        rescueContacts = listOf(
            TripRescueContact(id = "rescue-1", organization = "景区救援", phone = "110"),
        ),
        budgetItems = listOf(
            TripBudgetItem(id = "budget-1", name = "包车费用", quantity = 1, totalPriceCents = 80000),
        ),
        goals = listOf(
            TripGoalItem(id = "goal-1", scope = "team", content = "全员安全完成穿越"),
        ),
        weightSummaries = listOf(
            TripMemberGearWeightSummary(memberId = owner.id, allWeightG = 925, actualWeightG = 925),
        ),
    )
}

private fun fixturePackingList(): GearPackingListDetail = GearPackingListDetail(
    id = "packing-weekend",
    title = "武功山出发清单",
    description = "出发前逐项确认个人装备。",
    targetDate = "2026-06-19",
    stats = GearPackingListStats(totalItems = 2, packedItems = 1, totalWeightG = 945, packedWeightG = 900),
    items = listOf(
        GearPackingListItem(
            id = "packing-item-pack",
            gearId = "gear-pack",
            category = GearCategory.BACKPACK_SYSTEM,
            categoryLabel = GearCategory.BACKPACK_SYSTEM.label,
            name = "Osprey Talon 22",
            plannedQuantity = 1,
            packedQuantity = 1,
            unitWeightG = 900,
        ),
        GearPackingListItem(
            id = "packing-item-headlamp",
            gearId = "gear-headlamp",
            category = GearCategory.LIGHTING_SYSTEM,
            categoryLabel = GearCategory.LIGHTING_SYSTEM.label,
            name = "NITECORE NU25 UL",
            plannedQuantity = 1,
            packedQuantity = 0,
            unitWeightG = 45,
        ),
    ),
)

private fun fixtureAtlas() = listOf(
    GearAtlasPublicItem(
        id = "atlas-headlamp",
        category = GearCategory.LIGHTING_SYSTEM,
        categoryLabel = GearCategory.LIGHTING_SYSTEM.label,
        name = "NITECORE NU25 UL",
        brand = "NITECORE",
        model = "NU25 UL",
        description = "轻量头灯，适合夜间行走与营地照明。",
        weightG = 45,
        officialPriceCents = 29800,
        officialPriceCurrency = "CNY",
        specs = mapOf("续航" to "45h", "防水" to "IP66"),
        approvedAt = "2026-04-20T08:00:00Z",
        createdAt = "2026-04-18T08:00:00Z",
        updatedAt = "2026-04-20T08:00:00Z",
    ),
    GearAtlasPublicItem(
        id = "atlas-pack",
        category = GearCategory.BACKPACK_SYSTEM,
        categoryLabel = GearCategory.BACKPACK_SYSTEM.label,
        name = "Osprey Talon 22",
        brand = "Osprey",
        model = "Talon 22",
        description = "一日徒步背包，容量适中，背负透气。",
        weightG = 900,
        officialPriceCents = 129900,
        officialPriceCurrency = "CNY",
        specs = mapOf("容量" to "22L", "背负" to "AirScape"),
        approvedAt = "2026-04-12T08:00:00Z",
        createdAt = "2026-04-10T08:00:00Z",
        updatedAt = "2026-04-12T08:00:00Z",
    ),
)

private fun fixtureKnots() = listOf(
    KnotSummary(
        id = "taut-line",
        slug = "taut-line",
        title = "可调节绳结",
        summary = "调节绳索上的张力。",
        categories = listOf(KnotTaxonomyItem("camp", "camp", "露营")),
        types = listOf(KnotTaxonomyItem("tension", "tension", "张力调节")),
        media = listOf(
            KnotMediaAsset("taut-preview", "preview", "knots/taut-line.png", "image/png", attribution = "Knots 3D"),
            KnotMediaAsset("taut-draw", "draw_gif", "knots/taut-line-draw.gif", "image/gif", attribution = "Knots 3D"),
            KnotMediaAsset("taut-turntable", "turntable_gif", "knots/taut-line-turntable.gif", "image/gif", attribution = "Knots 3D"),
        ),
        href = "/api/v1/skills/knots/detail/taut-line",
    ),
    KnotSummary(
        id = "bowline",
        slug = "bowline",
        title = "单套结",
        summary = "快速打出固定绳圈，受力后仍较容易解开。",
        categories = listOf(KnotTaxonomyItem("basic", "basic", "基础")),
        types = listOf(KnotTaxonomyItem("loop", "loop", "固定绳圈")),
        href = "/api/v1/skills/knots/detail/bowline",
    ),
)
