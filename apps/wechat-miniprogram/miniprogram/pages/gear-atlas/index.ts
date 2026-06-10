import { getThemeViewData, syncPageTheme } from "../../utils/theme";
import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  hasAccessToken,
  isOfflineCacheMissError,
  isNotFoundApiError,
  listGearAtlas,
} from "../../utils/api-atlas";
import {
  formatGearPrice,
  formatGearWeight,
  getGearCategoryLabelForLocale,
  getGearCategoryOptionsForLocale,
  type GearAtlasPublicItem,
  type GearCategory,
} from "../../utils/gear-display";
import { loadAppLocale, type AppLocale } from "../../utils/locale";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../utils/network-state";
import { indexedAppendData } from "../../utils/page-data";

interface AtlasCategoryChip {
  id: "all" | GearCategory;
  label: string;
}

interface AtlasCard {
  id: string;
  categoryText: string;
  name: string;
  brandModelText: string;
  descriptionText: string;
  weightText: string;
  officialPriceText: string;
}

interface AtlasListCopy {
  eyebrow: string;
  title: string;
  subtitle: string;
  submit: string;
  searchPlaceholder: string;
  search: string;
  clear: string;
  allCategory: string;
  unavailable: string;
  loading: string;
  emptyTitle: string;
  emptySubtitle: string;
  emptyAction: string;
  status: string;
  weight: string;
  officialPrice: string;
  loadingMore: string;
  noMore: string;
  brandModelUnset: string;
  descriptionUnset: string;
  unset: string;
  loginTitle: string;
  loginContent: string;
  loginConfirm: string;
}

const ATLAS_LIST_COPY: Record<AppLocale, AtlasListCopy> = {
  "zh-CN": {
    eyebrow: "装备图鉴",
    title: "装备图鉴",
    subtitle: "浏览已审核收录的市面装备，也可以投稿补充新装备。",
    submit: "投稿",
    searchPlaceholder: "搜索装备名、品牌、型号",
    search: "搜索",
    clear: "清除",
    allCategory: "全部",
    unavailable: "装备图鉴服务正在更新，请稍后重试。",
    loading: "正在加载装备图鉴...",
    emptyTitle: "还没有收录装备",
    emptySubtitle: "可以先提交一件装备，审核通过后会展示在这里。",
    emptyAction: "投稿装备",
    status: "已收录",
    weight: "重量",
    officialPrice: "官方价",
    loadingMore: "继续加载...",
    noMore: "没有更多装备了",
    brandModelUnset: "未填写品牌型号",
    descriptionUnset: "暂无描述",
    unset: "未记录",
    loginTitle: "登录后投稿",
    loginContent: "投稿新装备需要登录。",
    loginConfirm: "去登录",
  },
  en: {
    eyebrow: "Gear Atlas",
    title: "Gear Atlas",
    subtitle: "Browse approved public gear entries and submit new gear.",
    submit: "Submit",
    searchPlaceholder: "Search name, brand, or model",
    search: "Search",
    clear: "Clear",
    allCategory: "All",
    unavailable: "Gear atlas is updating. Please try again later.",
    loading: "Loading gear atlas...",
    emptyTitle: "No gear entries yet",
    emptySubtitle: "Submit a gear item and it will appear here after review.",
    emptyAction: "Submit Gear",
    status: "Listed",
    weight: "Weight",
    officialPrice: "Official Price",
    loadingMore: "Loading more...",
    noMore: "No more gear",
    brandModelUnset: "Brand and model not set",
    descriptionUnset: "No description yet",
    unset: "Not recorded",
    loginTitle: "Sign in to submit",
    loginContent: "Sign in before submitting new gear.",
    loginConfirm: "Sign in",
  },
};

let atlasListRequestSeq = 0;
const initialLocale = loadAppLocale();

Page({
  data: {
    locale: initialLocale,
    copy: ATLAS_LIST_COPY[initialLocale],
    categories: buildCategoryChips(initialLocale),
    selectedCategory: "all" as "all" | GearCategory,
    q: "",
    items: [] as AtlasCard[],
    nextCursor: null as string | null,
    loading: false,
    loadingMore: false,
    error: "",
    errorIsUnavailable: false,
    offlineNotice: "",
    isLoggedIn: hasAccessToken(),
    ...getThemeViewData(),
  },

  onLoad() {
    this.refreshPage();
  },

  onShow() {
    syncPageTheme(this);
    const previousLocale = this.data.locale;
    const locale = loadAppLocale();
    this.setData({
      isLoggedIn: hasAccessToken(),
      locale,
      copy: ATLAS_LIST_COPY[locale],
      categories: buildCategoryChips(locale),
    });
    if (locale !== previousLocale) {
      this.setData({ nextCursor: null, items: [] });
      this.refreshPage();
    }
  },

  onPullDownRefresh() {
    this.refreshPage().finally(() => wx.stopPullDownRefresh());
  },

  onReachBottom() {
    this.loadMore();
  },

  async refreshPage() {
    const requestSeq = ++atlasListRequestSeq;
    this.setData({
      loading: true,
      loadingMore: false,
      error: "",
      errorIsUnavailable: false,
    });
    try {
      const response = await listGearAtlas(
        this.buildRequest(null),
        this.data.locale,
      );
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        items: response.items.map((item) =>
          mapAtlasCard(item, this.data.locale, this.data.copy),
        ),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      if (isOfflineCacheMissError(error) && this.data.items.length) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        error: atlasErrorMessage(error),
        errorIsUnavailable: isNotFoundApiError(error),
        items: [],
      });
    } finally {
      if (requestSeq === atlasListRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  async loadMore() {
    if (!this.data.nextCursor || this.data.loadingMore || this.data.loading) {
      return;
    }
    this.setData({ loadingMore: true, error: "", errorIsUnavailable: false });
    const requestSeq = atlasListRequestSeq;
    try {
      const response = await listGearAtlas(
        this.buildRequest(this.data.nextCursor),
        this.data.locale,
      );
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      const cards = response.items.map((item) =>
        mapAtlasCard(item, this.data.locale, this.data.copy),
      );
      this.setData({
        ...indexedAppendData("items", this.data.items.length, cards),
        nextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      if (isOfflineCacheMissError(error)) {
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        error: atlasErrorMessage(error),
        errorIsUnavailable: isNotFoundApiError(error),
      });
    } finally {
      if (requestSeq === atlasListRequestSeq) {
        this.setData({ loadingMore: false });
      }
    }
  },

  buildRequest(cursor: string | null) {
    const selectedCategory = this.data.selectedCategory;
    return {
      category: selectedCategory === "all" ? undefined : selectedCategory,
      q: this.data.q.trim() || undefined,
      sort: "approved_at_desc" as const,
      limit: 20,
      cursor: cursor ?? undefined,
    };
  },

  selectCategory(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as "all" | GearCategory;
    this.setData({ selectedCategory: id, nextCursor: null, items: [] });
    this.refreshPage();
  },

  onSearchInput(event: WechatMiniprogram.BaseEvent) {
    this.setData({ q: (event as any).detail.value });
  },

  submitSearch() {
    this.setData({ nextCursor: null, items: [] });
    this.refreshPage();
  },

  clearSearch() {
    this.setData({ q: "", nextCursor: null, items: [] });
    this.refreshPage();
  },

  goSubmit() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (!hasAccessToken()) {
      wx.showModal({
        title: this.data.copy.loginTitle,
        content: this.data.copy.loginContent,
        confirmText: this.data.copy.loginConfirm,
        confirmColor: "#0f766e",
        success: (result) => {
          if (result.confirm) {
            wx.navigateTo({
              url: "/pages/login/index?redirect=%2Fpages%2Fgear-atlas%2Fsubmit%2Findex",
            });
          }
        },
      });
      return;
    }
    wx.navigateTo({ url: "/pages/gear-atlas/submit/index" });
  },

  goDetail(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (id) {
      wx.navigateTo({
        url: `/pages/gear-atlas/detail/index?id=${encodeURIComponent(id)}`,
      });
    }
  },
});

function atlasErrorMessage(error: unknown): string {
  if (isNotFoundApiError(error)) {
    return ATLAS_LIST_COPY[loadAppLocale()].unavailable;
  }
  return getErrorMessage(error);
}

function mapAtlasCard(
  item: GearAtlasPublicItem,
  locale: AppLocale,
  copy: AtlasListCopy,
): AtlasCard {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    id: item.id,
    categoryText:
      item.category_label ||
      getGearCategoryLabelForLocale(item.category, locale),
    name: item.name,
    brandModelText: brandModel || copy.brandModelUnset,
    descriptionText: item.description || copy.descriptionUnset,
    weightText: localizeUnset(formatGearWeight(item.weight_g), copy),
    officialPriceText: localizeUnset(
      formatGearPrice(item.official_price_cents, item.official_price_currency),
      copy,
    ),
  };
}

function buildCategoryChips(locale: AppLocale): AtlasCategoryChip[] {
  return [
    { id: "all", label: ATLAS_LIST_COPY[locale].allCategory },
    ...getGearCategoryOptionsForLocale(locale).map((item) => ({
      id: item.value,
      label: item.label,
    })),
  ];
}

function localizeUnset(value: string, copy: AtlasListCopy): string {
  return value === ATLAS_LIST_COPY["zh-CN"].unset ? copy.unset : value;
}
