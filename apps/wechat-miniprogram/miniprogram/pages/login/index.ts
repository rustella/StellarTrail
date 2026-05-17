import {
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  loginWithWechat,
} from "../../utils/api";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";

const TAB_PAGES = new Set([
  "/pages/index/index",
  "/pages/gears/index",
  "/pages/skills/index",
  "/pages/profile/index",
]);

Page({
  data: {
    redirect: "/pages/profile/index",
    loggedIn: hasAccessToken(),
    userDisplay: buildUserDisplay(),
    loading: false,
    error: "",
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    this.setData({
      redirect: decodeRedirect(options.redirect),
      loggedIn: hasAccessToken(),
      userDisplay: buildUserDisplay(),
    });
  },

  onShow() {
    syncPageTheme(this);
    this.setData({
      loggedIn: hasAccessToken(),
      userDisplay: buildUserDisplay(),
    });
  },

  async login() {
    if (this.data.loading) {
      return;
    }
    this.setData({ loading: true, error: "" });
    try {
      await loginWithWechat();
      this.setData({ loggedIn: true, userDisplay: buildUserDisplay() });
      wx.showToast({ title: "登录成功", icon: "success" });
      navigateToRedirect(this.data.redirect);
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  skip() {
    navigateToRedirect(this.data.redirect || "/pages/index/index");
  },
});

function decodeRedirect(value?: string): string {
  if (!value) {
    return "/pages/profile/index";
  }
  try {
    const decoded = decodeURIComponent(value);
    return decoded.startsWith("/pages/") ? decoded : "/pages/profile/index";
  } catch {
    return "/pages/profile/index";
  }
}

function buildUserDisplay(): string {
  const user = getStoredUser();
  if (!user) {
    return "未登录";
  }
  return user.nickname || user.username || user.email || "微信用户";
}

function navigateToRedirect(redirect: string): void {
  const [path] = redirect.split("?");
  if (TAB_PAGES.has(path)) {
    wx.switchTab({ url: path });
    return;
  }
  wx.redirectTo({
    url: redirect,
    fail: () => wx.switchTab({ url: "/pages/index/index" }),
  });
}
