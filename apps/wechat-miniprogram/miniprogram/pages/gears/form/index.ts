import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createGear,
  getErrorMessage,
  getGear,
  getGearSpecKeyRankings,
  getGearTagSuggestions,
  hasAccessToken,
  isLoginRequiredError,
  updateGear,
} from "../../../utils/api";
import {
  addGearTagViews,
  buildGearPayload,
  combineSpecValue,
  createGearTagSuggestionViews,
  createDefaultGearFormData,
  gearToFormData,
  GEAR_CATEGORY_OPTIONS,
  GEAR_CURRENCY_OPTIONS,
  GEAR_STATUS_OPTIONS,
  GEAR_TAG_COLOR_OPTIONS,
  GEAR_WEIGHT_UNIT_OPTIONS,
  PURCHASE_LOCATION_OPTIONS,
  getGearSpecFieldViews,
  normalizeGearTagColor,
  randomGearTagColor,
  optionIndex,
  type GearCategory,
  type GearCurrency,
  type GearFormData,
  type GearStatus,
  type GearTagColor,
  type GearTagSuggestionView,
  type GearTagView,
  type GearWeightUnit,
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
    weightUnitOptions: GEAR_WEIGHT_UNIT_OPTIONS,
    weightUnitLabels: GEAR_WEIGHT_UNIT_OPTIONS.map((item) => item.label),
    weightUnitIndex: 0,
    currencyOptions: GEAR_CURRENCY_OPTIONS,
    currencyLabels: GEAR_CURRENCY_OPTIONS.map((item) => item.label),
    officialPriceCurrencyIndex: 0,
    purchasePriceCurrencyIndex: 0,
    purchaseLocationOptions: PURCHASE_LOCATION_OPTIONS,
    purchaseLocationSheetVisible: false,
    customPurchaseLocationVisible: false,
    customPurchaseLocationText: "",
    tagSheetVisible: false,
    tagSuggestions: [] as GearTagSuggestionView[],
    tagInputText: "",
    tagColorOptions: GEAR_TAG_COLOR_OPTIONS.map((item) => ({
      ...item,
      colorClass: `tag-color-${item.value}`,
    })),
    selectedTagColor: "" as "" | GearTagColor,
    specRankedKeys: [] as string[],
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
      this.loadSpecRankings(this.data.form.category);
    }
  },

  onShow() {
    syncPageTheme(this);
    if (this.data.requiresLogin && hasAccessToken()) {
      this.setData({ requiresLogin: false });
      if (this.data.id) {
        this.loadGearForEdit(this.data.id);
      } else {
        this.loadSpecRankings(this.data.form.category);
      }
    }
  },

  async loadGearForEdit(id: string) {
    this.setData({ loading: true, error: "" });
    try {
      const item = await getGear(id);
      const form = gearToFormData(item);
      this.setForm(form);
      this.loadSpecRankings(form.category);
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
      weightUnitIndex: optionIndex(GEAR_WEIGHT_UNIT_OPTIONS, form.weightUnit),
      officialPriceCurrencyIndex: optionIndex(
        GEAR_CURRENCY_OPTIONS,
        form.officialPriceCurrency,
      ),
      purchasePriceCurrencyIndex: optionIndex(
        GEAR_CURRENCY_OPTIONS,
        form.purchasePriceCurrency,
      ),
      specRankedKeys: [],
      specFields: getGearSpecFieldViews(form.category, form.specs),
    });
  },

  async loadSpecRankings(category: GearCategory) {
    if (!hasAccessToken()) {
      return;
    }
    try {
      const response = await getGearSpecKeyRankings(category);
      if (this.data.form.category !== category) {
        return;
      }
      this.setData({
        specRankedKeys: response.keys,
        specFields: getGearSpecFieldViews(
          category,
          this.data.form.specs,
          response.keys,
        ),
      });
    } catch {
      if (this.data.form.category !== category) {
        return;
      }
      this.setData({
        specRankedKeys: [],
        specFields: getGearSpecFieldViews(category, this.data.form.specs),
      });
    }
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
      specRankedKeys: [],
      specFields: getGearSpecFieldViews(category, this.data.form.specs),
    });
    this.loadSpecRankings(category);
  },

  onStatusChange(event: any) {
    const index = Number(event.detail.value || 0);
    const status = GEAR_STATUS_OPTIONS[index].value as GearStatus;
    this.setData({
      statusIndex: index,
      "form.status": status,
    });
  },

  confirmEnableShare() {
    wx.showModal({
      title: "提交共享审核？",
      content: "确认后，保存装备时会提交共享审核；审核通过后会出现在装备图鉴。",
      confirmText: "提交审核",
      confirmColor: "#0f766e",
      success: (result) => {
        if (result.confirm) {
          this.setData({ "form.shareEnabled": true });
        }
      },
    });
  },

  disableShare() {
    this.setData({ "form.shareEnabled": false });
  },

  onPurchaseDateChange(event: any) {
    this.setData({ "form.purchaseDate": event.detail.value });
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

  onPurchasePriceCurrencyChange(event: any) {
    const index = Number(event.detail.value || 0);
    const currency = GEAR_CURRENCY_OPTIONS[index].value as GearCurrency;
    this.setData({
      purchasePriceCurrencyIndex: index,
      "form.purchasePriceCurrency": currency,
    });
  },

  openPurchaseLocationSheet() {
    this.setData({ purchaseLocationSheetVisible: true });
  },

  closePurchaseLocationSheet() {
    this.setData({ purchaseLocationSheetVisible: false });
  },

  selectPurchaseLocation(event: WechatMiniprogram.BaseEvent) {
    const value = String(event.currentTarget.dataset.value || "");
    if (!value) {
      return;
    }
    this.setData({
      "form.purchaseLocation": value,
      purchaseLocationSheetVisible: false,
    });
  },

  openCustomPurchaseLocation() {
    this.setData({
      purchaseLocationSheetVisible: false,
      customPurchaseLocationVisible: true,
      customPurchaseLocationText: this.data.form.purchaseLocation,
    });
  },

  onCustomPurchaseLocationInput(event: WechatMiniprogram.BaseEvent) {
    this.setData({ customPurchaseLocationText: (event as any).detail.value });
  },

  saveCustomPurchaseLocation() {
    this.setData({
      "form.purchaseLocation": this.data.customPurchaseLocationText.trim(),
      customPurchaseLocationVisible: false,
      customPurchaseLocationText: "",
    });
  },

  cancelCustomPurchaseLocation() {
    this.setData({
      customPurchaseLocationVisible: false,
      customPurchaseLocationText: "",
    });
  },

  async openTagSheet() {
    this.setData({
      tagSheetVisible: true,
      tagInputText: "",
      selectedTagColor: "",
    });
    if (!hasAccessToken()) {
      this.setData({ tagSuggestions: [] });
      return;
    }
    try {
      const response = await getGearTagSuggestions(20);
      this.setData({
        tagSuggestions: createGearTagSuggestionViews(response.items),
      });
    } catch {
      this.setData({ tagSuggestions: [] });
    }
  },

  closeTagSheet() {
    this.setData({
      tagSheetVisible: false,
      tagInputText: "",
      selectedTagColor: "",
    });
  },

  onTagInput(event: WechatMiniprogram.BaseEvent) {
    this.setData({ tagInputText: (event as any).detail.value });
  },

  selectTagColor(event: WechatMiniprogram.BaseEvent) {
    const value = String(event.currentTarget.dataset.value || "");
    const color = normalizeGearTagColor(value);
    if (!color) {
      return;
    }
    this.setData({ selectedTagColor: color });
  },

  clearTagColor() {
    this.setData({ selectedTagColor: "" });
  },

  selectTagSuggestion(event: WechatMiniprogram.BaseEvent) {
    const name = String(event.currentTarget.dataset.name || "");
    const suggestedColor = normalizeGearTagColor(
      String(event.currentTarget.dataset.color || ""),
    );
    const color =
      this.data.selectedTagColor || suggestedColor || randomGearTagColor();
    this.addTagsToForm(name, color, true);
  },

  saveCustomTag() {
    const color = this.data.selectedTagColor || null;
    this.addTagsToForm(this.data.tagInputText, color, true);
  },

  removeTag(event: WechatMiniprogram.BaseEvent) {
    const index = Number(event.currentTarget.dataset.index);
    if (!Number.isFinite(index)) {
      return;
    }
    const tags = [...this.data.form.tags];
    tags.splice(index, 1);
    this.setData({ "form.tags": tags });
  },

  addTagsToForm(
    input: string,
    color: GearTagColor | null,
    closeAfterAdd: boolean,
  ) {
    const nextTags = addGearTagViews(
      this.data.form.tags,
      input,
      color || undefined,
    );
    if (nextTags.length === this.data.form.tags.length) {
      wx.showToast({ title: "标签已存在或为空", icon: "none" });
      return;
    }
    this.setData({
      "form.tags": nextTags as GearTagView[],
      tagInputText: "",
      ...(closeAfterAdd
        ? { tagSheetVisible: false, selectedTagColor: "" }
        : {}),
    });
  },

  stopTap() {},

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
      specFields: getGearSpecFieldViews(
        this.data.form.category,
        specs,
        this.data.specRankedKeys,
      ),
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
      specFields: getGearSpecFieldViews(
        this.data.form.category,
        specs,
        this.data.specRankedKeys,
      ),
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
