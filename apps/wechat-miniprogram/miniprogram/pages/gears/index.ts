import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  consumeOfflineCacheNotice,
  deleteGear,
  getErrorMessage,
  getGearOverview,
  hasAccessToken,
  isOfflineCacheMissError,
  isLoginRequiredError,
  listGears,
} from "../../utils/api-gears";
import {
  categoryFilterItems,
  formatGearQuantity,
  formatGearPrice,
  formatGearWeight,
  GEAR_SORT_OPTIONS,
  GEAR_STATUS_FILTER_OPTIONS,
  getGearCategoryLabel,
  getGearStatusLabel,
  getStatusTone,
  type GearCategory,
  type GearSort,
  type GearStatsResponse,
  type GearStatus,
  type GearSummary,
} from "../../utils/gear-display";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  loginPageUrl,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../utils/network-state";
import { indexedAppendData } from "../../utils/page-data";

interface GearCard extends GearSummary {
  categoryText: string;
  statusText: string;
  statusTone: string;
  weightText: string;
  priceText: string;
  purchaseDateText: string;
  brandModelText: string;
  quantityText: string;
  showQuantityBadge: boolean;
}

interface StatCard {
  label: string;
  value: string;
  hint: string;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};
const DEFAULT_CATEGORY = "all";
const DEFAULT_STATUS = "";
const DEFAULT_SORT = "created_at_desc" as GearSort;
const DEFAULT_STATUS_INDEX = Math.max(
  GEAR_STATUS_FILTER_OPTIONS.findIndex((item) => item.value === DEFAULT_STATUS),
  0,
);
const DEFAULT_SORT_INDEX = Math.max(
  GEAR_SORT_OPTIONS.findIndex((item) => item.value === DEFAULT_SORT),
  0,
);
let gearListRequestSeq = 0;

Page({
  data: {
    statusOptions: GEAR_STATUS_FILTER_OPTIONS,
    statusLabels: GEAR_STATUS_FILTER_OPTIONS.map((item) => item.label),
    sortOptions: GEAR_SORT_OPTIONS,
    sortLabels: GEAR_SORT_OPTIONS.map((item) => item.label),
    categories: [{ id: "all", label: "全部装备", count: 0 }],
    selectedCategory: DEFAULT_CATEGORY,
    selectedStatus: DEFAULT_STATUS,
    selectedStatusIndex: DEFAULT_STATUS_INDEX,
    selectedSort: DEFAULT_SORT,
    selectedSortIndex: DEFAULT_SORT_INDEX,
    draftSelectedCategory: DEFAULT_CATEGORY,
    draftSelectedStatus: DEFAULT_STATUS,
    draftSelectedStatusIndex: DEFAULT_STATUS_INDEX,
    draftSelectedSort: DEFAULT_SORT,
    draftSelectedSortIndex: DEFAULT_SORT_INDEX,
    activeFilterText: buildActiveFilterText(
      [{ id: "all", label: "全部装备", count: 0 }],
      DEFAULT_CATEGORY,
      DEFAULT_STATUS_INDEX,
      DEFAULT_SORT_INDEX,
    ),
    activeFilterCount: 0,
    filterSheetVisible: false,
    q: "",
    stats: EMPTY_STATS,
    statCards: buildStatCards(EMPTY_STATS, 0),
    gears: [] as GearCard[],
    nextCursor: null as string | null,
    isLoggedIn: hasAccessToken(),
    loading: false,
    loadingMore: false,
    error: "",
    offlineNotice: "",
    emptyText: "还没有装备，先添加第一件户外装备吧",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad() {
    this.refreshPage();
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(
      "stellartrail_gears_should_refresh",
    );
    if (shouldRefresh) {
      wx.removeStorageSync("stellartrail_gears_should_refresh");
      this.refreshPage();
      return;
    }
    if (this.data.isLoggedIn !== hasAccessToken()) {
      this.refreshPage();
    }
  },

  onPullDownRefresh() {
    this.refreshPage().finally(() => wx.stopPullDownRefresh());
  },

  onReachBottom() {
    this.loadMore();
  },

  async refreshPage() {
    const isLoggedIn = hasAccessToken();
    this.setData({ isLoggedIn, error: "" });
    if (!isLoggedIn) {
      this.setData({
        loading: false,
        loadingMore: false,
        categories: [{ id: "all", label: "全部装备", count: 0 }],
        stats: EMPTY_STATS,
        statCards: buildStatCards(EMPTY_STATS, 0),
        gears: [] as GearCard[],
        nextCursor: null,
        emptyText: "登录后查看我的装备",
        filterSheetVisible: false,
      });
      return;
    }

    const requestSeq = ++gearListRequestSeq;
    this.setData({ loading: true, error: "" });
    try {
      const overview = await getGearOverview({
        limit: 20,
        sort: this.data.selectedSort,
      });
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      const categories = categoryFilterItems(overview.categories.items);
      const selectedCategory = normalizeSelectedCategory(
        this.data.selectedCategory as "all" | GearCategory,
        categories,
      );
      this.setData({
        categories,
        selectedCategory,
        draftSelectedCategory: this.data.filterSheetVisible
          ? this.data.draftSelectedCategory
          : selectedCategory,
        stats: overview.stats,
        statCards: buildStatCards(overview.stats, categoryCount(categories)),
        gears: overview.list.items.map(mapGearCard),
        nextCursor: overview.list.next_cursor ?? null,
        activeFilterText: buildActiveFilterText(
          categories,
          selectedCategory,
          this.data.selectedStatusIndex,
          this.data.selectedSortIndex,
        ),
        activeFilterCount: buildActiveFilterCount(
          selectedCategory,
          this.data.selectedStatus,
          this.data.selectedSort,
        ),
        emptyText: "还没有装备，先添加第一件户外装备吧",
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, error: "", loading: false });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看自己的装备。",
          redirectUrl: "/pages/gears/index",
        });
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.gears.length) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      if (requestSeq === gearListRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  async loadMore() {
    if (
      !this.data.isLoggedIn ||
      !this.data.nextCursor ||
      this.data.loadingMore ||
      this.data.loading
    ) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = gearListRequestSeq;
    try {
      const response = await listGears(
        this.buildListRequest(this.data.nextCursor),
      );
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      const cards = response.items.map(mapGearCard);
      this.setData({
        ...indexedAppendData("gears", this.data.gears.length, cards),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, loadingMore: false });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后继续查看自己的装备。",
          redirectUrl: "/pages/gears/index",
        });
        return;
      }
      if (isOfflineCacheMissError(error)) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loadingMore: false });
    }
  },

  async loadFirstPage() {
    if (!this.data.isLoggedIn) {
      return;
    }
    const requestSeq = ++gearListRequestSeq;
    this.setData({ loading: true, loadingMore: false, error: "" });
    try {
      const response = await listGears(this.buildListRequest(null));
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        gears: response.items.map(mapGearCard),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, error: "", loading: false });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看自己的装备。",
          redirectUrl: "/pages/gears/index",
        });
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.gears.length) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({ error: getErrorMessage(error), gears: [] as GearCard[] });
    } finally {
      if (requestSeq === gearListRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  buildListRequest(cursor: string | null) {
    const selectedCategory = this.data.selectedCategory as "all" | GearCategory;
    const selectedStatus = this.data.selectedStatus as "" | GearStatus;
    return {
      category: selectedCategory === "all" ? undefined : selectedCategory,
      status: selectedStatus || undefined,
      q: this.data.q.trim() || undefined,
      sort: this.data.selectedSort as GearSort,
      limit: 20,
      cursor: cursor ?? undefined,
    };
  },

  openFilterSheet() {
    if (!this.data.isLoggedIn) {
      return;
    }
    this.setData({
      filterSheetVisible: true,
      draftSelectedCategory: this.data.selectedCategory,
      draftSelectedStatus: this.data.selectedStatus,
      draftSelectedStatusIndex: this.data.selectedStatusIndex,
      draftSelectedSort: this.data.selectedSort,
      draftSelectedSortIndex: this.data.selectedSortIndex,
    });
  },

  closeFilterSheet() {
    this.setData({ filterSheetVisible: false });
  },

  stopTap() {},

  selectDraftCategory(event: WechatMiniprogram.BaseEvent) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const id = event.currentTarget.dataset.id as "all" | GearCategory;
    this.setData({ draftSelectedCategory: id || DEFAULT_CATEGORY });
  },

  selectDraftStatus(event: WechatMiniprogram.BaseEvent) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const index = Number(event.currentTarget.dataset.index || 0);
    const option =
      GEAR_STATUS_FILTER_OPTIONS[index] ||
      GEAR_STATUS_FILTER_OPTIONS[DEFAULT_STATUS_INDEX];
    this.setData({
      draftSelectedStatusIndex: index,
      draftSelectedStatus: option.value,
    });
  },

  selectDraftSort(event: WechatMiniprogram.BaseEvent) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const index = Number(
      event.currentTarget.dataset.index || DEFAULT_SORT_INDEX,
    );
    const option =
      GEAR_SORT_OPTIONS[index] || GEAR_SORT_OPTIONS[DEFAULT_SORT_INDEX];
    this.setData({
      draftSelectedSortIndex: index,
      draftSelectedSort: option.value,
    });
  },

  resetFilterDrafts() {
    this.setData(defaultDraftFilterData());
  },

  applyFilters() {
    if (!this.data.isLoggedIn) {
      return;
    }
    const selectedCategory = this.data.draftSelectedCategory as
      | "all"
      | GearCategory;
    const selectedStatus = this.data.draftSelectedStatus as "" | GearStatus;
    const selectedSort = this.data.draftSelectedSort as GearSort;
    this.setData({
      selectedCategory,
      selectedStatus,
      selectedStatusIndex: this.data.draftSelectedStatusIndex,
      selectedSort,
      selectedSortIndex: this.data.draftSelectedSortIndex,
      activeFilterText: buildActiveFilterText(
        this.data.categories,
        selectedCategory,
        this.data.draftSelectedStatusIndex,
        this.data.draftSelectedSortIndex,
      ),
      activeFilterCount: buildActiveFilterCount(
        selectedCategory,
        selectedStatus,
        selectedSort,
      ),
      filterSheetVisible: false,
      nextCursor: null,
      gears: [],
    });
    this.loadFirstPage();
  },

  clearAppliedFilters() {
    if (!this.data.isLoggedIn) {
      return;
    }
    this.setData({
      selectedCategory: DEFAULT_CATEGORY,
      selectedStatus: DEFAULT_STATUS,
      selectedStatusIndex: DEFAULT_STATUS_INDEX,
      selectedSort: DEFAULT_SORT,
      selectedSortIndex: DEFAULT_SORT_INDEX,
      ...defaultDraftFilterData(),
      activeFilterText: buildActiveFilterText(
        this.data.categories,
        DEFAULT_CATEGORY,
        DEFAULT_STATUS_INDEX,
        DEFAULT_SORT_INDEX,
      ),
      activeFilterCount: 0,
      filterSheetVisible: false,
      nextCursor: null,
      gears: [],
    });
    this.loadFirstPage();
  },

  onSearchInput(event: any) {
    this.setData({ q: event.detail.value });
  },

  submitSearch() {
    if (!this.data.isLoggedIn) {
      return;
    }
    this.setData({ nextCursor: null, gears: [] });
    this.loadFirstPage();
  },

  clearSearch() {
    if (!this.data.isLoggedIn) {
      this.setData({ q: "" });
      return;
    }
    this.setData({ q: "", nextCursor: null, gears: [] });
    this.loadFirstPage();
  },

  goCreate() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后就能把这件装备保存到自己的清单里。",
        redirectUrl: "/pages/gears/form/index",
      })
    ) {
      return;
    }
    wx.navigateTo({ url: "/pages/gears/form/index" });
  },

  goGearAtlas() {
    wx.navigateTo({ url: "/pages/gear-atlas/index" });
  },

  goPackingLists() {
    wx.navigateTo({ url: "/pages/packing-lists/index" });
  },

  goStatsDetail() {
    wx.navigateTo({ url: "/pages/gears/stats/index" });
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/gears/index") });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id;
    if (id) {
      wx.navigateTo({ url: `/pages/gears/detail/index?id=${id}` });
    }
  },

  goEdit(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以编辑自己的装备。",
        redirectUrl: "/pages/gears/index",
      })
    ) {
      return;
    }
    const id = event.currentTarget.dataset.id;
    if (id) {
      wx.navigateTo({ url: `/pages/gears/form/index?id=${id}` });
    }
  },

  deleteItem(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以删除自己的装备。",
        redirectUrl: "/pages/gears/index",
      })
    ) {
      return;
    }
    const id = event.currentTarget.dataset.id as string;
    wx.showModal({
      title: "删除这件装备？",
      content: "删除后不会出现在装备列表中，已有打包清单会保留历史条目。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await deleteGear(id);
          wx.showToast({ title: "已删除", icon: "success" });
          this.refreshPage();
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后删除装备。",
              redirectUrl: "/pages/gears/index",
            });
            return;
          }
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function mapGearCard(item: GearSummary): GearCard {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    ...item,
    categoryText: item.category_label || getGearCategoryLabel(item.category),
    statusText: item.status_label || getGearStatusLabel(item.status),
    statusTone: getStatusTone(item.status),
    quantityText: formatGearQuantity(item.quantity),
    showQuantityBadge: (item.quantity ?? 1) > 1,
    weightText: formatGearWeight(item.weight_g),
    priceText: formatGearPrice(
      item.purchase_price_cents,
      item.purchase_price_currency,
    ),
    purchaseDateText: item.purchase_date
      ? item.purchase_date.slice(0, 10)
      : "未记录",
    brandModelText: brandModel || "未填写品牌型号",
  };
}

function buildStatCards(
  stats: GearStatsResponse,
  categoryTotal: number,
): StatCard[] {
  return [
    {
      label: "装备数量",
      value: String(stats.current_count),
      hint: "当前库存",
    },
    {
      label: "分类数",
      value: String(categoryTotal),
      hint: "已有分类",
    },
    {
      label: "总重量",
      value: formatGearWeight(stats.total_weight_g),
      hint: "当前库存",
    },
    {
      label: "估值",
      value: formatGearPrice(stats.total_value_cents),
      hint: "按购买价汇总",
    },
  ];
}

function categoryCount(
  categories: Array<{ id: string; label: string; count: number }>,
): number {
  return categories.filter((item) => item.id !== DEFAULT_CATEGORY).length;
}

function normalizeSelectedCategory(
  selectedCategory: "all" | GearCategory,
  categories: Array<{ id: string; label: string; count: number }>,
): "all" | GearCategory {
  return categories.some((item) => item.id === selectedCategory)
    ? selectedCategory
    : DEFAULT_CATEGORY;
}

function buildActiveFilterText(
  categories: Array<{ id: string; label: string; count: number }>,
  selectedCategory: "all" | GearCategory,
  selectedStatusIndex: number,
  selectedSortIndex: number,
): string {
  const category =
    categories.find((item) => item.id === selectedCategory)?.label ||
    "全部装备";
  const status =
    GEAR_STATUS_FILTER_OPTIONS[selectedStatusIndex]?.label || "全部状态";
  const sort = GEAR_SORT_OPTIONS[selectedSortIndex]?.label || "最近添加";
  return `${category} · ${status} · ${sort}`;
}

function buildActiveFilterCount(
  selectedCategory: "all" | GearCategory,
  selectedStatus: string,
  selectedSort: GearSort,
): number {
  return [
    selectedCategory !== DEFAULT_CATEGORY,
    Boolean(selectedStatus),
    selectedSort !== DEFAULT_SORT,
  ].filter(Boolean).length;
}

function defaultDraftFilterData() {
  return {
    draftSelectedCategory: DEFAULT_CATEGORY,
    draftSelectedStatus: DEFAULT_STATUS,
    draftSelectedStatusIndex: DEFAULT_STATUS_INDEX,
    draftSelectedSort: DEFAULT_SORT,
    draftSelectedSortIndex: DEFAULT_SORT_INDEX,
  };
}
