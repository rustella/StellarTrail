import {
  clearLoginState,
  createFeedback,
  type FeedbackCategory,
  getCurrentUser,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  isNotFoundApiError,
  uploadWechatAvatar,
} from "../../utils/api";
import { buildAccountProfile } from "../../utils/account-profile";
import { loginPageUrl } from "../../utils/auth-prompt";
import {
  getThemeViewData,
  syncPageTheme,
  togglePageTheme,
} from "../../utils/theme";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../utils/network-state";
import {
  clearKnotOfflineCache,
  deleteCachedKnot,
  getKnotOfflineCacheInventory,
  refreshKnotOfflineCacheInventory,
  type CachedKnotPreview,
  type KnotOfflineCacheInventory,
} from "../../utils/knot-offline-cache";

const KNOT_CACHE_ENTRY_KEY = "stellartrail_open_knots_cache";

const FEEDBACK_CATEGORY_OPTIONS: Array<{
  value: FeedbackCategory;
  label: string;
}> = [
  { value: "suggestion", label: "功能建议" },
  { value: "bug", label: "问题反馈" },
  { value: "content_correction", label: "内容纠错" },
  { value: "other", label: "其他" },
];

Page({
  data: {
    title: "我的寻径星野",
    loggedIn: hasAccessToken(),
    accountProfile: buildAccountProfile(hasAccessToken()),
    feedbackCategoryLabels: FEEDBACK_CATEGORY_OPTIONS.map((item) => item.label),
    feedbackCategoryIndex: 0,
    feedbackContent: "",
    feedbackContact: "",
    feedbackModalVisible: false,
    feedbackLoading: false,
    feedbackError: "",
    aboutModalVisible: false,
    aboutInfo: buildAboutInfo(),
    cachedKnotsModalVisible: false,
    cachedKnots: [] as CachedKnotPreview[],
    cachedKnotsInfo: buildCachedKnotsInfo(
      getKnotOfflineCacheInventory("zh-CN"),
    ),
    cachedKnotsStatsLoading: false,
    accountError: "",
    avatarLoading: false,
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    void this.refreshAccountState();
  },

  async refreshAccountState() {
    const loggedIn = hasAccessToken();
    this.setData({
      loggedIn,
      accountProfile: buildAccountProfile(loggedIn),
      accountError: "",
    });
    if (!loggedIn) {
      return;
    }
    try {
      await getCurrentUser();
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        accountError: "",
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({
          loggedIn: false,
          accountProfile: buildAccountProfile(false),
          accountError: "",
        });
        return;
      }
      if (isNotFoundApiError(error)) {
        this.setData({
          loggedIn: hasAccessToken(),
          accountProfile: buildAccountProfile(hasAccessToken()),
          accountError: "",
        });
        return;
      }
      this.setData({ accountError: getErrorMessage(error) });
    }
  },

  onChooseWechatAvatar(
    event: WechatMiniprogram.CustomEvent<{ avatarUrl?: string }>,
  ) {
    const avatarPath = event.detail.avatarUrl || "";
    if (!avatarPath || !this.data.loggedIn || this.data.avatarLoading) {
      return;
    }
    void this.uploadProfileAvatar(avatarPath);
  },

  async uploadProfileAvatar(filePath: string) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({ avatarLoading: true, accountError: "" });
    try {
      await uploadWechatAvatar(filePath);
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        accountError: "",
      });
      wx.showToast({ title: "头像已更新", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.resetLoggedOutAccountState();
        return;
      }
      if (isNotFoundApiError(error)) {
        this.setData({ accountError: "头像保存暂不可用，请稍后再试" });
        return;
      }
      this.setData({
        accountError: `头像保存失败：${getErrorMessage(error)}`,
      });
    } finally {
      this.setData({ avatarLoading: false });
    }
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/profile/index") });
  },

  openUserSettings() {
    if (!this.data.loggedIn) {
      this.goLogin();
      return;
    }
    wx.navigateTo({ url: "/pages/profile/settings/index" });
  },

  logout() {
    wx.showModal({
      title: "退出登录？",
      content: "退出后仍可浏览装备图鉴和绳结教学。",
      confirmText: "退出",
      confirmColor: "#dc2626",
      success: (result) => {
        if (!result.confirm) {
          return;
        }
        clearLoginState();
        void this.refreshAccountState();
        wx.showToast({ title: "已退出", icon: "success" });
      },
    });
  },

  toggleTheme() {
    togglePageTheme(this);
  },

  openCachedKnotsModal() {
    const inventory = getKnotOfflineCacheInventory("zh-CN");
    this.setData({
      cachedKnotsModalVisible: true,
      cachedKnots: inventory.items,
      cachedKnotsInfo: buildCachedKnotsInfo(inventory),
    });
    void this.refreshCachedKnotsInfo();
  },

  closeCachedKnotsModal() {
    this.setData({ cachedKnotsModalVisible: false });
  },

  async refreshCachedKnotsInfo() {
    this.setData({ cachedKnotsStatsLoading: true });
    try {
      const inventory = await refreshKnotOfflineCacheInventory("zh-CN");
      this.applyKnotCacheInventory(inventory);
    } catch {
      this.applyKnotCacheInventory(getKnotOfflineCacheInventory("zh-CN"));
    } finally {
      this.setData({ cachedKnotsStatsLoading: false });
    }
  },

  applyKnotCacheInventory(inventory: KnotOfflineCacheInventory) {
    this.setData({
      cachedKnots: inventory.items,
      cachedKnotsInfo: buildCachedKnotsInfo(inventory),
    });
  },

  goKnotOfflineCache() {
    this.setData({ cachedKnotsModalVisible: false });
    wx.setStorageSync(KNOT_CACHE_ENTRY_KEY, true);
    wx.switchTab({ url: "/pages/skills/index" });
  },

  openCachedKnotDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    this.setData({ cachedKnotsModalVisible: false });
    wx.navigateTo({
      url: `/pages/skills/detail/index?id=${encodeURIComponent(id)}`,
    });
  },

  removeCachedKnot(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    const title = event.currentTarget.dataset.title as string | undefined;
    if (!id) {
      return;
    }
    wx.showModal({
      title: "删除绳结缓存？",
      content: `将删除「${title || "这个绳结"}」的离线详情和媒体资源。`,
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (!result.confirm) {
          return;
        }
        const inventory = deleteCachedKnot(id, "zh-CN");
        this.applyKnotCacheInventory(inventory);
        wx.showToast({ title: "已删除缓存", icon: "success" });
      },
    });
  },

  clearCachedKnots() {
    if (!this.data.cachedKnots.length) {
      return;
    }
    wx.showModal({
      title: "删除全部绳结缓存？",
      content: "将删除已缓存的绳结详情、离线列表和对应媒体资源。",
      confirmText: "删除全部",
      confirmColor: "#dc2626",
      success: (result) => {
        if (!result.confirm) {
          return;
        }
        const inventory = clearKnotOfflineCache("zh-CN");
        this.applyKnotCacheInventory(inventory);
        wx.showToast({ title: "已清空缓存", icon: "success" });
      },
    });
  },

  openFeedbackModal() {
    if (!this.data.loggedIn) {
      wx.showModal({
        title: "登录后反馈",
        content: "登录后可以提交意见反馈，帮助我们改进寻径星野。",
        confirmText: "去登录",
        success: (result) => {
          if (result.confirm) {
            this.goLogin();
          }
        },
      });
      return;
    }
    const email = this.data.accountProfile.email;
    this.setData({
      feedbackModalVisible: true,
      feedbackCategoryIndex: 0,
      feedbackContent: "",
      feedbackContact: email,
      feedbackError: "",
    });
  },

  closeFeedbackModal() {
    if (this.data.feedbackLoading) {
      return;
    }
    this.setData({
      feedbackModalVisible: false,
      feedbackContent: "",
      feedbackContact: "",
      feedbackError: "",
    });
  },

  onFeedbackCategoryChange(event: any) {
    const index = Number(event.detail.value || 0);
    this.setData({ feedbackCategoryIndex: index, feedbackError: "" });
  },

  onFeedbackInput(event: WechatMiniprogram.BaseEvent) {
    const field = event.currentTarget.dataset.field as
      | "feedbackContent"
      | "feedbackContact"
      | undefined;
    if (!field) {
      return;
    }
    this.setData({ [field]: (event as any).detail.value, feedbackError: "" });
  },

  async submitFeedback() {
    if (this.data.feedbackLoading) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const content = this.data.feedbackContent.trim();
    if (!content) {
      this.setData({ feedbackError: "请填写反馈内容" });
      return;
    }
    const category =
      FEEDBACK_CATEGORY_OPTIONS[this.data.feedbackCategoryIndex]?.value ??
      "suggestion";
    this.setData({ feedbackLoading: true, feedbackError: "" });
    try {
      await createFeedback({
        category,
        content,
        contact: normalizeOptionalText(this.data.feedbackContact),
        page: currentPagePath(),
        client_platform: "wechat_miniprogram",
        client_version: miniProgramVersion(),
        device_model: deviceModel(),
        image_ids: [],
      });
      this.setData({
        feedbackModalVisible: false,
        feedbackContent: "",
        feedbackContact: "",
        feedbackError: "",
      });
      wx.showToast({ title: "反馈已提交", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.resetLoggedOutAccountState();
        return;
      }
      this.setData({ feedbackError: getErrorMessage(error) });
    } finally {
      this.setData({ feedbackLoading: false });
    }
  },

  openAboutModal() {
    this.setData({
      aboutModalVisible: true,
      aboutInfo: buildAboutInfo(),
    });
  },

  closeAboutModal() {
    this.setData({ aboutModalVisible: false });
  },

  resetLoggedOutAccountState() {
    this.setData({
      loggedIn: false,
      accountProfile: buildAccountProfile(false),
      cachedKnotsModalVisible: false,
      feedbackModalVisible: false,
      feedbackContent: "",
      feedbackContact: "",
      feedbackError: "",
      accountError: "",
    });
    wx.showToast({ title: "请重新登录", icon: "none" });
  },
});

function normalizeOptionalText(value: string): string | null {
  const normalized = value.trim();
  return normalized || null;
}

function buildCachedKnotsInfo(inventory: KnotOfflineCacheInventory): {
  cachedCount: number;
  totalCountText: string;
  uncachedCountText: string;
} {
  return {
    cachedCount: inventory.cachedCount,
    totalCountText: String(inventory.totalCount),
    uncachedCountText: String(inventory.uncachedCount),
  };
}

function currentPagePath(): string {
  const pages = getCurrentPages();
  const current = pages[pages.length - 1];
  return current?.route ? `/${current.route}` : "/pages/profile/index";
}

function miniProgramVersion(): string {
  const info = safeAccountInfo();
  return info?.miniProgram?.version || "dev";
}

function buildAboutInfo(): {
  envText: string;
  versionText: string;
} {
  const miniProgram = safeAccountInfo()?.miniProgram;
  return {
    envText: envVersionText(miniProgram?.envVersion),
    versionText: miniProgram?.version || "dev",
  };
}

function safeAccountInfo(): WechatMiniprogram.AccountInfo | undefined {
  try {
    return wx.getAccountInfoSync();
  } catch {
    return undefined;
  }
}

function envVersionText(value?: string): string {
  if (value === "release") {
    return "正式版";
  }
  if (value === "trial") {
    return "体验版";
  }
  return "开发版";
}

function deviceModel(): string | null {
  try {
    return wx.getSystemInfoSync().model || null;
  } catch {
    return null;
  }
}
