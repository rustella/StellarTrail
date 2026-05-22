import {
  getKnotOfflineManifest,
  knotDetailPath,
  knotListPath,
  resolveAssetUrl,
} from "./api";
import {
  cacheMediaUrlForOffline,
  isMediaUrlCached,
  removeCachedMediaUrls,
} from "./media-cache";
import {
  listOfflineCaches,
  makeOfflineCacheKey,
  readOfflineCache,
  removeOfflineCacheByKey,
  removeOfflineCachesWhere,
  writeOfflineCache,
  writeOfflineCaches,
  type OfflineCacheEnvelope,
  type OfflineCacheDescriptor,
} from "./offline-cache";
import type {
  KnotDetail,
  KnotListResponse,
  KnotMediaAsset,
  KnotSummary,
  SkillLocale,
} from "./skill-utils";
import { getSkillDifficultyLabel } from "./skill-utils";

export type KnotOfflineCachePhase = "list" | "detail" | "media";

export interface KnotOfflineCacheProgress {
  phase: KnotOfflineCachePhase;
  loadedCount?: number;
  totalCount?: number;
  currentIndex?: number;
  currentTitle?: string;
  mediaReadyCount?: number;
  mediaTotal?: number;
  failedMediaCount?: number;
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

export interface CachedKnotPreview {
  id: string;
  title: string;
  summary: string;
  categoryText: string;
  difficultyText: string;
  cachedAt: string;
}

export interface KnotOfflineCacheInventory {
  items: CachedKnotPreview[];
  cachedCount: number;
  totalCount: number;
  uncachedCount: number;
}

interface KnotOfflineCacheMeta {
  locale: SkillLocale;
  totalCount: number;
  mediaTotal: number;
  estimatedBytes: number;
  updatedAt: string;
}

interface CachedKnotDetailRecord {
  cache: OfflineCacheEnvelope<KnotDetail>;
  item: KnotDetail;
}

interface CacheAllKnotsForOfflineOptions {
  locale?: SkillLocale;
  pageSize?: number;
  plan?: KnotOfflineCachePlan;
  onProgress?: (progress: KnotOfflineCacheProgress) => void;
}

const DEFAULT_PAGE_SIZE = 10;
const DEFAULT_MEDIA_CONCURRENCY = 3;
const KNOT_OFFLINE_META_PATH = "/api/skills/knots/offline-cache-meta";
const KNOT_DETAIL_PATH_PREFIX = "/api/skills/knots/detail/";
const KNOT_LIST_PATH_PREFIX = "/api/skills/knots/list";

export async function cacheAllKnotsForOffline(
  options: CacheAllKnotsForOfflineOptions = {},
): Promise<KnotOfflineCacheResult> {
  const plan = options.plan ?? (await prepareAllKnotsOfflineCache(options));
  const urls = plan.mediaUrls.filter((url) => !isMediaUrlCached(url));
  writeKnotOfflineApiCaches(
    plan.items,
    plan.locale,
    options.pageSize ?? DEFAULT_PAGE_SIZE,
  );
  writeKnotOfflineCacheMeta(plan.locale, {
    totalCount: plan.detailCount,
    mediaTotal: plan.mediaTotal,
    estimatedBytes: plan.estimatedBytes,
  });
  let mediaReadyCount = plan.mediaUrls.length - urls.length;
  let failedMediaCount = 0;
  const notify = () =>
    options.onProgress?.({
      phase: "media",
      mediaReadyCount,
      mediaTotal: plan.mediaUrls.length,
      failedMediaCount,
    });
  notify();
  await runWithConcurrency(urls, DEFAULT_MEDIA_CONCURRENCY, async (url) => {
    try {
      await cacheMediaUrlForOffline(url);
      mediaReadyCount += 1;
    } catch {
      failedMediaCount += 1;
    }
    notify();
  });

  return {
    items: plan.items,
    detailCount: plan.detailCount,
    mediaReadyCount,
    mediaTotal: plan.mediaUrls.length,
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

export function listCachedKnotPreviews(
  locale: SkillLocale = "zh-CN",
): CachedKnotPreview[] {
  return getKnotOfflineCacheInventory(locale).items;
}

export function getKnotOfflineCacheInventory(
  locale: SkillLocale = "zh-CN",
): KnotOfflineCacheInventory {
  const previews = new Map<string, CachedKnotPreview>();
  const detailRecords = listCachedKnotDetailRecords(locale);

  detailRecords.forEach(({ cache, item }) => {
    addCachedKnotPreview(previews, item, cache.cachedAt);
  });

  const items = Array.from(previews.values());
  const meta = readKnotOfflineCacheMeta(locale);
  const totalCount = Math.max(meta?.totalCount ?? items.length, items.length);
  const cachedCount = items.length;
  return {
    items,
    cachedCount,
    totalCount,
    uncachedCount: Math.max(totalCount - cachedCount, 0),
  };
}

export async function refreshKnotOfflineCacheInventory(
  locale: SkillLocale = "zh-CN",
): Promise<KnotOfflineCacheInventory> {
  const manifest = await getKnotOfflineManifest(locale);
  writeKnotOfflineCacheMeta(locale, {
    totalCount: manifest.item_count,
    mediaTotal: manifest.media_count,
    estimatedBytes: manifest.estimated_bytes,
  });
  return getKnotOfflineCacheInventory(locale);
}

export function deleteCachedKnot(
  id: string,
  locale: SkillLocale = "zh-CN",
): KnotOfflineCacheInventory {
  const records = listCachedKnotDetailRecords(locale);
  const target = records.find(({ item }) => item.id === id);
  if (!target) {
    return getKnotOfflineCacheInventory(locale);
  }
  ensureKnotOfflineCacheMeta(locale, records.length);
  removeOfflineCacheByKey(target.cache.key);

  const remainingItems = records
    .filter(({ item }) => item.id !== id)
    .map(({ item }) => item);
  rewriteKnotListCaches(remainingItems, locale);
  removeUnreferencedMedia(target.item, remainingItems);
  return getKnotOfflineCacheInventory(locale);
}

export function clearKnotOfflineCache(
  locale: SkillLocale = "zh-CN",
): KnotOfflineCacheInventory {
  const records = listCachedKnotDetailRecords(locale);
  ensureKnotOfflineCacheMeta(locale, records.length);
  const mediaUrls = collectItemMediaUrls(records.map(({ item }) => item));

  removeOfflineCachesWhere((cache) => {
    if (
      cache.scope !== "public" ||
      !isCacheLocale(cache.key, cache.locale, locale)
    ) {
      return false;
    }
    const path = cachePath(cache.key);
    return (
      path.startsWith(KNOT_DETAIL_PATH_PREFIX) ||
      path.startsWith(KNOT_LIST_PATH_PREFIX)
    );
  });
  removeCachedMediaUrls(mediaUrls);
  return getKnotOfflineCacheInventory(locale);
}

function writeKnotOfflineApiCaches(
  items: KnotDetail[],
  locale: SkillLocale,
  pageSize: number,
): void {
  writeOfflineCaches(
    items.map((item) => ({
      descriptor: publicOfflineCacheDescriptor(knotDetailPath(item.id), locale),
      data: item,
    })),
  );
  writeKnotOfflineListCaches(items, locale, pageSize);
}

function writeKnotOfflineListCaches(
  items: KnotDetail[],
  locale: SkillLocale,
  pageSize: number,
): void {
  const effectivePageSize = Math.max(1, pageSize || DEFAULT_PAGE_SIZE);
  if (!items.length) {
    writeKnotListCache(locale, effectivePageSize, 0, []);
    return;
  }
  const entries: Array<{
    descriptor: OfflineCacheDescriptor;
    data: KnotListResponse;
  }> = [];
  for (let offset = 0; offset < items.length; offset += effectivePageSize) {
    entries.push(
      knotListCacheEntry(
        locale,
        effectivePageSize,
        offset,
        items.slice(offset, offset + effectivePageSize).map(toKnotSummary),
        offset + effectivePageSize < items.length
          ? offset + effectivePageSize
          : null,
      ),
    );
  }
  writeOfflineCaches(entries);
}

function writeKnotListCache(
  locale: SkillLocale,
  limit: number,
  offset: number,
  items: KnotSummary[],
  nextOffset: number | null = null,
): void {
  const entry = knotListCacheEntry(locale, limit, offset, items, nextOffset);
  writeOfflineCaches([entry]);
}

function knotListCacheEntry(
  locale: SkillLocale,
  limit: number,
  offset: number,
  items: KnotSummary[],
  nextOffset: number | null = null,
): { descriptor: OfflineCacheDescriptor; data: KnotListResponse } {
  const payload: KnotListResponse = {
    locale,
    items,
    page: {
      limit,
      offset,
      next_offset: nextOffset,
    },
  };
  return {
    descriptor: publicOfflineCacheDescriptor(
      knotListPath({ offset, limit }),
      locale,
    ),
    data: payload,
  };
}

function writePublicOfflineCache<T>(
  path: string,
  locale: SkillLocale,
  data: T,
): void {
  writeOfflineCache(publicOfflineCacheDescriptor(path, locale), data);
}

function publicOfflineCacheDescriptor(
  path: string,
  locale: SkillLocale,
): OfflineCacheDescriptor {
  return {
    key: makeOfflineCacheKey(path, { locale }),
    scope: "public",
    locale,
  };
}

function writeKnotOfflineCacheMeta(
  locale: SkillLocale,
  meta: Omit<KnotOfflineCacheMeta, "locale" | "updatedAt">,
): void {
  writePublicOfflineCache(KNOT_OFFLINE_META_PATH, locale, {
    locale,
    ...meta,
    updatedAt: new Date().toISOString(),
  });
}

function readKnotOfflineCacheMeta(
  locale: SkillLocale,
): KnotOfflineCacheMeta | null {
  return (
    readOfflineCache<KnotOfflineCacheMeta>({
      key: makeOfflineCacheKey(KNOT_OFFLINE_META_PATH, { locale }),
      scope: "public",
      locale,
    })?.data ?? null
  );
}

function ensureKnotOfflineCacheMeta(
  locale: SkillLocale,
  currentTotalCount: number,
): void {
  if (readKnotOfflineCacheMeta(locale)) {
    return;
  }
  writeKnotOfflineCacheMeta(locale, {
    totalCount: currentTotalCount,
    mediaTotal: 0,
    estimatedBytes: 0,
  });
}

function listCachedKnotDetailRecords(
  locale: SkillLocale,
): CachedKnotDetailRecord[] {
  return listOfflineCaches<KnotDetail>("public")
    .filter(
      (cache) =>
        isCacheLocale(cache.key, cache.locale, locale) &&
        cachePath(cache.key).startsWith(KNOT_DETAIL_PATH_PREFIX) &&
        Boolean(cache.data?.id),
    )
    .map((cache) => ({ cache, item: cache.data }));
}

function rewriteKnotListCaches(items: KnotDetail[], locale: SkillLocale): void {
  removeOfflineCachesWhere(
    (cache) =>
      cache.scope === "public" &&
      isCacheLocale(cache.key, cache.locale, locale) &&
      cachePath(cache.key).startsWith(KNOT_LIST_PATH_PREFIX),
  );
  if (items.length) {
    writeKnotOfflineListCaches(items, locale, DEFAULT_PAGE_SIZE);
  }
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

function collectItemMediaUrls(items: KnotDetail[]): string[] {
  const mediaByUrl = new Map<string, number>();
  items.forEach((item) => collectMediaUrls(item.media, mediaByUrl));
  return Array.from(mediaByUrl.keys());
}

function removeUnreferencedMedia(
  target: KnotDetail,
  remainingItems: KnotDetail[],
): void {
  const targetUrls = collectItemMediaUrls([target]);
  if (!targetUrls.length) {
    return;
  }
  const remainingUrls = new Set(collectItemMediaUrls(remainingItems));
  removeCachedMediaUrls(targetUrls.filter((url) => !remainingUrls.has(url)));
}

async function runWithConcurrency<T>(
  items: T[],
  concurrency: number,
  worker: (item: T) => Promise<void>,
): Promise<void> {
  let nextIndex = 0;
  const workerCount = Math.min(Math.max(1, concurrency), items.length);
  await Promise.all(
    Array.from({ length: workerCount }, async () => {
      while (nextIndex < items.length) {
        const item = items[nextIndex];
        nextIndex += 1;
        await worker(item);
      }
    }),
  );
}

function addCachedKnotPreview(
  previews: Map<string, CachedKnotPreview>,
  item: KnotSummary | KnotDetail,
  cachedAt: string,
): void {
  if (!item?.id || previews.has(item.id)) {
    return;
  }
  previews.set(item.id, {
    id: item.id,
    title: item.title || "未命名绳结",
    summary:
      item.summary || ("description" in item ? item.description || "" : ""),
    categoryText: item.categories[0]?.title ?? "绳结",
    difficultyText: getSkillDifficultyLabel(item.difficulty),
    cachedAt,
  });
}

function cachePath(key: string): string {
  return key.split("|")[0] ?? key;
}

function isCacheLocale(
  key: string,
  envelopeLocale: string | undefined,
  locale: SkillLocale,
): boolean {
  return (envelopeLocale || key.split("|")[1] || "zh-CN") === locale;
}
