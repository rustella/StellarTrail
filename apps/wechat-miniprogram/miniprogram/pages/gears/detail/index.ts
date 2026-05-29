import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  deleteGear,
  getErrorMessage,
  getGear,
  hasAccessToken,
  isOfflineCacheMissError,
  isLoginRequiredError,
  listMyGearAtlasSubmissions,
  submitGearToAtlas,
} from "../../../utils/api-gears";
import {
  createGearTagViews,
  formatDateText,
  formatGearQuantity,
  formatGearPrice,
  formatGearWeight,
  getGearAtlasStatusLabel,
  getGearCategoryLabel,
  getGearSpecFieldViews,
  getGearStatusLabel,
  getStatusTone,
  valueOrUnset,
  type GearAtlasStatus,
  type GearAtlasSubmission,
  type GearItem,
  type GearTagView,
} from "../../../utils/gear-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";

interface DetailRow {
  label: string;
  value: string;
}

interface DetailGroup {
  title: string;
  items: DetailRow[];
}

Page({
  data: {
    id: "",
    item: null as GearItem | null,
    categoryText: "",
    statusText: "",
    statusTone: "",
    atlasSubmissionStatus: "" as "" | GearAtlasStatus,
    atlasSubmissionText: "未投稿",
    atlasSubmissionHint:
      "投稿只复制可公开信息，不包含购入价、购买渠道、存放位置和备注。",
    submittingAtlas: false,
    weightText: "未记录",
    quantityText: "x1",
    priceText: "未记录",
    tagViews: [] as GearTagView[],
    groups: [] as DetailGroup[],
    loading: false,
    requiresLogin: false,
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    if (!id) {
      this.setData({ error: "没有找到这条内容，请返回后重试" });
      return;
    }
    this.setData({ id });
    this.loadDetail();
  },

  onShow() {
    syncPageTheme(this);
    if (this.data.requiresLogin && hasAccessToken()) {
      this.loadDetail();
    }
  },

  onPullDownRefresh() {
    this.loadDetail().finally(() => wx.stopPullDownRefresh());
  },

  async loadDetail() {
    if (!this.data.id) {
      return;
    }
    if (!hasAccessToken()) {
      this.setData({
        requiresLogin: true,
        loading: false,
        error: "",
        item: null,
      });
      showLoginPrompt(this, {
        message: "登录后可以查看自己的装备详情。",
        redirectUrl: detailUrl(this.data.id),
      });
      return;
    }
    this.setData({ loading: true, requiresLogin: false, error: "" });
    try {
      const item = await getGear(this.data.id);
      const submission = await findAtlasSubmission(item.id);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...buildDetailData(item, submission),
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ requiresLogin: true, item: null, error: "" });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看装备详情。",
          redirectUrl: detailUrl(this.data.id),
        });
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.item) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  goEdit() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以编辑自己的装备。",
        redirectUrl: `/pages/gears/form/index?id=${encodeURIComponent(this.data.id)}`,
      })
    ) {
      return;
    }
    wx.navigateTo({ url: `/pages/gears/form/index?id=${this.data.id}` });
  },

  deleteItem() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以删除自己的装备。",
        redirectUrl: detailUrl(this.data.id),
      })
    ) {
      return;
    }
    wx.showModal({
      title: "删除这件装备？",
      content: "删除后不会出现在装备列表中，已有打包清单会保留历史条目。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await deleteGear(this.data.id);
          wx.setStorageSync("stellartrail_gears_should_refresh", true);
          wx.showToast({ title: "已删除", icon: "success" });
          if (getCurrentPages().length > 1) {
            wx.navigateBack();
          } else {
            wx.switchTab({ url: "/pages/gears/index" });
          }
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后删除装备。",
              redirectUrl: detailUrl(this.data.id),
            });
            return;
          }
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },

  submitToAtlas() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以把自己的装备投稿到装备图鉴。",
        redirectUrl: detailUrl(this.data.id),
      })
    ) {
      return;
    }
    if (
      this.data.submittingAtlas ||
      this.data.atlasSubmissionStatus === "pending" ||
      this.data.atlasSubmissionStatus === "approved"
    ) {
      return;
    }
    wx.showModal({
      title: "投稿到装备图鉴？",
      content:
        "只会复制分类、名称、品牌、型号、描述、重量、官方价和详细信息，不包含购入价、购买渠道、存放位置、备注、标签等个人信息。",
      confirmText: "提交审核",
      confirmColor: "#0f766e",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        this.setData({ submittingAtlas: true });
        try {
          const submission = await submitGearToAtlas(this.data.id);
          this.setData(buildAtlasSubmissionData(submission));
          wx.showToast({ title: "已提交审核", icon: "success" });
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后投稿装备。",
              redirectUrl: detailUrl(this.data.id),
            });
            return;
          }
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        } finally {
          this.setData({ submittingAtlas: false });
        }
      },
    });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

async function findAtlasSubmission(
  gearId: string,
): Promise<GearAtlasSubmission | null> {
  try {
    const response = await listMyGearAtlasSubmissions({ limit: 100 });
    return (
      response.items.find((item) => item.source_user_gear_id === gearId) ?? null
    );
  } catch {
    return null;
  }
}

function buildDetailData(
  item: GearItem,
  submission: GearAtlasSubmission | null,
) {
  return {
    item,
    requiresLogin: false,
    categoryText: getGearCategoryLabel(item.category),
    statusText: getGearStatusLabel(item.status),
    statusTone: getStatusTone(item.status),
    ...buildAtlasSubmissionData(submission),
    weightText: formatGearWeight(item.weight_g),
    quantityText: formatGearQuantity(item.quantity),
    priceText: formatGearPrice(
      item.purchase_price_cents,
      item.purchase_price_currency,
    ),
    tagViews: createGearTagViews(item.tags ?? [], item.tag_colors ?? {}),
    groups: buildGroups(item),
  };
}

function detailUrl(id: string): string {
  return `/pages/gears/detail/index?id=${encodeURIComponent(id)}`;
}

function buildAtlasSubmissionData(submission: GearAtlasSubmission | null) {
  if (!submission) {
    return {
      atlasSubmissionStatus: "" as "" | GearAtlasStatus,
      atlasSubmissionText: "未投稿",
      atlasSubmissionHint:
        "投稿只复制可公开信息，不包含购入价、购买渠道、存放位置和备注。",
    };
  }
  const hint =
    submission.status === "rejected" && submission.rejection_reason
      ? submission.rejection_reason
      : submission.status === "approved"
        ? formatApprovedAtlasSubmissionHint(submission)
        : "已提交审核，审核通过后会出现在装备图鉴。";
  return {
    atlasSubmissionStatus: submission.status,
    atlasSubmissionText: getGearAtlasStatusLabel(submission.status),
    atlasSubmissionHint: hint,
  };
}

function formatApprovedAtlasSubmissionHint(
  submission: GearAtlasSubmission,
): string {
  if (!submission.review_changes?.length) {
    return "审核通过后已出现在装备图鉴。";
  }
  const summary = submission.review_changes
    .map(
      (change) =>
        `${change.label} 从 ${change.before || "—"} 改为 ${
          change.after || "—"
        }`,
    )
    .join("；");
  return `审核通过，管理员调整：${summary}`;
}

function buildGroups(item: GearItem): DetailGroup[] {
  const specs = getGearSpecFieldViews(item.category, item.specs ?? {})
    .map((field) => ({
      label: field.label,
      value: valueOrUnset((item.specs ?? {})[field.key]),
    }))
    .filter((row) => row.value !== "未记录");
  return [
    {
      title: "基本信息",
      items: [
        { label: "分类", value: getGearCategoryLabel(item.category) },
        { label: "状态", value: getGearStatusLabel(item.status) },
        { label: "数量", value: formatGearQuantity(item.quantity) },
        { label: "品牌", value: valueOrUnset(item.brand) },
        { label: "型号", value: valueOrUnset(item.model) },
        { label: "描述", value: valueOrUnset(item.description) },
      ],
    },
    {
      title: "重量与详细信息",
      items: [
        { label: "重量", value: formatGearWeight(item.weight_g) },
        ...specs,
      ],
    },
    {
      title: "购买与存放",
      items: [
        { label: "购买日期", value: formatDateText(item.purchase_date) },
        {
          label: "官方价格",
          value: formatGearPrice(
            item.official_price_cents,
            item.official_price_currency,
          ),
        },
        {
          label: "购入价格",
          value: formatGearPrice(
            item.purchase_price_cents,
            item.purchase_price_currency,
          ),
        },
        { label: "购买渠道", value: valueOrUnset(item.purchase_location) },
        { label: "存放位置", value: valueOrUnset(item.storage_location) },
      ],
    },
    {
      title: "备注",
      items: [
        { label: "备注", value: valueOrUnset(item.notes) },
        { label: "创建时间", value: formatDateText(item.created_at) },
        { label: "更新时间", value: formatDateText(item.updated_at) },
      ],
    },
  ];
}
