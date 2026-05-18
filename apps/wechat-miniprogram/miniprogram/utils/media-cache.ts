import { isOffline } from "./network-state";

interface MediaCacheEntry {
  url: string;
  filePath: string;
  cachedAt: string;
}

interface SaveFileOptions {
  tempFilePath: string;
  success: (result: { savedFilePath: string }) => void;
}

interface GetFileInfoOptions {
  filePath: string;
  success: () => void;
  fail: () => void;
}

const MEDIA_CACHE_PREFIX = "stellartrail_media_cache_v1";
const MEDIA_CACHE_INDEX_KEY = "stellartrail_media_cache_index_v1";
const pendingDownloads = new Set<string>();
const mediaWx = wx as typeof wx & {
  saveFile(options: SaveFileOptions): void;
  getFileInfo(options: GetFileInfoOptions): void;
};

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
  pendingDownloads.add(url);
  wx.downloadFile({
    url,
    success: (download) => {
      if (download.statusCode && download.statusCode >= 400) {
        return;
      }
      mediaWx.saveFile({
        tempFilePath: download.tempFilePath,
        success: (saved) => {
          writeMediaCacheEntry({
            url,
            filePath: saved.savedFilePath,
            cachedAt: new Date().toISOString(),
          });
        },
      });
    },
    complete: () => {
      pendingDownloads.delete(url);
    },
  });
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
    mediaWx.getFileInfo({
      filePath,
      success: () => resolve(true),
      fail: () => resolve(false),
    });
  });
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
