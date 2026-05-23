package com.rustella.stellartrail.screenshot

import android.content.Context
import android.content.Intent
import com.rustella.stellartrail.core.config.AppConfig
import com.rustella.stellartrail.core.config.AppConfigStore
import com.rustella.stellartrail.core.config.InMemoryAppConfigStore
import com.rustella.stellartrail.core.network.ApiClient
import com.rustella.stellartrail.core.session.InMemorySessionStore
import com.rustella.stellartrail.core.session.SessionStore
import com.rustella.stellartrail.core.theme.InMemoryThemeRepository
import com.rustella.stellartrail.core.theme.ThemeMode
import com.rustella.stellartrail.core.theme.ThemeRepository
import com.rustella.stellartrail.data.atlas.GearAtlasRepositoryContract
import com.rustella.stellartrail.data.auth.AuthRepositoryContract
import com.rustella.stellartrail.data.gear.GearRepositoryContract
import com.rustella.stellartrail.data.skills.SkillRepositoryContract
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
import com.rustella.stellartrail.domain.gear.label
import com.rustella.stellartrail.domain.skills.KnotDetail
import com.rustella.stellartrail.domain.skills.KnotListResponse
import com.rustella.stellartrail.domain.skills.KnotMediaAsset
import com.rustella.stellartrail.domain.skills.KnotSummary
import com.rustella.stellartrail.domain.skills.KnotTaxonomyItem
import com.rustella.stellartrail.domain.skills.ListKnotsRequest
import com.rustella.stellartrail.domain.skills.PageInfo
import com.rustella.stellartrail.domain.skills.SkillCategoriesResponse
import com.rustella.stellartrail.domain.skills.SkillCategorySummary
import com.rustella.stellartrail.domain.skills.SkillLocale
import kotlinx.coroutines.flow.StateFlow

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
    override val apiClient: ApiClient = ApiClient(configProvider = { configStore.config.value })
    override val authRepository: AuthRepositoryContract = FixtureAuthRepository(sessionStore)
    override val gearRepository: GearRepositoryContract = FixtureGearRepository()
    override val gearAtlasRepository: GearAtlasRepositoryContract = FixtureGearAtlasRepository()
    override val skillRepository: SkillRepositoryContract = FixtureSkillRepository()
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
    override suspend fun createCaptcha(account: String): CaptchaChallengeResponse =
        CaptchaChallengeResponse("fixture-ticket", "image", "<svg />", "2099-01-01T00:00:00Z", "1234")
    override suspend fun login(account: String, password: String, captchaTicket: String?, captchaAnswer: String?): LoginResponse = login(account)
    override suspend fun register(request: RegisterRequest): LoginResponse = login(request.email)
    override fun logout() = sessionStore.clear()

    private fun codeResponse(email: String) = EmailVerificationCodeResponse(email, "2099-01-01T00:10:00Z", "246810")
    private fun login(account: String): LoginResponse = LoginResponse(
        accessToken = "fixture-access-token",
        expiresAt = "2099-01-01T00:00:00Z",
        refreshToken = "fixture-refresh-token",
        refreshExpiresAt = "2099-01-02T00:00:00Z",
        user = LoginUser(id = "fixture-user", username = account, email = "trail@example.test", nickname = "星野徒步者"),
    ).also(sessionStore::save)
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
        currentCount = 6,
        archivedCount = 2,
        totalValueCents = 429_700,
        totalWeightG = 5_860,
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

private class FixtureSkillRepository : SkillRepositoryContract {
    private val categories = listOf(
        SkillCategorySummary("knots", "knots", "绳结", "常用露营、钓鱼、连接和固定绳结，按场景快速复习。", 3, "/api/v1/skills/knots"),
    )
    private val knots = fixtureKnots()

    override suspend fun listSkills(locale: SkillLocale): SkillCategoriesResponse = SkillCategoriesResponse(categories)
    override suspend fun listKnots(locale: SkillLocale, request: ListKnotsRequest): KnotListResponse =
        KnotListResponse(locale, knots, PageInfo(limit = request.limit, offset = request.offset))
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
    override fun resolveMediaUrl(pathOrUrl: String): String = pathOrUrl
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
        id = "bowline",
        slug = "bowline",
        title = "单套结",
        summary = "快速打出固定绳圈，受力后仍较容易解开。",
        categories = listOf(KnotTaxonomyItem("basic", "basic", "基础")),
        types = listOf(KnotTaxonomyItem("loop", "loop", "固定绳圈")),
        media = listOf(KnotMediaAsset("thumbnail", "preview", "knots/bowline.png", "image/png")),
        href = "/api/v1/skills/knots/detail/bowline",
    ),
    KnotSummary(
        id = "taut-line",
        slug = "taut-line",
        title = "调节结",
        summary = "适合帐篷风绳微调张力。",
        categories = listOf(KnotTaxonomyItem("camp", "camp", "营地")),
        types = listOf(KnotTaxonomyItem("tension", "tension", "张力调节")),
        href = "/api/v1/skills/knots/detail/taut-line",
    ),
)
