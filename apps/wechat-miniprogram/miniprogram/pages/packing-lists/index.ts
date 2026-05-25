import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  consumeOfflineCacheNotice,
  deleteGearPackingList,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  listGearPackingLists,
} from "../../utils/api-gears";
import {
  formatGearWeight,
  formatPackingMeta,
  formatPackingProgress,
  type GearPackingListSummary,
} from "../../utils/gear-utils";
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

interface PackingListCard extends GearPackingListSummary {
  metaText: string;
  progressText: string;
  weightText: string;
}

let packingListRequestSeq = 0;

Page({
  data: {
    isLoggedIn: hasAccessToken(),
    lists: [] as PackingListCard[],
    nextCursor: null as string | null,
    loading: false,
    loadingMore: false,
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad() {
    this.refreshPage();
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(
      "stellartrail_packing_lists_should_refresh",
    );
    if (shouldRefresh) {
      wx.removeStorageSync("stellartrail_packing_lists_should_refresh");
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
        lists: [] as PackingListCard[],
        nextCursor: null,
      });
      return;
    }
    const requestSeq = ++packingListRequestSeq;
    this.setData({ loading: true, loadingMore: false, error: "" });
    try {
      const response = await listGearPackingLists({ limit: 20 });
      if (requestSeq !== packingListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        lists: response.items.map(mapPackingListCard),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== packingListRequestSeq) {
        return;
      }
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, error: "", loading: false });
        showLoginPrompt(this, {
          message: "登录后可以查看自己的打包清单。",
          redirectUrl: "/pages/packing-lists/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error), lists: [] });
    } finally {
      if (requestSeq === packingListRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  async loadMore() {
    if (
      !this.data.isLoggedIn ||
      !this.data.nextCursor ||
      this.data.loading ||
      this.data.loadingMore
    ) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = packingListRequestSeq;
    try {
      const response = await listGearPackingLists({
        limit: 20,
        cursor: this.data.nextCursor,
      });
      if (requestSeq !== packingListRequestSeq) {
        return;
      }
      const cards = response.items.map(mapPackingListCard);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...indexedAppendData("lists", this.data.lists.length, cards),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, loadingMore: false });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看打包清单。",
          redirectUrl: "/pages/packing-lists/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loadingMore: false });
    }
  },

  goCreate() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以创建自己的打包清单。",
        redirectUrl: "/pages/packing-lists/form/index",
      })
    ) {
      return;
    }
    wx.navigateTo({ url: "/pages/packing-lists/form/index" });
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/packing-lists/index") });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id;
    if (id) {
      wx.navigateTo({ url: `/pages/packing-lists/detail/index?id=${id}` });
    }
  },

  deleteList(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const id = event.currentTarget.dataset.id as string;
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
          await deleteGearPackingList(id);
          wx.showToast({ title: "已删除", icon: "success" });
          this.refreshPage();
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后删除打包清单。",
              redirectUrl: "/pages/packing-lists/index",
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

function mapPackingListCard(item: GearPackingListSummary): PackingListCard {
  return {
    ...item,
    metaText: formatPackingMeta(item.route_name, item.duration_label),
    progressText: formatPackingProgress(item),
    weightText: formatGearWeight(item.total_weight_g),
  };
}
