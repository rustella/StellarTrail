import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  archiveGear,
  consumeOfflineCacheNotice,
  getErrorMessage,
  getGearOverview,
  hasAccessToken,
  isOfflineCacheMissError,
  isLoginRequiredError,
  listGears,
  restoreGear,
} from "../../utils/api-gears";
import {
  categoryFilterItems,
  formatGearPrice,
  formatGearWeight,
  GEAR_SORT_OPTIONS,
  GEAR_STATUS_FILTER_OPTIONS,
  GEAR_TAB_OPTIONS,
  getGearCategoryLabel,
  getGearStatusLabel,
  getStatusTone,
  type GearCategory,
  type GearSort,
  type GearStatsResponse,
  type GearStatus,
  type GearSummary,
  type GearTab,
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
}

interface StatCard {
  label: string;
  value: string;
  hint: string;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  archived_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};
let gearListRequestSeq = 0;

Page({
  data: {
    tabOptions: GEAR_TAB_OPTIONS,
    statusOptions: GEAR_STATUS_FILTER_OPTIONS,
    statusLabels: GEAR_STATUS_FILTER_OPTIONS.map((item) => item.label),
    sortOptions: GEAR_SORT_OPTIONS,
    sortLabels: GEAR_SORT_OPTIONS.map((item) => item.label),
    tab: "available" as GearTab,
    categories: [{ id: "all", label: "全部装备", count: 0 }],
    selectedCategory: "all",
    selectedStatus: "",
    selectedStatusIndex: 0,
    selectedSort: "created_at_desc" as GearSort,
    selectedSortIndex: 0,
    q: "",
    stats: EMPTY_STATS,
    statCards: buildStatCards(EMPTY_STATS),
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
        statCards: buildStatCards(EMPTY_STATS),
        gears: [] as GearCard[],
        nextCursor: null,
        emptyText: "登录后查看我的装备",
      });
      return;
    }

    const requestSeq = ++gearListRequestSeq;
    this.setData({ loading: true, error: "" });
    try {
      const tab = this.data.tab as GearTab;
      const overview = await getGearOverview({
        tab,
        limit: 20,
        sort: this.data.selectedSort,
      });
      if (requestSeq !== gearListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        categories: categoryFilterItems(overview.categories.items),
        stats: overview.stats,
        statCards: buildStatCards(overview.stats),
        gears: overview.list.items.map(mapGearCard),
        nextCursor: overview.list.next_cursor ?? null,
        emptyText:
          tab === "history"
            ? "历史装备为空，删除后的装备会出现在这里"
            : "还没有装备，先添加第一件户外装备吧",
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
      tab: this.data.tab as GearTab,
      category: selectedCategory === "all" ? undefined : selectedCategory,
      status: selectedStatus || undefined,
      q: this.data.q.trim() || undefined,
      sort: this.data.selectedSort as GearSort,
      limit: 20,
      cursor: cursor ?? undefined,
    };
  },

  switchTab(event: WechatMiniprogram.BaseEvent) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const tab = event.currentTarget.dataset.value as GearTab;
    if (!tab || tab === this.data.tab) {
      return;
    }
    this.setData({
      tab,
      selectedCategory: "all",
      selectedStatus: "",
      selectedStatusIndex: 0,
      nextCursor: null,
      gears: [],
    });
    this.refreshPage();
  },

  selectCategory(event: WechatMiniprogram.BaseEvent) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const id = event.currentTarget.dataset.id as "all" | GearCategory;
    this.setData({ selectedCategory: id, nextCursor: null, gears: [] });
    this.loadFirstPage();
  },

  onStatusChange(event: any) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const index = Number(event.detail.value || 0);
    const option = GEAR_STATUS_FILTER_OPTIONS[index];
    this.setData({
      selectedStatusIndex: index,
      selectedStatus: option.value,
      nextCursor: null,
      gears: [],
    });
    this.loadFirstPage();
  },

  onSortChange(event: any) {
    if (!this.data.isLoggedIn) {
      return;
    }
    const index = Number(event.detail.value || 0);
    const option = GEAR_SORT_OPTIONS[index];
    this.setData({
      selectedSortIndex: index,
      selectedSort: option.value,
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

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/gears/index") });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id;
    if (id) {
      wx.navigateTo({
        url: `/pages/gears/detail/index?id=${id}&tab=${this.data.tab}`,
      });
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

  archiveItem(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以归档或恢复自己的装备。",
        redirectUrl: "/pages/gears/index",
      })
    ) {
      return;
    }
    const id = event.currentTarget.dataset.id as string;
    wx.showModal({
      title: "移入历史装备？",
      content: "该装备会从可用列表移入历史装备，可在历史装备中恢复。",
      confirmText: "移入历史",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await archiveGear(id);
          wx.showToast({ title: "已归档", icon: "success" });
          this.refreshPage();
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后更新装备状态。",
              redirectUrl: "/pages/gears/index",
            });
            return;
          }
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },

  async restoreItem(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以把历史装备恢复到可用列表。",
        redirectUrl: "/pages/gears/index",
      })
    ) {
      return;
    }
    const id = event.currentTarget.dataset.id as string;
    try {
      await restoreGear(id);
      wx.showToast({ title: "已恢复", icon: "success" });
      this.refreshPage();
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后恢复装备。",
          redirectUrl: "/pages/gears/index",
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
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

function buildStatCards(stats: GearStatsResponse): StatCard[] {
  return [
    {
      label: "可用装备",
      value: String(stats.current_count),
      hint: "当前可直接使用",
    },
    {
      label: "历史装备",
      value: String(stats.archived_count),
      hint: "已归档",
    },
    {
      label: "总重量",
      value: formatGearWeight(stats.total_weight_g),
      hint: "按当前筛选统计",
    },
    {
      label: "估值",
      value: formatGearPrice(stats.total_value_cents),
      hint: "按购买价汇总",
    },
  ];
}
