import {
  createCaptcha,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isCaptchaRequiredError,
  loginWithEmailCode,
  loginWithPassword,
  loginWithWechat,
  resetPassword,
  sendEmailLoginCode,
  sendPasswordResetCode,
} from "../../utils/api-auth";
import {
  decodeRedirect,
  navigateToGuestFallback,
  navigateToRedirect,
} from "../../utils/navigation";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  isOffline,
  OFFLINE_WRITE_BLOCKED_MESSAGE,
} from "../../utils/network-state";

type LoginMode = "wechat" | "password" | "email" | "reset";

Page({
  data: {
    redirect: "/pages/profile/index",
    loggedIn: hasAccessToken(),
    userDisplay: buildUserDisplay(),
    loading: false,
    codeLoading: false,
    captchaLoading: false,
    error: "",
    notice: "",
    loginMode: "wechat" as LoginMode,
    account: "",
    password: "",
    email: "",
    emailCode: "",
    resetPassword: "",
    resetConfirmPassword: "",
    captchaAnswer: "",
    captchaTicket: "",
    captchaImageSrc: "",
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

  switchToWechat() {
    this.setData({
      loginMode: "wechat",
      error: "",
      notice: "",
    });
  },

  switchToPassword() {
    this.setData({ loginMode: "password", error: "", notice: "" });
  },

  switchToEmail() {
    this.setData({ loginMode: "email", error: "", notice: "" });
  },

  switchToReset() {
    this.setData({ loginMode: "reset", error: "", notice: "" });
  },

  onFieldInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "account"
      | "password"
      | "email"
      | "emailCode"
      | "resetPassword"
      | "resetConfirmPassword"
      | "captchaAnswer";
    if (!field) {
      return;
    }
    this.setData({ [field]: event.detail.value });
  },

  async loginWechat() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    this.setData({ loading: true, error: "", notice: "" });
    try {
      await loginWithWechat();
      this.afterLoginSuccess();
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  async loginWithAccount() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const account = this.data.account.trim();
    const password = this.data.password;
    const captchaAnswer = this.data.captchaAnswer.trim();
    if (!account) {
      this.setData({ error: "请填写账号或邮箱", notice: "" });
      return;
    }
    if (!password) {
      this.setData({ error: "请填写密码", notice: "" });
      return;
    }
    if (this.data.captchaTicket && !captchaAnswer) {
      this.setData({ error: "请填写图形验证码", notice: "" });
      return;
    }

    this.setData({ loading: true, error: "", notice: "" });
    try {
      await loginWithPassword({
        account,
        password,
        captcha_ticket: this.data.captchaTicket || undefined,
        captcha_answer: captchaAnswer || undefined,
      });
      this.afterLoginSuccess();
    } catch (error) {
      if (isCaptchaRequiredError(error)) {
        await this.refreshCaptcha(account, "请先完成图形验证");
      } else {
        this.setData({ error: getErrorMessage(error) });
      }
    } finally {
      this.setData({ loading: false });
    }
  },

  async sendLoginCode() {
    const email = this.data.email.trim();
    if (this.data.codeLoading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    if (!isEmailLike(email)) {
      this.setData({ error: "请填写可用邮箱", notice: "" });
      return;
    }
    this.setData({ codeLoading: true, error: "", notice: "" });
    try {
      await sendEmailLoginCode(email);
      this.setData({ notice: buildCodeNotice() });
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ codeLoading: false });
    }
  },

  async loginWithEmailCode() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const email = this.data.email.trim();
    const emailCode = this.data.emailCode.trim();
    if (!isEmailLike(email)) {
      this.setData({ error: "请填写可用邮箱", notice: "" });
      return;
    }
    if (!emailCode) {
      this.setData({ error: "请填写邮箱验证码", notice: "" });
      return;
    }
    this.setData({ loading: true, error: "", notice: "" });
    try {
      await loginWithEmailCode({
        email,
        email_verification_code: emailCode,
      });
      this.afterLoginSuccess();
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  async sendResetCode() {
    const email = this.data.email.trim();
    if (this.data.codeLoading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    if (!isEmailLike(email)) {
      this.setData({ error: "请填写可用邮箱", notice: "" });
      return;
    }
    this.setData({ codeLoading: true, error: "", notice: "" });
    try {
      await sendPasswordResetCode(email);
      this.setData({ notice: buildCodeNotice() });
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ codeLoading: false });
    }
  },

  async submitPasswordReset() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const email = this.data.email.trim();
    const emailCode = this.data.emailCode.trim();
    const password = this.data.resetPassword;
    const confirmPassword = this.data.resetConfirmPassword;
    if (!isEmailLike(email)) {
      this.setData({ error: "请填写可用邮箱", notice: "" });
      return;
    }
    if (!emailCode) {
      this.setData({ error: "请填写邮箱验证码", notice: "" });
      return;
    }
    if (password.length < 8) {
      this.setData({ error: "密码至少 8 位", notice: "" });
      return;
    }
    if (password !== confirmPassword) {
      this.setData({ error: "两次密码不一致", notice: "" });
      return;
    }

    this.setData({ loading: true, error: "", notice: "" });
    try {
      await resetPassword({
        email,
        email_verification_code: emailCode,
        password,
        confirm_password: confirmPassword,
      });
      this.afterLoginSuccess();
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  async refreshCaptcha(account?: string, message = "已刷新图形验证码") {
    const targetAccount = (account ?? this.data.account).trim();
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    if (!targetAccount) {
      this.setData({ error: "请先填写账号或邮箱", notice: "" });
      return;
    }
    if (this.data.captchaLoading) {
      return;
    }
    this.setData({ captchaLoading: true, error: "", notice: "" });
    try {
      const captcha = await createCaptcha(targetAccount);
      this.setData({
        captchaTicket: captcha.captcha_ticket,
        captchaImageSrc: toSvgDataUri(captcha.image_svg),
        captchaAnswer: "",
        error: message,
      });
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ captchaLoading: false });
    }
  },

  goRegister() {
    wx.navigateTo({
      url: `/pages/register/index?redirect=${encodeURIComponent(this.data.redirect)}`,
    });
  },

  skip() {
    navigateToGuestFallback();
  },

  afterLoginSuccess(target?: string) {
    this.setData({ loggedIn: true, userDisplay: buildUserDisplay() });
    wx.showToast({ title: "登录成功", icon: "success" });
    navigateToRedirect(target || this.data.redirect);
  },
});

function buildUserDisplay(): string {
  const user = getStoredUser();
  if (!user) {
    return "未登录";
  }
  return user.nickname || user.username || user.email || "微信用户";
}

function buildCodeNotice(): string {
  return "验证码已发送，请查看邮箱";
}

function isEmailLike(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}

function toSvgDataUri(svg: string): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}
