import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  getGearAtlasItem,
  isOfflineCacheMissError,
} from "../../../utils/api-atlas";
import {
  formatDateText,
  formatGearPrice,
  formatGearWeight,
  getGearSpecFieldViews,
  type GearAtlasPublicItem,
} from "../../../utils/gear-utils";
import { getGearCategoryLabelForLocale } from "../../../utils/gear-display";
import { loadAppLocale, type AppLocale } from "../../../utils/locale";

interface DetailRow {
  label: string;
  value: string;
}

interface DetailGroup {
  title: string;
  items: DetailRow[];
}

interface AtlasDetailCopy {
  notFound: string;
  loading: string;
  empty: string;
  retry: string;
  status: string;
  brandModelUnset: string;
  unset: string;
  basicInfo: string;
  publicInfo: string;
  listingInfo: string;
  category: string;
  brand: string;
  model: string;
  description: string;
  weight: string;
  officialPrice: string;
  approvedAt: string;
  updatedAt: string;
}

const ATLAS_DETAIL_COPY: Record<AppLocale, AtlasDetailCopy> = {
  "zh-CN": {
    notFound: "没有找到这条内容，请返回后重试",
    loading: "正在加载图鉴详情...",
    empty: "暂无图鉴详情",
    retry: "重试",
    status: "已收录",
    brandModelUnset: "未填写品牌型号",
    unset: "未记录",
    basicInfo: "基本信息",
    publicInfo: "可公开信息",
    listingInfo: "收录信息",
    category: "分类",
    brand: "品牌",
    model: "型号",
    description: "描述",
    weight: "重量",
    officialPrice: "官方价格",
    approvedAt: "收录时间",
    updatedAt: "更新时间",
  },
  en: {
    notFound: "Could not find this gear entry. Please go back and try again.",
    loading: "Loading atlas detail...",
    empty: "No atlas detail available",
    retry: "Retry",
    status: "Listed",
    brandModelUnset: "Brand and model not set",
    unset: "Not recorded",
    basicInfo: "Basic Info",
    publicInfo: "Public Info",
    listingInfo: "Listing Info",
    category: "Category",
    brand: "Brand",
    model: "Model",
    description: "Description",
    weight: "Weight",
    officialPrice: "Official Price",
    approvedAt: "Listed At",
    updatedAt: "Updated At",
  },
};

const SPEC_LABELS_EN: Record<string, string> = {
  accessories: "Accessories",
  back_length: "Back Length",
  backpack_size: "Size",
  battery_capacity: "Battery Capacity",
  battery_type: "Battery Type",
  beam_distance: "Beam Distance",
  breathability_rating: "Breathability",
  capacity: "Capacity",
  certification: "Certification",
  charging_port: "Charging Port",
  days: "Days",
  expiry_date: "Expiry Date",
  filling: "Filling",
  fuel_type: "Fuel Type",
  kit_size: "Kit Size",
  layer: "Layer",
  length: "Length",
  material: "Material",
  max_brightness: "Max Brightness",
  net_content: "Net Content",
  output_power: "Output Power",
  packed_size: "Packed Size",
  people_count: "People",
  ports: "Ports",
  power: "Power",
  rated_energy: "Rated Energy",
  recommended_load: "Recommended Load",
  restock_threshold: "Restock Threshold",
  retirement_date: "Retirement Date",
  runtime: "Runtime",
  season: "Season",
  size: "Size",
  size_or_length: "Size / Length",
  specification: "Specification",
  storage_condition: "Storage Condition",
  strength: "Strength",
  support: "Cushion / Support",
  temperature_or_r_value: "Temperature / R-Value",
  terrain: "Terrain",
  type: "Type",
  use_case: "Use Case",
  waterproof_packaging: "Waterproof Packaging",
  waterproof_rating: "Water Resistance",
  warmth_rating: "Warmth Rating",
  working_temperature: "Working Temperature",
};

const initialLocale = loadAppLocale();

Page({
  data: {
    locale: initialLocale,
    copy: ATLAS_DETAIL_COPY[initialLocale],
    id: "",
    item: null as GearAtlasPublicItem | null,
    categoryText: "",
    brandModelText: "",
    weightText: ATLAS_DETAIL_COPY[initialLocale].unset,
    officialPriceText: ATLAS_DETAIL_COPY[initialLocale].unset,
    approvedAtText: ATLAS_DETAIL_COPY[initialLocale].unset,
    groups: [] as DetailGroup[],
    loading: false,
    error: "",
    offlineNotice: "",
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    if (!id) {
      this.setData({ error: this.data.copy.notFound });
      return;
    }
    this.setData({ id });
    this.loadDetail();
  },

  onShow() {
    syncPageTheme(this);
    const previousLocale = this.data.locale;
    const locale = loadAppLocale();
    this.setData({
      locale,
      copy: ATLAS_DETAIL_COPY[locale],
    });
    if (locale !== previousLocale && this.data.id) {
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
    this.setData({ loading: true, error: "" });
    try {
      const item = await getGearAtlasItem(this.data.id, this.data.locale);
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        ...buildDetailData(item, this.data.locale, this.data.copy),
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

function buildDetailData(
  item: GearAtlasPublicItem,
  locale: AppLocale,
  copy: AtlasDetailCopy,
) {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    item,
    categoryText:
      item.category_label ||
      getGearCategoryLabelForLocale(item.category, locale),
    brandModelText: brandModel || copy.brandModelUnset,
    weightText: localizeUnset(formatGearWeight(item.weight_g), copy),
    officialPriceText: localizeUnset(
      formatGearPrice(item.official_price_cents, item.official_price_currency),
      copy,
    ),
    approvedAtText: localizeUnset(formatDateText(item.approved_at), copy),
    groups: buildGroups(item, locale, copy),
  };
}

function buildGroups(
  item: GearAtlasPublicItem,
  locale: AppLocale,
  copy: AtlasDetailCopy,
): DetailGroup[] {
  const specs = getGearSpecFieldViews(item.category, item.specs ?? {})
    .map((field) => ({
      label:
        locale === "en" ? SPEC_LABELS_EN[field.key] || field.key : field.label,
      value: localizedValueOrUnset((item.specs ?? {})[field.key], copy),
    }))
    .filter((row) => row.value !== copy.unset);
  return [
    {
      title: copy.basicInfo,
      items: [
        {
          label: copy.category,
          value:
            item.category_label ||
            getGearCategoryLabelForLocale(item.category, locale),
        },
        { label: copy.brand, value: localizedValueOrUnset(item.brand, copy) },
        { label: copy.model, value: localizedValueOrUnset(item.model, copy) },
        {
          label: copy.description,
          value: localizedValueOrUnset(item.description, copy),
        },
      ],
    },
    {
      title: copy.publicInfo,
      items: [
        {
          label: copy.weight,
          value: localizeUnset(formatGearWeight(item.weight_g), copy),
        },
        {
          label: copy.officialPrice,
          value: localizeUnset(
            formatGearPrice(
              item.official_price_cents,
              item.official_price_currency,
            ),
            copy,
          ),
        },
        ...specs,
      ],
    },
    {
      title: copy.listingInfo,
      items: [
        {
          label: copy.approvedAt,
          value: localizeUnset(formatDateText(item.approved_at), copy),
        },
        {
          label: copy.updatedAt,
          value: localizeUnset(formatDateText(item.updated_at), copy),
        },
      ],
    },
  ];
}

function localizedValueOrUnset(
  value: string | number | null | undefined,
  copy: AtlasDetailCopy,
): string {
  if (value === undefined || value === null || value === "") {
    return copy.unset;
  }
  return String(value);
}

function localizeUnset(value: string, copy: AtlasDetailCopy): string {
  return value === ATLAS_DETAIL_COPY["zh-CN"].unset ? copy.unset : value;
}
