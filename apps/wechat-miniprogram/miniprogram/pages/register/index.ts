import {
  getErrorMessage,
  registerWithPassword,
  sendEmailVerificationCode,
} from "../../utils/api";
import { decodeRedirect, navigateToRedirect } from "../../utils/navigation";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  isOffline,
  OFFLINE_WRITE_BLOCKED_MESSAGE,
} from "../../utils/network-state";

Page({
  data: {
    redirect: "/pages/profile/index",
    username: "",
    email: "",
    emailCode: "",
    password: "",
    confirmPassword: "",
    loading: false,
    codeLoading: false,
    codeCountdown: 0,
    codeButtonText: "获取验证码",
    error: "",
    notice: "",
    ...getThemeViewData(),
  },

  codeTimer: 0 as ReturnType<typeof setInterval> | 0,

  onLoad(options: Record<string, string | undefined>) {
    this.setData({ redirect: decodeRedirect(options.redirect) });
  },

  onShow() {
    syncPageTheme(this);
  },

  onUnload() {
    this.stopCodeCountdown();
  },

  onFieldInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | "username"
      | "email"
      | "emailCode"
      | "password"
      | "confirmPassword";
    if (!field) {
      return;
    }
    this.setData({ [field]: event.detail.value });
  },

  async sendCode() {
    const email = this.data.email.trim();
    if (this.data.codeLoading || this.data.codeCountdown > 0) {
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
      await sendEmailVerificationCode(email);
      this.setData({ notice: "验证码已发送，请查看邮箱" });
      this.startCodeCountdown();
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ codeLoading: false });
    }
  },

  async register() {
    if (this.data.loading) {
      return;
    }
    if (isOffline()) {
      this.setData({ error: OFFLINE_WRITE_BLOCKED_MESSAGE, notice: "" });
      return;
    }
    const username = this.data.username.trim();
    const email = this.data.email.trim();
    const emailCode = this.data.emailCode.trim();
    const password = this.data.password;
    const confirmPassword = this.data.confirmPassword;
    const validationError = validateForm({
      username,
      email,
      emailCode,
      password,
      confirmPassword,
    });
    if (validationError) {
      this.setData({ error: validationError, notice: "" });
      return;
    }

    this.setData({ loading: true, error: "", notice: "" });
    try {
      await registerWithPassword({
        username,
        email,
        password,
        confirm_password: confirmPassword,
        email_verification_code: emailCode,
      });
      wx.showToast({ title: "注册成功", icon: "success" });
      navigateToRedirect(this.data.redirect);
    } catch (error) {
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  goLogin() {
    wx.navigateBack({
      delta: 1,
      fail: () =>
        wx.redirectTo({
          url: `/pages/login/index?redirect=${encodeURIComponent(this.data.redirect)}`,
        }),
    });
  },

  startCodeCountdown() {
    this.stopCodeCountdown();
    this.setData({ codeCountdown: 60, codeButtonText: "60 秒" });
    this.codeTimer = setInterval(() => {
      const next = Math.max(0, this.data.codeCountdown - 1);
      this.setData({
        codeCountdown: next,
        codeButtonText: next > 0 ? `${next} 秒` : "获取验证码",
      });
      if (next === 0) {
        this.stopCodeCountdown();
      }
    }, 1000);
  },

  stopCodeCountdown() {
    if (this.codeTimer) {
      clearInterval(this.codeTimer);
      this.codeTimer = 0;
    }
  },
});

function validateForm(values: {
  username: string;
  email: string;
  emailCode: string;
  password: string;
  confirmPassword: string;
}): string {
  if (!values.username) {
    return "请填写账号";
  }
  if (!isEmailLike(values.email)) {
    return "请填写可用邮箱";
  }
  if (!values.emailCode) {
    return "请填写邮箱验证码";
  }
  if (values.password.length < 8) {
    return "密码至少 8 位";
  }
  if (values.password !== values.confirmPassword) {
    return "两次密码不一致";
  }
  return "";
}

function isEmailLike(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
}
