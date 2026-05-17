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
    userDisplay: buildUserDisplay(),
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    this.refreshAccountState();
  },

  refreshAccountState() {
    this.setData({
      loggedIn: hasAccessToken(),
      userDisplay: buildUserDisplay(),
    });
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/profile/index") });
  },

  logout() {
    wx.showModal({
      title: "退出登录？",
      content: "退出后仍可浏览装备模板、公告清单和绳结详情。",
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

function buildUserDisplay(): string {
  const user = getStoredUser();
  if (!user) {
    return "游客模式";
  }
  return user.nickname || user.username || user.email || "微信用户";
}
