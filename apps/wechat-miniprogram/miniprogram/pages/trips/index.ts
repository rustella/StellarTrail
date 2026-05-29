import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  consumeOfflineCacheNotice,
  convertTripToOutdoorExperience,
  deleteTrip,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  listTrips,
} from "../../utils/api-trips";
import { getStoredUser } from "../../utils/api-auth";
import {
  formatTripDurationText,
  type TripSummary,
} from "../../utils/trip-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  loginPageUrl,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../utils/auth-prompt";
import { navigateToGuestFallback } from "../../utils/navigation";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../utils/network-state";
import { indexedAppendData } from "../../utils/page-data";

const TRIP_HOME_SHOULD_REFRESH_KEY = "stellartrail_home_trip_refresh";
const OUTDOOR_EXPERIENCES_SHOULD_REFRESH_KEY =
  "stellartrail_outdoor_experiences_refresh";

interface TripCard extends TripSummary {
  canDelete: boolean;
  canConvertToExperience: boolean;
  convertedToExperience: boolean;
  dateText: string;
  durationText: string;
  typeText: string;
  readinessText: string;
}

interface TripGroupView {
  key: string;
  title: string;
  subtitle: string;
  trips: TripCard[];
}

let tripRequestSeq = 0;

Page({
  data: {
    isLoggedIn: hasAccessToken(),
    plans: [] as TripCard[],
    groups: [] as TripGroupView[],
    showCreateSheet: false,
    nextCursor: null as string | null,
    loading: false,
    loadingMore: false,
    convertingTripId: "",
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
    const shouldRefresh = wx.getStorageSync("stellartrail_trips_refresh");
    if (shouldRefresh) {
      wx.removeStorageSync("stellartrail_trips_refresh");
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
      this.setData({ plans: [], groups: [], nextCursor: null, loading: false });
      return;
    }
    const requestSeq = ++tripRequestSeq;
    this.setData({ loading: true, loadingMore: false });
    try {
      const response = await listTrips({ limit: 20 });
      if (requestSeq !== tripRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        plans: response.items.map(mapPlanCard),
        groups: groupTripCards(response.items.map(mapPlanCard)),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== tripRequestSeq) {
        return;
      }
      if (isLoginRequiredError(error)) {
        this.setData({ isLoggedIn: false, loading: false, error: "" });
        showLoginPrompt(this, {
          message: "登录后可以查看自己的行程。",
          redirectUrl: "/pages/trips/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error), plans: [] });
    } finally {
      if (requestSeq === tripRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  async loadMore() {
    if (!this.data.nextCursor || this.data.loading || this.data.loadingMore) {
      return;
    }
    this.setData({ loadingMore: true, error: "" });
    const requestSeq = tripRequestSeq;
    try {
      const response = await listTrips({
        limit: 20,
        cursor: this.data.nextCursor,
      });
      if (requestSeq !== tripRequestSeq) {
        return;
      }
      const cards = response.items.map(mapPlanCard);
      const plans = this.data.plans.concat(cards);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...indexedAppendData("plans", this.data.plans.length, cards),
        groups: groupTripCards(plans),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看行程。",
          redirectUrl: "/pages/trips/index",
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
        message: "登录后可以制作行程计划。",
        redirectUrl: "/pages/trips/form/index",
      })
    ) {
      return;
    }
    this.setData({ showCreateSheet: true });
  },

  closeCreateSheet() {
    this.setData({ showCreateSheet: false });
  },

  createSoloTrip() {
    this.setData({ showCreateSheet: false });
    wx.navigateTo({ url: "/pages/trips/form/index?tripType=solo" });
  },

  createGroupTrip() {
    this.setData({ showCreateSheet: false });
    wx.navigateTo({ url: "/pages/trips/form/index?tripType=team" });
  },

  goJoin() {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以加入多人行程。",
        redirectUrl: "/pages/trips/join/index",
      })
    ) {
      return;
    }
    wx.navigateTo({ url: "/pages/trips/join/index" });
  },

  goGearAtlas() {
    navigateToGuestFallback();
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/trips/index") });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id;
    if (id) {
      wx.navigateTo({ url: `/pages/trips/detail/index?id=${id}` });
    }
  },

  deletePlan(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const id = event.currentTarget.dataset.id as string;
    const plan = this.data.plans.find((item) => item.id === id);
    if (!plan?.canDelete) {
      return;
    }
    wx.showModal({
      title: "删除行程？",
      content: "删除后这份行程不会继续展示。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await deleteTrip(id);
          wx.setStorageSync(TRIP_HOME_SHOULD_REFRESH_KEY, true);
          wx.showToast({ title: "已删除", icon: "success" });
          this.refreshPage();
        } catch (error) {
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },

  convertPlanToExperience(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const id = event.currentTarget.dataset.id as string;
    const plan = this.data.plans.find((item) => item.id === id);
    if (!plan?.canConvertToExperience || this.data.convertingTripId) {
      return;
    }
    this.setData({ convertingTripId: id, error: "" });
    convertTripToOutdoorExperience(id)
      .then(() => {
        wx.setStorageSync(TRIP_HOME_SHOULD_REFRESH_KEY, true);
        wx.setStorageSync(OUTDOOR_EXPERIENCES_SHOULD_REFRESH_KEY, true);
        wx.showToast({ title: "已转为户外经历", icon: "success" });
        this.refreshPage();
      })
      .catch((error) => {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
      })
      .finally(() => {
        this.setData({ convertingTripId: "" });
      });
  },

  noop() {},

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function mapPlanCard(plan: TripSummary): TripCard {
  const dateText =
    [plan.start_date, plan.end_date].filter(Boolean).join(" 至 ") ||
    "未设置行程时间";
  const currentUserId = getStoredUser()?.id ?? "";
  return {
    ...plan,
    canDelete: currentUserId ? plan.owner_user_id === currentUserId : false,
    canConvertToExperience:
      plan.time_bucket === "past" && !plan.outdoor_experience_id,
    convertedToExperience: Boolean(plan.outdoor_experience_id),
    dateText,
    durationText: formatTripDurationText(plan),
    typeText: plan.trip_type === "solo" ? "单人" : "多人",
    readinessText: formatReadinessText(plan),
  };
}

function groupTripCards(plans: TripCard[]): TripGroupView[] {
  const groups: Array<{
    key: TripSummary["time_bucket"];
    title: string;
    subtitle: string;
  }> = [
    { key: "ongoing", title: "进行中", subtitle: "正在路上的行程" },
    { key: "upcoming", title: "未来行程", subtitle: "准备中的出发计划" },
    { key: "undated", title: "待定", subtitle: "还没确定日期的想法" },
    { key: "past", title: "历史行程", subtitle: "已结束，可转为户外经历" },
  ];
  return groups
    .map((group) => ({
      ...group,
      trips: plans.filter((plan) => plan.time_bucket === group.key),
    }))
    .filter((group) => group.trips.length > 0);
}

function formatReadinessText(plan: TripSummary): string {
  if (
    plan.time_bucket === "upcoming" &&
    typeof plan.days_until_start === "number"
  ) {
    return `${Math.max(0, plan.days_until_start)}天后出发`;
  }
  if (
    plan.time_bucket === "ongoing" &&
    typeof plan.days_until_end === "number"
  ) {
    return `还剩${Math.max(0, plan.days_until_end)}天`;
  }
  const missing = plan.readiness?.missing_count || 0;
  if (missing > 0) {
    return `待完善 ${missing} 项`;
  }
  return `${plan.readiness?.completion_percent ?? 0}% 已完善`;
}
