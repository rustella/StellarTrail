import { hasAccessToken } from "./api";

export interface LoginPromptViewData {
  visible: boolean;
  title: string;
  message: string;
  redirectUrl: string;
}

interface PromptPage {
  data?: {
    loginPrompt?: LoginPromptViewData;
  };
  setData(data: Record<string, unknown>): void;
}

interface LoginPromptOptions {
  title?: string;
  message: string;
  redirectUrl?: string;
  durationMs?: number;
}

const DEFAULT_DURATION_MS = 4500;
const promptTimers = new WeakMap<object, ReturnType<typeof setTimeout>>();

export function getDefaultLoginPrompt(): LoginPromptViewData {
  return {
    visible: false,
    title: "登录后继续",
    message: "登录后可以保存和管理自己的装备。",
    redirectUrl: "/pages/profile/index",
  };
}

export function requireLoginForAction(
  page: PromptPage,
  options: LoginPromptOptions,
): boolean {
  if (hasAccessToken()) {
    return true;
  }
  showLoginPrompt(page, options);
  return false;
}

export function showLoginPrompt(
  page: PromptPage,
  options: LoginPromptOptions,
): void {
  clearPromptTimer(page);
  page.setData({
    loginPrompt: {
      visible: true,
      title: options.title ?? "登录后继续",
      message: options.message,
      redirectUrl: options.redirectUrl ?? currentPageUrl(),
    },
  });
  const timer = setTimeout(
    () => hideLoginPrompt(page),
    options.durationMs ?? DEFAULT_DURATION_MS,
  );
  promptTimers.set(page, timer);
}

export function hideLoginPrompt(page: PromptPage): void {
  clearPromptTimer(page);
  page.setData({
    "loginPrompt.visible": false,
  });
}

export function openLoginPageFromPrompt(page: PromptPage): void {
  clearPromptTimer(page);
  const redirectUrl = page.data?.loginPrompt?.redirectUrl || currentPageUrl();
  page.setData({
    "loginPrompt.visible": false,
  });
  wx.navigateTo({
    url: loginPageUrl(redirectUrl),
  });
}

export function loginPageUrl(redirectUrl: string): string {
  return `/pages/login/index?redirect=${encodeURIComponent(redirectUrl)}`;
}

function clearPromptTimer(page: PromptPage): void {
  const timer = promptTimers.get(page);
  if (timer) {
    clearTimeout(timer);
    promptTimers.delete(page);
  }
}

function currentPageUrl(): string {
  const pages = getCurrentPages();
  const current = pages[pages.length - 1];
  if (!current?.route) {
    return "/pages/index/index";
  }
  const options = current.options ?? {};
  const query = Object.entries(options)
    .filter(([, value]) => value !== undefined && value !== "")
    .map(
      ([key, value]) =>
        `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`,
    )
    .join("&");
  return `/${current.route}${query ? `?${query}` : ""}`;
}
