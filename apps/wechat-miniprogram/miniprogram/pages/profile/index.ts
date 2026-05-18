import {
  clearLoginState,
  getStoredUser,
  hasAccessToken,
} from "../../utils/api";
import { loginPageUrl } from "../../utils/auth-prompt";
import {
  getThemeViewData,
  syncPageTheme,
  togglePageTheme,
} from "../../utils/theme";

Page({
  data: {
    title: "我的寻径星野",
    loggedIn: hasAccessToken(),
    accountProfile: buildAccountProfile(),
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    this.refreshAccountState();
  },

  refreshAccountState() {
    this.setData({
      loggedIn: hasAccessToken(),
      accountProfile: buildAccountProfile(),
    });
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/profile/index") });
  },

  logout() {
    wx.showModal({
      title: "退出登录？",
      content: "退出后仍可浏览出行装备参考和绳结教学。",
      confirmText: "退出",
      confirmColor: "#dc2626",
      success: (result) => {
        if (!result.confirm) {
          return;
        }
        clearLoginState();
        this.refreshAccountState();
        wx.showToast({ title: "已退出", icon: "success" });
      },
    });
  },

  toggleTheme() {
    togglePageTheme(this);
  },
});

function buildAccountProfile(): {
  displayName: string;
  avatarUrl: string;
  avatarInitial: string;
} {
  const user = getStoredUser();
  if (!user) {
    return {
      displayName: "未登录",
      avatarUrl: "",
      avatarInitial: "未",
    };
  }
  const displayName = user.nickname || user.username || user.email || "微信用户";
  return {
    displayName,
    avatarUrl: user.avatar_url || "",
    avatarInitial: displayName.trim().slice(0, 1) || "微",
  };
}
