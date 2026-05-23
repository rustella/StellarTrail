import {
  clearLoginState,
  createFeedback,
  type FeedbackCategory,
  getCurrentUser,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isLoginRequiredError,
  isNotFoundApiError,
  listClientVersions,
  type ClientVersion,
  uploadFeedbackImage,
  uploadWechatAvatar,
} from "../../utils/api-profile";
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
import { consumeProfileShouldRefresh } from "../../utils/profile-refresh";

const KNOT_CACHE_ENTRY_KEY = "stellartrail_open_knots_cache";
const PROFILE_SOFT_REFRESH_MS = 60_000;
let lastProfileRefreshAt = 0;
let lastProfileRefreshUserId = "";
const MAX_FEEDBACK_IMAGE_COUNT = 6;
const FEEDBACK_SUCCESS_MESSAGE =
  "感谢你的反馈。反馈内容会进入后台处理，我们会及时查看并持续改进。你的建议会让寻径星野变得更好。";
const FEEDBACK_SUCCESS_VISIBLE_MS = 10_000;
const FEEDBACK_SUCCESS_TICK_MS = 1_000;
const FEEDBACK_SUCCESS_VISIBLE_SECONDS =
  FEEDBACK_SUCCESS_VISIBLE_MS / FEEDBACK_SUCCESS_TICK_MS;
let feedbackSuccessTimer: ReturnType<typeof setTimeout> | null = null;

const FEEDBACK_CATEGORY_OPTIONS: Array<{
  value: FeedbackCategory;
  label: string;
}> = [
  { value: "suggestion", label: "功能建议" },
  { value: "bug", label: "问题反馈" },
  { value: "content_correction", label: "内容纠错" },
  { value: "other", label: "其他" },
];

interface FeedbackImageItem {
  id: string;
  path: string;
  sizeText: string;
}

interface ClientVersionView extends ClientVersion {
  publishedAtText: string;
}

Page({
  data: {
    title: "我的寻径星野",
    loggedIn: hasAccessToken(),
    accountProfile: buildAccountProfile(hasAccessToken()),
    feedbackCategoryLabels: FEEDBACK_CATEGORY_OPTIONS.map((item) => item.label),
    feedbackCategoryIndex: 0,
    feedbackContent: "",
    feedbackContact: "",
    feedbackImages: [] as FeedbackImageItem[],
    feedbackImageLimitText: `最多 ${MAX_FEEDBACK_IMAGE_COUNT} 张`,
    feedbackCanAddImage: true,
    feedbackModalVisible: false,
    feedbackLoading: false,
    feedbackError: "",
    feedbackSuccessVisible: false,
    feedbackSuccessMessage: FEEDBACK_SUCCESS_MESSAGE,
    feedbackSuccessSecondsRemaining: 0,
    aboutModalVisible: false,
    versionInfoDesc: "点击查看版本更新",
    versionModalVisible: false,
    versionLoading: false,
    versionError: "",
    clientVersions: [] as ClientVersionView[],
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
    void this.refreshAccountState({
      force: consumeProfileShouldRefresh(),
    });
    void this.refreshClientVersionSummary();
  },

  onHide() {
    this.hideFeedbackSuccess();
  },

  onUnload() {
    this.hideFeedbackSuccess();
  },

  async refreshAccountState(options: { force?: boolean } = {}) {
    const loggedIn = hasAccessToken();
    const userId = getStoredUser()?.id ?? "";
    this.setData({
      loggedIn,
      accountProfile: buildAccountProfile(loggedIn),
      accountError: "",
    });
    if (!loggedIn) {
      lastProfileRefreshAt = 0;
      lastProfileRefreshUserId = "";
      return;
    }
    if (
      !options.force &&
      lastProfileRefreshUserId === userId &&
      Date.now() - lastProfileRefreshAt < PROFILE_SOFT_REFRESH_MS
    ) {
      return;
    }
    try {
      await getCurrentUser();
      lastProfileRefreshAt = Date.now();
      lastProfileRefreshUserId = getStoredUser()?.id ?? userId;
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
      await this.refreshAccountState({ force: true });
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

  openRoadmap() {
    wx.navigateTo({ url: "/pages/profile/roadmap/index" });
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
    this.hideFeedbackSuccess();
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
      feedbackImages: [],
      feedbackCanAddImage: true,
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
      feedbackImages: [],
      feedbackCanAddImage: true,
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

  chooseFeedbackImages() {
    if (this.data.feedbackLoading) {
      return;
    }
    const remaining =
      MAX_FEEDBACK_IMAGE_COUNT - this.data.feedbackImages.length;
    if (remaining <= 0) {
      wx.showToast({
        title: `最多添加 ${MAX_FEEDBACK_IMAGE_COUNT} 张`,
        icon: "none",
      });
      return;
    }
    const chooseMedia = (wx as any).chooseMedia as
      | ((options: {
          count: number;
          mediaType: string[];
          sourceType: string[];
          success(result: {
            tempFiles?: Array<{ tempFilePath?: string; size?: number }>;
          }): void;
          fail(error: { errMsg?: string }): void;
        }) => void)
      | undefined;
    if (chooseMedia) {
      chooseMedia({
        count: remaining,
        mediaType: ["image"],
        sourceType: ["album", "camera"],
        success: (result) => {
          this.appendFeedbackImages(
            (result.tempFiles || [])
              .map((file) => ({
                path: file.tempFilePath || "",
                size: file.size,
              }))
              .filter((file) => Boolean(file.path)),
          );
        },
        fail: (error) => this.handleFeedbackImageChooseFail(error),
      });
      return;
    }

    wx.chooseImage({
      count: remaining,
      sizeType: ["compressed"],
      sourceType: ["album", "camera"],
      success: (result) => {
        this.appendFeedbackImages(
          result.tempFilePaths.map((path, index) => ({
            path,
            size: result.tempFiles?.[index]?.size,
          })),
        );
      },
      fail: (error) => this.handleFeedbackImageChooseFail(error),
    });
  },

  appendFeedbackImages(files: Array<{ path: string; size?: number }>) {
    if (!files.length) {
      return;
    }
    const remaining =
      MAX_FEEDBACK_IMAGE_COUNT - this.data.feedbackImages.length;
    const appended = files.slice(0, remaining).map((file) => ({
      id: `feedback-image-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      path: file.path,
      sizeText: formatFileSize(file.size),
    }));
    const nextImages = [...this.data.feedbackImages, ...appended];
    this.setData({
      feedbackImages: nextImages,
      feedbackCanAddImage: nextImages.length < MAX_FEEDBACK_IMAGE_COUNT,
      feedbackError: "",
    });
  },

  handleFeedbackImageChooseFail(error: { errMsg?: string }) {
    const message = error.errMsg || "";
    if (message.includes("cancel")) {
      return;
    }
    this.setData({ feedbackError: "图片选择失败，请稍后再试" });
  },

  previewFeedbackImage(event: WechatMiniprogram.BaseEvent) {
    const index = Number(event.currentTarget.dataset.index || 0);
    const urls = this.data.feedbackImages.map((image) => image.path);
    if (!urls.length) {
      return;
    }
    wx.previewImage({
      current: urls[index] || urls[0],
      urls,
    });
  },

  removeFeedbackImage(event: WechatMiniprogram.BaseEvent) {
    if (this.data.feedbackLoading) {
      return;
    }
    const index = Number(event.currentTarget.dataset.index);
    if (!Number.isFinite(index) || index < 0) {
      return;
    }
    const nextImages = this.data.feedbackImages.filter(
      (_image, currentIndex) => currentIndex !== index,
    );
    this.setData({
      feedbackImages: nextImages,
      feedbackCanAddImage: nextImages.length < MAX_FEEDBACK_IMAGE_COUNT,
      feedbackError: "",
    });
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
      const imageIds = await this.uploadFeedbackImages();
      await createFeedback({
        category,
        content,
        contact: normalizeOptionalText(this.data.feedbackContact),
        page: currentPagePath(),
        client_platform: "wechat_miniprogram",
        client_version: miniProgramVersion(),
        device_model: deviceModel(),
        image_ids: imageIds,
      });
      this.setData({
        feedbackModalVisible: false,
        feedbackContent: "",
        feedbackContact: "",
        feedbackImages: [],
        feedbackCanAddImage: true,
        feedbackError: "",
      });
      this.showFeedbackSuccess();
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

  async uploadFeedbackImages(): Promise<string[]> {
    const images = this.data.feedbackImages;
    if (!images.length) {
      return [];
    }
    this.setData({ feedbackError: "正在上传图片..." });
    const imageIds: string[] = [];
    for (const image of images) {
      const uploaded = await uploadFeedbackImage(image.path);
      imageIds.push(uploaded.id);
    }
    this.setData({ feedbackError: "" });
    return imageIds;
  },

  showFeedbackSuccess() {
    this.clearFeedbackSuccessTimer();
    this.setData({
      feedbackSuccessVisible: true,
      feedbackSuccessMessage: FEEDBACK_SUCCESS_MESSAGE,
      feedbackSuccessSecondsRemaining: FEEDBACK_SUCCESS_VISIBLE_SECONDS,
    });
    this.scheduleFeedbackSuccessCountdown();
  },

  scheduleFeedbackSuccessCountdown() {
    this.clearFeedbackSuccessTimer();
    feedbackSuccessTimer = setTimeout(() => {
      feedbackSuccessTimer = null;
      const nextSeconds = this.data.feedbackSuccessSecondsRemaining - 1;
      if (nextSeconds <= 0) {
        this.hideFeedbackSuccess();
        return;
      }
      this.setData({ feedbackSuccessSecondsRemaining: nextSeconds });
      this.scheduleFeedbackSuccessCountdown();
    }, FEEDBACK_SUCCESS_TICK_MS);
  },

  closeFeedbackSuccess() {
    this.hideFeedbackSuccess();
  },

  hideFeedbackSuccess() {
    this.clearFeedbackSuccessTimer();
    if (
      this.data.feedbackSuccessVisible ||
      this.data.feedbackSuccessSecondsRemaining
    ) {
      this.setData({
        feedbackSuccessVisible: false,
        feedbackSuccessSecondsRemaining: 0,
      });
    }
  },

  clearFeedbackSuccessTimer() {
    if (feedbackSuccessTimer) {
      clearTimeout(feedbackSuccessTimer);
      feedbackSuccessTimer = null;
    }
  },

  openAboutModal() {
    this.setData({ aboutModalVisible: true });
  },

  closeAboutModal() {
    this.setData({ aboutModalVisible: false });
  },

  openVersionInfoModal() {
    this.setData({ versionModalVisible: true });
    if (!this.data.clientVersions.length && !this.data.versionLoading) {
      void this.loadClientVersions();
    }
  },

  closeVersionInfoModal() {
    this.setData({ versionModalVisible: false });
  },

  retryClientVersions() {
    void this.loadClientVersions();
  },

  async refreshClientVersionSummary() {
    try {
      const response = await listClientVersions("wechat_miniprogram", {
        limit: 1,
      });
      const latest = response.items[0];
      if (latest) {
        this.setData({
          versionInfoDesc: `客户端版本 ${latest.version}`,
        });
      }
    } catch {
      this.setData({ versionInfoDesc: "点击查看版本更新" });
    }
  },

  async loadClientVersions() {
    this.setData({ versionLoading: true, versionError: "" });
    try {
      const response = await listClientVersions("wechat_miniprogram", {
        limit: 20,
      });
      const versions = response.items.map(versionToView);
      this.setData({
        clientVersions: versions,
        versionInfoDesc: versions[0]
          ? `客户端版本 ${versions[0].version}`
          : "暂无版本信息",
        versionError: "",
      });
    } catch (error) {
      this.setData({
        versionError: getErrorMessage(error),
        versionInfoDesc: "点击查看版本更新",
      });
    } finally {
      this.setData({ versionLoading: false });
    }
  },

  resetLoggedOutAccountState() {
    this.clearFeedbackSuccessTimer();
    this.setData({
      loggedIn: false,
      accountProfile: buildAccountProfile(false),
      cachedKnotsModalVisible: false,
      feedbackModalVisible: false,
      feedbackContent: "",
      feedbackContact: "",
      feedbackImages: [],
      feedbackCanAddImage: true,
      feedbackError: "",
      feedbackSuccessVisible: false,
      feedbackSuccessSecondsRemaining: 0,
      accountError: "",
    });
    lastProfileRefreshAt = 0;
    lastProfileRefreshUserId = "";
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

function versionToView(version: ClientVersion): ClientVersionView {
  return {
    ...version,
    publishedAtText: formatVersionDate(version.published_at),
  };
}

function formatVersionDate(value?: string | null): string {
  if (!value) {
    return "发布时间待定";
  }
  return value.slice(0, 10);
}

function currentPagePath(): string {
  const pages = getCurrentPages();
  const current = pages[pages.length - 1];
  return current?.route ? `/${current.route}` : "/pages/profile/index";
}

function formatFileSize(size?: number): string {
  if (!Number.isFinite(size) || !size || size <= 0) {
    return "";
  }
  if (size >= 1024 * 1024) {
    return `${(size / 1024 / 1024).toFixed(1)} MB`;
  }
  return `${Math.max(1, Math.round(size / 1024))} KB`;
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
