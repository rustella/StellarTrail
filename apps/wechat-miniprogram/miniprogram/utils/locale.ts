export type AppLocale = "zh-CN" | "en";

export interface AppLocaleOption {
  value: AppLocale;
  label: string;
  description: string;
}

export const APP_LOCALE_OPTIONS: AppLocaleOption[] = [
  {
    value: "zh-CN",
    label: "中文",
    description: "装备图鉴使用中文展示。",
  },
  {
    value: "en",
    label: "English",
    description: "Show gear atlas public content in English when available.",
  },
];

const APP_LOCALE_STORAGE_KEY = "stellartrail.wechat.locale";

export function loadAppLocale(): AppLocale {
  return normalizeAppLocale(wx.getStorageSync(APP_LOCALE_STORAGE_KEY));
}

export function saveAppLocale(locale: AppLocale): void {
  const normalized = normalizeAppLocale(locale);
  wx.setStorageSync(APP_LOCALE_STORAGE_KEY, normalized);
  syncGlobalLocale(normalized);
}

export function getAppLocaleLabel(locale = loadAppLocale()): string {
  return (
    APP_LOCALE_OPTIONS.find((option) => option.value === locale)?.label ??
    APP_LOCALE_OPTIONS[0].label
  );
}

export function normalizeAppLocale(value: unknown): AppLocale {
  return value === "en" ? "en" : "zh-CN";
}

export function syncGlobalLocale(locale = loadAppLocale()): AppLocale {
  const normalized = normalizeAppLocale(locale);
  const app = getApp<{
    globalData?: {
      appLocale?: AppLocale;
    };
  }>();
  if (app.globalData) {
    app.globalData.appLocale = normalized;
  }
  return normalized;
}
