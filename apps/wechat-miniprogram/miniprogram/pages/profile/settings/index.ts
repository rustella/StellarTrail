import {
  bindEmailToCurrentAccount,
  getCurrentUser,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  sendBindEmailCode,
} from "../../../utils/api-profile";
import {
  resetPassword,
  sendPasswordResetCode,
  updateWechatNickname,
} from "../../../utils/api-auth";
import { buildAccountProfile } from "../../../utils/account-profile";
import { loginPageUrl } from "../../../utils/auth-prompt";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { markProfileShouldRefresh } from "../../../utils/profile-refresh";

Page({
  data: {
    loggedIn: hasAccessToken(),
    accountProfile: buildAccountProfile(hasAccessToken()),
    accountError: "",
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
    passwordModalVisible: false,
    passwordCode: "",
    passwordValue: "",
    passwordConfirm: "",
    passwordNotice: "",
    passwordError: "",
    passwordCodeLoading: false,
    passwordLoading: false,
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
        this.resetLoggedOutState();
        return;
      }
      this.setData({ accountError: getErrorMessage(error) });
    }
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/profile/settings/index") });
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

  onNicknameInput(event: WechatMiniprogram.Input) {
    this.setData({ nicknameDraft: event.detail.value, accountError: "" });
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
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({ nicknameLoading: true, accountError: "" });
    try {
      await updateWechatNickname(nickname);
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        nicknameModalVisible: false,
        nicknameDraft: "",
        nicknameReviewBlocked: false,
        accountError: "",
      });
      markProfileShouldRefresh();
      wx.showToast({ title: "名称已更新", icon: "success" });
    } catch (error) {
      this.handleAuthError(error);
    } finally {
      this.setData({ nicknameLoading: false });
    }
  },

  openEmailBindingModal() {
    if (!this.data.loggedIn || this.data.emailBindingLoading) {
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
      this.handleAuthError(error);
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
      const hadEmail = this.data.accountProfile.hasEmail;
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
      markProfileShouldRefresh();
      wx.showToast({
        title: hadEmail ? "邮箱已更新" : "邮箱已绑定",
        icon: "success",
      });
    } catch (error) {
      this.handleAuthError(error);
    } finally {
      this.setData({ emailBindingLoading: false });
    }
  },

  openPasswordModal() {
    if (!this.data.loggedIn) {
      return;
    }
    if (!this.data.accountProfile.hasEmail) {
      wx.showModal({
        title: "先绑定邮箱",
        content: "修改密码需要通过邮箱验证码确认身份。",
        confirmText: "绑定邮箱",
        success: (result) => {
          if (result.confirm) {
            this.openEmailBindingModal();
          }
        },
      });
      return;
    }
    this.setData({
      passwordModalVisible: true,
      passwordCode: "",
      passwordValue: "",
      passwordConfirm: "",
      passwordNotice: "",
      passwordError: "",
    });
  },

  openOutdoorProfile() {
    if (!this.data.loggedIn) {
      this.goLogin();
      return;
    }
    wx.navigateTo({ url: "/pages/profile/outdoor/index" });
  },

  openOutdoorExperiences() {
    if (!this.data.loggedIn) {
      this.goLogin();
      return;
    }
    wx.navigateTo({ url: "/pages/profile/outdoor-experiences/index" });
  },

  closePasswordModal() {
    if (this.data.passwordCodeLoading || this.data.passwordLoading) {
      return;
    }
    this.setData({
      passwordModalVisible: false,
      passwordCode: "",
      passwordValue: "",
      passwordConfirm: "",
      passwordNotice: "",
      passwordError: "",
    });
  },

  onPasswordInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "passwordCode"
      | "passwordValue"
      | "passwordConfirm"
      | undefined;
    if (!field) {
      return;
    }
    this.setData({ [field]: event.detail.value, passwordError: "" });
  },

  async sendPasswordCode() {
    if (this.data.passwordCodeLoading || !this.data.accountProfile.email) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      passwordCodeLoading: true,
      passwordNotice: "",
      passwordError: "",
    });
    try {
      await sendPasswordResetCode(this.data.accountProfile.email);
      this.setData({ passwordNotice: "验证码已发送，请查看邮箱" });
    } catch (error) {
      this.setData({ passwordError: getErrorMessage(error) });
    } finally {
      this.setData({ passwordCodeLoading: false });
    }
  },

  async submitPasswordChange() {
    if (this.data.passwordLoading || !this.data.accountProfile.email) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const emailCode = this.data.passwordCode.trim();
    const password = this.data.passwordValue;
    const confirmPassword = this.data.passwordConfirm;
    if (!emailCode) {
      this.setData({ passwordError: "请填写邮箱验证码" });
      return;
    }
    if (password.length < 8) {
      this.setData({ passwordError: "密码至少 8 位" });
      return;
    }
    if (password !== confirmPassword) {
      this.setData({ passwordError: "两次输入的密码不一致" });
      return;
    }
    this.setData({ passwordLoading: true, passwordError: "" });
    try {
      await resetPassword({
        email: this.data.accountProfile.email,
        email_verification_code: emailCode,
        password,
        confirm_password: confirmPassword,
      });
      await getCurrentUser();
      this.setData({
        loggedIn: hasAccessToken(),
        accountProfile: buildAccountProfile(true),
        passwordModalVisible: false,
        passwordCode: "",
        passwordValue: "",
        passwordConfirm: "",
        passwordNotice: "",
        passwordError: "",
      });
      markProfileShouldRefresh();
      wx.showToast({ title: "密码已更新", icon: "success" });
    } catch (error) {
      this.setData({ passwordError: getErrorMessage(error) });
    } finally {
      this.setData({ passwordLoading: false });
    }
  },

  resetLoggedOutState() {
    this.setData({
      loggedIn: false,
      accountProfile: buildAccountProfile(false),
      nicknameModalVisible: false,
      emailBindingModalVisible: false,
      passwordModalVisible: false,
      accountError: "",
    });
    wx.showToast({ title: "请重新登录", icon: "none" });
  },

  handleAuthError(error: unknown) {
    if (isLoginRequiredError(error)) {
      this.resetLoggedOutState();
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

function isEmailLike(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}
