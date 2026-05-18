import {
  bindEmailToCurrentAccount,
  clearLoginState,
  getCurrentUser,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isLoginRequiredError,
  isNotFoundApiError,
  loginWithWechat,
  sendBindEmailCode,
  uploadWechatAvatar,
} from "../../utils/api";
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

Page({
  data: {
    title: "我的寻径星野",
    loggedIn: hasAccessToken(),
    accountProfile: buildAccountProfile(hasAccessToken()),
    accountError: "",
    avatarLoading: false,
    nicknameDraft: "",
    nicknameModalVisible: false,
    nicknameLoading: false,
    nicknameReviewBlocked: false,
    emailBindingModalVisible: false,
    emailBindingEmail: "",
    emailBindingCode: "",
    emailBindingNotice: "",
    emailCodeLoading: false,
    emailBindingLoading: false,
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
      nicknameModalVisible: true,
      nicknameDraft: this.data.accountProfile.hasNickname
        ? this.data.accountProfile.displayName
        : "",
      nicknameReviewBlocked: false,
      accountError: "",
    });
  },

  onNicknameInput(event: WechatMiniprogram.Input) {
    this.setData({ nicknameDraft: event.detail.value });
  },

  onWechatNicknameReview(
    event: WechatMiniprogram.CustomEvent<{ pass?: boolean; timeout?: boolean }>,
  ) {
    const reviewBlocked = event.detail.pass === false && !event.detail.timeout;
    this.setData({
      nicknameReviewBlocked: reviewBlocked,
      accountError: reviewBlocked
        ? "微信名称未通过安全检测，请重新选择或手动输入名称"
        : "",
    });
  },

  async saveWechatNickname(event: WechatNicknameSubmitEvent) {
    if (this.data.nicknameLoading) {
      return;
    }
    const nickname = getSubmittedWechatNickname(event, this.data.nicknameDraft);
    if (!nickname) {
      this.setData({ accountError: "请选择或输入名称" });
      return;
    }
    if (this.data.nicknameReviewBlocked) {
      this.setData({
        accountError: "微信名称未通过安全检测，请重新选择或手动输入名称",
      });
      return;
    }
    this.setData({ nicknameDraft: nickname, accountError: "" });
    await this.saveNicknameValue(nickname);
  },

  closeNicknameModal() {
    if (this.data.nicknameLoading) {
      return;
    }
    this.setData({
      nicknameModalVisible: false,
      nicknameDraft: "",
      nicknameReviewBlocked: false,
      accountError: "",
    });
  },

  openEmailBindingModal() {
    if (
      !this.data.loggedIn ||
      this.data.emailBindingLoading ||
      this.data.accountProfile.hasEmail
    ) {
      return;
    }
    this.setData({
      emailBindingModalVisible: true,
      emailBindingEmail: "",
      emailBindingCode: "",
      emailBindingNotice: "",
      accountError: "",
    });
  },

  closeEmailBindingModal() {
    if (this.data.emailCodeLoading || this.data.emailBindingLoading) {
      return;
    }
    this.setData({
      emailBindingModalVisible: false,
      emailBindingEmail: "",
      emailBindingCode: "",
      emailBindingNotice: "",
      accountError: "",
    });
  },

  onEmailBindingInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "emailBindingEmail"
      | "emailBindingCode"
      | undefined;
    if (!field) {
      return;
    }
    this.setData({ [field]: event.detail.value, accountError: "" });
  },

  async sendEmailBindingCode() {
    if (this.data.emailCodeLoading) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const email = this.data.emailBindingEmail.trim();
    if (!isEmailLike(email)) {
      this.setData({ accountError: "请填写可用邮箱", emailBindingNotice: "" });
      return;
    }
    this.setData({
      emailCodeLoading: true,
      accountError: "",
      emailBindingNotice: "",
    });
    try {
      await sendBindEmailCode(email);
      this.setData({ emailBindingNotice: "验证码已发送，请查看邮箱" });
    } catch (error) {
      this.handleAccountAuthError(error);
    } finally {
      this.setData({ emailCodeLoading: false });
    }
  },

  async submitEmailBinding() {
    if (this.data.emailBindingLoading) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const email = this.data.emailBindingEmail.trim();
    const emailCode = this.data.emailBindingCode.trim();
    if (!isEmailLike(email)) {
      this.setData({ accountError: "请填写可用邮箱", emailBindingNotice: "" });
      return;
    }
    if (!emailCode) {
      this.setData({
        accountError: "请填写邮箱验证码",
        emailBindingNotice: "",
      });
      return;
    }
    this.setData({
      emailBindingLoading: true,
      accountError: "",
      emailBindingNotice: "",
    });
    try {
      await bindEmailToCurrentAccount({
        email,
        email_verification_code: emailCode,
      });
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        emailBindingModalVisible: false,
        emailBindingEmail: "",
        emailBindingCode: "",
        emailBindingNotice: "",
        accountError: "",
      });
      wx.showToast({ title: "邮箱已绑定", icon: "success" });
    } catch (error) {
      this.handleAccountAuthError(error);
    } finally {
      this.setData({ emailBindingLoading: false });
    }
  },

  async saveNicknameValue(nickname: string) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({ nicknameLoading: true, accountError: "" });
    try {
      await loginWithWechat({ nickname });
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        nicknameModalVisible: false,
        nicknameDraft: "",
        nicknameReviewBlocked: false,
        accountError: "",
      });
      wx.showToast({ title: "名称已更新", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.resetLoggedOutAccountState();
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

  resetLoggedOutAccountState() {
    this.setData({
      loggedIn: false,
      accountProfile: buildAccountProfile(false),
      nicknameModalVisible: false,
      nicknameDraft: "",
      nicknameReviewBlocked: false,
      emailBindingModalVisible: false,
      emailBindingEmail: "",
      emailBindingCode: "",
      emailBindingNotice: "",
      accountError: "",
    });
    wx.showToast({ title: "请重新登录", icon: "none" });
  },

  handleAccountAuthError(error: unknown) {
    if (isLoginRequiredError(error)) {
      this.resetLoggedOutAccountState();
      return;
    }
    this.setData({ accountError: getErrorMessage(error) });
  },
});

interface WechatNicknameSubmitEvent {
  detail: {
    value?: {
      nickname?: string;
    };
  };
}

function getSubmittedWechatNickname(
  event: WechatNicknameSubmitEvent,
  fallback: string,
): string {
  return (event.detail.value?.nickname ?? fallback).trim();
}

function buildAccountProfile(loggedIn: boolean): {
  displayName: string;
  avatarUrl: string;
  avatarInitial: string;
  hasNickname: boolean;
  email: string;
  emailText: string;
  hasEmail: boolean;
} {
  const user = getStoredUser();
  if (!loggedIn) {
    return {
      displayName: "",
      avatarUrl: "",
      avatarInitial: "",
      hasNickname: false,
      email: "",
      emailText: "",
      hasEmail: false,
    };
  }
  if (!user) {
    return {
      displayName: "微信用户",
      avatarUrl: "",
      avatarInitial: "微",
      hasNickname: false,
      email: "",
      emailText: "还没有绑定邮箱",
      hasEmail: false,
    };
  }
  const nickname = normalizeProfileNickname(user.nickname);
  const displayName = nickname || user.username || user.email || "微信用户";
  const email = user.email?.trim() ?? "";
  return {
    displayName,
    avatarUrl: user.avatar_url || "",
    avatarInitial: displayName.trim().slice(0, 1) || "微",
    hasNickname: Boolean(nickname),
    email,
    emailText: email ? `已绑定 ${email}` : "还没有绑定邮箱",
    hasEmail: Boolean(email),
  };
}

function normalizeProfileNickname(value?: string | null): string {
  const nickname = value?.trim() ?? "";
  const defaultNames = ["寻径星野用户", "微信用户", "WeChat User"];
  return nickname && !defaultNames.includes(nickname) ? nickname : "";
}

function isEmailLike(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}
