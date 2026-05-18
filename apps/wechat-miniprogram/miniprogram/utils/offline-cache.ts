export type OfflineCacheScope = "public" | "user";

export interface OfflineCacheDescriptor {
  key: string;
  scope: OfflineCacheScope;
  userId?: string;
  locale?: string;
}

export interface OfflineCacheEnvelope<T> extends OfflineCacheDescriptor {
  version: 1;
  cachedAt: string;
  data: T;
}

interface OfflineCacheIndexItem {
  storageKey: string;
  scope: OfflineCacheScope;
  userId?: string;
}

const CACHE_VERSION = 1;
const CACHE_PREFIX = "stellartrail_offline_cache_v1";
const CACHE_INDEX_KEY = "stellartrail_offline_cache_index_v1";

export function makeOfflineCacheKey(
  path: string,
  options: { locale?: string; userId?: string } = {},
): string {
  return [path, options.locale ?? "", options.userId ?? ""].join("|");
}

export function readOfflineCache<T>(
  descriptor: OfflineCacheDescriptor,
): OfflineCacheEnvelope<T> | null {
  const envelope = safeGetStorage<OfflineCacheEnvelope<T>>(
    cacheStorageKey(descriptor.key),
  );
  if (
    !envelope ||
    envelope.version !== CACHE_VERSION ||
    envelope.scope !== descriptor.scope ||
    envelope.key !== descriptor.key
  ) {
    return null;
  }
  if (descriptor.scope === "user" && envelope.userId !== descriptor.userId) {
    return null;
  }
  return envelope;
}

export function writeOfflineCache<T>(
  descriptor: OfflineCacheDescriptor,
  data: T,
): void {
  const storageKey = cacheStorageKey(descriptor.key);
  const envelope: OfflineCacheEnvelope<T> = {
    version: CACHE_VERSION,
    key: descriptor.key,
    scope: descriptor.scope,
    ...(descriptor.userId ? { userId: descriptor.userId } : {}),
    ...(descriptor.locale ? { locale: descriptor.locale } : {}),
    cachedAt: new Date().toISOString(),
    data,
  };
  try {
    wx.setStorageSync(storageKey, envelope);
    addCacheIndexItem({
      storageKey,
      scope: descriptor.scope,
      ...(descriptor.userId ? { userId: descriptor.userId } : {}),
    });
  } catch {
    // Storage quota errors should not block online reads.
  }
}

export function clearUserOfflineCaches(userId?: string): void {
  const nextIndex: OfflineCacheIndexItem[] = [];
  readCacheIndex().forEach((item) => {
    const shouldRemove =
      item.scope === "user" && (!userId || item.userId === userId);
    if (shouldRemove) {
      try {
        wx.removeStorageSync(item.storageKey);
      } catch {
        // Best-effort cleanup.
      }
      return;
    }
    nextIndex.push(item);
  });
  writeCacheIndex(nextIndex);
}

function addCacheIndexItem(item: OfflineCacheIndexItem): void {
  const index = readCacheIndex();
  const existing = index.find(
    (current) => current.storageKey === item.storageKey,
  );
  if (existing) {
    existing.scope = item.scope;
    existing.userId = item.userId;
    writeCacheIndex(index);
    return;
  }
  index.push(item);
  writeCacheIndex(index);
}

function readCacheIndex(): OfflineCacheIndexItem[] {
  const index = safeGetStorage<OfflineCacheIndexItem[]>(CACHE_INDEX_KEY);
  return Array.isArray(index) ? index : [];
}

function writeCacheIndex(index: OfflineCacheIndexItem[]): void {
  try {
    wx.setStorageSync(CACHE_INDEX_KEY, index);
  } catch {
    // Best-effort index maintenance.
  }
}

function cacheStorageKey(key: string): string {
  return `${CACHE_PREFIX}:${encodeURIComponent(key)}`;
}

function safeGetStorage<T>(key: string): T | null {
  try {
    return (wx.getStorageSync(key) as T | undefined) ?? null;
  } catch {
    return null;
  }
}
