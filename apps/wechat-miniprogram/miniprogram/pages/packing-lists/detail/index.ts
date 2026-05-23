import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  deleteGearPackingList,
  getErrorMessage,
  getGearPackingList,
  hasAccessToken,
  isLoginRequiredError,
  removeGearPackingItem,
  updateGearPackingItem,
} from "../../../utils/api-gears";
import {
  formatGearPrice,
  formatGearQuantity,
  formatGearWeight,
  formatPackingMeta,
  formatPackingProgress,
  GEAR_CATEGORY_OPTIONS,
  getGearCategoryLabel,
  getGearStatusLabel,
  type GearCategory,
  type GearPackingListDetail,
  type GearPackingListItem,
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

interface PackingItemView extends GearPackingListItem {
  categoryText: string;
  weightText: string;
  priceText: string;
  statusText: string;
  brandModelText: string;
  quantityText: string;
  plannedText: string;
  packedText: string;
  canDecreasePlanned: boolean;
  canIncreasePlanned: boolean;
  unavailableText: string;
}

interface PackingGroup {
  category: GearCategory;
  title: string;
  items: PackingItemView[];
}

Page({
  data: {
    id: "",
    detail: null as GearPackingListDetail | null,
    groups: [] as PackingGroup[],
    metaText: "",
    progressText: "0/0",
    weightText: "未记录",
    isLoggedIn: hasAccessToken(),
    loading: false,
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
    this.setData({ id });
    this.loadDetail();
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(
      "stellartrail_packing_lists_should_refresh",
    );
    if (shouldRefresh && this.data.id) {
      wx.removeStorageSync("stellartrail_packing_lists_should_refresh");
      this.loadDetail();
      return;
    }
    if (this.data.isLoggedIn !== hasAccessToken()) {
      this.loadDetail();
    }
  },

  onPullDownRefresh() {
    this.loadDetail().finally(() => wx.stopPullDownRefresh());
  },

  async loadDetail() {
    if (!this.data.id) {
      return;
    }
    if (!hasAccessToken()) {
      this.setData({ isLoggedIn: false, detail: null, error: "" });
      showLoginPrompt(this, {
        message: "登录后可以查看自己的打包清单。",
        redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
      });
      return;
    }
    this.setData({ isLoggedIn: true, loading: true, error: "" });
    try {
      const detail = await getGearPackingList(this.data.id);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...buildDetailData(detail),
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, detail: null, error: "" });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看打包清单。",
          redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  goAddGear() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    wx.navigateTo({
      url: `/pages/packing-lists/select-gears/index?id=${this.data.id}`,
    });
  },

  goEdit() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    wx.navigateTo({
      url: `/pages/packing-lists/form/index?id=${this.data.id}`,
    });
  },

  async togglePacked(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const itemId = event.currentTarget.dataset.id as string;
    const current = this.data.detail?.items.find((item) => item.id === itemId);
    if (!current) {
      return;
    }
    try {
      const detail = await updateGearPackingItem(
        this.data.id,
        itemId,
        !current.packed,
      );
      wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
      this.setData(buildDetailData(detail));
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后勾选装备。",
          redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  decreasePlanned(event: WechatMiniprogram.BaseEvent) {
    this.changePlannedQuantity(event, -1);
  },

  increasePlanned(event: WechatMiniprogram.BaseEvent) {
    this.changePlannedQuantity(event, 1);
  },

  async changePlannedQuantity(
    event: WechatMiniprogram.BaseEvent,
    delta: number,
  ) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const itemId = event.currentTarget.dataset.id as string;
    const current = this.data.detail?.items.find((item) => item.id === itemId);
    if (!current) {
      return;
    }
    const stockQuantity = Math.max(1, current.gear.quantity ?? 1);
    const nextQuantity = Math.max(
      1,
      Math.min(stockQuantity, current.planned_quantity + delta),
    );
    if (nextQuantity === current.planned_quantity) {
      return;
    }
    try {
      const detail = await updateGearPackingItem(this.data.id, itemId, {
        planned_quantity: nextQuantity,
      });
      wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
      this.setData(buildDetailData(detail));
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后调整数量。",
          redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  async removeItem(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const itemId = event.currentTarget.dataset.id as string;
    try {
      const detail = await removeGearPackingItem(this.data.id, itemId);
      wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
      this.setData(buildDetailData(detail));
      wx.showToast({ title: "已移除", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后移除装备。",
          redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  deleteList() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    wx.showModal({
      title: "删除打包清单？",
      content: "删除后不会再显示在打包清单列表中。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await deleteGearPackingList(this.data.id);
          wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
          wx.showToast({ title: "已删除", icon: "success" });
          if (getCurrentPages().length > 1) {
            wx.navigateBack();
          } else {
            wx.redirectTo({ url: "/pages/packing-lists/index" });
          }
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后删除打包清单。",
              redirectUrl: `/pages/packing-lists/detail/index?id=${encodeURIComponent(this.data.id)}`,
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

function buildDetailData(detail: GearPackingListDetail) {
  return {
    detail,
    metaText: formatPackingMeta(detail.route_name, detail.duration_label),
    progressText: formatPackingProgress(detail.stats),
    weightText: formatGearWeight(detail.stats.total_weight_g),
    groups: groupItems(detail.items),
  };
}

function groupItems(items: GearPackingListItem[]): PackingGroup[] {
  const groups = new Map<GearCategory, PackingItemView[]>();
  items.map(mapItemView).forEach((item) => {
    const category = item.gear.category;
    groups.set(category, [...(groups.get(category) ?? []), item]);
  });
  const ordered = GEAR_CATEGORY_OPTIONS.map((option) => option.value);
  return [...groups.entries()]
    .sort((a, b) => ordered.indexOf(a[0]) - ordered.indexOf(b[0]))
    .map(([category, groupItems]) => ({
      category,
      title: getGearCategoryLabel(category),
      items: groupItems,
    }));
}

function mapItemView(item: GearPackingListItem): PackingItemView {
  const brandModel = [item.gear.brand, item.gear.model]
    .filter(Boolean)
    .join(" · ");
  const stockQuantity = Math.max(1, item.gear.quantity ?? 1);
  return {
    ...item,
    categoryText:
      item.gear.category_label || getGearCategoryLabel(item.gear.category),
    weightText: formatGearWeight(item.gear.weight_g),
    priceText: formatGearPrice(
      item.gear.purchase_price_cents,
      item.gear.purchase_price_currency,
    ),
    quantityText: formatGearQuantity(stockQuantity),
    plannedText: `计划 ${item.planned_quantity}`,
    packedText: `已装 ${item.packed_quantity}`,
    canDecreasePlanned: item.planned_quantity > 1,
    canIncreasePlanned: item.planned_quantity < stockQuantity,
    statusText:
      item.unavailable_reason === "archived"
        ? "已归档"
        : item.unavailable_reason === "deleted"
          ? "已删除"
          : getGearStatusLabel(item.gear.status),
    brandModelText: brandModel || "未填写品牌型号",
    unavailableText:
      item.unavailable_reason === "archived"
        ? "这件装备已归档，仍保留在旧清单中"
        : item.unavailable_reason === "deleted"
          ? "这件装备已删除，仍保留在旧清单中"
          : "",
  };
}
