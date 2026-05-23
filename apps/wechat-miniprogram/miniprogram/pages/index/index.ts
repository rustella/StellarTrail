import { getGearOverview } from "../../utils/api-gears";
import {
  consumeOfflineCacheNotice,
  isOfflineCacheMissError,
  listKnots,
  resolveAssetUrl,
} from "../../utils/api-skills";
import {
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  loginWithWechat,
} from "../../utils/api-auth";
import { uploadWechatAvatar } from "../../utils/api-profile";
import {
  formatGearPrice,
  formatGearWeight,
  type GearStatsResponse,
  type GearSummary,
} from "../../utils/gear-display";
import {
  mapSkillCard,
  type KnotMediaAsset,
  type KnotSummary,
  type SkillCard,
} from "../../utils/skill-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
} from "../../utils/auth-prompt";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import { resolveCachedMediaUrl } from "../../utils/media-cache";

interface HomeStatCard {
  label: string;
  value: string;
  hint: string;
}

interface HomeGearCard {
  id: string;
  name: string;
  brandModelText: string;
  weightText: string;
}

interface ChecklistItem {
  icon: string;
  title: string;
  description: string;
}

interface HomeSkillCard extends SkillCard {
  thumbnailUrl: string;
  hasThumbnail: boolean;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  archived_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};

const LOCKED_GEAR_STATS: HomeStatCard[] = [
  {
    label: "可用装备",
    value: "—",
    hint: "登录后可见",
  },
  {
    label: "总重量",
    value: "—",
    hint: "登录后可见",
  },
  {
    label: "装备估值",
    value: "—",
    hint: "登录后可见",
  },
];

const INITIAL_LOGGED_IN = hasAccessToken();
const WECHAT_PROFILE_PROMPT_SEEN_KEY =
  "stellartrail_wechat_profile_prompt_seen";
const WECHAT_PROFILE_PROMPT_PENDING_KEY =
  "stellartrail_wechat_profile_prompt_pending";
const HOME_SOFT_REFRESH_MS = 30_000;
const GEARS_SHOULD_REFRESH_KEY = "stellartrail_gears_should_refresh";
let lastHomeDashboardLoadedAt = 0;
let lastHomeDashboardLoginState = INITIAL_LOGGED_IN;

const CHECKLIST_ITEMS: ChecklistItem[] = [
  {
    icon: "✓",
    title: "装备清点",
    description: "确认重量、状态和存放位置",
  },
  {
    icon: "☁",
    title: "天气与风险自查",
    description: "出发前关注降雨、温差和风力",
  },
  {
    icon: "✚",
    title: "急救与应急联系人",
    description: "复习急救知识并告知行程信息",
  },
  {
    icon: "保",
    title: "户外保险",
    description: "确认保障范围、保单和紧急救援电话",
  },
];

Page({
  data: {
    title: "寻径星野",
    checklistItems: CHECKLIST_ITEMS,
    isLoggedIn: INITIAL_LOGGED_IN,
    heroStatusText: INITIAL_LOGGED_IN ? "正在同步装备状态" : "未登录也可先浏览",
    gearLoading: false,
    gearError: "",
    skillLoading: false,
    skillError: "",
    offlineNotice: "",
    gearStatCards: INITIAL_LOGGED_IN
      ? buildGearStatCards(EMPTY_STATS)
      : LOCKED_GEAR_STATS,
    recentGears: [] as HomeGearCard[],
    featuredSkills: [] as HomeSkillCard[],
    loginPrompt: getDefaultLoginPrompt(),
    wechatProfilePromptVisible: false,
    wechatProfilePromptCanRetry: false,
    wechatProfileLoading: false,
    wechatProfileError: "",
    wechatNickname: "",
    wechatAvatarPath: "",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(GEARS_SHOULD_REFRESH_KEY) === true;
    if (shouldRefresh) {
      wx.removeStorageSync(GEARS_SHOULD_REFRESH_KEY);
    }
    void this.loadHomeDashboard({ force: shouldRefresh });
    this.showWechatProfilePromptIfNeeded();
  },

  onPullDownRefresh() {
    this.loadHomeDashboard({ force: true }).finally(() =>
      wx.stopPullDownRefresh(),
    );
  },

  async loadHomeDashboard(options: { force?: boolean } = {}) {
    const isLoggedIn = hasAccessToken();
    const now = Date.now();
    if (
      !options.force &&
      lastHomeDashboardLoginState === isLoggedIn &&
      now - lastHomeDashboardLoadedAt < HOME_SOFT_REFRESH_MS &&
      (this.data.featuredSkills.length || this.data.skillError)
    ) {
      return;
    }
    lastHomeDashboardLoadedAt = now;
    lastHomeDashboardLoginState = isLoggedIn;
    this.setData({
      isLoggedIn,
      heroStatusText: isLoggedIn ? "正在同步装备状态" : "未登录也可先浏览",
      offlineNotice: "",
    });
    const tasks = [this.loadFeaturedSkills()];
    if (isLoggedIn) {
      tasks.push(this.loadGearSummary());
    } else {
      this.setData({
        gearLoading: false,
        gearError: "",
        heroStatusText: "未登录也可先浏览",
        gearStatCards: LOCKED_GEAR_STATS,
        recentGears: [] as HomeGearCard[],
      });
    }
    await Promise.all(tasks);
  },

  async loadGearSummary() {
    this.setData({ gearLoading: true, gearError: "" });
    try {
      const overview = await getGearOverview({
        tab: "available",
        limit: 2,
        sort: "created_at_desc",
      });
      const stats = overview.stats;
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        isLoggedIn: true,
        heroStatusText: buildHeroStatusText(stats),
        gearStatCards: buildGearStatCards(stats),
        recentGears: overview.list.items.map(mapHomeGearCard),
        gearLoading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({
          isLoggedIn: false,
          gearError: "",
          gearLoading: false,
          heroStatusText: "未登录也可先浏览",
          gearStatCards: buildGearStatCards(EMPTY_STATS),
          recentGears: [] as HomeGearCard[],
        });
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.recentGears.length) {
        this.setData({ gearLoading: false });
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        gearError: getErrorMessage(error),
        gearLoading: false,
        heroStatusText: "装备状态暂时不可用",
        recentGears: [] as HomeGearCard[],
      });
    }
  },

  async loadFeaturedSkills() {
    this.setData({ skillLoading: true, skillError: "" });
    try {
      const response = await listKnots({ offset: 0, limit: 3 });
      const featuredSkills = await Promise.all(
        response.items.slice(0, 3).map(mapHomeSkillCard),
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        featuredSkills,
        skillLoading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isOfflineCacheMissError(error) && this.data.featuredSkills.length) {
        this.setData({ skillLoading: false });
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        skillError: getErrorMessage(error),
        skillLoading: false,
        featuredSkills: [] as HomeSkillCard[],
      });
    }
  },

  showLoginForGearSummary() {
    requireLoginForAction(this, {
      message: "登录后可以查看自己的装备、重量和估值。",
      redirectUrl: "/pages/index/index",
    });
  },

  goGears() {
    wx.switchTab({ url: "/pages/gears/index" });
  },

  goSkills() {
    wx.switchTab({ url: "/pages/skills/index" });
  },

  goSkillDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (id) {
      wx.navigateTo({
        url: `/pages/skills/detail/index?id=${encodeURIComponent(id)}`,
      });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },

  showWechatProfilePromptIfNeeded() {
    if (
      hasAccessToken() &&
      wx.getStorageSync(WECHAT_PROFILE_PROMPT_PENDING_KEY) === true &&
      wx.getStorageSync(WECHAT_PROFILE_PROMPT_SEEN_KEY) !== true
    ) {
      this.setData({
        wechatProfilePromptVisible: true,
        wechatProfilePromptCanRetry: false,
        wechatProfileError: "",
        wechatNickname: "",
        wechatAvatarPath: "",
        loginPrompt: {
          ...this.data.loginPrompt,
          visible: false,
        },
      });
    }
  },

  onWechatNicknameInput(event: WechatMiniprogram.Input) {
    this.setData({ wechatNickname: event.detail.value });
  },

  onChooseWechatAvatar(
    event: WechatMiniprogram.CustomEvent<{ avatarUrl?: string }>,
  ) {
    const avatarPath = event.detail.avatarUrl || "";
    if (avatarPath) {
      this.setData({ wechatAvatarPath: avatarPath, wechatProfileError: "" });
    }
  },

  async importWechatProfile() {
    if (this.data.wechatProfileLoading) {
      return;
    }
    const nickname = this.data.wechatNickname.trim();
    const avatarPath = this.data.wechatAvatarPath;
    if (!nickname && !avatarPath) {
      this.setData({ wechatProfileError: "请选择头像或填写昵称，也可以跳过" });
      return;
    }

    this.setData({
      wechatProfileLoading: true,
      wechatProfileError: "",
      wechatProfilePromptCanRetry: false,
    });
    try {
      if (nickname) {
        await loginWithWechat({ nickname });
      }
      if (avatarPath) {
        try {
          await uploadWechatAvatar(avatarPath);
        } catch (avatarError) {
          this.setData({
            wechatProfilePromptCanRetry: true,
            wechatProfileError: `头像保存失败：${getErrorMessage(avatarError)}`,
          });
          return;
        }
      }
      this.finishWechatProfilePrompt("资料已导入");
    } catch (error) {
      this.setData({ wechatProfileError: getErrorMessage(error) });
    } finally {
      this.setData({ wechatProfileLoading: false });
    }
  },

  async retryWechatAvatarUpload() {
    if (this.data.wechatProfileLoading) {
      return;
    }
    if (!this.data.wechatAvatarPath) {
      this.setData({ wechatProfileError: "请先选择头像" });
      return;
    }
    this.setData({ wechatProfileLoading: true, wechatProfileError: "" });
    try {
      await uploadWechatAvatar(this.data.wechatAvatarPath);
      this.finishWechatProfilePrompt("头像已保存");
    } catch (error) {
      this.setData({
        wechatProfileError: `头像保存失败：${getErrorMessage(error)}`,
      });
    } finally {
      this.setData({ wechatProfileLoading: false });
    }
  },

  skipWechatProfileImport() {
    if (this.data.wechatProfileLoading) {
      return;
    }
    this.finishWechatProfilePrompt("已跳过资料导入");
  },

  finishWechatProfilePrompt(title: string) {
    wx.setStorageSync(WECHAT_PROFILE_PROMPT_SEEN_KEY, true);
    wx.removeStorageSync(WECHAT_PROFILE_PROMPT_PENDING_KEY);
    this.setData({
      wechatProfilePromptVisible: false,
      wechatProfilePromptCanRetry: false,
      wechatProfileError: "",
    });
    wx.showToast({ title, icon: "none" });
  },
});

function buildGearStatCards(stats: GearStatsResponse): HomeStatCard[] {
  return [
    {
      label: "可用装备",
      value: String(stats.current_count),
      hint: "当前可直接使用",
    },
    {
      label: "总重量",
      value: formatGearWeight(stats.total_weight_g),
      hint: "已记录装备重量",
    },
    {
      label: "装备估值",
      value: formatGearPrice(stats.total_value_cents),
      hint: "按 CNY 购入价汇总",
    },
  ];
}

function buildHeroStatusText(stats: GearStatsResponse): string {
  const availableCount = stats.current_count;
  const archivedCount = stats.archived_count;
  if (availableCount > 0 && archivedCount > 0) {
    return `可用 ${availableCount} 件 · 已归档 ${archivedCount} 件`;
  }
  if (availableCount > 0) {
    return `可用装备 ${availableCount} 件`;
  }
  if (archivedCount > 0) {
    return `已归档装备 ${archivedCount} 件`;
  }
  return "还没有装备记录";
}

function mapHomeGearCard(item: GearSummary): HomeGearCard {
  const brandModelText = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    id: item.id,
    name: item.name,
    brandModelText: brandModelText || "未记录品牌型号",
    weightText: formatGearWeight(item.weight_g),
  };
}

async function mapHomeSkillCard(item: KnotSummary): Promise<HomeSkillCard> {
  const thumbnail = findThumbnail(item.media);
  const thumbnailUrl = thumbnail
    ? await resolveCachedMediaUrl(resolveAssetUrl(thumbnail.url))
    : "";
  return {
    ...mapSkillCard(item),
    thumbnailUrl,
    hasThumbnail: Boolean(thumbnailUrl),
  };
}

function findThumbnail(media: KnotMediaAsset[]): KnotMediaAsset | undefined {
  return (
    media.find((item) => item.media_type === "thumbnail") ??
    media.find((item) => item.mime_type.startsWith("image/"))
  );
}
