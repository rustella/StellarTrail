import { getKnotDetail, listKnots, resolveAssetUrl } from "./api";
import { cacheMediaUrlForOffline } from "./media-cache";
import type { KnotMediaAsset, KnotSummary, SkillLocale } from "./skill-utils";

export type KnotOfflineCachePhase = "list" | "detail" | "media";

export interface KnotOfflineCacheProgress {
  phase: KnotOfflineCachePhase;
  loadedCount?: number;
  totalCount?: number;
  currentIndex?: number;
  currentTitle?: string;
  mediaReadyCount?: number;
  mediaTotal?: number;
}

export interface KnotOfflineCacheResult {
  items: KnotSummary[];
  detailCount: number;
  mediaReadyCount: number;
  mediaTotal: number;
  failedDetailCount: number;
  failedMediaCount: number;
}

interface CacheAllKnotsForOfflineOptions {
  locale?: SkillLocale;
  pageSize?: number;
  onProgress?: (progress: KnotOfflineCacheProgress) => void;
}

const DEFAULT_PAGE_SIZE = 10;

export async function cacheAllKnotsForOffline(
  options: CacheAllKnotsForOfflineOptions = {},
): Promise<KnotOfflineCacheResult> {
  const locale = options.locale ?? "zh-CN";
  const pageSize = options.pageSize ?? DEFAULT_PAGE_SIZE;
  const itemById = new Map<string, KnotSummary>();
  const mediaUrls = new Set<string>();
  let offset = 0;

  for (;;) {
    const response = await listKnots({ offset, limit: pageSize }, locale);
    response.items.forEach((item) => {
      if (!itemById.has(item.id)) {
        itemById.set(item.id, item);
      }
      collectMediaUrls(item.media, mediaUrls);
    });
    options.onProgress?.({
      phase: "list",
      loadedCount: itemById.size,
    });
    if (response.page.next_offset == null) {
      break;
    }
    offset = response.page.next_offset;
  }

  const items = Array.from(itemById.values());
  let detailCount = 0;
  let failedDetailCount = 0;
  for (let index = 0; index < items.length; index += 1) {
    const item = items[index];
    options.onProgress?.({
      phase: "detail",
      currentIndex: index + 1,
      totalCount: items.length,
      currentTitle: item.title,
    });
    try {
      const detail = await getKnotDetail(item.id, locale);
      detailCount += 1;
      collectMediaUrls(detail.media, mediaUrls);
    } catch {
      failedDetailCount += 1;
    }
  }

  const urls = Array.from(mediaUrls);
  let mediaReadyCount = 0;
  let failedMediaCount = 0;
  for (let index = 0; index < urls.length; index += 1) {
    options.onProgress?.({
      phase: "media",
      mediaReadyCount,
      mediaTotal: urls.length,
    });
    try {
      await cacheMediaUrlForOffline(urls[index]);
      mediaReadyCount += 1;
    } catch {
      failedMediaCount += 1;
    }
  }
  options.onProgress?.({
    phase: "media",
    mediaReadyCount,
    mediaTotal: urls.length,
  });

  return {
    items,
    detailCount,
    mediaReadyCount,
    mediaTotal: urls.length,
    failedDetailCount,
    failedMediaCount,
  };
}

function collectMediaUrls(media: KnotMediaAsset[], urls: Set<string>): void {
  media.forEach((item) => {
    if (item.url) {
      urls.add(resolveAssetUrl(item.url));
    }
  });
}
