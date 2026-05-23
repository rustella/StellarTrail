const PROFILE_SHOULD_REFRESH_KEY = "stellartrail_profile_should_refresh";

export function markProfileShouldRefresh(): void {
  wx.setStorageSync(PROFILE_SHOULD_REFRESH_KEY, true);
}

export function consumeProfileShouldRefresh(): boolean {
  const shouldRefresh = wx.getStorageSync(PROFILE_SHOULD_REFRESH_KEY) === true;
  if (shouldRefresh) {
    wx.removeStorageSync(PROFILE_SHOULD_REFRESH_KEY);
  }
  return shouldRefresh;
}
