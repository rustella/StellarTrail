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
  GEAR_CATEGORY_OPTIONS,
  getGearCategoryLabel,
  type GearAtlasPublicItem,
  type GearCategory,
} from "../../utils/gear-display";
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

const CATEGORY_CHIPS: AtlasCategoryChip[] = [
  { id: "all", label: "全部" },
  ...GEAR_CATEGORY_OPTIONS.map((item) => ({
    id: item.value,
    label: item.label,
  })),
];
let atlasListRequestSeq = 0;

Page({
  data: {
    categories: CATEGORY_CHIPS,
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
    this.setData({ isLoggedIn: hasAccessToken() });
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
      const response = await listGearAtlas(this.buildRequest(null));
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      this.setData({
        items: response.items.map(mapAtlasCard),
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
      );
      if (requestSeq !== atlasListRequestSeq) {
        return;
      }
      const offlineNotice = consumeOfflineCacheNotice();
      const cards = response.items.map(mapAtlasCard);
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
        title: "登录后投稿",
        content: "装备图鉴可以先浏览；投稿新装备需要登录。",
        confirmText: "去登录",
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
    return "装备图鉴服务正在更新，请稍后重试。";
  }
  return getErrorMessage(error);
}

function mapAtlasCard(item: GearAtlasPublicItem): AtlasCard {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    id: item.id,
    categoryText: item.category_label || getGearCategoryLabel(item.category),
    name: item.name,
    brandModelText: brandModel || "未填写品牌型号",
    descriptionText: item.description || "暂无描述",
    weightText: formatGearWeight(item.weight_g),
    officialPriceText: formatGearPrice(
      item.official_price_cents,
      item.official_price_currency,
    ),
  };
}
