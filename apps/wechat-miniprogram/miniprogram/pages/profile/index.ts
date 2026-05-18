import {
  clearLoginState,
  getCurrentUser,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isLoginRequiredError,
  isNotFoundApiError,
  loginWithWechat,
  uploadWechatAvatar,
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
    accountProfile: buildAccountProfile(hasAccessToken()),
    accountError: "",
    avatarLoading: false,
    nicknameDraft: "",
    nicknameEditMode: "" as "" | "wechat" | "custom",
    nicknameModalVisible: false,
    nicknameLoading: false,
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

  openNicknameModal() {
    if (!this.data.loggedIn || this.data.nicknameLoading) {
      return;
    }
    this.setData({
      nicknameEditMode: "",
      nicknameModalVisible: true,
      nicknameDraft: "",
      accountError: "",
    });
  },

  async importWechatNickname() {
    if (!this.data.loggedIn || this.data.nicknameLoading) {
      return;
    }
    this.setData({ nicknameLoading: true, accountError: "" });
    try {
      const profile = await requestWechatUserProfile();
      const nickname = normalizeWechatNickname(profile.userInfo?.nickName);
      if (!nickname) {
        this.showWechatNicknameSelector("微信未返回可用名称，请选择微信名称");
        return;
      }
      await this.saveNicknameValue(nickname);
    } catch (_error) {
      this.showWechatNicknameSelector("未获取到微信名称，请选择微信名称");
    } finally {
      this.setData({ nicknameLoading: false });
    }
  },

  showWechatNicknameSelector(accountError: string) {
    this.setData({
      nicknameEditMode: "wechat",
      nicknameDraft: "",
      accountError,
    });
  },

  useCustomNickname() {
    if (this.data.nicknameLoading) {
      return;
    }
    this.setData({
      nicknameEditMode: "custom",
      nicknameDraft: this.data.accountProfile.hasNickname
        ? this.data.accountProfile.displayName
        : "",
      accountError: "",
    });
  },

  onNicknameInput(event: WechatMiniprogram.Input) {
    this.setData({ nicknameDraft: event.detail.value });
  },

  async saveWechatNickname(event: WechatMiniprogram.Input) {
    const nickname = event.detail.value.trim();
    if (!nickname || this.data.nicknameLoading) {
      return;
    }
    this.setData({ nicknameDraft: nickname, accountError: "" });
    await this.saveNicknameValue(nickname);
  },

  backToNicknameMethods() {
    if (this.data.nicknameLoading) {
      return;
    }
    this.setData({
      nicknameEditMode: "",
      nicknameDraft: "",
      accountError: "",
    });
  },

  closeNicknameModal() {
    if (this.data.nicknameLoading) {
      return;
    }
    this.setData({
      nicknameEditMode: "",
      nicknameModalVisible: false,
      nicknameDraft: "",
      accountError: "",
    });
  },

  async saveNickname() {
    if (!this.data.loggedIn || this.data.nicknameLoading) {
      return;
    }
    if (
      this.data.nicknameEditMode !== "custom" &&
      this.data.nicknameEditMode !== "wechat"
    ) {
      this.setData({ accountError: "请选择修改方式" });
      return;
    }
    const nickname = this.data.nicknameDraft.trim();
    if (!nickname) {
      this.setData({
        accountError:
          this.data.nicknameEditMode === "wechat"
            ? "请选择微信名称"
            : "请输入自定义名称",
      });
      return;
    }
    await this.saveNicknameValue(nickname);
  },

  async saveNicknameValue(nickname: string) {
    this.setData({ nicknameLoading: true, accountError: "" });
    try {
      await loginWithWechat({ nickname });
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        nicknameEditMode: "",
        nicknameModalVisible: false,
        nicknameDraft: "",
        accountError: "",
      });
      wx.showToast({ title: "名称已更新", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({
          loggedIn: false,
          accountProfile: buildAccountProfile(false),
          nicknameEditMode: "",
          nicknameModalVisible: false,
          nicknameDraft: "",
          accountError: "",
        });
        wx.showToast({ title: "请重新登录", icon: "none" });
        return;
      }
      this.setData({ accountError: getErrorMessage(error) });
    } finally {
      this.setData({ nicknameLoading: false });
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
        this.setData({
          loggedIn: false,
          accountProfile: buildAccountProfile(false),
          accountError: "",
        });
        wx.showToast({ title: "请重新登录", icon: "none" });
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
        void this.refreshAccountState();
        wx.showToast({ title: "已退出", icon: "success" });
      },
    });
  },

  toggleTheme() {
    togglePageTheme(this);
  },
});

interface WechatProfileResult {
  userInfo?: {
    nickName?: string;
  };
}

interface WechatProfileApi {
  getUserProfile?: (options: {
    desc: string;
    lang?: string;
    success: (result: WechatProfileResult) => void;
    fail: (error: unknown) => void;
  }) => void;
}

function requestWechatUserProfile(): Promise<WechatProfileResult> {
  return new Promise((resolve, reject) => {
    const getUserProfile = (wx as WechatProfileApi).getUserProfile;
    if (!getUserProfile) {
      reject(new Error("当前微信版本不支持直接获取名称"));
      return;
    }
    getUserProfile({
      desc: "用于导入你的微信名称",
      lang: "zh_CN",
      success: resolve,
      fail: reject,
    });
  });
}

function buildAccountProfile(loggedIn: boolean): {
  displayName: string;
  avatarUrl: string;
  avatarInitial: string;
  hasNickname: boolean;
} {
  const user = getStoredUser();
  if (!loggedIn) {
    return {
      displayName: "",
      avatarUrl: "",
      avatarInitial: "",
      hasNickname: false,
    };
  }
  if (!user) {
    return {
      displayName: "微信用户",
      avatarUrl: "",
      avatarInitial: "微",
      hasNickname: false,
    };
  }
  const nickname = normalizeProfileNickname(user.nickname);
  const displayName = nickname || user.username || user.email || "微信用户";
  return {
    displayName,
    avatarUrl: user.avatar_url || "",
    avatarInitial: displayName.trim().slice(0, 1) || "微",
    hasNickname: Boolean(nickname),
  };
}

function normalizeProfileNickname(value?: string | null): string {
  return normalizeWechatNickname(value);
}

function normalizeWechatNickname(value?: string | null): string {
  const nickname = value?.trim() ?? "";
  const defaultNames = ["寻径星野用户", "微信用户", "WeChat User"];
  return nickname && !defaultNames.includes(nickname) ? nickname : "";
}
