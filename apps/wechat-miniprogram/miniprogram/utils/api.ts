import type {
  CreateGearRequest,
  GearCategoriesResponse,
  GearItem,
  GearStatsResponse,
  ListGearsRequest,
  ListGearsResponse,
  UpdateGearRequest,
  WechatLoginResponse,
} from "./gear-utils";
import type { ListSkillsResponse, SkillContent } from "./skill-utils";

const TOKEN_STORAGE_KEY = "stellartrail_access_token";
const USER_STORAGE_KEY = "stellartrail_user";
const API_BASE_URL_STORAGE_KEY = "stellartrail_api_base_url";

let loginPromise: Promise<string> | null = null;

interface ApiRequestOptions {
  method?: "GET" | "POST" | "PATCH" | "DELETE";
  data?: unknown;
  auth?: boolean;
}

interface WechatLoginRequest {
  code: string;
  profile?: {
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export function getApiBaseUrl(): string {
  const stored = wx.getStorageSync(API_BASE_URL_STORAGE_KEY) as
    | string
    | undefined;
  if (stored) {
    return stored.replace(/\/$/, "");
  }
  const app = getApp<{
    globalData?: {
      apiBaseUrl?: string;
    };
  }>();
  return (app.globalData?.apiBaseUrl ?? "http://127.0.0.1:8080").replace(
    /\/$/,
    "",
  );
}

export function setApiBaseUrl(baseUrl: string): void {
  wx.setStorageSync(API_BASE_URL_STORAGE_KEY, baseUrl.replace(/\/$/, ""));
}

export async function ensureAccessToken(): Promise<string> {
  const cached = wx.getStorageSync(TOKEN_STORAGE_KEY) as string | undefined;
  if (cached) {
    return cached;
  }
  if (!loginPromise) {
    loginPromise = loginWithWechat().finally(() => {
      loginPromise = null;
    });
  }
  return loginPromise;
}

export async function loginWithWechat(): Promise<string> {
  const code = await getWechatLoginCode();
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/wechat-login",
    {
      method: "POST",
      data: {
        code,
        profile: {
          nickname: "小程序本地用户",
          avatar_url: null,
        },
      } satisfies WechatLoginRequest,
    },
  );
  wx.setStorageSync(TOKEN_STORAGE_KEY, response.access_token);
  wx.setStorageSync(USER_STORAGE_KEY, response.user);
  return response.access_token;
}

export function clearLoginState(): void {
  wx.removeStorageSync(TOKEN_STORAGE_KEY);
  wx.removeStorageSync(USER_STORAGE_KEY);
}

export async function listGearCategories(
  tab: "available" | "history",
): Promise<GearCategoriesResponse> {
  return requestJson(`/api/me/gears/categories${queryString({ tab })}`, {
    auth: true,
  });
}

export async function getGearStats(
  tab: "available" | "history",
): Promise<GearStatsResponse> {
  return requestJson(`/api/me/gears/stats${queryString({ tab })}`, {
    auth: true,
  });
}

export async function listGears(
  request: ListGearsRequest,
): Promise<ListGearsResponse> {
  return requestJson(`/api/me/gears${queryString(request)}`, { auth: true });
}

export async function getGear(id: string): Promise<GearItem> {
  return requestJson(`/api/me/gears/${encodeURIComponent(id)}`, { auth: true });
}

export async function createGear(
  request: CreateGearRequest,
): Promise<GearItem> {
  return requestJson("/api/me/gears", {
    method: "POST",
    data: request,
    auth: true,
  });
}

export async function updateGear(
  id: string,
  request: UpdateGearRequest,
): Promise<GearItem> {
  return requestJson(`/api/me/gears/${encodeURIComponent(id)}`, {
    method: "PATCH",
    data: request,
    auth: true,
  });
}

export async function archiveGear(id: string): Promise<void> {
  await requestJson<void>(`/api/me/gears/${encodeURIComponent(id)}`, {
    method: "DELETE",
    auth: true,
  });
}

export async function restoreGear(id: string): Promise<GearItem> {
  return requestJson(`/api/me/gears/${encodeURIComponent(id)}/restore`, {
    method: "POST",
    auth: true,
  });
}

export async function listSkills(): Promise<ListSkillsResponse> {
  return requestJson("/api/skills");
}

export async function getSkill(id: string): Promise<SkillContent> {
  return requestJson(`/api/skills/${encodeURIComponent(id)}`);
}

export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return "操作失败，请稍后重试";
}

async function requestJson<T>(
  path: string,
  options: ApiRequestOptions = {},
): Promise<T> {
  const token = options.auth ? await ensureAccessToken() : undefined;
  const header: Record<string, string> = {};
  if (options.data !== undefined) {
    header["content-type"] = "application/json";
  }
  if (token) {
    header.authorization = `Bearer ${token}`;
  }

  return new Promise<T>((resolve, reject) => {
    wx.request({
      url: `${getApiBaseUrl()}${path}`,
      method: (options.method ?? "GET") as any,
      data: options.data as any,
      header,
      success: (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(response.data as T);
          return;
        }
        if (response.statusCode === 401) {
          clearLoginState();
        }
        reject(new Error(readErrorMessage(response.data, response.statusCode)));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || "网络请求失败，请检查 API 服务"));
      },
    });
  });
}

function getWechatLoginCode(): Promise<string> {
  return new Promise((resolve) => {
    wx.login({
      success: (result) => {
        resolve(result.code || "local-dev-user");
      },
      fail: () => {
        resolve("local-dev-user");
      },
    });
  });
}

function queryString(params: object): string {
  const parts: string[] = [];
  Object.entries(params as Record<string, unknown>).forEach(([key, value]) => {
    if (
      value !== undefined &&
      value !== null &&
      value !== "" &&
      value !== "all"
    ) {
      parts.push(
        `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`,
      );
    }
  });
  return parts.length > 0 ? `?${parts.join("&")}` : "";
}

function readErrorMessage(data: unknown, statusCode: number): string {
  if (typeof data === "object" && data !== null && "message" in data) {
    const message = (data as { message?: unknown }).message;
    if (typeof message === "string" && message) {
      return message;
    }
  }
  return `请求失败（${statusCode}）`;
}
