import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  archiveGear,
  getErrorMessage,
  getGear,
  hasAccessToken,
  isLoginRequiredError,
  restoreGear,
} from "../../../utils/api";
import {
  formatDateText,
  formatGearPrice,
  formatGearWeight,
  getGearCategoryLabel,
  getGearShareStatusLabel,
  getGearStatusLabel,
  getStatusTone,
  valueOrUnset,
  type GearItem,
  type GearTab,
} from "../../../utils/gear-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../../utils/auth-prompt";

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
    tab: "available" as GearTab,
    item: null as GearItem | null,
    categoryText: "",
    statusText: "",
    statusTone: "",
    shareText: "",
    weightText: "未记录",
    priceText: "未记录",
    tagList: [] as string[],
    groups: [] as DetailGroup[],
    loading: false,
    requiresLogin: false,
    error: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    if (!id) {
      this.setData({ error: "缺少装备 ID" });
      return;
    }
    this.setData({ id, tab: (options.tab as GearTab) || "available" });
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
        message: "登录后可以查看你的个人装备详情。",
        redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
      });
      return;
    }
    this.setData({ loading: true, requiresLogin: false, error: "" });
    try {
      const item = await getGear(this.data.id);
      this.setData(buildDetailData(item));
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ requiresLogin: true, item: null, error: "" });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看装备详情。",
          redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  goEdit() {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以编辑和同步你的个人装备。",
        redirectUrl: `/pages/gears/form/index?id=${encodeURIComponent(this.data.id)}`,
      })
    ) {
      return;
    }
    wx.navigateTo({ url: `/pages/gears/form/index?id=${this.data.id}` });
  },

  archiveItem() {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以更新装备状态。",
        redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
      })
    ) {
      return;
    }
    wx.showModal({
      title: "移入历史装备？",
      content: "移入历史后不会出现在可用装备列表，可随时恢复。",
      confirmText: "移入历史",
      confirmColor: "#dc2626",
      success: async (result) => {
        if (!result.confirm) {
          return;
        }
        try {
          await archiveGear(this.data.id);
          wx.setStorageSync("stellartrail_gears_should_refresh", true);
          wx.showToast({ title: "已移入历史", icon: "success" });
          wx.navigateBack();
        } catch (error) {
          if (isLoginRequiredError(error)) {
            showLoginPrompt(this, {
              message: "登录状态已过期，请重新登录后更新装备状态。",
              redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
            });
            return;
          }
          wx.showToast({ title: getErrorMessage(error), icon: "none" });
        }
      },
    });
  },

  async restoreItem() {
    if (
      !requireLoginForAction(this, {
        message: "登录后可以恢复历史装备。",
        redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
      })
    ) {
      return;
    }
    try {
      await restoreGear(this.data.id);
      wx.setStorageSync("stellartrail_gears_should_refresh", true);
      wx.showToast({ title: "已恢复", icon: "success" });
      this.loadDetail();
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后恢复装备。",
          redirectUrl: `/pages/gears/detail/index?id=${encodeURIComponent(this.data.id)}&tab=${this.data.tab}`,
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function buildDetailData(item: GearItem) {
  const archived = Boolean(item.archived_at);
  return {
    item,
    requiresLogin: false,
    categoryText: getGearCategoryLabel(item.category),
    statusText: getGearStatusLabel(item.status),
    statusTone: getStatusTone(item.status),
    shareText: item.share_enabled
      ? `已开启 · ${getGearShareStatusLabel(item.share_status)}`
      : "未开启共享",
    weightText: formatGearWeight(item.weight_g),
    priceText: formatGearPrice(item.purchase_price_cents),
    tagList: item.tags ?? [],
    tab: (archived ? "history" : "available") as GearTab,
    groups: buildGroups(item),
  };
}

function buildGroups(item: GearItem): DetailGroup[] {
  return [
    {
      title: "基本信息",
      items: [
        { label: "分类", value: getGearCategoryLabel(item.category) },
        { label: "状态", value: getGearStatusLabel(item.status) },
        { label: "品牌", value: valueOrUnset(item.brand) },
        { label: "型号", value: valueOrUnset(item.model) },
        { label: "颜色", value: valueOrUnset(item.color) },
        { label: "材质", value: valueOrUnset(item.material) },
        { label: "容量", value: valueOrUnset(item.capacity) },
        { label: "尺寸", value: valueOrUnset(item.size) },
        { label: "描述", value: valueOrUnset(item.description) },
      ],
    },
    {
      title: "性能指标",
      items: [
        { label: "重量", value: formatGearWeight(item.weight_g) },
        { label: "保暖指数", value: valueOrUnset(item.warmth_index) },
        { label: "防水指数", value: valueOrUnset(item.waterproof_index) },
      ],
    },
    {
      title: "购买与存放",
      items: [
        { label: "购买日期", value: formatDateText(item.purchase_date) },
        {
          label: "购买价格",
          value: formatGearPrice(item.purchase_price_cents),
        },
        {
          label: "保修/到期",
          value: formatDateText(item.expiry_or_warranty_date),
        },
        { label: "购买渠道", value: valueOrUnset(item.purchase_location) },
        { label: "存放位置", value: valueOrUnset(item.storage_location) },
      ],
    },
    {
      title: "共享与备注",
      items: [
        {
          label: "共享状态",
          value: item.share_enabled
            ? `已开启 · ${getGearShareStatusLabel(item.share_status)}`
            : "未开启共享",
        },
        { label: "备注", value: valueOrUnset(item.notes) },
        { label: "创建时间", value: formatDateText(item.created_at) },
        { label: "更新时间", value: formatDateText(item.updated_at) },
      ],
    },
  ];
}
