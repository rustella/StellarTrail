import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createTrip,
  getErrorMessage,
  getTrip,
  hasAccessToken,
  isLoginRequiredError,
  updateTrip,
} from "../../../utils/api-trips";
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

const TRIP_HOME_SHOULD_REFRESH_KEY = "stellartrail_home_trip_refresh";

Page({
  data: {
    id: "",
    tripType: "team" as "solo" | "team",
    name: "",
    startDate: "",
    endDate: "",
    description: "",
    fieldVersions: {} as Record<string, number>,
    saving: false,
    loading: false,
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id || "";
    const tripType = options.tripType === "solo" ? "solo" : "team";
    this.setData({ id, tripType });
    wx.setNavigationBarTitle({ title: id ? "编辑行程" : tripType === "solo" ? "新建单人行程" : "新建多人行程" });
    if (id) {
      this.loadDetail();
    }
  },

  onShow() {
    syncPageTheme(this);
  },

  async loadDetail() {
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后可以编辑行程。",
        redirectUrl: `/pages/trips/form/index?id=${encodeURIComponent(this.data.id)}`,
      });
      return;
    }
    this.setData({ loading: true, error: "" });
    try {
      const detail = await getTrip(this.data.id);
      this.setData({
        name: detail.plan.name,
        startDate: detail.plan.start_date || "",
        endDate: detail.plan.end_date || "",
        description: detail.plan.description || "",
        fieldVersions: detail.plan.field_versions,
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后编辑行程。",
          redirectUrl: `/pages/trips/form/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  onInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as string;
    this.setData({ [field]: event.detail.value });
  },

  onDateChange(event: WechatMiniprogram.PickerChange) {
    const field = event.currentTarget.dataset.field as string;
    this.setData({ [field]: event.detail.value as string });
  },

  async submit() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const name = this.data.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写行程名称", icon: "none" });
      return;
    }
    this.setData({ saving: true, error: "" });
    const editableFields = {
      title: name,
      start_date: nullableText(this.data.startDate),
      end_date: nullableText(this.data.endDate),
      description: nullableText(this.data.description),
    };
    try {
      const detail = this.data.id
        ? await updateTrip(this.data.id, {
            ...editableFields,
            base_field_versions: {
              title: this.data.fieldVersions.title ?? this.data.fieldVersions.name ?? 0,
              start_date: this.data.fieldVersions.start_date ?? 0,
              end_date: this.data.fieldVersions.end_date ?? 0,
              description: this.data.fieldVersions.description ?? 0,
            },
          })
        : await createTrip({
            trip_type: this.data.tripType,
            ...editableFields,
          });
      wx.setStorageSync("stellartrail_trips_refresh", true);
      wx.setStorageSync(TRIP_HOME_SHOULD_REFRESH_KEY, true);
      wx.redirectTo({
        url: `/pages/trips/detail/index?id=${detail.plan.id}`,
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后保存行程。",
          redirectUrl: "/pages/trips/form/index",
        });
        return;
      }
      showMutationError(error, "保存失败");
    } finally {
      this.setData({ saving: false });
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

function showMutationError(error: unknown, fallback: string) {
  const code = (error as { code?: string }).code;
  if (code === "edit_conflict") {
    wx.showModal({
      title: "内容已被队友修改",
      content: "请返回详情刷新后再保存，避免覆盖同一字段。",
      showCancel: false,
    });
    return;
  }
  wx.showToast({ title: getErrorMessage(error) || fallback, icon: "none" });
}
