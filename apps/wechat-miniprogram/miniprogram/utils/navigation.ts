const TAB_PAGES = new Set([
  "/pages/index/index",
  "/pages/gears/index",
  "/pages/trips/index",
  "/pages/skills/index",
  "/pages/profile/index",
]);
export const GUEST_FALLBACK_PAGE = "/pages/gear-atlas/index";

const GUEST_ACCESSIBLE_PAGES = new Set([
  GUEST_FALLBACK_PAGE,
  "/pages/gear-atlas/detail/index",
  "/pages/login/index",
  "/pages/register/index",
]);

export function decodeRedirect(
  value?: string,
  fallback = "/pages/profile/index",
): string {
  if (!value) {
    return fallback;
  }
  try {
    const decoded = decodeURIComponent(value);
    return isSafeMiniProgramPage(decoded) ? decoded : fallback;
  } catch {
    return fallback;
  }
}

export function navigateToRedirect(
  redirect: string,
  fallback = "/pages/index/index",
): void {
  const safeRedirect = isSafeMiniProgramPage(redirect) ? redirect : fallback;
  const [path] = safeRedirect.split("?");
  if (TAB_PAGES.has(path)) {
    wx.switchTab({ url: path });
    return;
  }
  wx.redirectTo({
    url: safeRedirect,
    fail: () => wx.switchTab({ url: fallback }),
  });
}

export function isGuestAccessiblePage(value: string): boolean {
  const [path] = value.split("?");
  return GUEST_ACCESSIBLE_PAGES.has(path);
}

export function navigateToGuestFallback(): void {
  wx.redirectTo({
    url: GUEST_FALLBACK_PAGE,
    fail: () => {
      wx.navigateTo({ url: GUEST_FALLBACK_PAGE });
    },
  });
}

function isSafeMiniProgramPage(value: string): boolean {
  return (
    value.startsWith("/pages/") &&
    !value.includes("//") &&
    !value.includes("..")
  );
}
