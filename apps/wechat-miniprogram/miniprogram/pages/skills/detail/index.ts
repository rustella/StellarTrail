import {
  consumeOfflineCacheNotice,
  favoriteKnot,
  getErrorMessage,
  getFavoriteKnotStatus,
  getKnotDetail,
  hasAccessToken,
  isOfflineCacheMissError,
  resolveAssetUrl,
  unfavoriteKnot,
} from "../../../utils/api-skills";
import {
  formatKnotAliasText,
  type KnotDetail,
  type KnotMediaAsset,
} from "../../../utils/skill-utils";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import { resolveCachedMediaUrl } from "../../../utils/media-cache";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
} from "../../../utils/auth-prompt";
import { isOffline, showOfflineWriteBlockedToast } from "../../../utils/network-state";

interface MediaView extends KnotMediaAsset {
  resolvedUrl: string;
  mediaTypeText: string;
  helpText: string;
  icon: string;
}

Page({
  data: {
    id: "",
    detail: null as KnotDetail | null,
    detailAliasText: "",
    detailTags: [] as string[],
    media: [] as MediaView[],
    activeMediaIndex: 0,
    activeMedia: null as MediaView | null,
    mediaCredit: "系法动图 · Knots 3D",
    mediaHelpText: "自动循环演示打结步骤。",
    loading: false,
    isFavorited: false,
    favoriteLoading: false,
    favoritedAt: "",
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
      const media = await Promise.all(
        detail.media.filter(isDetailMediaAsset).map(mapMedia),
      );
      const activeMediaIndex = preferredMediaIndex(media);
      const activeMedia = media[activeMediaIndex] ?? null;
      const favoriteStatus = await loadFavoriteStatus(detail.id);
      const offlineNotice = consumeOfflineCacheNotice();
      wx.setNavigationBarTitle({ title: detail.title });
      this.setData({
        detail,
        detailAliasText: formatKnotAliasText(detail.aliases),
        detailTags: detailTags(detail),
        media,
        activeMediaIndex,
        activeMedia,
        mediaCredit: mediaCredit(activeMedia),
        mediaHelpText: activeMedia?.helpText ?? "",
        isFavorited: favoriteStatus.isFavorited,
        favoriteLoading: false,
        favoritedAt: favoriteStatus.favoritedAt,
        loading: false,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isOfflineCacheMissError(error) && this.data.detail) {
        this.setData({ loading: false });
        wx.showToast({ title: getErrorMessage(error), icon: "none" });
        return;
      }
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        detail: null as KnotDetail | null,
        detailAliasText: "",
      });
    }
  },

  selectMedia(event: WechatMiniprogram.BaseEvent) {
    const index = Number(event.currentTarget.dataset.index);
    const activeMedia = this.data.media[index];
    if (!activeMedia) {
      return;
    }
    this.setData({
      activeMediaIndex: index,
      activeMedia,
      mediaCredit: mediaCredit(activeMedia),
      mediaHelpText: activeMedia.helpText,
    });
  },

  async toggleFavorite() {
    const id = this.data.detail?.id ?? this.data.id;
    if (!id) {
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以收藏技能，并在收藏清单里快速找到。",
        redirectUrl: `/pages/skills/detail/index?id=${encodeURIComponent(id)}`,
      })
    ) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const previousFavorited = this.data.isFavorited;
    const previousFavoritedAt = this.data.favoritedAt;
    this.setData({ favoriteLoading: true });
    try {
      const status = previousFavorited
        ? await unfavoriteKnot(id)
        : await favoriteKnot(id);
      this.setData({
        isFavorited: status.is_favorited,
        favoritedAt: status.favorited_at ?? "",
        favoriteLoading: false,
      });
    } catch (error) {
      this.setData({
        isFavorited: previousFavorited,
        favoritedAt: previousFavoritedAt,
        favoriteLoading: false,
      });
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

async function loadFavoriteStatus(id: string): Promise<{
  isFavorited: boolean;
  favoritedAt: string;
}> {
  if (!hasAccessToken()) {
    return { isFavorited: false, favoritedAt: "" };
  }
  try {
    const status = await getFavoriteKnotStatus(id);
    return {
      isFavorited: status.is_favorited,
      favoritedAt: status.favorited_at ?? "",
    };
  } catch {
    return { isFavorited: false, favoritedAt: "" };
  }
}

function isDetailMediaAsset(item: KnotMediaAsset): boolean {
  return (
    item.media_type === "preview" ||
    item.media_type === "draw_gif" ||
    item.media_type === "turntable_gif"
  );
}

async function mapMedia(item: KnotMediaAsset): Promise<MediaView> {
  const meta = mediaMeta(item.media_type);
  return {
    ...item,
    resolvedUrl: await resolveCachedMediaUrl(resolveAssetUrl(item.url)),
    mediaTypeText: meta.label,
    helpText: meta.helpText,
    icon: meta.icon,
  };
}

function mediaMeta(mediaType: string): {
  label: string;
  icon: string;
  helpText: string;
} {
  if (mediaType === "preview") {
    return { label: "静态图", icon: "◉", helpText: "查看绳结的清晰定格图。" };
  }
  if (mediaType === "draw_gif") {
    return { label: "系法动图", icon: "▷", helpText: "自动循环演示打结步骤。" };
  }
  if (mediaType === "turntable_gif") {
    return {
      label: "旋转动图",
      icon: "◎",
      helpText: "自动循环展示绳结结构的旋转动图。",
    };
  }
  return { label: "动图", icon: "•", helpText: "查看绳结动图。" };
}

function preferredMediaIndex(media: MediaView[]): number {
  const drawGifIndex = media.findIndex((item) => item.media_type === "draw_gif");
  if (drawGifIndex >= 0) {
    return drawGifIndex;
  }
  return 0;
}

function detailTags(detail: KnotDetail): string[] {
  const tags = [
    ...detail.categories.map((item) => item.title),
    ...detail.types.map((item) => item.title),
  ];
  return tags.filter((item, index) => item && tags.indexOf(item) === index);
}

function mediaCredit(media: MediaView | null): string {
  if (!media) {
    return "演示素材 · Knots 3D";
  }
  return `${media.mediaTypeText} · ${media.attribution || "Knots 3D"}`;
}
