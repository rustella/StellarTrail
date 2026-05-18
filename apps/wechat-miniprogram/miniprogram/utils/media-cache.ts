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

const MEDIA_CACHE_PREFIX = "stellartrail_media_cache_v1";
const MEDIA_CACHE_INDEX_KEY = "stellartrail_media_cache_index_v1";
const pendingDownloads = new Map<string, Promise<void>>();

export async function resolveCachedMediaUrl(url: string): Promise<string> {
  if (!isHttpUrl(url)) {
    return url;
  }
  const cachedPath = await readValidCachedMediaPath(url);
  if (cachedPath) {
    return cachedPath;
  }
  if (!isOffline()) {
    cacheMediaUrl(url);
    return url;
  }
  return "";
}

export function cacheMediaUrl(url: string): void {
  if (!isHttpUrl(url) || pendingDownloads.has(url) || isOffline()) {
    return;
  }
  const download = downloadAndSaveMediaUrl(url);
  pendingDownloads.set(url, download);
  void download
    .catch(() => {})
    .finally(() => {
      if (pendingDownloads.get(url) === download) {
        pendingDownloads.delete(url);
      }
    });
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

async function readValidCachedMediaPath(url: string): Promise<string | null> {
  const entry = readMediaCacheEntry(url);
  if (!entry) {
    return null;
  }
  const exists = await savedFileExists(entry.filePath);
  if (exists) {
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
} {
  return wx as typeof wx & {
    saveFile(options: SaveFileOptions): void;
    getFileInfo(options: GetFileInfoOptions): void;
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
    wx.setStorageSync(mediaStorageKey(entry.url), entry);
    const index = readMediaCacheIndex();
    if (!index.includes(entry.url)) {
      index.push(entry.url);
      wx.setStorageSync(MEDIA_CACHE_INDEX_KEY, index);
    }
  } catch {
    // Media caching is opportunistic.
  }
}

function removeMediaCacheEntry(url: string): void {
  try {
    wx.removeStorageSync(mediaStorageKey(url));
    const index = readMediaCacheIndex().filter((item) => item !== url);
    wx.setStorageSync(MEDIA_CACHE_INDEX_KEY, index);
  } catch {
    // Best-effort cleanup.
  }
}

function readMediaCacheIndex(): string[] {
  try {
    const index = wx.getStorageSync(MEDIA_CACHE_INDEX_KEY) as
      | string[]
      | undefined;
    return Array.isArray(index) ? index : [];
  } catch {
    return [];
  }
}

function mediaStorageKey(url: string): string {
  return `${MEDIA_CACHE_PREFIX}:${encodeURIComponent(url)}`;
}

function isHttpUrl(value: string): boolean {
  return /^https?:\/\//i.test(value);
}
