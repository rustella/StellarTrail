import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createGearPackingList,
  getErrorMessage,
  getGearPackingList,
  hasAccessToken,
  isLoginRequiredError,
  updateGearPackingList,
} from "../../../utils/api-gears";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  showLoginPrompt,
} from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";

interface PackingListFormData {
  name: string;
  routeName: string;
  durationLabel: string;
}

Page({
  data: {
    mode: "create" as "create" | "edit",
    id: "",
    form: {
      name: "",
      routeName: "",
      durationLabel: "",
    } as PackingListFormData,
    loading: false,
    submitting: false,
    requiresLogin: false,
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id || "";
    this.setData({ id, mode: id ? "edit" : "create" });
    wx.setNavigationBarTitle({ title: id ? "编辑打包清单" : "新建打包清单" });
    if (!hasAccessToken()) {
      this.setData({ requiresLogin: true });
      showLoginPrompt(this, {
        message: "登录后可以创建和编辑自己的打包清单。",
        redirectUrl: `/pages/packing-lists/form/index${id ? `?id=${encodeURIComponent(id)}` : ""}`,
      });
      return;
    }
    if (id) {
      this.loadList(id);
    }
  },

  onShow() {
    syncPageTheme(this);
    if (this.data.requiresLogin && hasAccessToken()) {
      this.setData({ requiresLogin: false });
      if (this.data.id) {
        this.loadList(this.data.id);
      }
    }
  },

  async loadList(id: string) {
    this.setData({ loading: true, error: "" });
    try {
      const detail = await getGearPackingList(id);
      this.setData({
        form: {
          name: detail.name,
          routeName: detail.route_name || "",
          durationLabel: detail.duration_label || "",
        },
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ requiresLogin: true, error: "" });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后编辑打包清单。",
          redirectUrl: `/pages/packing-lists/form/index?id=${encodeURIComponent(id)}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  onInput(event: WechatMiniprogram.BaseEvent) {
    const field = event.currentTarget.dataset
      .field as keyof PackingListFormData;
    const value = (event as any).detail.value;
    this.setData({ [`form.${field}`]: value });
  },

  async submit() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后可以保存打包清单。",
        redirectUrl: "/pages/packing-lists/form/index",
      });
      return;
    }
    const payload = {
      name: this.data.form.name.trim(),
      route_name: nullableText(this.data.form.routeName),
      duration_label: nullableText(this.data.form.durationLabel),
    };
    if (!payload.name) {
      this.setData({ error: "请填写清单名称" });
      return;
    }
    this.setData({ submitting: true, error: "" });
    try {
      const detail =
        this.data.mode === "edit"
          ? await updateGearPackingList(this.data.id, payload)
          : await createGearPackingList(payload);
      wx.setStorageSync("stellartrail_packing_lists_should_refresh", true);
      if (this.data.mode === "edit") {
        wx.redirectTo({
          url: `/pages/packing-lists/detail/index?id=${detail.id}`,
        });
      } else {
        wx.redirectTo({
          url: `/pages/packing-lists/select-gears/index?id=${detail.id}`,
        });
      }
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后保存打包清单。",
          redirectUrl: "/pages/packing-lists/form/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ submitting: false });
    }
  },

  cancel() {
    if (getCurrentPages().length > 1) {
      wx.navigateBack();
    } else {
      wx.redirectTo({ url: "/pages/packing-lists/index" });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function nullableText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}
