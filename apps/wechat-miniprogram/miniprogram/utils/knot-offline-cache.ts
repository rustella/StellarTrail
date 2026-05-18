import {
  getKnotOfflineManifest,
  knotDetailPath,
  knotListPath,
  resolveAssetUrl,
} from "./api";
import { cacheMediaUrlForOffline } from "./media-cache";
import { makeOfflineCacheKey, writeOfflineCache } from "./offline-cache";
import type {
  KnotDetail,
  KnotListResponse,
  KnotMediaAsset,
  KnotSummary,
  SkillLocale,
} from "./skill-utils";

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
  items: KnotDetail[];
  detailCount: number;
  mediaReadyCount: number;
  mediaTotal: number;
  estimatedBytes: number;
  failedDetailCount: number;
  failedMediaCount: number;
}

export interface KnotOfflineCachePlan {
  locale: SkillLocale;
  items: KnotDetail[];
  mediaUrls: string[];
  detailCount: number;
  mediaTotal: number;
  estimatedBytes: number;
  failedDetailCount: number;
}

interface CacheAllKnotsForOfflineOptions {
  locale?: SkillLocale;
  pageSize?: number;
  plan?: KnotOfflineCachePlan;
  onProgress?: (progress: KnotOfflineCacheProgress) => void;
}

const DEFAULT_PAGE_SIZE = 10;

export async function cacheAllKnotsForOffline(
  options: CacheAllKnotsForOfflineOptions = {},
): Promise<KnotOfflineCacheResult> {
  const plan = options.plan ?? (await prepareAllKnotsOfflineCache(options));
  const urls = plan.mediaUrls;
  writeKnotOfflineApiCaches(
    plan.items,
    plan.locale,
    options.pageSize ?? DEFAULT_PAGE_SIZE,
  );
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
    items: plan.items,
    detailCount: plan.detailCount,
    mediaReadyCount,
    mediaTotal: urls.length,
    estimatedBytes: plan.estimatedBytes,
    failedDetailCount: plan.failedDetailCount,
    failedMediaCount,
  };
}

export async function prepareAllKnotsOfflineCache(
  options: CacheAllKnotsForOfflineOptions = {},
): Promise<KnotOfflineCachePlan> {
  const locale = options.locale ?? "zh-CN";
  const manifest = await getKnotOfflineManifest(locale);
  const mediaByUrl = new Map<string, number>();
  manifest.items.forEach((item) => collectMediaUrls(item.media, mediaByUrl));
  options.onProgress?.({
    phase: "list",
    loadedCount: manifest.item_count,
    totalCount: manifest.item_count,
  });

  const mediaUrls = Array.from(mediaByUrl.keys());

  return {
    locale,
    items: manifest.items,
    detailCount: manifest.item_count,
    mediaUrls,
    mediaTotal: manifest.media_count,
    estimatedBytes: manifest.estimated_bytes || sumMediaBytes(mediaByUrl),
    failedDetailCount: 0,
  };
}

function writeKnotOfflineApiCaches(
  items: KnotDetail[],
  locale: SkillLocale,
  pageSize: number,
): void {
  items.forEach((item) => {
    writePublicOfflineCache(knotDetailPath(item.id), locale, item);
  });

  const effectivePageSize = Math.max(1, pageSize || DEFAULT_PAGE_SIZE);
  if (!items.length) {
    writeKnotListCache(locale, effectivePageSize, 0, []);
    return;
  }
  for (let offset = 0; offset < items.length; offset += effectivePageSize) {
    writeKnotListCache(
      locale,
      effectivePageSize,
      offset,
      items.slice(offset, offset + effectivePageSize).map(toKnotSummary),
      offset + effectivePageSize < items.length
        ? offset + effectivePageSize
        : null,
    );
  }
}

function writeKnotListCache(
  locale: SkillLocale,
  limit: number,
  offset: number,
  items: KnotSummary[],
  nextOffset: number | null = null,
): void {
  const payload: KnotListResponse = {
    locale,
    items,
    page: {
      limit,
      offset,
      next_offset: nextOffset,
    },
  };
  writePublicOfflineCache(knotListPath({ offset, limit }), locale, payload);
}

function writePublicOfflineCache<T>(
  path: string,
  locale: SkillLocale,
  data: T,
): void {
  writeOfflineCache(
    {
      key: makeOfflineCacheKey(path, { locale }),
      scope: "public",
      locale,
    },
    data,
  );
}

function toKnotSummary(item: KnotDetail): KnotSummary {
  return {
    id: item.id,
    slug: item.slug,
    title: item.title,
    summary: item.summary,
    difficulty: item.difficulty,
    categories: item.categories,
    types: item.types,
    media: item.media,
    href: item.href || knotDetailPath(item.id),
  };
}

function collectMediaUrls(
  media: KnotMediaAsset[],
  urls: Map<string, number>,
): void {
  media.forEach((item) => {
    if (item.url) {
      const url = resolveAssetUrl(item.url);
      const current = urls.get(url) ?? 0;
      urls.set(url, Math.max(current, item.size_bytes || 0));
    }
  });
}

function sumMediaBytes(mediaByUrl: Map<string, number>): number {
  let total = 0;
  mediaByUrl.forEach((sizeBytes) => {
    total += sizeBytes;
  });
  return total;
}
