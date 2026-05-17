import {
  createCaptcha,
  getErrorMessage,
  getStoredUser,
  hasAccessToken,
  isCaptchaRequiredError,
  loginWithPassword,
  loginWithWechat,
} from "../../utils/api";
import { decodeRedirect, navigateToRedirect } from "../../utils/navigation";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";

type LoginMode = "wechat" | "password";

Page({
  data: {
    redirect: "/pages/profile/index",
    loggedIn: hasAccessToken(),
    userDisplay: buildUserDisplay(),
    loading: false,
    captchaLoading: false,
    error: "",
    loginMode: "wechat" as LoginMode,
    account: "",
    password: "",
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
    this.setData({ loginMode: "wechat", error: "" });
  },

  switchToPassword() {
    this.setData({ loginMode: "password", error: "" });
  },

  onFieldInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "account"
      | "password"
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
    this.setData({ loading: true, error: "" });
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
    const account = this.data.account.trim();
    const password = this.data.password;
    const captchaAnswer = this.data.captchaAnswer.trim();
    if (!account) {
      this.setData({ error: "请填写账号或邮箱" });
      return;
    }
    if (!password) {
      this.setData({ error: "请填写密码" });
      return;
    }
    if (this.data.captchaTicket && !captchaAnswer) {
      this.setData({ error: "请填写图形验证码" });
      return;
    }

    this.setData({ loading: true, error: "" });
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

  async refreshCaptcha(account?: string, message = "已刷新图形验证码") {
    const targetAccount = (account ?? this.data.account).trim();
    if (!targetAccount) {
      this.setData({ error: "请先填写账号或邮箱" });
      return;
    }
    if (this.data.captchaLoading) {
      return;
    }
    this.setData({ captchaLoading: true, error: "" });
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
    navigateToRedirect(this.data.redirect || "/pages/index/index");
  },

  afterLoginSuccess() {
    this.setData({ loggedIn: true, userDisplay: buildUserDisplay() });
    wx.showToast({ title: "登录成功", icon: "success" });
    navigateToRedirect(this.data.redirect);
  },
});

function buildUserDisplay(): string {
  const user = getStoredUser();
  if (!user) {
    return "未登录";
  }
  return user.nickname || user.username || user.email || "寻径星野用户";
}

function toSvgDataUri(svg: string): string {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}
