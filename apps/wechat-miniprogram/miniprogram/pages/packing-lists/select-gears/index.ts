import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  addGearPackingItems,
  consumeOfflineCacheNotice,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  listGears,
} from "../../../utils/api-gears";
import { importTripPackingList } from "../../../utils/api-trips";
import {
  formatGearPrice,
  formatGearQuantity,
  formatGearWeight,
  GEAR_CATEGORY_OPTIONS,
  GEAR_SORT_OPTIONS,
  GEAR_STATUS_FILTER_OPTIONS,
  getGearCategoryLabel,
  getGearStatusLabel,
  type GearCategory,
  type GearSort,
  type GearStatus,
  type GearSummary,
} from "../../../utils/gear-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  showLoginPrompt,
} from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { indexedAppendData } from "../../../utils/page-data";

interface CategoryOption {
  id: "all" | GearCategory;
  label: string;
}

interface GearSelectCard extends GearSummary {
  selected: boolean;
  categoryText: string;
  statusText: string;
  weightText: string;
  priceText: string;
  quantityText: string;
  showQuantityBadge: boolean;
  brandModelText: string;
}

const CATEGORY_OPTIONS: CategoryOption[] = [
  { id: "all", label: "全部装备" },
  ...GEAR_CATEGORY_OPTIONS.map((item) => ({
    id: item.value,
    label: item.label,
  })),
];

let selectGearRequestSeq = 0;

Page({
  data: {
    id: "",
    returnTripId: "",
    categoryOptions: CATEGORY_OPTIONS,
    statusOptions: GEAR_STATUS_FILTER_OPTIONS,
    statusLabels: GEAR_STATUS_FILTER_OPTIONS.map((item) => item.label),
    sortOptions: GEAR_SORT_OPTIONS,
    sortLabels: GEAR_SORT_OPTIONS.map((item) => item.label),
    selectedCategory: "all" as "all" | GearCategory,
    selectedStatus: "" as "" | GearStatus,
    selectedStatusIndex: 0,
    selectedSort: "created_at_desc" as GearSort,
    selectedSortIndex: 0,
    q: "",
    selectedIds: [] as string[],
    gears: [] as GearSelectCard[],
    nextCursor: null as string | null,
    isLoggedIn: hasAccessToken(),
    loading: false,
    loadingMore: false,
    submitting: false,
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id || "";
    if (!id) {
      this.setData({ error: "没有找到这份打包清单，请返回后重试" });
      return;
    }
    this.setData({ id, returnTripId: options.returnTripId || "" });
    this.refreshPage();
  },

  onShow() {
    syncPageTheme(this);
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
      showLoginPrompt(this, {
        message: "登录后可以从个人装备中挑选打包物品。",
        redirectUrl: selectGearPageUrl(
          this.data.id,
          this.data.returnTripId,
        ),
      });
      return;
    }
    const requestSeq = ++selectGearRequestSeq;
    this.setData({ loading: true, loadingMore: false, error: "" });
    try {
      const response = await listGears(this.buildListRequest(null));
      if (requestSeq !== selectGearRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        gears: response.items.map((item) =>
          mapGearCard(item, this.data.selectedIds),
        ),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== selectGearRequestSeq) {
        return;
      }
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, loading: false });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后挑选装备。",
          redirectUrl: selectGearPageUrl(
            this.data.id,
            this.data.returnTripId,
          ),
        });
        return;
      }
      this.setData({ error: getErrorMessage(error), gears: [] });
    } finally {
      if (requestSeq === selectGearRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  async loadMore() {
    if (!this.data.nextCursor || this.data.loading || this.data.loadingMore) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = selectGearRequestSeq;
    try {
      const response = await listGears(
        this.buildListRequest(this.data.nextCursor),
      );
      if (requestSeq !== selectGearRequestSeq) {
        return;
      }
      const cards = response.items.map((item) =>
        mapGearCard(item, this.data.selectedIds),
      );
      this.setData({
        ...indexedAppendData("gears", this.data.gears.length, cards),
        nextCursor: response.next_cursor ?? null,
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后挑选装备。",
          redirectUrl: selectGearPageUrl(
            this.data.id,
            this.data.returnTripId,
          ),
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loadingMore: false });
    }
  },

  buildListRequest(cursor: string | null) {
    const selectedCategory = this.data.selectedCategory;
    const selectedStatus = this.data.selectedStatus;
    return {
      category: selectedCategory === "all" ? undefined : selectedCategory,
      status: selectedStatus || undefined,
      q: this.data.q.trim() || undefined,
      sort: this.data.selectedSort,
      limit: 20,
      cursor: cursor ?? undefined,
    };
  },

  selectCategory(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as "all" | GearCategory;
    this.setData({ selectedCategory: id, gears: [], nextCursor: null });
    this.refreshPage();
  },

  onStatusChange(event: any) {
    const index = Number(event.detail.value || 0);
    const option = GEAR_STATUS_FILTER_OPTIONS[index];
    this.setData({
      selectedStatusIndex: index,
      selectedStatus: option.value,
      gears: [],
      nextCursor: null,
    });
    this.refreshPage();
  },

  onSortChange(event: any) {
    const index = Number(event.detail.value || 0);
    const option = GEAR_SORT_OPTIONS[index];
    this.setData({
      selectedSortIndex: index,
      selectedSort: option.value,
      gears: [],
      nextCursor: null,
    });
    this.refreshPage();
  },

  onSearchInput(event: any) {
    this.setData({ q: event.detail.value });
  },

  submitSearch() {
    this.setData({ gears: [], nextCursor: null });
    this.refreshPage();
  },

  clearSearch() {
    this.setData({ q: "", gears: [], nextCursor: null });
    this.refreshPage();
  },

  toggleSelect(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string;
    const selectedIds = this.data.selectedIds.includes(id)
      ? this.data.selectedIds.filter((item) => item !== id)
      : [...this.data.selectedIds, id];
    this.setData({
      selectedIds,
      gears: this.data.gears.map((item) => mapGearCard(item, selectedIds)),
    });
  },

  async submitSelection() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (!this.data.selectedIds.length) {
      wx.showToast({ title: "请选择要加入的装备", icon: "none" });
      return;
    }
    this.setData({ submitting: true });
    try {
      await addGearPackingItems(this.data.id, this.data.selectedIds);
      wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
      if (this.data.returnTripId) {
        await importTripPackingList(this.data.returnTripId, {
          packing_list_id: this.data.id,
        });
        wx.setStorageSync("stellartrail_trips_refresh", true);
        wx.redirectTo({
          url: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.returnTripId)}&section=personal_gear`,
        });
        return;
      }
      wx.redirectTo({
        url: `/pages/packing-lists/detail/index?id=${this.data.id}`,
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后加入装备。",
          redirectUrl: selectGearPageUrl(
            this.data.id,
            this.data.returnTripId,
          ),
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ submitting: false });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function mapGearCard(item: GearSummary, selectedIds: string[]): GearSelectCard {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    ...item,
    selected: selectedIds.includes(item.id),
    categoryText: item.category_label || getGearCategoryLabel(item.category),
    statusText: item.status_label || getGearStatusLabel(item.status),
    weightText: formatGearWeight(item.weight_g),
    quantityText: formatGearQuantity(item.quantity),
    showQuantityBadge: (item.quantity ?? 1) > 1,
    priceText: formatGearPrice(
      item.purchase_price_cents,
      item.purchase_price_currency,
    ),
    brandModelText: brandModel || "未填写品牌型号",
  };
}

function selectGearPageUrl(id: string, returnTripId: string): string {
  const params = [`id=${encodeURIComponent(id)}`];
  if (returnTripId) {
    params.push(`returnTripId=${encodeURIComponent(returnTripId)}`);
  }
  return `/pages/packing-lists/select-gears/index?${params.join("&")}`;
}
