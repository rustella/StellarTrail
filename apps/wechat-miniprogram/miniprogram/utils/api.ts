import type {
  CreateGearRequest,
  GearCategoriesResponse,
  GearItem,
  GearStatsResponse,
  GearTemplate,
  ListGearTemplatesResponse,
  ListGearsRequest,
  ListGearsResponse,
  UpdateGearRequest,
  WechatLoginResponse,
} from "./gear-utils";
import type {
  KnotDetail,
  KnotListResponse,
  ListKnotsRequest,
  ListSkillsResponse,
  SkillLocale,
} from "./skill-utils";

const TOKEN_STORAGE_KEY = "stellartrail_access_token";
const ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_access_token_expires_at";
const REFRESH_TOKEN_STORAGE_KEY = "stellartrail_refresh_token";
const REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_refresh_token_expires_at";
const USER_STORAGE_KEY = "stellartrail_user";
const API_BASE_URL_STORAGE_KEY = "stellartrail_api_base_url";

let loginPromise: Promise<string> | null = null;
let refreshPromise: Promise<string> | null = null;
interface ApiRequestOptions {
  method?: "GET" | "POST" | "PATCH" | "DELETE";
  data?: unknown;
  auth?: boolean;
  locale?: SkillLocale;
}

interface WechatLoginRequest {
  code: string;
  profile?: {
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export interface EmailVerificationCodeRequest {
  email: string;
}

export interface EmailVerificationCodeResponse {
  email: string;
  expires_at: string;
  debug_code?: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  confirm_password: string;
  email_verification_code: string;
}

export interface CaptchaChallengeRequest {
  account: string;
}

export interface CaptchaChallengeResponse {
  captcha_ticket: string;
  captcha_type: "image";
  image_svg: string;
  expires_at: string;
  debug_answer?: string;
}

export interface PasswordLoginRequest {
  account: string;
  password: string;
  captcha_ticket?: string | null;
  captcha_answer?: string | null;
}

export interface ApiResponseCaptchaRequirement {
  type?: string;
  captcha_type?: string;
  endpoint?: string;
}

export interface ApiResponseFieldViolation {
  field?: string;
  message?: string;
}

interface ApiErrorBody {
  code?: string;
  message?: string;
  fields?: ApiResponseFieldViolation[];
  captcha?: ApiResponseCaptchaRequirement;
  parameter?: string;
}

export class LoginRequiredError extends Error {
  readonly code = "LOGIN_REQUIRED";

  constructor(message = "登录后继续") {
    super(message);
    this.name = "LoginRequiredError";
  }
}

export function isLoginRequiredError(
  error: unknown,
): error is LoginRequiredError {
  return (
    error instanceof LoginRequiredError ||
    (typeof error === "object" &&
      error !== null &&
      (error as { code?: unknown }).code === "LOGIN_REQUIRED")
  );
}

export class ApiResponseError extends Error {
  readonly statusCode: number;
  readonly code?: string;
  readonly fields?: ApiResponseFieldViolation[];
  readonly captcha?: ApiResponseCaptchaRequirement;
  readonly parameter?: string;
  readonly responseData: unknown;

  constructor(statusCode: number, data: unknown) {
    super(readErrorMessage(data, statusCode));
    this.name = "ApiResponseError";
    this.statusCode = statusCode;
    this.responseData = data;
    if (isApiErrorBody(data)) {
      this.code = data.code;
      this.fields = data.fields;
      this.captcha = data.captcha;
      this.parameter = data.parameter;
    }
  }
}

export function isApiResponseError(error: unknown): error is ApiResponseError {
  return (
    error instanceof ApiResponseError ||
    (typeof error === "object" &&
      error !== null &&
      typeof (error as { statusCode?: unknown }).statusCode === "number")
  );
}

export function isCaptchaRequiredError(error: unknown): boolean {
  return (
    isApiResponseError(error) &&
    (error.statusCode === 428 || error.code === "captcha_required")
  );
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

export function hasAccessToken(): boolean {
  const cached = wx.getStorageSync(TOKEN_STORAGE_KEY) as string | undefined;
  return Boolean(cached);
}

export function getStoredUser(): WechatLoginResponse["user"] | null {
  const user = wx.getStorageSync(USER_STORAGE_KEY) as
    | WechatLoginResponse["user"]
    | undefined;
  return user ?? null;
}

export async function ensureAccessToken(): Promise<string> {
  const cached = wx.getStorageSync(TOKEN_STORAGE_KEY) as string | undefined;
  if (cached && !shouldRefreshAccessToken()) {
    return cached;
  }
  if (wx.getStorageSync(REFRESH_TOKEN_STORAGE_KEY)) {
    try {
      return await refreshAccessToken();
    } catch {
      clearLoginState();
    }
  }
  throw new LoginRequiredError("登录后继续");
}

export async function loginWithWechat(): Promise<string> {
  if (!loginPromise) {
    loginPromise = runWechatLogin().finally(() => {
      loginPromise = null;
    });
  }
  return loginPromise;
}

export async function loginWithPassword(
  request: PasswordLoginRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>("/api/auth/login", {
    method: "POST",
    data: normalizePasswordLoginRequest(request),
  });
  saveLoginResponse(response);
  return response.access_token;
}

export async function registerWithPassword(
  request: RegisterRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/register",
    {
      method: "POST",
      data: request,
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export function sendEmailVerificationCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/auth/email-verification-code", {
    method: "POST",
    data: { email } satisfies EmailVerificationCodeRequest,
  });
}

export function createCaptcha(
  account: string,
): Promise<CaptchaChallengeResponse> {
  return requestJson("/api/auth/captcha", {
    method: "POST",
    data: { account } satisfies CaptchaChallengeRequest,
  });
}

async function runWechatLogin(): Promise<string> {
  const code = await getWechatLoginCode();
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/wechat-login",
    {
      method: "POST",
      data: {
        code,
        profile: {
          nickname: "寻径星野用户",
          avatar_url: null,
        },
      } satisfies WechatLoginRequest,
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export async function refreshAccessToken(): Promise<string> {
  if (!refreshPromise) {
    refreshPromise = refreshAccessTokenOnce().finally(() => {
      refreshPromise = null;
    });
  }
  return refreshPromise;
}

export function clearLoginState(): void {
  wx.removeStorageSync(TOKEN_STORAGE_KEY);
  wx.removeStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY);
  wx.removeStorageSync(REFRESH_TOKEN_STORAGE_KEY);
  wx.removeStorageSync(REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY);
  wx.removeStorageSync(USER_STORAGE_KEY);
}

export async function listGearTemplates(): Promise<ListGearTemplatesResponse> {
  return requestJson("/api/gear-templates");
}

export async function getGearTemplate(id: string): Promise<GearTemplate> {
  return requestJson(`/api/gear-templates/${encodeURIComponent(id)}`);
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

export async function listSkills(
  locale: SkillLocale = "zh-CN",
): Promise<ListSkillsResponse> {
  return requestJson("/api/skills", { locale });
}

export async function listKnots(
  request: ListKnotsRequest = {},
  locale: SkillLocale = "zh-CN",
): Promise<KnotListResponse> {
  return requestJson(`/api/skills/knots/list${queryString(request)}`, {
    locale,
  });
}

export async function getKnotDetail(
  id: string,
  locale: SkillLocale = "zh-CN",
): Promise<KnotDetail> {
  return requestJson(`/api/skills/knots/detail/${encodeURIComponent(id)}`, {
    locale,
  });
}

export function getErrorMessage(error: unknown): string {
  if (isApiResponseError(error)) {
    const fieldMessage = error.fields?.find((field) => field.message)?.message;
    return fieldMessage || error.message;
  }
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
  didRetryUnauthorized = false,
): Promise<T> {
  const token = options.auth ? await ensureAccessToken() : undefined;
  const header: Record<string, string> = {};
  if (options.data !== undefined) {
    header["content-type"] = "application/json";
  }
  if (options.locale) {
    header["X-StellarTrail-Locale"] = options.locale;
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
        if (
          response.statusCode === 401 &&
          options.auth &&
          !didRetryUnauthorized
        ) {
          void refreshAccessToken()
            .then(() => requestJson<T>(path, options, true))
            .then(resolve)
            .catch((error) => {
              clearLoginState();
              reject(
                error instanceof Error
                  ? error
                  : new Error("登录已过期，请重新登录"),
              );
            });
          return;
        }
        if (response.statusCode === 401 && options.auth) {
          clearLoginState();
          reject(new LoginRequiredError("登录状态已过期，请重新登录"));
          return;
        }
        if (response.statusCode === 403 && options.auth) {
          reject(new LoginRequiredError("当前账号暂无权限，请重新登录后再试"));
          return;
        }
        reject(new ApiResponseError(response.statusCode, response.data));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || "网络请求失败，请稍后再试"));
      },
    });
  });
}

function saveLoginResponse(response: WechatLoginResponse): void {
  wx.setStorageSync(TOKEN_STORAGE_KEY, response.access_token);
  wx.setStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY, response.expires_at);
  wx.setStorageSync(REFRESH_TOKEN_STORAGE_KEY, response.refresh_token);
  wx.setStorageSync(
    REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY,
    response.refresh_expires_at,
  );
  wx.setStorageSync(USER_STORAGE_KEY, response.user);
}

function normalizePasswordLoginRequest(
  request: PasswordLoginRequest,
): PasswordLoginRequest {
  return {
    account: request.account,
    password: request.password,
    ...(request.captcha_ticket
      ? { captcha_ticket: request.captcha_ticket }
      : {}),
    ...(request.captcha_answer
      ? { captcha_answer: request.captcha_answer }
      : {}),
  };
}

async function refreshAccessTokenOnce(): Promise<string> {
  const refreshToken = wx.getStorageSync(REFRESH_TOKEN_STORAGE_KEY) as
    | string
    | undefined;
  if (!refreshToken) {
    throw new Error("登录已过期，请重新登录");
  }
  const response = await requestJson<WechatLoginResponse>("/api/auth/refresh", {
    method: "POST",
    data: { refresh_token: refreshToken },
  });
  saveLoginResponse(response);
  return response.access_token;
}

function shouldRefreshAccessToken(): boolean {
  const expiresAt = wx.getStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY) as
    | string
    | undefined;
  if (!expiresAt) {
    return false;
  }
  const timestamp = Date.parse(expiresAt);
  return Number.isFinite(timestamp) && timestamp <= Date.now() + 60_000;
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
  if (
    isApiErrorBody(data) &&
    typeof data.message === "string" &&
    data.message
  ) {
    return data.message;
  }
  return `请求失败（${statusCode}）`;
}

function isApiErrorBody(data: unknown): data is ApiErrorBody {
  return typeof data === "object" && data !== null;
}
