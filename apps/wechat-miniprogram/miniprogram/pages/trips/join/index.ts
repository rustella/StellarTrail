import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  acceptTripInvitation,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
} from "../../../utils/api-trips";
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
import {
  buildTripJoinPath,
  extractTripInvitationToken,
} from "../../../utils/trip-utils";

const TRIP_HOME_SHOULD_REFRESH_KEY = "stellartrail_home_trip_refresh";

Page({
  data: {
    token: "",
    loading: false,
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const token = extractTripInvitationToken(options.token || "");
    this.setData({ token });
    if (token) {
      this.accept();
    }
  },

  onShow() {
    syncPageTheme(this);
  },

  onInput(event: WechatMiniprogram.Input) {
    this.setData({ token: event.detail.value });
  },

  pasteToken() {
    wx.getClipboardData({
      success: (result) => {
        const token = extractTripInvitationToken(result.data);
        if (!token) {
          wx.showToast({ title: "剪贴板没有可用口令", icon: "none" });
          return;
        }
        this.setData({ token, error: "" });
      },
    });
  },

  async accept() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const token = extractTripInvitationToken(this.data.token);
    if (!token) {
      wx.showToast({ title: "请填写有效邀请口令", icon: "none" });
      return;
    }
    if (this.data.token !== token) {
      this.setData({ token });
    }
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后才能加入组队计划。",
        redirectUrl: buildTripJoinPath(token),
      });
      return;
    }
    this.setData({ loading: true, error: "" });
    try {
      const detail = await acceptTripInvitation(token);
      wx.setStorageSync("stellartrail_trips_refresh", true);
      wx.setStorageSync(TRIP_HOME_SHOULD_REFRESH_KEY, true);
      wx.redirectTo({
        url: `/pages/trips/detail/index?id=${detail.plan.id}`,
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后加入计划。",
          redirectUrl: buildTripJoinPath(token),
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});
