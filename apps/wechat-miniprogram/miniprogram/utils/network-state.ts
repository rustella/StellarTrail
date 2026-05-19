export const OFFLINE_CACHE_NOTICE = "当前离线，正在显示已缓存内容";
export const OFFLINE_CACHE_MISS_MESSAGE = "当前离线且暂无已缓存内容";
export const OFFLINE_WRITE_BLOCKED_MESSAGE =
  "当前离线，仅支持查看已缓存内容，请联网后再操作";

export interface NetworkState {
  isOffline: boolean;
  networkType?: string;
  lastOnlineAt?: string;
  updatedAt: string;
}

const NETWORK_STATE_STORAGE_KEY = "stellartrail_network_state";

let currentNetworkState: NetworkState = readStoredNetworkState() ?? {
  isOffline: false,
  updatedAt: new Date().toISOString(),
};

export class OfflineCacheMissError extends Error {
  readonly code = "OFFLINE_CACHE_MISS";

  constructor(message = OFFLINE_CACHE_MISS_MESSAGE) {
    super(message);
    this.name = "OfflineCacheMissError";
  }
}

export class OfflineWriteBlockedError extends Error {
  readonly code = "OFFLINE_WRITE_BLOCKED";

  constructor(message = OFFLINE_WRITE_BLOCKED_MESSAGE) {
    super(message);
    this.name = "OfflineWriteBlockedError";
  }
}

export function initNetworkState(): void {
  if (typeof wx.getNetworkType === "function") {
    wx.getNetworkType({
      success: (result) => {
        updateNetworkState(result.networkType, result.networkType === "none");
      },
    });
  }
  if (typeof wx.onNetworkStatusChange === "function") {
    wx.onNetworkStatusChange((result) => {
      updateNetworkState(result.networkType, !result.isConnected);
    });
  }
}

export function getNetworkState(): NetworkState {
  return currentNetworkState;
}

export function isOffline(): boolean {
  return currentNetworkState.isOffline;
}

export function isOfflineCacheMissError(
  error: unknown,
): error is OfflineCacheMissError {
  return (
    error instanceof OfflineCacheMissError ||
    (typeof error === "object" &&
      error !== null &&
      (error as { code?: unknown }).code === "OFFLINE_CACHE_MISS")
  );
}

export function isOfflineWriteBlockedError(
  error: unknown,
): error is OfflineWriteBlockedError {
  return (
    error instanceof OfflineWriteBlockedError ||
    (typeof error === "object" &&
      error !== null &&
      (error as { code?: unknown }).code === "OFFLINE_WRITE_BLOCKED")
  );
}

export function showOfflineWriteBlockedToast(): void {
  wx.showToast({
    title: OFFLINE_WRITE_BLOCKED_MESSAGE,
    icon: "none",
  });
}

export function markNetworkFailure(): void {
  const now = new Date().toISOString();
  currentNetworkState = {
    ...currentNetworkState,
    updatedAt: now,
  };
  persistNetworkState();
}

function updateNetworkState(networkType: string | undefined, offline: boolean) {
  const now = new Date().toISOString();
  currentNetworkState = {
    isOffline: offline,
    ...(networkType ? { networkType } : {}),
    lastOnlineAt: offline
      ? currentNetworkState.lastOnlineAt
      : currentNetworkState.lastOnlineAt || now,
    updatedAt: now,
  };
  persistNetworkState();
}

function persistNetworkState() {
  try {
    wx.setStorageSync(NETWORK_STATE_STORAGE_KEY, currentNetworkState);
  } catch {
    // Network state is useful but not critical.
  }
}

function readStoredNetworkState(): NetworkState | null {
  try {
    const stored =
      (wx.getStorageSync(NETWORK_STATE_STORAGE_KEY) as
        | NetworkState
        | undefined) ?? null;
    if (!stored) {
      return null;
    }
    if (stored.isOffline) {
      return {
        ...stored,
        isOffline: false,
        networkType:
          stored.networkType === "none" ? undefined : stored.networkType,
        updatedAt: new Date().toISOString(),
      };
    }
    return stored;
  } catch {
    return null;
  }
}
