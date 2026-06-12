import type { AppLocale } from "@stellartrail/shared-types";

export function detectPreferredAppLocale(): AppLocale {
  if (typeof document !== "undefined") {
    const pageLocale = localeFromLanguage(document.documentElement.lang);
    if (pageLocale) {
      return pageLocale;
    }
  }

  if (typeof navigator === "undefined") {
    return "zh-CN";
  }
  const languages = [
    ...(Array.isArray(navigator.languages) ? navigator.languages : []),
    navigator.language,
  ].filter((language): language is string => Boolean(language));

  for (const language of languages) {
    const locale = localeFromLanguage(language);
    if (locale) {
      return locale;
    }
  }

  return "zh-CN";
}

function localeFromLanguage(language: string | undefined): AppLocale | null {
  const normalized = language?.toLowerCase().replace("_", "-");
  if (!normalized) {
    return null;
  }
  if (
    normalized === "zh" ||
    normalized.startsWith("zh-") ||
    normalized === "cn"
  ) {
    return "zh-CN";
  }
  if (normalized === "en" || normalized.startsWith("en-")) {
    return "en";
  }
  return null;
}
