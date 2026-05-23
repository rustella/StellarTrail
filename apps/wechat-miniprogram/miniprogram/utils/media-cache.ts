import { isOffline } from "./network-state";

interface MediaCacheEntry {
  url: string;
  filePath: string;
  cachedAt: string;
}

interface SaveFileOptions {
  tempFilePath: string;
  success: (result: { savedFilePath: string }) => void;
  fail?: (error: unknown) => void;
}

interface GetFileInfoOptions {
  filePath: string;
  success: () => void;
  fail: () => void;
}

interface RemoveSavedFileOptions {
  filePath: string;
  success?: () => void;
  fail?: (error: unknown) => void;
  complete?: () => void;
}

const MEDIA_CACHE_PREFIX = "stellartrail_media_cache_v1";
const MEDIA_CACHE_INDEX_KEY = "stellartrail_media_cache_index_v1";
const OPPORTUNISTIC_MEDIA_CONCURRENCY = 2;
const MAX_OPPORTUNISTIC_MEDIA_QUEUE = 30;
const pendingDownloads = new Map<string, Promise<void>>();
const resolvedMediaPathMemo = new Map<string, string>();
const opportunisticQueue: string[] = [];
const queuedOpportunisticUrls = new Set<string>();
let activeOpportunisticDownloads = 0;
let mediaCacheIndexMemo: Set<string> | null = null;

export async function resolveCachedMediaUrl(url: string): Promise<string> {
  if (!isHttpUrl(url)) {
    return url;
  }
  const memoPath = resolvedMediaPathMemo.get(url);
  if (memoPath) {
    return memoPath;
  }
  const cachedPath = await readValidCachedMediaPath(url);
  if (cachedPath) {
    resolvedMediaPathMemo.set(url, cachedPath);
    return cachedPath;
  }
  if (!isOffline()) {
    cacheMediaUrl(url);
    return url;
  }
  return "";
}

export function cacheMediaUrl(url: string): void {
  if (
    !isHttpUrl(url) ||
    pendingDownloads.has(url) ||
    queuedOpportunisticUrls.has(url) ||
    isOffline() ||
    readMediaCacheIndexSet().has(url)
  ) {
    return;
  }
  if (opportunisticQueue.length >= MAX_OPPORTUNISTIC_MEDIA_QUEUE) {
    return;
  }
  queuedOpportunisticUrls.add(url);
  opportunisticQueue.push(url);
  drainOpportunisticQueue();
}

export async function cacheMediaUrlForOffline(url: string): Promise<boolean> {
  if (!isHttpUrl(url)) {
    return false;
  }
  const cachedPath = await readValidCachedMediaPath(url);
  if (cachedPath) {
    return false;
  }
  if (isOffline()) {
    throw new Error("当前离线，无法下载媒体资源");
  }
  const pending = pendingDownloads.get(url);
  if (pending) {
    await pending;
    return false;
  }
  dropQueuedOpportunisticUrl(url);
  const download = downloadAndSaveMediaUrl(url);
  pendingDownloads.set(url, download);
  try {
    await download;
    return true;
  } finally {
    if (pendingDownloads.get(url) === download) {
      pendingDownloads.delete(url);
    }
  }
}

export function filterUncachedMediaUrls(urls: string[]): string[] {
  const index = readMediaCacheIndexSet();
  const seen = new Set<string>();
  return urls.filter((url) => {
    if (seen.has(url) || index.has(url)) {
      return false;
    }
    seen.add(url);
    return true;
  });
}

export function removeCachedMediaUrls(urls: string[]): number {
  let removedCount = 0;
  const uniqueUrls = Array.from(new Set(urls));
  const index = readMediaCacheIndex();
  const removedUrls = new Set<string>();
  uniqueUrls.forEach((url) => {
    const entry = readMediaCacheEntry(url);
    if (!entry) {
      return;
    }
    removeSavedMediaFile(entry.filePath);
    removeMediaCacheEntryStorage(url);
    removedUrls.add(url);
    removedCount += 1;
  });
  if (removedUrls.size) {
    const nextIndex = index.filter((url) => !removedUrls.has(url));
    writeMediaCacheIndex(nextIndex);
    mediaCacheIndexMemo = new Set(nextIndex);
  }
  return removedCount;
}

export function removeCachedMediaUrl(url: string): boolean {
  return removeCachedMediaUrls([url]) > 0;
}

export function isMediaUrlCached(url: string): boolean {
  return readMediaCacheIndexSet().has(url);
}

function drainOpportunisticQueue(): void {
  while (
    activeOpportunisticDownloads < OPPORTUNISTIC_MEDIA_CONCURRENCY &&
    opportunisticQueue.length
  ) {
    const url = opportunisticQueue.shift();
    if (!url) {
      return;
    }
    queuedOpportunisticUrls.delete(url);
    if (pendingDownloads.has(url) || isOffline()) {
      continue;
    }
    activeOpportunisticDownloads += 1;
    const download = downloadAndSaveMediaUrl(url);
    pendingDownloads.set(url, download);
    void download
      .catch(() => {})
      .finally(() => {
        if (pendingDownloads.get(url) === download) {
          pendingDownloads.delete(url);
        }
        activeOpportunisticDownloads = Math.max(
          0,
          activeOpportunisticDownloads - 1,
        );
        drainOpportunisticQueue();
      });
  }
}

function dropQueuedOpportunisticUrl(url: string): void {
  if (!queuedOpportunisticUrls.delete(url)) {
    return;
  }
  const index = opportunisticQueue.indexOf(url);
  if (index >= 0) {
    opportunisticQueue.splice(index, 1);
  }
}

async function readValidCachedMediaPath(url: string): Promise<string | null> {
  const entry = readMediaCacheEntry(url);
  if (!entry) {
    return null;
  }
  const exists = await savedFileExists(entry.filePath);
  if (exists) {
    resolvedMediaPathMemo.set(url, entry.filePath);
    return entry.filePath;
  }
  removeMediaCacheEntry(url);
  return null;
}

function savedFileExists(filePath: string): Promise<boolean> {
  return new Promise((resolve) => {
    getMediaWx().getFileInfo({
      filePath,
      success: () => resolve(true),
      fail: () => resolve(false),
    });
  });
}

function downloadAndSaveMediaUrl(url: string): Promise<void> {
  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = () => {
      if (!settled) {
        settled = true;
        resolve();
      }
    };
    const fail = (error: unknown) => {
      if (!settled) {
        settled = true;
        reject(error instanceof Error ? error : new Error("媒体资源缓存失败"));
      }
    };

    wx.downloadFile({
      url,
      success: (download) => {
        if (download.statusCode && download.statusCode >= 400) {
          fail(new Error("媒体资源下载失败"));
          return;
        }
        getMediaWx().saveFile({
          tempFilePath: download.tempFilePath,
          success: (saved) => {
            writeMediaCacheEntry({
              url,
              filePath: saved.savedFilePath,
              cachedAt: new Date().toISOString(),
            });
            finish();
          },
          fail,
        });
      },
      fail,
      complete: () => {},
    });
  });
}

function getMediaWx(): typeof wx & {
  saveFile(options: SaveFileOptions): void;
  getFileInfo(options: GetFileInfoOptions): void;
  removeSavedFile?: (options: RemoveSavedFileOptions) => void;
} {
  return wx as typeof wx & {
    saveFile(options: SaveFileOptions): void;
    getFileInfo(options: GetFileInfoOptions): void;
    removeSavedFile?: (options: RemoveSavedFileOptions) => void;
  };
}

function readMediaCacheEntry(url: string): MediaCacheEntry | null {
  try {
    return (
      (wx.getStorageSync(mediaStorageKey(url)) as
        | MediaCacheEntry
        | undefined) ?? null
    );
  } catch {
    return null;
  }
}

function writeMediaCacheEntry(entry: MediaCacheEntry): void {
  try {
    resolvedMediaPathMemo.set(entry.url, entry.filePath);
    wx.setStorageSync(mediaStorageKey(entry.url), entry);
    const index = readMediaCacheIndex();
    if (!index.includes(entry.url)) {
      index.push(entry.url);
      writeMediaCacheIndex(index);
    }
  } catch {
    // Media caching is opportunistic.
  }
}

function removeMediaCacheEntry(url: string): void {
  try {
    resolvedMediaPathMemo.delete(url);
    removeMediaCacheEntryStorage(url);
    writeMediaCacheIndex(readMediaCacheIndex().filter((item) => item !== url));
  } catch {
    // Best-effort cleanup.
  }
}

function removeMediaCacheEntryStorage(url: string): void {
  resolvedMediaPathMemo.delete(url);
  try {
    wx.removeStorageSync(mediaStorageKey(url));
  } catch {
    // Best-effort cleanup.
  }
}

function removeSavedMediaFile(filePath: string): void {
  const mediaWx = getMediaWx();
  if (typeof mediaWx.removeSavedFile === "function") {
    mediaWx.removeSavedFile({
      filePath,
      complete: () => {},
    });
  }
}

function readMediaCacheIndex(): string[] {
  try {
    if (mediaCacheIndexMemo) {
      return Array.from(mediaCacheIndexMemo);
    }
    const index = wx.getStorageSync(MEDIA_CACHE_INDEX_KEY) as
      | string[]
      | undefined;
    const urls = Array.isArray(index) ? index : [];
    mediaCacheIndexMemo = new Set(urls);
    return urls;
  } catch {
    return [];
  }
}

function readMediaCacheIndexSet(): Set<string> {
  if (!mediaCacheIndexMemo) {
    mediaCacheIndexMemo = new Set(readMediaCacheIndex());
  }
  return mediaCacheIndexMemo;
}

function writeMediaCacheIndex(index: string[]): void {
  mediaCacheIndexMemo = new Set(index);
  try {
    wx.setStorageSync(MEDIA_CACHE_INDEX_KEY, index);
  } catch {
    // Best-effort index maintenance.
  }
}

function mediaStorageKey(url: string): string {
  return `${MEDIA_CACHE_PREFIX}:${encodeURIComponent(url)}`;
}

function isHttpUrl(value: string): boolean {
  return /^https?:\/\//i.test(value);
}
