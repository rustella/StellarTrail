import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  isOfflineCacheMissError,
  listKnots,
  resolveAssetUrl,
} from "../../utils/api";
import {
  mapSkillCard,
  type KnotMediaAsset,
  type KnotSummary,
  type SkillCard,
} from "../../utils/skill-utils";
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
    loading: false,
    error: "",
    offlineNotice: "",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },

  onPullDownRefresh() {
    if (this.data.mode === "knots") {
      this.loadKnots().finally(() => wx.stopPullDownRefresh());
      return;
    }
    wx.stopPullDownRefresh();
  },

  openSkillCategory(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as SkillCategoryCard["id"] | undefined;
    if (id !== "knots") {
      return;
    }
    wx.setNavigationBarTitle({ title: "绳结" });
    this.setData({ mode: "knots", error: "", offlineNotice: "" });
    this.loadKnots();
  },

  showSkillCatalog() {
    wx.setNavigationBarTitle({ title: "户外技能" });
    this.setData({ mode: "catalog", error: "", loading: false });
  },

  async loadKnots() {
    this.setData({ loading: true, error: "" });
    try {
      const items = await loadAllKnots();
      const allKnots = await Promise.all(items.map(mapKnotListCard));
      const categoryFilters = buildCategoryFilters(allKnots);
      const selectedCategoryIndex = validCategoryIndex(
        categoryFilters,
        this.data.selectedCategoryId,
      );
      const selectedCategoryId = categoryFilters[selectedCategoryIndex]?.id ?? "all";
      const filteredKnots = filterKnots(
        allKnots,
        selectedCategoryId,
        this.data.searchQuery,
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        allKnots,
        knots: filteredKnots,
        categoryFilters,
        categoryFilterLabels: categoryFilters.map(formatCategoryFilterLabel),
        selectedCategoryId,
        selectedCategoryIndex,
        hasActiveFilters: hasActiveFilters(selectedCategoryId, this.data.searchQuery),
        listResultText: listResultText(filteredKnots.length, allKnots.length),
        loading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isOfflineCacheMissError(error) && this.data.allKnots.length) {
        this.setData({ loading: false });
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        allKnots: [] as KnotListCard[],
        knots: [] as KnotListCard[],
      });
    }
  },

  onSearchInput(event: any) {
    const searchQuery = String(event.detail.value ?? "");
    this.applyFilters({
      searchQuery,
      selectedCategoryId: this.data.selectedCategoryId,
      selectedCategoryIndex: this.data.selectedCategoryIndex,
    });
  },

  onCategoryFilterChange(event: any) {
    const selectedCategoryIndex = Number(event.detail.value || 0);
    const selectedCategoryId = this.data.categoryFilters[selectedCategoryIndex]?.id ?? "all";
    this.applyFilters({
      searchQuery: this.data.searchQuery,
      selectedCategoryId,
      selectedCategoryIndex,
    });
  },

  clearKnotFilters() {
    this.applyFilters({
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
    const knots = filterKnots(
      this.data.allKnots,
      filterState.selectedCategoryId,
      filterState.searchQuery,
    );
    this.setData({
      ...filterState,
      knots,
      hasActiveFilters: hasActiveFilters(
        filterState.selectedCategoryId,
        filterState.searchQuery,
      ),
      listResultText: listResultText(knots.length, this.data.allKnots.length),
    });
  },
});

async function loadAllKnots(): Promise<KnotSummary[]> {
  const limit = 100;
  let offset = 0;
  const items: KnotSummary[] = [];
  for (;;) {
    const response = await listKnots({ offset, limit });
    response.items.forEach((item) => items.push(item));
    if (response.page.next_offset == null) {
      return items;
    }
    offset = response.page.next_offset;
  }
}

async function mapKnotListCard(item: KnotSummary): Promise<KnotListCard> {
  const thumbnail = findThumbnail(item.media);
  const thumbnailUrl = thumbnail
    ? await resolveCachedMediaUrl(resolveAssetUrl(thumbnail.url))
    : "";
  const categoryIds = item.categories.map((category) => category.id || category.slug);
  const categoryTitles = item.categories.map((category) => category.title);
  const searchParts = [item.title, item.summary].concat(
    categoryTitles,
    item.types.map((type) => type.title),
  );
  return {
    ...mapSkillCard(item),
    thumbnailUrl,
    hasThumbnail: Boolean(thumbnailUrl),
    categoryIds,
    categoryTitles,
    searchText: searchParts.join(" ").toLocaleLowerCase(),
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

function filterKnots(
  knots: KnotListCard[],
  selectedCategoryId: string,
  searchQuery: string,
): KnotListCard[] {
  const query = searchQuery.trim().toLocaleLowerCase();
  return knots.filter((knot) => {
    const matchesCategory =
      selectedCategoryId === "all" || knot.categoryIds.includes(selectedCategoryId);
    const matchesSearch = !query || knot.searchText.includes(query);
    return matchesCategory && matchesSearch;
  });
}

function hasActiveFilters(selectedCategoryId: string, searchQuery: string): boolean {
  return selectedCategoryId !== "all" || Boolean(searchQuery.trim());
}

function listResultText(filteredCount: number, totalCount: number): string {
  if (!totalCount) {
    return "";
  }
  if (filteredCount === totalCount) {
    return `共 ${totalCount} 个绳结`;
  }
  return `已筛出 ${filteredCount} / ${totalCount} 个绳结`;
}
