import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createGear,
  getErrorMessage,
  getGear,
  hasAccessToken,
  isLoginRequiredError,
  updateGear,
} from "../../../utils/api";
import {
  buildGearPayload,
  combineSpecValue,
  createDefaultGearFormData,
  gearToFormData,
  GEAR_CATEGORY_OPTIONS,
  GEAR_CURRENCY_OPTIONS,
  GEAR_STATUS_OPTIONS,
  getGearSpecFieldViews,
  optionIndex,
  type GearCategory,
  type GearCurrency,
  type GearFormData,
  type GearStatus,
} from "../../../utils/gear-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../../utils/auth-prompt";

Page({
  data: {
    mode: "create" as "create" | "edit",
    id: "",
    templateId: "",
    form: createDefaultGearFormData(),
    categoryOptions: GEAR_CATEGORY_OPTIONS,
    categoryLabels: GEAR_CATEGORY_OPTIONS.map((item) => item.label),
    categoryHints: GEAR_CATEGORY_OPTIONS.map((item) => item.hint || ""),
    categoryIndex: 0,
    statusOptions: GEAR_STATUS_OPTIONS,
    statusLabels: GEAR_STATUS_OPTIONS.map((item) => item.label),
    statusIndex: 0,
    currencyOptions: GEAR_CURRENCY_OPTIONS,
    currencyLabels: GEAR_CURRENCY_OPTIONS.map((item) => item.label),
    officialPriceCurrencyIndex: 0,
    purchasePriceCurrencyIndex: 0,
    specFields: getGearSpecFieldViews("backpack_system", {}),
    loading: false,
    submitting: false,
    requiresLogin: false,
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    const templateId = options.template || "";
    this.setData({ templateId });
    if (!hasAccessToken()) {
      this.setData({
        requiresLogin: true,
        mode: id ? "edit" : "create",
        id: id || "",
      });
      wx.setNavigationBarTitle({ title: id ? "编辑装备" : "添加装备" });
      showLoginPrompt(this, {
        message: id
          ? "登录后可以编辑自己的装备。"
          : "登录后就能把这件装备保存到自己的清单里。",
        redirectUrl: `/pages/gears/form/index${id ? `?id=${encodeURIComponent(id)}` : templateId ? `?template=${encodeURIComponent(templateId)}` : ""}`,
      });
      return;
    }
    this.setData({ requiresLogin: false });
    if (id) {
      this.setData({ id, mode: "edit" });
      wx.setNavigationBarTitle({ title: "编辑装备" });
      this.loadGearForEdit(id);
    } else {
      wx.setNavigationBarTitle({ title: "添加装备" });
    }
  },

  onShow() {
    syncPageTheme(this);
    if (this.data.requiresLogin && hasAccessToken()) {
      this.setData({ requiresLogin: false });
      if (this.data.id) {
        this.loadGearForEdit(this.data.id);
      }
    }
  },

  async loadGearForEdit(id: string) {
    this.setData({ loading: true, error: "" });
    try {
      const item = await getGear(id);
      const form = gearToFormData(item);
      this.setForm(form);
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ requiresLogin: true, error: "" });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后编辑装备。",
          redirectUrl: `/pages/gears/form/index?id=${encodeURIComponent(id)}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  setForm(form: GearFormData) {
    this.setData({
      form,
      categoryIndex: optionIndex(GEAR_CATEGORY_OPTIONS, form.category),
      statusIndex: optionIndex(GEAR_STATUS_OPTIONS, form.status),
      officialPriceCurrencyIndex: optionIndex(
        GEAR_CURRENCY_OPTIONS,
        form.officialPriceCurrency,
      ),
      purchasePriceCurrencyIndex: optionIndex(
        GEAR_CURRENCY_OPTIONS,
        form.purchasePriceCurrency,
      ),
      specFields: getGearSpecFieldViews(form.category, form.specs),
    });
  },

  onInput(event: WechatMiniprogram.BaseEvent) {
    const field = event.currentTarget.dataset.field as keyof GearFormData;
    const value = (event as any).detail.value;
    this.setData({ [`form.${field}`]: value });
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

  onStatusChange(event: any) {
    const index = Number(event.detail.value || 0);
    const status = GEAR_STATUS_OPTIONS[index].value as GearStatus;
    this.setData({
      statusIndex: index,
      "form.status": status,
    });
  },

  onShareSwitch(event: any) {
    this.setData({ "form.shareEnabled": Boolean(event.detail.value) });
  },

  onOfficialPriceCurrencyChange(event: any) {
    const index = Number(event.detail.value || 0);
    const currency = GEAR_CURRENCY_OPTIONS[index].value as GearCurrency;
    this.setData({
      officialPriceCurrencyIndex: index,
      "form.officialPriceCurrency": currency,
    });
  },

  onPurchasePriceCurrencyChange(event: any) {
    const index = Number(event.detail.value || 0);
    const currency = GEAR_CURRENCY_OPTIONS[index].value as GearCurrency;
    this.setData({
      purchasePriceCurrencyIndex: index,
      "form.purchasePriceCurrency": currency,
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
    if (
      !requireLoginForAction(this, {
        message:
          this.data.mode === "edit"
            ? "登录后可以编辑自己的装备。"
            : "登录后就能把这件装备保存到自己的清单里。",
        redirectUrl: `/pages/gears/form/index${this.data.id ? `?id=${encodeURIComponent(this.data.id)}` : ""}`,
      })
    ) {
      return;
    }
    let payload;
    try {
      payload = buildGearPayload(this.data.form);
    } catch (error) {
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
      return;
    }

    this.setData({ submitting: true, error: "" });
    try {
      const item =
        this.data.mode === "edit"
          ? await updateGear(this.data.id, payload)
          : await createGear(payload);
      wx.setStorageSync("stellartrail_gears_should_refresh", true);
      wx.showToast({
        title: this.data.mode === "edit" ? "已保存" : "已添加",
        icon: "success",
      });
      wx.redirectTo({ url: `/pages/gears/detail/index?id=${item.id}` });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后保存装备。",
          redirectUrl: `/pages/gears/form/index${this.data.id ? `?id=${encodeURIComponent(this.data.id)}` : ""}`,
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
    if (this.data.mode === "edit" && this.data.id) {
      wx.redirectTo({
        url: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}`,
      });
      return;
    }
    wx.switchTab({ url: "/pages/gears/index" });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});
