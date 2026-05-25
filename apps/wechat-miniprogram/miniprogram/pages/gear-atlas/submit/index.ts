import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createGearAtlasSubmission,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
} from "../../../utils/api-atlas";
import {
  buildGearAtlasPayload,
  combineSpecValue,
  createDefaultGearFormData,
  GEAR_CATEGORY_OPTIONS,
  GEAR_CURRENCY_OPTIONS,
  GEAR_WEIGHT_UNIT_OPTIONS,
  getGearSpecFieldViews,
  optionIndex,
  type GearCategory,
  type GearCurrency,
  type GearFormData,
  type GearWeightUnit,
} from "../../../utils/gear-form";
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

Page({
  data: {
    form: createDefaultGearFormData(),
    categoryOptions: GEAR_CATEGORY_OPTIONS,
    categoryLabels: GEAR_CATEGORY_OPTIONS.map((item) => item.label),
    categoryHints: GEAR_CATEGORY_OPTIONS.map((item) => item.hint || ""),
    categoryIndex: 0,
    weightUnitOptions: GEAR_WEIGHT_UNIT_OPTIONS,
    weightUnitLabels: GEAR_WEIGHT_UNIT_OPTIONS.map((item) => item.label),
    weightUnitIndex: 0,
    currencyOptions: GEAR_CURRENCY_OPTIONS,
    currencyLabels: GEAR_CURRENCY_OPTIONS.map((item) => item.label),
    officialPriceCurrencyIndex: 0,
    specFields: getGearSpecFieldViews("backpack_system", {}),
    submitting: false,
    requiresLogin: !hasAccessToken(),
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad() {
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后可以把新装备投稿到图鉴审核。",
        redirectUrl: "/pages/gear-atlas/submit/index",
      });
    }
  },

  onShow() {
    syncPageTheme(this);
    if (this.data.requiresLogin && hasAccessToken()) {
      this.setData({ requiresLogin: false });
    }
  },

  onInput(event: WechatMiniprogram.BaseEvent) {
    const field = event.currentTarget.dataset.field as keyof GearFormData;
    this.setData({ [`form.${field}`]: (event as any).detail.value });
  },

  onCategoryChange(event: any) {
    const index = Number(event.detail.value || 0);
    const category = GEAR_CATEGORY_OPTIONS[index].value as GearCategory;
    this.setData({
      categoryIndex: index,
      "form.category": category,
      specFields: getGearSpecFieldViews(category, this.data.form.specs),
    });
  },

  onWeightUnitChange(event: any) {
    const index = Number(event.detail.value || 0);
    const unit = GEAR_WEIGHT_UNIT_OPTIONS[index].value as GearWeightUnit;
    this.setData({
      weightUnitIndex: index,
      "form.weightUnit": unit,
    });
  },

  onOfficialPriceCurrencyChange(event: any) {
    const index = Number(event.detail.value || 0);
    const currency = GEAR_CURRENCY_OPTIONS[index].value as GearCurrency;
    this.setData({
      officialPriceCurrencyIndex: index,
      "form.officialPriceCurrency": currency,
    });
  },

  onSpecInput(event: WechatMiniprogram.BaseEvent) {
    const key = event.currentTarget.dataset.key as string;
    const index = Number(event.currentTarget.dataset.index || 0);
    const value = (event as any).detail.value;
    const field = this.data.specFields[index];
    const unit = field?.units?.[field.unitIndex] ?? "";
    const specs = {
      ...this.data.form.specs,
      [key]: combineSpecValue(value, unit),
    };
    this.setData({
      "form.specs": specs,
      specFields: getGearSpecFieldViews(this.data.form.category, specs),
    });
  },

  onSpecUnitChange(event: any) {
    const key = event.currentTarget.dataset.key as string;
    const fieldIndex = Number(event.currentTarget.dataset.index || 0);
    const unitIndex = Number(event.detail.value || 0);
    const field = this.data.specFields[fieldIndex];
    const unit = field?.units?.[unitIndex] ?? "";
    const specs = {
      ...this.data.form.specs,
      [key]: combineSpecValue(field?.valueText ?? "", unit),
    };
    this.setData({
      "form.specs": specs,
      specFields: getGearSpecFieldViews(this.data.form.category, specs),
    });
  },

  async submitForm() {
    if (this.data.submitting) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后可以把新装备投稿到图鉴审核。",
        redirectUrl: "/pages/gear-atlas/submit/index",
      });
      return;
    }
    let payload;
    try {
      payload = buildGearAtlasPayload(this.data.form);
    } catch (error) {
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
      return;
    }
    this.setData({ submitting: true, error: "" });
    try {
      await createGearAtlasSubmission(payload);
      wx.showToast({ title: "已提交审核", icon: "success" });
      wx.redirectTo({ url: "/pages/gear-atlas/index" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ requiresLogin: true });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后投稿装备。",
          redirectUrl: "/pages/gear-atlas/submit/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ submitting: false });
    }
  },

  cancel() {
    if (getCurrentPages().length > 1) {
      wx.navigateBack();
      return;
    }
    wx.redirectTo({ url: "/pages/gear-atlas/index" });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});
