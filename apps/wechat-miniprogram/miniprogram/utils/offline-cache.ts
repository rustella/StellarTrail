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

export function writeOfflineCaches<T>(
  entries: Array<{ descriptor: OfflineCacheDescriptor; data: T }>,
): void {
  if (!entries.length) {
    return;
  }
  const index = readCacheIndex();
  let indexChanged = false;
  entries.forEach(({ descriptor, data }) => {
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
      indexChanged =
        upsertCacheIndexItem(index, {
          storageKey,
          scope: descriptor.scope,
          ...(descriptor.userId ? { userId: descriptor.userId } : {}),
        }) || indexChanged;
    } catch {
      // Storage quota errors should not block online reads.
    }
  });
  if (indexChanged) {
    writeCacheIndex(index);
  }
}

export function listOfflineCaches<T = unknown>(
  scope?: OfflineCacheScope,
): Array<OfflineCacheEnvelope<T>> {
  return readCacheIndex()
    .map((item) => {
      const envelope = safeGetStorage<OfflineCacheEnvelope<T>>(item.storageKey);
      if (
        !envelope ||
        envelope.version !== CACHE_VERSION ||
        envelope.scope !== item.scope ||
        (item.scope === "user" && envelope.userId !== item.userId)
      ) {
        return null;
      }
      if (scope && envelope.scope !== scope) {
        return null;
      }
      return envelope;
    })
    .filter((item): item is OfflineCacheEnvelope<T> => Boolean(item));
}

export function removeOfflineCacheByKey(key: string): boolean {
  const storageKey = cacheStorageKey(key);
  const index = readCacheIndex();
  const nextIndex = index.filter((item) => item.storageKey !== storageKey);
  try {
    wx.removeStorageSync(storageKey);
  } catch {
    // Best-effort cleanup.
  }
  if (nextIndex.length !== index.length) {
    writeCacheIndex(nextIndex);
    return true;
  }
  return false;
}

export function removeOfflineCachesWhere(
  shouldRemove: (envelope: OfflineCacheEnvelope<unknown>) => boolean,
): number {
  let removedCount = 0;
  const nextIndex: OfflineCacheIndexItem[] = [];
  readCacheIndex().forEach((item) => {
    const envelope = safeGetStorage<OfflineCacheEnvelope<unknown>>(
      item.storageKey,
    );
    if (!envelope) {
      return;
    }
    if (shouldRemove(envelope)) {
      try {
        wx.removeStorageSync(item.storageKey);
      } catch {
        // Best-effort cleanup.
      }
      removedCount += 1;
      return;
    }
    nextIndex.push(item);
  });
  writeCacheIndex(nextIndex);
  return removedCount;
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
  if (upsertCacheIndexItem(index, item)) {
    writeCacheIndex(index);
  }
}

function upsertCacheIndexItem(
  index: OfflineCacheIndexItem[],
  item: OfflineCacheIndexItem,
): boolean {
  const existing = index.find(
    (current) => current.storageKey === item.storageKey,
  );
  if (existing) {
    const changed =
      existing.scope !== item.scope || existing.userId !== item.userId;
    existing.scope = item.scope;
    existing.userId = item.userId;
    return changed;
  }
  index.push(item);
  return true;
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
