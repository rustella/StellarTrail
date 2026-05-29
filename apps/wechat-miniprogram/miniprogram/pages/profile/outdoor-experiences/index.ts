import {
  createOutdoorExperience,
  consumeOfflineCacheNotice,
  deleteOutdoorExperience,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  listOutdoorExperiences,
  updateOutdoorExperience,
  type OutdoorExperience,
  type OutdoorExperienceRequest,
} from "../../../utils/api-profile";
import { loginPageUrl } from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";

const TRIPS_REFRESH_KEY = "stellartrail_trips_refresh";
const EXPERIENCES_REFRESH_KEY = "stellartrail_outdoor_experiences_refresh";

interface OutdoorExperienceCard extends OutdoorExperience {
  dateText: string;
  metaText: string;
  sourceText: string;
  summaryLines: Array<{ label: string; text: string }>;
}

interface ExperienceForm {
  title: string;
  startDate: string;
  endDate: string;
  dayCount: string;
  companionCount: string;
  routeSummary: string;
  gearSummary: string;
  foodSummary: string;
  budgetSummary: string;
  notes: string;
}

const EMPTY_FORM: ExperienceForm = {
  title: "",
  startDate: "",
  endDate: "",
  dayCount: "",
  companionCount: "",
  routeSummary: "",
  gearSummary: "",
  foodSummary: "",
  budgetSummary: "",
  notes: "",
};

Page({
  data: {
    loggedIn: hasAccessToken(),
    loading: false,
    saving: false,
    error: "",
    offlineNotice: "",
    experiences: [] as OutdoorExperienceCard[],
    editorVisible: false,
    editorTitle: "新增户外经历",
    editingExperienceId: "",
    experienceForm: { ...EMPTY_FORM },
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync(EXPERIENCES_REFRESH_KEY);
    if (shouldRefresh) {
      wx.removeStorageSync(EXPERIENCES_REFRESH_KEY);
    }
    void this.loadExperiences();
  },

  async loadExperiences() {
    const loggedIn = hasAccessToken();
    this.setData({ loggedIn, error: "" });
    if (!loggedIn) {
      return;
    }
    this.setData({ loading: true });
    try {
      const response = await listOutdoorExperiences();
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        experiences: response.items.map(mapExperienceCard),
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ loggedIn: false });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  goLogin() {
    wx.navigateTo({
      url: loginPageUrl("/pages/profile/outdoor-experiences/index"),
    });
  },

  openCreateEditor() {
    if (!this.data.loggedIn) {
      this.goLogin();
      return;
    }
    this.setData({
      editorVisible: true,
      editorTitle: "新增户外经历",
      editingExperienceId: "",
      experienceForm: { ...EMPTY_FORM },
      error: "",
    });
  },

  openEditEditor(event: WechatMiniprogram.BaseEvent) {
    const id = String(event.currentTarget.dataset.id || "");
    const item = this.data.experiences.find(
      (experience) => experience.id === id,
    );
    if (!item) {
      return;
    }
    this.setData({
      editorVisible: true,
      editorTitle: "编辑户外经历",
      editingExperienceId: id,
      experienceForm: experienceToForm(item),
      error: "",
    });
  },

  closeEditor() {
    if (this.data.saving) {
      return;
    }
    this.setData({
      editorVisible: false,
      editingExperienceId: "",
      experienceForm: { ...EMPTY_FORM },
      error: "",
    });
  },

  noop() {},

  onFormInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | keyof ExperienceForm
      | undefined;
    if (!field) {
      return;
    }
    this.setData({
      [`experienceForm.${field}`]: event.detail.value,
      error: "",
    });
  },

  onStartDateChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      "experienceForm.startDate": String(event.detail.value || ""),
      error: "",
    });
  },

  onEndDateChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      "experienceForm.endDate": String(event.detail.value || ""),
      error: "",
    });
  },

  clearStartDate() {
    this.setData({ "experienceForm.startDate": "", error: "" });
  },

  clearEndDate() {
    this.setData({ "experienceForm.endDate": "", error: "" });
  },

  async saveExperience() {
    if (this.data.saving) {
      return;
    }
    if (!hasAccessToken()) {
      this.goLogin();
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const request = buildExperienceRequest(this.data.experienceForm);
    if (!request) {
      this.setData({ error: "请填写经历标题，天数和同行人数需为有效数字" });
      return;
    }
    this.setData({ saving: true, error: "" });
    try {
      if (this.data.editingExperienceId) {
        await updateOutdoorExperience(this.data.editingExperienceId, request);
      } else {
        await createOutdoorExperience(request);
      }
      wx.setStorageSync(EXPERIENCES_REFRESH_KEY, true);
      wx.showToast({
        title: this.data.editingExperienceId ? "经历已更新" : "经历已新增",
        icon: "success",
      });
      this.setData({
        editorVisible: false,
        editingExperienceId: "",
        experienceForm: { ...EMPTY_FORM },
      });
      await this.loadExperiences();
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ loggedIn: false });
        this.goLogin();
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ saving: false });
    }
  },

  confirmDeleteExperience(event: WechatMiniprogram.BaseEvent) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const id = String(event.currentTarget.dataset.id || "");
    const item = this.data.experiences.find(
      (experience) => experience.id === id,
    );
    if (!item) {
      return;
    }
    wx.showModal({
      title: "删除户外经历？",
      content: `会删除「${item.title}」这条经历记录，不影响原行程。`,
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await deleteOutdoorExperience(id);
          wx.setStorageSync(EXPERIENCES_REFRESH_KEY, true);
          wx.setStorageSync(TRIPS_REFRESH_KEY, true);
          wx.showToast({ title: "已删除", icon: "success" });
          await this.loadExperiences();
        } catch (error) {
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },
});

function mapExperienceCard(item: OutdoorExperience): OutdoorExperienceCard {
  const summaryLines = [
    { label: "路线", text: item.route_summary || "" },
    { label: "装备", text: item.gear_summary || "" },
    { label: "食品", text: item.food_summary || "" },
    { label: "预算", text: item.budget_summary || "" },
    { label: "备注", text: item.notes || "" },
  ].filter((line) => line.text);
  return {
    ...item,
    dateText: formatDateRange(item.start_date, item.end_date),
    metaText: formatMetaText(item),
    sourceText: item.source_trip_id ? "来自历史行程" : "手动记录",
    summaryLines,
  };
}

function formatDateRange(
  startDate: string | null | undefined,
  endDate: string | null | undefined,
): string {
  if (startDate && endDate && startDate !== endDate) {
    return `${startDate} 至 ${endDate}`;
  }
  return startDate || endDate || "未设置日期";
}

function formatMetaText(item: OutdoorExperience): string {
  const parts = [
    item.trip_type === "solo" ? "单人" : "多人",
    item.day_count ? `${item.day_count} 天` : "",
    typeof item.companion_count === "number"
      ? `同行 ${item.companion_count} 人`
      : "",
  ].filter(Boolean);
  return parts.join(" · ") || "经历记录";
}

function experienceToForm(item: OutdoorExperience): ExperienceForm {
  return {
    title: item.title || "",
    startDate: item.start_date || "",
    endDate: item.end_date || "",
    dayCount: typeof item.day_count === "number" ? String(item.day_count) : "",
    companionCount:
      typeof item.companion_count === "number"
        ? String(item.companion_count)
        : "",
    routeSummary: item.route_summary || "",
    gearSummary: item.gear_summary || "",
    foodSummary: item.food_summary || "",
    budgetSummary: item.budget_summary || "",
    notes: item.notes || "",
  };
}

function buildExperienceRequest(
  form: ExperienceForm,
): OutdoorExperienceRequest | null {
  const title = form.title.trim();
  if (!title) {
    return null;
  }
  const dayCount = parseOptionalInteger(form.dayCount, 366);
  const companionCount = parseOptionalInteger(form.companionCount, 999);
  if (dayCount === undefined || companionCount === undefined) {
    return null;
  }
  return {
    title,
    start_date: nullableText(form.startDate),
    end_date: nullableText(form.endDate),
    day_count: dayCount,
    companion_count: companionCount,
    route_summary: nullableText(form.routeSummary),
    gear_summary: nullableText(form.gearSummary),
    food_summary: nullableText(form.foodSummary),
    budget_summary: nullableText(form.budgetSummary),
    notes: nullableText(form.notes),
  };
}

function parseOptionalInteger(
  value: string,
  max: number,
): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  if (!/^\d+$/.test(trimmed)) {
    return undefined;
  }
  const numeric = Number(trimmed);
  return numeric >= 0 && numeric <= max ? numeric : undefined;
}

function nullableText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}
