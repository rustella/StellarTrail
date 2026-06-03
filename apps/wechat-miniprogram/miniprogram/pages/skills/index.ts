import {
  acceptKnotDisclaimer as submitKnotDisclaimerAcceptance,
  consumeOfflineCacheNotice,
  favoriteKnot,
  getErrorMessage,
  getKnotDisclaimer,
  getKnotFilters,
  hasAccessToken,
  hasLocalKnotDisclaimerAcceptance,
  isOfflineCacheMissError,
  listFavoriteSkills,
  listKnots,
  resolveAssetUrl,
  unfavoriteKnot,
} from "../../utils/api-skills";
import {
  mapSkillCard,
  type FavoriteSkillCategory,
  type FavoriteSkillFilterOption,
  type KnotDetail,
  type KnotFilterOption,
  type KnotMediaAsset,
  type KnotSummary,
  type SkillCard,
} from "../../utils/skill-utils";
import {
  cacheAllKnotsForOffline,
  prepareAllKnotsOfflineCache,
  type KnotOfflineCachePlan,
  type KnotOfflineCacheProgress,
  type KnotOfflineCacheResult,
} from "../../utils/knot-offline-cache";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import { resolveCachedMediaUrl } from "../../utils/media-cache";
import { delayNextTick, indexedAppendData } from "../../utils/page-data";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
} from "../../utils/auth-prompt";
import { isOffline, showOfflineWriteBlockedToast } from "../../utils/network-state";

type SkillsMode = "catalog" | "knots" | "favorites";

interface SkillCategoryCard {
  id: "knots";
  icon: string;
  title: string;
  subtitle: string;
  summary: string;
  actionText: string;
}

interface KnotListCard extends SkillCard {
  thumbnailUrl: string;
  hasThumbnail: boolean;
  categoryIds: string[];
  categoryTitles: string[];
  searchText: string;
  isFavorited: boolean;
  favoriteLoading: boolean;
  favoritedAt: string;
}

interface KnotCategoryFilter {
  id: string;
  label: string;
  count: number;
}

const KNOTS_PAGE_SIZE = 10;
const FAVORITE_SKILLS_PAGE_SIZE = 20;
const KNOT_CACHE_ENTRY_KEY = "stellartrail_open_knots_cache";
let knotSearchTimer: ReturnType<typeof setTimeout> | null = null;
let knotListRequestSeq = 0;
let favoriteListRequestSeq = 0;

const DEFAULT_FAVORITE_FILTERS: FavoriteSkillFilterOption[] = [
  { id: "all", title: "全部收藏", count: 0 },
  { id: "knots", title: "绳结", count: 0 },
];

const SKILL_CATEGORIES: SkillCategoryCard[] = [
  {
    id: "knots",
    icon: "🪢",
    title: "绳结",
    subtitle: "Knots",
    summary: "常用露营、钓鱼、连接和固定绳结，按场景快速复习。",
    actionText: "查看绳结列表",
  },
];

Page({
  data: {
    title: "户外技能",
    mode: "catalog" as SkillsMode,
    skillCategories: SKILL_CATEGORIES,
    allKnots: [] as KnotListCard[],
    knots: [] as KnotListCard[],
    favoriteKnots: [] as KnotListCard[],
    favoriteFilters: DEFAULT_FAVORITE_FILTERS,
    favoriteFilterLabels: favoriteFilterLabels(DEFAULT_FAVORITE_FILTERS),
    selectedFavoriteFilter: "all" as FavoriteSkillCategory,
    selectedFavoriteFilterIndex: 0,
    favoriteNextOffset: null as number | null,
    categoryFilters: [{ id: "all", label: "全部类别", count: 0 }] as KnotCategoryFilter[],
    categoryFilterLabels: ["全部类别"] as string[],
    selectedCategoryId: "all",
    selectedCategoryIndex: 0,
    searchQuery: "",
    hasActiveFilters: false,
    listResultText: "",
    nextOffset: null as number | null,
    loading: false,
    loadingMore: false,
    preparingKnotCache: false,
    cachingKnots: false,
    cacheProgressText: "",
    cacheSummaryText: "",
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    checkingKnotDisclaimer: false,
    knotDisclaimerVisible: false,
    knotDisclaimerTitle: "",
    knotDisclaimerContent: "",
    knotDisclaimerVersion: "",
    knotDisclaimerError: "",
    acceptingKnotDisclaimer: false,
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    this.ensureSkillsPageReady();
  },

  onTabItemTap() {
    this.showSkillCatalog();
    wx.pageScrollTo({ scrollTop: 0, duration: 0 });
  },

  ensureSkillsPageReady() {
    if (this.data.mode === "knots" && !hasAccessToken()) {
      this.showSkillCatalog();
      return;
    }
    if (wx.getStorageSync(KNOT_CACHE_ENTRY_KEY) === true) {
      wx.removeStorageSync(KNOT_CACHE_ENTRY_KEY);
      this.openKnotsFromEntry();
      return;
    }
    if (this.data.mode === "catalog") {
      if (!this.data.skillCategories.length) {
        this.setData({ skillCategories: SKILL_CATEGORIES });
      }
      return;
    }
    if (
      this.data.mode === "knots" &&
      !this.data.allKnots.length &&
      !this.data.knots.length &&
      !this.data.loading &&
      !this.data.error
    ) {
      this.loadKnots();
      return;
    }
    if (
      this.data.mode === "favorites" &&
      !this.data.favoriteKnots.length &&
      !this.data.loading &&
      !this.data.error
    ) {
      this.loadFavoriteSkills();
    }
  },

  onPullDownRefresh() {
    if (this.data.mode === "knots") {
      this.loadKnots().finally(() => wx.stopPullDownRefresh());
      return;
    }
    if (this.data.mode === "favorites") {
      this.loadFavoriteSkills().finally(() => wx.stopPullDownRefresh());
      return;
    }
    wx.stopPullDownRefresh();
  },

  onReachBottom() {
    if (this.data.mode === "knots") {
      this.loadMoreKnots();
      return;
    }
    if (this.data.mode === "favorites") {
      this.loadMoreFavoriteSkills();
    }
  },

  openSkillCategory(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as SkillCategoryCard["id"] | undefined;
    if (id !== "knots") {
      return;
    }
    this.openKnotsFromEntry();
  },

  async openKnotsFromEntry() {
    if (
      !requireLoginForAction(this, {
        message: "登录并同意绳结教程免责声明后，可以查看绳结列表。",
        redirectUrl: "/pages/skills/index",
      })
    ) {
      return;
    }
    this.setData({
      checkingKnotDisclaimer: true,
      knotDisclaimerError: "",
      error: "",
      offlineNotice: "",
    });
    try {
      const disclaimer = await getKnotDisclaimer();
      if (disclaimer.accepted) {
        this.enterKnotsList();
        return;
      }
      this.setData({
        checkingKnotDisclaimer: false,
        knotDisclaimerVisible: true,
        knotDisclaimerTitle: disclaimer.title,
        knotDisclaimerContent: disclaimer.content,
        knotDisclaimerVersion: disclaimer.version,
      });
    } catch (error) {
      this.setData({ checkingKnotDisclaimer: false });
      if (hasLocalKnotDisclaimerAcceptance()) {
        this.enterKnotsList();
        wx.showToast({
          title: "当前离线，正在显示已缓存绳结",
          icon: "none",
        });
        return;
      }
      if (!hasAccessToken()) {
        requireLoginForAction(this, {
          message: "登录并同意绳结教程免责声明后，可以查看绳结列表。",
          redirectUrl: "/pages/skills/index",
        });
        return;
      }
      wx.showToast({
        title:
          isOffline() || isOfflineCacheMissError(error)
            ? "当前离线，请联网确认绳结免责声明后再查看"
            : getErrorMessage(error),
        icon: "none",
      });
    }
  },

  enterKnotsList() {
    wx.setNavigationBarTitle({ title: "绳结" });
    this.setData({
      mode: "knots",
      error: "",
      offlineNotice: "",
      checkingKnotDisclaimer: false,
      knotDisclaimerVisible: false,
      knotDisclaimerError: "",
      acceptingKnotDisclaimer: false,
    });
    if (!this.data.allKnots.length) {
      this.loadKnots();
    }
  },

  rejectKnotDisclaimer() {
    this.setData({
      knotDisclaimerVisible: false,
      knotDisclaimerError: "",
      acceptingKnotDisclaimer: false,
      checkingKnotDisclaimer: false,
    });
    this.showSkillCatalog();
  },

  async acceptKnotDisclaimer() {
    if (this.data.acceptingKnotDisclaimer) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      acceptingKnotDisclaimer: true,
      knotDisclaimerError: "",
    });
    try {
      const response = await submitKnotDisclaimerAcceptance({
        client_platform: "wechat_miniprogram",
        client_version: miniProgramVersion(),
        device_model: deviceModel(),
      });
      if (!response.accepted) {
        this.setData({
          acceptingKnotDisclaimer: false,
          knotDisclaimerError: "同意状态未保存，请稍后重试。",
        });
        return;
      }
      this.enterKnotsList();
    } catch (error) {
      this.setData({
        acceptingKnotDisclaimer: false,
        knotDisclaimerError: getErrorMessage(error),
      });
      if (!hasAccessToken()) {
        this.setData({ knotDisclaimerVisible: false });
        requireLoginForAction(this, {
          message: "登录状态已过期，请重新登录后查看绳结列表。",
          redirectUrl: "/pages/skills/index",
        });
      }
    }
  },

  openFavoriteSkills() {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以同步收藏技能，并在收藏清单里快速找到。",
        redirectUrl: "/pages/skills/index",
      })
    ) {
      return;
    }
    wx.setNavigationBarTitle({ title: "收藏技能" });
    this.setData({
      mode: "favorites",
      error: "",
      offlineNotice: "",
      loadingMore: false,
    });
    this.loadFavoriteSkills();
  },

  showSkillCatalog() {
    wx.setNavigationBarTitle({ title: "户外技能" });
    this.setData({
      mode: "catalog",
      error: "",
      offlineNotice: "",
      loading: false,
      loadingMore: false,
      checkingKnotDisclaimer: false,
      knotDisclaimerVisible: false,
      knotDisclaimerError: "",
      acceptingKnotDisclaimer: false,
    });
  },

  async loadKnots(
    filterState: {
      searchQuery?: string;
      selectedCategoryId?: string;
      selectedCategoryIndex?: number;
    } = {},
  ) {
    const searchQuery = filterState.searchQuery ?? this.data.searchQuery;
    const selectedCategoryId =
      filterState.selectedCategoryId ?? this.data.selectedCategoryId;
    const selectedCategoryIndex =
      filterState.selectedCategoryIndex ?? this.data.selectedCategoryIndex;
    this.setData({
      loading: true,
      loadingMore: false,
      error: "",
      searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
    });
    const requestSeq = ++knotListRequestSeq;
    try {
      const filtersPromise = loadKnotCategoryFilters().catch(() => null);
      const favoriteIdsPromise = loadFavoriteKnotIdSet().catch(() => null);
      const response = await loadKnotsPage(0, searchQuery, selectedCategoryId);
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const favoriteIds = await favoriteIdsPromise;
      const allKnots = await Promise.all(
        response.items.map((item) => mapKnotListCard(item, favoriteIds)),
      );
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const loadedCategoryFilters = await filtersPromise;
      const categoryFilters = loadedCategoryFilters ?? buildCategoryFilters(allKnots);
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const nextOffset = response.page.next_offset ?? null;
      const listState = buildKnotListState(
        allKnots,
        selectedCategoryId,
        searchQuery,
        nextOffset,
        categoryFilters,
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        allKnots,
        ...listState,
        nextOffset,
        loading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.allKnots.length) {
        const listState = buildKnotListState(
          this.data.allKnots,
          selectedCategoryId,
          searchQuery,
          this.data.nextOffset,
          this.data.categoryFilters,
        );
        this.setData({
          ...listState,
          searchQuery,
          loading: false,
        });
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        loadingMore: false,
        allKnots: [] as KnotListCard[],
        knots: [] as KnotListCard[],
        nextOffset: null,
      });
    }
  },

  async loadMoreKnots() {
    const nextOffset = this.data.nextOffset;
    if (nextOffset == null || this.data.loadingMore || this.data.loading) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = knotListRequestSeq;
    try {
      const favoriteIdsPromise = loadFavoriteKnotIdSet().catch(() => null);
      const response = await loadKnotsPage(
        nextOffset,
        this.data.searchQuery,
        this.data.selectedCategoryId,
      );
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const favoriteIds = await favoriteIdsPromise;
      const nextKnots = await Promise.all(
        response.items.map((item) => mapKnotListCard(item, favoriteIds)),
      );
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const allKnots = appendUniqueKnots(this.data.allKnots, nextKnots);
      const addedKnots = allKnots.slice(this.data.allKnots.length);
      const nextPageOffset = response.page.next_offset ?? null;
      const listState = buildKnotListState(
        allKnots,
        this.data.selectedCategoryId,
        this.data.searchQuery,
        nextPageOffset,
        this.data.categoryFilters,
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...indexedAppendData("allKnots", this.data.allKnots.length, addedKnots),
        ...indexedAppendData(
          "knots",
          this.data.knots.length,
          filterKnots(
            addedKnots,
            this.data.selectedCategoryId,
            this.data.searchQuery,
          ),
        ),
        ...withoutKnotItems(listState),
        nextOffset: nextPageOffset,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ loadingMore: false });
    }
  },

  async loadFavoriteSkills(
    filterState: {
      selectedFavoriteFilter?: FavoriteSkillCategory;
      selectedFavoriteFilterIndex?: number;
    } = {},
  ) {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以查看收藏技能清单。",
        redirectUrl: "/pages/skills/index",
      })
    ) {
      return;
    }
    const selectedFavoriteFilter =
      filterState.selectedFavoriteFilter ?? this.data.selectedFavoriteFilter;
    const selectedFavoriteFilterIndex =
      filterState.selectedFavoriteFilterIndex ??
      this.data.selectedFavoriteFilterIndex;
    const requestSeq = ++favoriteListRequestSeq;
    this.setData({
      loading: true,
      loadingMore: false,
      error: "",
      selectedFavoriteFilter,
      selectedFavoriteFilterIndex,
    });
    try {
      const response = await loadFavoriteSkillsPage(0, selectedFavoriteFilter);
      if (requestSeq !== favoriteListRequestSeq) {
        return;
      }
      const favoriteKnots = await Promise.all(
        response.items.map((item) =>
          mapKnotListCard(item.knot, new Set([item.knot.id]), item.favorited_at),
        ),
      );
      if (requestSeq !== favoriteListRequestSeq) {
        return;
      }
      const filters = normalizeFavoriteFilters(response.filters);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        favoriteKnots,
        favoriteFilters: filters,
        favoriteFilterLabels: favoriteFilterLabels(filters),
        favoriteNextOffset: response.page.next_offset ?? null,
        loading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== favoriteListRequestSeq) {
        return;
      }
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        loadingMore: false,
        favoriteKnots: [] as KnotListCard[],
        favoriteNextOffset: null,
      });
    }
  },

  async loadMoreFavoriteSkills() {
    const nextOffset = this.data.favoriteNextOffset;
    if (nextOffset == null || this.data.loadingMore || this.data.loading) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = favoriteListRequestSeq;
    try {
      const response = await loadFavoriteSkillsPage(
        nextOffset,
        this.data.selectedFavoriteFilter,
      );
      if (requestSeq !== favoriteListRequestSeq) {
        return;
      }
      const nextKnots = await Promise.all(
        response.items.map((item) =>
          mapKnotListCard(item.knot, new Set([item.knot.id]), item.favorited_at),
        ),
      );
      if (requestSeq !== favoriteListRequestSeq) {
        return;
      }
      const filters = normalizeFavoriteFilters(response.filters);
      this.setData({
        ...indexedAppendData(
          "favoriteKnots",
          this.data.favoriteKnots.length,
          nextKnots,
        ),
        favoriteFilters: filters,
        favoriteFilterLabels: favoriteFilterLabels(filters),
        favoriteNextOffset: response.page.next_offset ?? null,
      });
    } catch (error) {
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ loadingMore: false });
    }
  },

  async cacheAllKnots() {
    if (this.data.preparingKnotCache || this.data.cachingKnots) {
      return;
    }
    const updateCacheProgress = createCacheProgressUpdater(this);
    this.setData({
      preparingKnotCache: true,
      cachingKnots: false,
      cacheProgressText: "正在统计绳结数量和资源大小...",
      cacheSummaryText: "",
      error: "",
    });
    try {
      const plan = await prepareAllKnotsOfflineCache({
        pageSize: KNOTS_PAGE_SIZE,
        onProgress: updateCacheProgress,
      });
      updateCacheProgress.flush();
      this.setData({
        cacheProgressText: `已统计 ${plan.items.length} 个绳结，预计约 ${formatBytes(
          plan.estimatedBytes,
        )}`,
      });
      const confirmed = await confirmKnotOfflineCache(plan);
      if (!confirmed) {
        this.setData({
          preparingKnotCache: false,
          cachingKnots: false,
          cacheProgressText: "",
        });
        return;
      }
      this.setData({
        preparingKnotCache: false,
        cachingKnots: true,
        cacheProgressText: "准备下载离线媒体资源...",
      });
      const result = await cacheAllKnotsForOffline({
        plan,
        pageSize: KNOTS_PAGE_SIZE,
        onProgress: updateCacheProgress,
      });
      updateCacheProgress.flush();
      this.setData({
        allKnots: [] as KnotListCard[],
        knots: [] as KnotListCard[],
        nextOffset: null,
        cacheProgressText: "正在整理离线绳结列表 0/" + result.items.length,
      });
      const favoriteIds = await loadFavoriteKnotIdSet().catch(() => null);
      const allKnots = await mapKnotListCardsInBatches(
        this,
        result.items,
        favoriteIds,
      );
      const listState = buildKnotListState(
        allKnots,
        this.data.selectedCategoryId,
        this.data.searchQuery,
        null,
        this.data.categoryFilters,
      );
      const cacheSummaryText = formatKnotCacheResult(result);
      this.setData({
        ...withoutKnotItems(listState),
        nextOffset: null,
        preparingKnotCache: false,
        cachingKnots: false,
        cacheProgressText: "",
        cacheSummaryText,
      });
      wx.showToast({
        title:
          result.failedDetailCount || result.failedMediaCount
            ? "部分缓存完成"
            : "缓存完成",
        icon:
          result.failedDetailCount || result.failedMediaCount
            ? "none"
            : "success",
      });
    } catch (error) {
      updateCacheProgress.flush();
      this.setData({
        preparingKnotCache: false,
        cachingKnots: false,
        cacheProgressText: "",
      });
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  onSearchInput(event: any) {
    const searchQuery = String(event.detail.value ?? "");
    const clearingSearch = !searchQuery.trim();
    const selectedCategoryId = clearingSearch ? "all" : this.data.selectedCategoryId;
    const selectedCategoryIndex = clearingSearch ? 0 : this.data.selectedCategoryIndex;
    clearKnotSearchTimer();
    knotListRequestSeq += 1;
    this.applyFilters({
      searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
      nextOffset: null,
    });
    if (clearingSearch) {
      this.loadKnots({ searchQuery: "", selectedCategoryId: "all", selectedCategoryIndex: 0 });
      return;
    }
    knotSearchTimer = setTimeout(() => {
      knotSearchTimer = null;
      this.loadKnots({
        searchQuery: this.data.searchQuery,
        selectedCategoryId: this.data.selectedCategoryId,
        selectedCategoryIndex: this.data.selectedCategoryIndex,
      });
    }, 250);
  },

  submitKnotSearch() {
    clearKnotSearchTimer();
    const clearingSearch = !this.data.searchQuery.trim();
    const selectedCategoryId = clearingSearch ? "all" : this.data.selectedCategoryId;
    const selectedCategoryIndex = clearingSearch ? 0 : this.data.selectedCategoryIndex;
    this.applyFilters({
      searchQuery: this.data.searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
      nextOffset: null,
    });
    this.loadKnots({
      searchQuery: this.data.searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
    });
  },

  onCategoryFilterChange(event: any) {
    clearKnotSearchTimer();
    const selectedCategoryIndex = Number(event.detail.value || 0);
    const selectedCategoryId = this.data.categoryFilters[selectedCategoryIndex]?.id ?? "all";
    this.loadKnots({
      searchQuery: this.data.searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
    });
  },

  onFavoriteFilterChange(event: any) {
    const selectedFavoriteFilterIndex = Number(event.detail.value || 0);
    const selectedFavoriteFilter =
      this.data.favoriteFilters[selectedFavoriteFilterIndex]?.id ?? "all";
    this.loadFavoriteSkills({
      selectedFavoriteFilter,
      selectedFavoriteFilterIndex,
    });
  },

  clearKnotFilters() {
    clearKnotSearchTimer();
    this.applyFilters({
      searchQuery: "",
      selectedCategoryId: "all",
      selectedCategoryIndex: 0,
      nextOffset: null,
    });
    this.loadKnots({
      searchQuery: "",
      selectedCategoryId: "all",
      selectedCategoryIndex: 0,
    });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (id) {
      wx.navigateTo({ url: `/pages/skills/detail/index?id=${encodeURIComponent(id)}` });
    }
  },

  async toggleFavorite(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以收藏技能，并在收藏清单里快速找到。",
        redirectUrl: "/pages/skills/index",
      })
    ) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const current = findKnotCard(this, id);
    const previousFavorited = Boolean(current?.isFavorited);
    const nextFavorited = !previousFavorited;
    this.setKnotFavoriteState(id, previousFavorited, true);
    try {
      const status = nextFavorited
        ? await favoriteKnot(id)
        : await unfavoriteKnot(id);
      this.setKnotFavoriteState(
        id,
        status.is_favorited,
        false,
        status.favorited_at ?? "",
      );
      if (this.data.mode === "favorites" && !status.is_favorited) {
        this.removeFavoriteKnot(id);
      }
    } catch (error) {
      this.setKnotFavoriteState(
        id,
        previousFavorited,
        false,
        current?.favoritedAt ?? "",
      );
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  setKnotFavoriteState(
    id: string,
    isFavorited: boolean,
    favoriteLoading: boolean,
    favoritedAt = "",
  ) {
    this.setData({
      allKnots: updateKnotFavoriteCards(
        this.data.allKnots,
        id,
        isFavorited,
        favoriteLoading,
        favoritedAt,
      ),
      knots: updateKnotFavoriteCards(
        this.data.knots,
        id,
        isFavorited,
        favoriteLoading,
        favoritedAt,
      ),
      favoriteKnots: updateKnotFavoriteCards(
        this.data.favoriteKnots,
        id,
        isFavorited,
        favoriteLoading,
        favoritedAt,
      ),
    });
  },

  removeFavoriteKnot(id: string) {
    const favoriteKnots = this.data.favoriteKnots.filter((item) => item.id !== id);
    const favoriteFilters = this.data.favoriteFilters.map((filter) => ({
      ...filter,
      count: Math.max(0, filter.count - 1),
    }));
    this.setData({
      favoriteKnots,
      favoriteFilters,
      favoriteFilterLabels: favoriteFilterLabels(favoriteFilters),
    });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },

  applyFilters(filterState: {
    searchQuery: string;
    selectedCategoryId: string;
    selectedCategoryIndex: number;
    nextOffset?: number | null;
  }) {
    const nextOffset =
      filterState.nextOffset === undefined ? this.data.nextOffset : filterState.nextOffset;
    const listState = buildKnotListState(
      this.data.allKnots,
      filterState.selectedCategoryId,
      filterState.searchQuery,
      nextOffset,
      this.data.categoryFilters,
    );
    this.setData({
      ...listState,
      searchQuery: filterState.searchQuery,
      nextOffset,
      loadingMore: false,
    });
  },
});

function clearKnotSearchTimer() {
  if (!knotSearchTimer) {
    return;
  }
  clearTimeout(knotSearchTimer);
  knotSearchTimer = null;
}

function loadKnotsPage(
  offset: number,
  searchQuery = "",
  selectedCategoryId = "all",
) {
  return listKnots({
    offset,
    limit: KNOTS_PAGE_SIZE,
    q: normalizeOptionalFilter(searchQuery),
    category:
      selectedCategoryId && selectedCategoryId !== "all"
        ? selectedCategoryId
        : undefined,
  });
}

function loadFavoriteSkillsPage(
  offset: number,
  selectedFavoriteFilter: FavoriteSkillCategory,
) {
  return listFavoriteSkills({
    offset,
    limit: FAVORITE_SKILLS_PAGE_SIZE,
    skill_category: selectedFavoriteFilter,
  });
}

async function loadFavoriteKnotIdSet(): Promise<Set<string> | null> {
  if (!hasAccessToken()) {
    return null;
  }
  const ids = new Set<string>();
  let offset = 0;
  while (true) {
    const response = await listFavoriteSkills({
      skill_category: "knots",
      offset,
      limit: 100,
    });
    response.items.forEach((item) => ids.add(item.knot.id));
    if (response.page.next_offset == null) {
      break;
    }
    offset = response.page.next_offset;
  }
  return ids;
}

function normalizeFavoriteFilters(
  filters: FavoriteSkillFilterOption[],
): FavoriteSkillFilterOption[] {
  const byId = new Map(filters.map((item) => [item.id, item]));
  return DEFAULT_FAVORITE_FILTERS.map((fallback) => byId.get(fallback.id) ?? fallback);
}

function favoriteFilterLabels(filters: FavoriteSkillFilterOption[]): string[] {
  return filters.map((item) => `${item.title} ${item.count}`);
}

function normalizeOptionalFilter(value: string): string | undefined {
  const trimmed = value.trim();
  return trimmed ? trimmed : undefined;
}

function confirmKnotOfflineCache(plan: KnotOfflineCachePlan): Promise<boolean> {
  return new Promise((resolve) => {
    const detailWarning = plan.failedDetailCount
      ? `\n${plan.failedDetailCount} 个详情暂未统计，实际大小可能略有差异。`
      : "";
    wx.showModal({
      title: "缓存全部绳结",
      content: `主要用于离线模式下查询绳结。将缓存 ${plan.items.length} 个绳结的详情和图片、动图资源，预计约 ${formatBytes(
        plan.estimatedBytes,
      )}。建议在 Wi-Fi 下进行。${detailWarning}`,
      confirmText: "开始缓存",
      success: (result) => resolve(Boolean(result.confirm)),
      fail: () => resolve(false),
    });
  });
}

function formatKnotCacheProgress(progress: KnotOfflineCacheProgress): string {
  if (progress.phase === "list") {
    return `已读取 ${progress.loadedCount ?? 0} 个绳结`;
  }
  if (progress.phase === "detail") {
    return `正在读取详情 ${progress.currentIndex ?? 0}/${progress.totalCount ?? 0}：${
      progress.currentTitle ?? "绳结"
    }`;
  }
  const failedText = progress.failedMediaCount
    ? `，失败 ${progress.failedMediaCount}`
    : "";
  return `正在下载媒体 ${progress.mediaReadyCount ?? 0}/${
    progress.mediaTotal ?? 0
  }${failedText}`;
}

function createCacheProgressUpdater(page: {
  setData(data: { cacheProgressText: string }): void;
}): ((progress: KnotOfflineCacheProgress) => void) & { flush(): void } {
  let lastAppliedAt = 0;
  let pendingText = "";
  let timer: ReturnType<typeof setTimeout> | null = null;
  const apply = () => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (!pendingText) {
      return;
    }
    lastAppliedAt = Date.now();
    page.setData({ cacheProgressText: pendingText });
  };
  const update = ((progress: KnotOfflineCacheProgress) => {
    pendingText = formatKnotCacheProgress(progress);
    const remainingMs = Math.max(0, 200 - (Date.now() - lastAppliedAt));
    if (remainingMs === 0) {
      apply();
      return;
    }
    if (!timer) {
      timer = setTimeout(apply, remainingMs);
    }
  }) as ((progress: KnotOfflineCacheProgress) => void) & { flush(): void };
  update.flush = apply;
  return update;
}

function formatKnotCacheResult(result: KnotOfflineCacheResult): string {
  const failedCount = result.failedDetailCount + result.failedMediaCount;
  const failedText = failedCount ? `，${failedCount} 项未成功，可稍后重试` : "";
  return `已缓存 ${result.items.length} 个绳结、${result.detailCount} 个详情、${result.mediaReadyCount}/${result.mediaTotal} 个媒体资源，约 ${formatBytes(
    result.estimatedBytes,
  )}${failedText}`;
}

function formatBytes(value: number): string {
  if (!Number.isFinite(value) || value <= 0) {
    return "0 MB";
  }
  if (value < 1024 * 1024) {
    return `${Math.max(1, Math.round(value / 1024))} KB`;
  }
  const mb = value / 1024 / 1024;
  return `${mb >= 10 ? Math.round(mb) : mb.toFixed(1)} MB`;
}

function appendUniqueKnots(
  current: KnotListCard[],
  incoming: KnotListCard[],
): KnotListCard[] {
  const seen = new Set(current.map((item) => item.id));
  const merged = current.slice();
  incoming.forEach((item) => {
    if (seen.has(item.id)) {
      return;
    }
    seen.add(item.id);
    merged.push(item);
  });
  return merged;
}

async function mapKnotListCard(
  item: KnotSummary | KnotDetail,
  favoriteIds: Set<string> | null = null,
  favoritedAt = "",
): Promise<KnotListCard> {
  const thumbnail = findThumbnail(item.media);
  const thumbnailUrl = thumbnail
    ? await resolveCachedMediaUrl(resolveAssetUrl(thumbnail.url))
    : "";
  const categoryIds = item.categories.map((category) => category.id || category.slug);
  const categoryTitles = item.categories.map((category) => category.title);
  const description = "description" in item ? item.description : "";
  const steps = "steps" in item ? item.steps : [];
  const searchParts = [
    item.id,
    item.slug,
    item.title,
    item.summary,
    ...(item.aliases || []),
    description ?? "",
  ].concat(
    steps,
    item.categories.flatMap((category) => [
      category.id,
      category.slug,
      category.title,
    ]),
    item.types.flatMap((type) => [type.id, type.slug, type.title]),
  );
  return {
    ...mapSkillCard(item),
    thumbnailUrl,
    hasThumbnail: Boolean(thumbnailUrl),
    categoryIds,
    categoryTitles,
    searchText: buildKnotSearchText(searchParts),
    isFavorited: favoriteIds?.has(item.id) ?? false,
    favoriteLoading: false,
    favoritedAt,
  };
}

function findKnotCard(
  page: {
    data: {
      allKnots: KnotListCard[];
      knots: KnotListCard[];
      favoriteKnots: KnotListCard[];
    };
  },
  id: string,
): KnotListCard | undefined {
  return (
    page.data.knots.find((item) => item.id === id) ??
    page.data.allKnots.find((item) => item.id === id) ??
    page.data.favoriteKnots.find((item) => item.id === id)
  );
}

function updateKnotFavoriteCards(
  cards: KnotListCard[],
  id: string,
  isFavorited: boolean,
  favoriteLoading: boolean,
  favoritedAt: string,
): KnotListCard[] {
  return cards.map((item) =>
    item.id === id
      ? {
          ...item,
          isFavorited,
          favoriteLoading,
          favoritedAt,
        }
      : item,
  );
}

function findThumbnail(media: KnotMediaAsset[]): KnotMediaAsset | undefined {
  return (
    media.find((item) => item.media_type === "thumbnail") ??
    media.find((item) => item.mime_type.startsWith("image/"))
  );
}

function buildCategoryFilters(knots: KnotListCard[]): KnotCategoryFilter[] {
  const categories = new Map<string, KnotCategoryFilter>();
  knots.forEach((knot) => {
    knot.categoryIds.forEach((id, index) => {
      const label = knot.categoryTitles[index] ?? "绳结";
      const current = categories.get(id);
      if (current) {
        current.count += 1;
        return;
      }
      categories.set(id, { id, label, count: 1 });
    });
  });
  const sortedCategories: KnotCategoryFilter[] = [];
  categories.forEach((category) => sortedCategories.push(category));
  sortedCategories.sort((left, right) =>
    left.label.localeCompare(right.label, "zh-Hans-CN"),
  );
  return [{ id: "all", label: "全部类别", count: knots.length }].concat(
    sortedCategories,
  );
}

async function loadKnotCategoryFilters(): Promise<KnotCategoryFilter[]> {
  const response = await getKnotFilters();
  return buildCategoryFiltersFromResponse(response.categories);
}

function buildCategoryFiltersFromResponse(
  categories: KnotFilterOption[],
): KnotCategoryFilter[] {
  const total = categories.reduce((sum, item) => sum + item.count, 0);
  return [{ id: "all", label: "全部类别", count: total }].concat(
    categories.map((item) => ({
      id: item.id || item.slug || item.title,
      label: item.title,
      count: item.count,
    })),
  );
}

function validCategoryIndex(filters: KnotCategoryFilter[], selectedCategoryId: string): number {
  const index = filters.findIndex((filter) => filter.id === selectedCategoryId);
  return index >= 0 ? index : 0;
}

function formatCategoryFilterLabel(filter: KnotCategoryFilter): string {
  return `${filter.label}（${filter.count}）`;
}

function buildKnotListState(
  allKnots: KnotListCard[],
  selectedCategoryId: string,
  searchQuery: string,
  nextOffset: number | null,
  knownCategoryFilters?: KnotCategoryFilter[],
) {
  const categoryFilters = knownCategoryFilters?.length
    ? knownCategoryFilters
    : buildCategoryFilters(allKnots);
  const selectedCategoryIndex = validCategoryIndex(
    categoryFilters,
    selectedCategoryId,
  );
  const resolvedCategoryId = categoryFilters[selectedCategoryIndex]?.id ?? "all";
  const knots = filterKnots(allKnots, resolvedCategoryId, searchQuery);
  return {
    knots,
    categoryFilters,
    categoryFilterLabels: categoryFilters.map(formatCategoryFilterLabel),
    selectedCategoryId: resolvedCategoryId,
    selectedCategoryIndex,
    hasActiveFilters: hasActiveFilters(resolvedCategoryId, searchQuery),
    listResultText: listResultText(
      knots.length,
      allKnots.length,
      nextOffset !== null,
    ),
  };
}

function filterKnots(
  knots: KnotListCard[],
  selectedCategoryId: string,
  searchQuery: string,
): KnotListCard[] {
  const query = normalizeKnotSearchText(searchQuery);
  return knots.filter((knot) => {
    const matchesCategory =
      selectedCategoryId === "all" || knot.categoryIds.includes(selectedCategoryId);
    const matchesSearch = !query || knot.searchText.includes(query);
    return matchesCategory && matchesSearch;
  });
}

function buildKnotSearchText(parts: Array<string | null | undefined>): string {
  const raw = parts
    .map((part) => (typeof part === "string" ? part.trim() : ""))
    .filter(Boolean)
    .join(" ")
    .toLocaleLowerCase();
  const normalized = normalizeKnotSearchText(raw);
  return `${raw} ${normalized}`;
}

function normalizeKnotSearchText(value: string): string {
  return value
    .trim()
    .toLocaleLowerCase()
    .replace(/[\s\-_/.,，。:：;；·•()（）【】\[\]]+/g, "");
}

function hasActiveFilters(selectedCategoryId: string, searchQuery: string): boolean {
  return selectedCategoryId !== "all" || Boolean(searchQuery.trim());
}

function listResultText(
  filteredCount: number,
  loadedCount: number,
  hasMore: boolean,
): string {
  if (!loadedCount) {
    return "";
  }
  if (filteredCount === loadedCount) {
    return hasMore ? `已加载 ${loadedCount} 个绳结` : `共 ${loadedCount} 个绳结`;
  }
  const totalText = hasMore ? `已加载 ${loadedCount}` : `${loadedCount}`;
  return `已筛出 ${filteredCount} / ${totalText} 个绳结`;
}

function withoutKnotItems(
  state: ReturnType<typeof buildKnotListState>,
): Omit<ReturnType<typeof buildKnotListState>, "knots"> {
  const { knots: _knots, ...meta } = state;
  return meta;
}

async function mapKnotListCardsInBatches(
  page: {
    data: {
      allKnots: KnotListCard[];
      knots: KnotListCard[];
      selectedCategoryId: string;
      searchQuery: string;
    };
    setData(data: Record<string, unknown>): void;
  },
  items: KnotDetail[],
  favoriteIds: Set<string> | null = null,
): Promise<KnotListCard[]> {
  const batchSize = 24;
  const allCards: KnotListCard[] = [];
  let allStartIndex = page.data.allKnots.length;
  let visibleStartIndex = page.data.knots.length;
  for (let offset = 0; offset < items.length; offset += batchSize) {
    const batch = items.slice(offset, offset + batchSize);
    const cards = await Promise.all(
      batch.map((item) => mapKnotListCard(item, favoriteIds)),
    );
    const visibleCards = filterKnots(
      cards,
      page.data.selectedCategoryId,
      page.data.searchQuery,
    );
    allCards.push(...cards);
    page.setData({
      ...indexedAppendData("allKnots", allStartIndex, cards),
      ...indexedAppendData("knots", visibleStartIndex, visibleCards),
      cacheProgressText: `正在整理离线绳结列表 ${Math.min(
        offset + batch.length,
        items.length,
      )}/${items.length}`,
    });
    allStartIndex += cards.length;
    visibleStartIndex += visibleCards.length;
    await delayNextTick();
  }
  return allCards;
}

function miniProgramVersion(): string {
  const info = safeAccountInfo();
  return info?.miniProgram?.version || "dev";
}

function safeAccountInfo(): WechatMiniprogram.AccountInfo | undefined {
  try {
    return wx.getAccountInfoSync();
  } catch {
    return undefined;
  }
}

function deviceModel(): string | null {
  try {
    return wx.getSystemInfoSync().model || null;
  } catch {
    return null;
  }
}
