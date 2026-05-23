import { getGearStats } from "../../utils/api-gears";
import {
  consumeOfflineCacheNotice,
  getKnotDisclaimer,
  isOfflineCacheMissError,
  listKnots,
  resolveAssetUrl,
} from "../../utils/api-skills";
import {
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
} from "../../utils/api-auth";
import {
  formatGearPrice,
  formatGearWeight,
  type GearStatsResponse,
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
    featuredSkillsGateChecked: false,
    offlineNotice: "",
    gearStatCards: INITIAL_LOGGED_IN
      ? buildGearStatCards(EMPTY_STATS)
      : LOCKED_GEAR_STATS,
    featuredSkills: [] as HomeSkillCard[],
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(GEARS_SHOULD_REFRESH_KEY) === true;
    if (shouldRefresh) {
      wx.removeStorageSync(GEARS_SHOULD_REFRESH_KEY);
    }
    void this.loadHomeDashboard({ force: shouldRefresh });
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
      (this.data.featuredSkillsGateChecked ||
        this.data.featuredSkills.length ||
        this.data.skillError)
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
      });
    }
    await Promise.all(tasks);
  },

  async loadGearSummary() {
    this.setData({ gearLoading: true, gearError: "" });
    try {
      const stats = await getGearStats("available");
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        isLoggedIn: true,
        heroStatusText: buildHeroStatusText(stats),
        gearStatCards: buildGearStatCards(stats),
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
        });
        return;
      }
      this.setData({
        gearError: getErrorMessage(error),
        gearLoading: false,
        heroStatusText: "装备状态暂时不可用",
      });
    }
  },

  async loadFeaturedSkills() {
    this.setData({ skillLoading: true, skillError: "" });
    if (!hasAccessToken()) {
      this.hideFeaturedSkills();
      return;
    }
    try {
      const disclaimer = await getKnotDisclaimer();
      if (!disclaimer.accepted) {
        this.hideFeaturedSkills();
        return;
      }
    } catch {
      this.hideFeaturedSkills();
      return;
    }
    try {
      const response = await listKnots({ offset: 0, limit: 3 });
      const featuredSkills = await Promise.all(
        response.items.slice(0, 3).map(mapHomeSkillCard),
      );
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        featuredSkills,
        featuredSkillsGateChecked: true,
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
        featuredSkillsGateChecked: true,
        featuredSkills: [] as HomeSkillCard[],
      });
    }
  },

  hideFeaturedSkills() {
    this.setData({
      featuredSkills: [] as HomeSkillCard[],
      featuredSkillsGateChecked: true,
      skillLoading: false,
      skillError: "",
    });
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
