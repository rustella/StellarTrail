const TAB_PAGES = new Set([
  "/pages/index/index",
  "/pages/gears/index",
  "/pages/skills/index",
  "/pages/profile/index",
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

function isSafeMiniProgramPage(value: string): boolean {
  return (
    value.startsWith("/pages/") &&
    !value.includes("//") &&
    !value.includes("..")
  );
}
