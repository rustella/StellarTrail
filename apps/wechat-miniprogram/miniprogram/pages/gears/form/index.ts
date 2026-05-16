import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  createGear,
  getErrorMessage,
  getGear,
  updateGear,
} from "../../../utils/api";
import {
  buildGearPayload,
  createDefaultGearFormData,
  gearToFormData,
  GEAR_CATEGORY_OPTIONS,
  GEAR_STATUS_OPTIONS,
  optionIndex,
  type GearCategory,
  type GearFormData,
  type GearStatus,
} from "../../../utils/gear-utils";

Page({
  data: {
    mode: "create" as "create" | "edit",
    id: "",
    form: createDefaultGearFormData(),
    categoryOptions: GEAR_CATEGORY_OPTIONS,
    categoryLabels: GEAR_CATEGORY_OPTIONS.map((item) => item.label),
    categoryHints: GEAR_CATEGORY_OPTIONS.map((item) => item.hint || ""),
    categoryIndex: 0,
    statusOptions: GEAR_STATUS_OPTIONS,
    statusLabels: GEAR_STATUS_OPTIONS.map((item) => item.label),
    statusIndex: 0,
    loading: false,
    submitting: false,
    error: "",
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
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
  },

  async loadGearForEdit(id: string) {
    this.setData({ loading: true, error: "" });
    try {
      const item = await getGear(id);
      const form = gearToFormData(item);
      this.setForm(form);
    } catch (error) {
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

  async submitForm() {
    if (this.data.submitting) {
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
      this.setData({ error: getErrorMessage(error) });
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ submitting: false });
    }
  },

  cancel() {
    wx.navigateBack();
  },
});
