import {
  createCaptcha,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isApiResponseError,
  isCaptchaRequiredError,
  loginWithEmailCode,
  loginWithPassword,
  loginWithSmsCode,
  loginWithWechat,
  resetPassword,
  sendEmailLoginCode,
  sendPasswordResetCode,
  sendSmsLoginCode,
} from "../../utils/api-auth";
import {
  decodeRedirect,
  isGuestAccessiblePage,
  navigateToGuestFallback,
  navigateToRedirect,
} from "../../utils/navigation";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  isOffline,
  OFFLINE_WRITE_BLOCKED_MESSAGE,
} from "../../utils/network-state";

type LoginMode = "phone" | "wechat" | "password" | "email" | "reset";
type PhoneLoginMode = "sms" | "password";

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
    loginMode: "phone" as LoginMode,
    phoneLoginMode: "sms" as PhoneLoginMode,
    phone: "",
    phonePassword: "",
    smsCode: "",
    smsTicket: "",
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
      ...clearCaptchaState(),
    });
  },

  switchToPhone() {
    this.setData({
      loginMode: "phone",
      error: "",
      notice: "",
      ...clearCaptchaState(),
    });
  },

  switchPhoneMode(event: WechatMiniprogram.BaseEvent) {
    const mode = event.currentTarget.dataset.mode as PhoneLoginMode;
    if (mode !== "sms" && mode !== "password") {
      return;
    }
    this.setData({
      phoneLoginMode: mode,
      error: "",
      notice: "",
      ...clearCaptchaState(),
    });
  },

  switchToPassword() {
    this.setData({
      loginMode: "password",
      error: "",
      notice: "",
      ...clearCaptchaState(),
    });
  },

  switchToEmail() {
    this.setData({
      loginMode: "email",
      error: "",
      notice: "",
      ...clearCaptchaState(),
    });
  },

  switchToReset() {
    this.setData({
      loginMode: "reset",
      error: "",
      notice: "",
      ...clearCaptchaState(),
    });
  },

  onFieldInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "phone"
      | "phonePassword"
      | "smsCode"
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
    if (field === "phone") {
      this.setData({
        phone: event.detail.value,
        smsTicket: "",
      });
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

  async sendPhoneLoginCode() {
    if (this.data.codeLoading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const phone = normalizePhoneInput(this.data.phone);
    if (!isMainlandPhone(phone)) {
      this.setData({ error: "请填写 11 位大陆手机号", notice: "" });
      return;
    }
    this.setData({ codeLoading: true, error: "", notice: "" });
    try {
      const response = await sendSmsLoginCode(phone);
      this.setData({
        phone: response.phone,
        smsTicket: response.sms_ticket,
        notice: buildSmsCodeNotice(response.debug_code),
      });
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ codeLoading: false });
    }
  },

  async loginWithPhoneSms() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const phone = normalizePhoneInput(this.data.phone);
    const smsCode = this.data.smsCode.trim();
    if (!isMainlandPhone(phone)) {
      this.setData({ error: "请填写 11 位大陆手机号", notice: "" });
      return;
    }
    if (!this.data.smsTicket) {
      this.setData({ error: "请先获取短信验证码", notice: "" });
      return;
    }
    if (!smsCode) {
      this.setData({ error: "请填写短信验证码", notice: "" });
      return;
    }
    this.setData({ loading: true, error: "", notice: "" });
    try {
      await loginWithSmsCode({
        phone,
        sms_ticket: this.data.smsTicket,
        sms_verification_code: smsCode,
      });
      this.afterLoginSuccess();
    } catch (error) {
      this.setData({ error: phoneLoginErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  async loginWithPhonePassword() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const phone = normalizePhoneInput(this.data.phone);
    const password = this.data.phonePassword;
    const captchaAnswer = this.data.captchaAnswer.trim();
    if (!isMainlandPhone(phone)) {
      this.setData({ error: "请填写 11 位大陆手机号", notice: "" });
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
        account: phone,
        password,
        captcha_ticket: this.data.captchaTicket || undefined,
        captcha_answer: captchaAnswer || undefined,
      });
      this.afterLoginSuccess();
    } catch (error) {
      if (isCaptchaRequiredError(error)) {
        await this.refreshCaptcha(phone, "请先完成图形验证");
      } else {
        this.setData({ error: phoneLoginErrorMessage(error) });
      }
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
      this.setData({ error: "请填写账号、手机号或邮箱", notice: "" });
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
      this.setData({ error: "请先填写账号、手机号或邮箱", notice: "" });
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
    if (isGuestAccessiblePage(this.data.redirect)) {
      navigateToRedirect(this.data.redirect);
      return;
    }
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
  return user.nickname || user.username || user.email || user.phone || "微信用户";
}

function buildCodeNotice(): string {
  return "验证码已发送，请查看邮箱";
}

function buildSmsCodeNotice(debugCode?: string): string {
  if (debugCode) {
    return `验证码已发送，测试验证码：${debugCode}`;
  }
  return "验证码已发送，请查看短信";
}

function isEmailLike(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}

function normalizePhoneInput(value: string): string {
  const digits = value.replace(/\D/g, "");
  if (digits.length === 13 && digits.startsWith("86")) {
    return digits.slice(2);
  }
  return digits;
}

function isMainlandPhone(value: string): boolean {
  return /^1[3-9]\d{9}$/.test(normalizePhoneInput(value));
}

function phoneLoginErrorMessage(error: unknown): string {
  if (isApiResponseError(error) && error.statusCode === 401) {
    return "手机号未注册或未绑定，请先注册账号或登录后绑定手机号。";
  }
  return getErrorMessage(error);
}

function clearCaptchaState() {
  return {
    captchaAnswer: "",
    captchaTicket: "",
    captchaImageSrc: "",
  };
}

function toSvgDataUri(svg: string): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}
