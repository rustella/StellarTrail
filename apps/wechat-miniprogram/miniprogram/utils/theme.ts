export type ThemeMode = "light" | "dark";

export interface ThemeViewData {
  theme: ThemeMode;
  themeClass: string;
  isDarkTheme: boolean;
  themeToggleIcon: string;
  themeToggleText: string;
  themeToggleAriaLabel: string;
}

const THEME_STORAGE_KEY = "stellartrail.wechat.theme";

interface ThemePage {
  data?: Partial<ThemeViewData>;
  setData(data: Partial<ThemeViewData>): void;
}

export function loadThemePreference(): ThemeMode {
  return normalizeTheme(wx.getStorageSync(THEME_STORAGE_KEY));
}

export function saveThemePreference(theme: ThemeMode): void {
  wx.setStorageSync(THEME_STORAGE_KEY, theme);
  syncGlobalTheme(theme);
}

export function getThemeViewData(theme = loadThemePreference()): ThemeViewData {
  const isDarkTheme = theme === "dark";
  return {
    theme,
    themeClass: isDarkTheme ? "theme-dark" : "theme-light",
    isDarkTheme,
    themeToggleIcon: isDarkTheme ? "☀️" : "🌙",
    themeToggleText: isDarkTheme ? "白天模式" : "黑夜模式",
    themeToggleAriaLabel: isDarkTheme ? "切换到白天模式" : "切换到黑夜模式",
  };
}

export function syncPageTheme(
  page: ThemePage,
  theme = loadThemePreference(),
): ThemeMode {
  syncGlobalTheme(theme);
  applyThemeToSystem(theme);
  const viewData = getThemeViewData(theme);
  if (!hasSameThemeViewData(page.data, viewData)) {
    page.setData(viewData);
  }
  return theme;
}

export function togglePageTheme(page: ThemePage): ThemeMode {
  const current = normalizeTheme(page.data?.theme ?? loadThemePreference());
  const next: ThemeMode = current === "dark" ? "light" : "dark";
  saveThemePreference(next);
  syncPageTheme(page, next);
  wx.showToast({
    title: next === "dark" ? "已切换黑夜模式" : "已切换白天模式",
    icon: "none",
  });
  return next;
}

export function applyThemeToSystem(theme: ThemeMode): void {
  if (theme === "dark") {
    wx.setNavigationBarColor({
      frontColor: "#ffffff",
      backgroundColor: "#12082e",
    });
    setBackgroundColor("#07051a");
    wx.setTabBarStyle({
      color: "#c7b9f4",
      selectedColor: "#a78bfa",
      backgroundColor: "#17112f",
      borderStyle: "black",
      fail: noop,
    });
    return;
  }

  wx.setNavigationBarColor({
    frontColor: "#ffffff",
    backgroundColor: "#0f172a",
  });
  setBackgroundColor("#f8fafc");
  wx.setTabBarStyle({
    color: "#64748b",
    selectedColor: "#0f766e",
    backgroundColor: "#ffffff",
    borderStyle: "white",
    fail: noop,
  });
}

function normalizeTheme(value: unknown): ThemeMode {
  return value === "dark" ? "dark" : "light";
}

function hasSameThemeViewData(
  current: ThemePage["data"],
  next: ThemeViewData,
): boolean {
  return (
    current?.theme === next.theme &&
    current.themeClass === next.themeClass &&
    current.isDarkTheme === next.isDarkTheme &&
    current.themeToggleIcon === next.themeToggleIcon &&
    current.themeToggleText === next.themeToggleText &&
    current.themeToggleAriaLabel === next.themeToggleAriaLabel
  );
}

function syncGlobalTheme(theme: ThemeMode): void {
  const app = getApp<{
    globalData?: {
      theme?: ThemeMode;
    };
  }>();
  if (app.globalData) {
    app.globalData.theme = theme;
  }
}

function setBackgroundColor(backgroundColor: string): void {
  const backgroundApi = wx as WechatMiniprogram.Wx & {
    setBackgroundColor?: (options: {
      backgroundColor: string;
      backgroundColorTop?: string;
      backgroundColorBottom?: string;
      fail?: () => void;
    }) => void;
  };
  backgroundApi.setBackgroundColor?.({
    backgroundColor,
    backgroundColorTop: backgroundColor,
    backgroundColorBottom: backgroundColor,
    fail: noop,
  });
}

function noop(): void {
  // Keep non-TabBar pages and older base libraries safe: system appearance API failures must not block in-page theme rendering.
}
