import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  getGearAtlasItem,
  isOfflineCacheMissError,
} from "../../../utils/api";
import {
  formatDateText,
  formatGearPrice,
  formatGearWeight,
  getGearCategoryLabel,
  getGearSpecFieldViews,
  valueOrUnset,
  type GearAtlasPublicItem,
} from "../../../utils/gear-utils";

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
    item: null as GearAtlasPublicItem | null,
    categoryText: "",
    brandModelText: "",
    weightText: "未记录",
    officialPriceText: "未记录",
    approvedAtText: "未记录",
    groups: [] as DetailGroup[],
    loading: false,
    error: "",
    offlineNotice: "",
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    if (!id) {
      this.setData({ error: "缺少图鉴 ID" });
      return;
    }
    this.setData({ id });
    this.loadDetail();
  },

  onShow() {
    syncPageTheme(this);
  },

  onPullDownRefresh() {
    this.loadDetail().finally(() => wx.stopPullDownRefresh());
  },

  async loadDetail() {
    if (!this.data.id) {
      return;
    }
    this.setData({ loading: true, error: "" });
    try {
      const item = await getGearAtlasItem(this.data.id);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...buildDetailData(item),
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isOfflineCacheMissError(error) && this.data.item) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({ error: getErrorMessage(error), item: null });
    } finally {
      this.setData({ loading: false });
    }
  },
});

function buildDetailData(item: GearAtlasPublicItem) {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    item,
    categoryText: item.category_label || getGearCategoryLabel(item.category),
    brandModelText: brandModel || "未填写品牌型号",
    weightText: formatGearWeight(item.weight_g),
    officialPriceText: formatGearPrice(
      item.official_price_cents,
      item.official_price_currency,
    ),
    approvedAtText: formatDateText(item.approved_at),
    groups: buildGroups(item),
  };
}

function buildGroups(item: GearAtlasPublicItem): DetailGroup[] {
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
        {
          label: "分类",
          value: item.category_label || getGearCategoryLabel(item.category),
        },
        { label: "品牌", value: valueOrUnset(item.brand) },
        { label: "型号", value: valueOrUnset(item.model) },
        { label: "描述", value: valueOrUnset(item.description) },
      ],
    },
    {
      title: "公开参数",
      items: [
        { label: "重量", value: formatGearWeight(item.weight_g) },
        {
          label: "官方价格",
          value: formatGearPrice(
            item.official_price_cents,
            item.official_price_currency,
          ),
        },
        ...specs,
      ],
    },
    {
      title: "收录信息",
      items: [
        { label: "收录时间", value: formatDateText(item.approved_at) },
        { label: "更新时间", value: formatDateText(item.updated_at) },
      ],
    },
  ];
}
