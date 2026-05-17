import { getErrorMessage, getKnotDetail, resolveAssetUrl } from "../../../utils/api";
import {
  getSkillDifficultyLabel,
  getSkillDifficultyTone,
  type KnotDetail,
  type KnotMediaAsset,
} from "../../../utils/skill-utils";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";

interface MediaView extends KnotMediaAsset {
  resolvedUrl: string;
  mediaTypeText: string;
}

Page({
  data: {
    id: "",
    detail: null as KnotDetail | null,
    categoriesText: "绳结",
    typesText: "",
    difficultyText: "未分级",
    difficultyTone: "success",
    media: [] as MediaView[],
    steps: [] as string[],
    loading: false,
    error: "",
    ...getThemeViewData(),
  },

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id;
    if (!id) {
      this.setData({ error: "缺少绳结 ID" });
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
      const detail = await getKnotDetail(this.data.id);
      wx.setNavigationBarTitle({ title: detail.title });
      this.setData({
        detail,
        categoriesText: detail.categories.map((item) => item.title).join(" · ") || "绳结",
        typesText: detail.types.map((item) => item.title).join(" · "),
        difficultyText: getSkillDifficultyLabel(detail.difficulty),
        difficultyTone: getSkillDifficultyTone(detail.difficulty),
        media: detail.media.map(mapMedia),
        steps: detail.steps,
        loading: false,
      });
    } catch (error) {
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        detail: null as KnotDetail | null,
      });
    }
  },
});

function mapMedia(item: KnotMediaAsset): MediaView {
  return {
    ...item,
    resolvedUrl: resolveAssetUrl(item.url),
    mediaTypeText: item.media_type.toUpperCase(),
  };
}
