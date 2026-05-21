import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  isOfflineCacheMissError,
  listKnots,
  resolveAssetUrl,
} from "../../utils/api";
import {
  mapSkillCard,
  type KnotDetail,
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

type SkillsMode = "catalog" | "knots";

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
}

interface KnotCategoryFilter {
  id: string;
  label: string;
  count: number;
}

const KNOTS_PAGE_SIZE = 10;
const KNOT_CACHE_ENTRY_KEY = "stellartrail_open_knots_cache";
let knotSearchTimer: ReturnType<typeof setTimeout> | null = null;
let knotListRequestSeq = 0;

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
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    if (wx.getStorageSync(KNOT_CACHE_ENTRY_KEY) === true) {
      wx.removeStorageSync(KNOT_CACHE_ENTRY_KEY);
      this.openKnotsFromEntry();
    }
  },

  onPullDownRefresh() {
    if (this.data.mode === "knots") {
      this.loadKnots().finally(() => wx.stopPullDownRefresh());
      return;
    }
    wx.stopPullDownRefresh();
  },

  onReachBottom() {
    if (this.data.mode === "knots") {
      this.loadMoreKnots();
    }
  },

  openSkillCategory(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as SkillCategoryCard["id"] | undefined;
    if (id !== "knots") {
      return;
    }
    this.openKnotsFromEntry();
  },

  openKnotsFromEntry() {
    wx.setNavigationBarTitle({ title: "绳结" });
    this.setData({ mode: "knots", error: "", offlineNotice: "" });
    if (!this.data.allKnots.length) {
      this.loadKnots();
    }
  },

  showSkillCatalog() {
    wx.setNavigationBarTitle({ title: "户外技能" });
    this.setData({ mode: "catalog", error: "", loading: false });
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
      const response = await loadKnotsPage(0, searchQuery, selectedCategoryId);
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const allKnots = await Promise.all(response.items.map(mapKnotListCard));
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const nextOffset = response.page.next_offset ?? null;
      const listState = buildKnotListState(
        allKnots,
        selectedCategoryId,
        searchQuery,
        nextOffset,
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
      const response = await loadKnotsPage(
        nextOffset,
        this.data.searchQuery,
        this.data.selectedCategoryId,
      );
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const nextKnots = await Promise.all(response.items.map(mapKnotListCard));
      if (requestSeq !== knotListRequestSeq) {
        return;
      }
      const allKnots = appendUniqueKnots(this.data.allKnots, nextKnots);
      const nextPageOffset = response.page.next_offset ?? null;
      const listState = buildKnotListState(
        allKnots,
        this.data.selectedCategoryId,
        this.data.searchQuery,
        nextPageOffset,
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        allKnots,
        ...listState,
        nextOffset: nextPageOffset,
        ...(offlineNotice ? { offlineNotice } : {}),
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
        onProgress: (progress) => {
          this.setData({
            cacheProgressText: formatKnotCacheProgress(progress),
          });
        },
      });
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
        onProgress: (progress) => {
          this.setData({
            cacheProgressText: formatKnotCacheProgress(progress),
          });
        },
      });
      const allKnots = await Promise.all(result.items.map(mapKnotListCard));
      const listState = buildKnotListState(
        allKnots,
        this.data.selectedCategoryId,
        this.data.searchQuery,
        null,
      );
      const cacheSummaryText = formatKnotCacheResult(result);
      this.setData({
        allKnots,
        ...listState,
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
    this.setData({ searchQuery, nextOffset: null, loadingMore: false });
    if (knotSearchTimer) {
      clearTimeout(knotSearchTimer);
    }
    knotSearchTimer = setTimeout(() => {
      knotSearchTimer = null;
      this.loadKnots({
        searchQuery,
        selectedCategoryId: this.data.selectedCategoryId,
        selectedCategoryIndex: this.data.selectedCategoryIndex,
      });
    }, 250);
  },

  onCategoryFilterChange(event: any) {
    const selectedCategoryIndex = Number(event.detail.value || 0);
    const selectedCategoryId = this.data.categoryFilters[selectedCategoryIndex]?.id ?? "all";
    this.loadKnots({
      searchQuery: this.data.searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
    });
  },

  clearKnotFilters() {
    if (knotSearchTimer) {
      clearTimeout(knotSearchTimer);
      knotSearchTimer = null;
    }
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

  applyFilters(filterState: {
    searchQuery: string;
    selectedCategoryId: string;
    selectedCategoryIndex: number;
  }) {
    const listState = buildKnotListState(
      this.data.allKnots,
      filterState.selectedCategoryId,
      filterState.searchQuery,
      this.data.nextOffset,
    );
    this.setData({
      ...listState,
      searchQuery: filterState.searchQuery,
    });
  },
});

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
  return `正在下载媒体 ${progress.mediaReadyCount ?? 0}/${
    progress.mediaTotal ?? 0
  }`;
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
  };
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
) {
  const categoryFilters = buildCategoryFilters(allKnots);
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
