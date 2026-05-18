import type {
  CreateGearRequest,
  CreateGearAtlasSubmissionRequest,
  GearCategoriesResponse,
  GearAtlasPublicItem,
  GearAtlasSubmission,
  GearCategory,
  GearItem,
  GearSpecKeyRankingsResponse,
  GearStatsResponse,
  GearTagSuggestionsResponse,
  GearTemplate,
  ListGearTemplatesResponse,
  ListGearAtlasRequest,
  ListGearAtlasResponse,
  ListGearAtlasSubmissionsResponse,
  ListGearsRequest,
  ListGearsResponse,
  UpdateGearRequest,
  WechatLoginResponse,
} from "./gear-utils";
import type {
  KnotDetail,
  KnotListResponse,
  KnotOfflineManifestResponse,
  ListKnotsRequest,
  ListSkillsResponse,
  SkillLocale,
} from "./skill-utils";
import {
  clearUserOfflineCaches,
  makeOfflineCacheKey,
  readOfflineCache,
  writeOfflineCache,
  type OfflineCacheDescriptor,
} from "./offline-cache";
import {
  OfflineCacheMissError,
  OfflineWriteBlockedError,
  isOffline,
  isOfflineCacheMissError,
  isOfflineWriteBlockedError,
  markNetworkFailure,
  OFFLINE_CACHE_NOTICE,
} from "./network-state";
export {
  isOfflineCacheMissError,
  isOfflineWriteBlockedError,
} from "./network-state";

const TOKEN_STORAGE_KEY = "stellartrail_access_token";
const ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_access_token_expires_at";
const REFRESH_TOKEN_STORAGE_KEY = "stellartrail_refresh_token";
const REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_refresh_token_expires_at";
const USER_STORAGE_KEY = "stellartrail_user";
const API_BASE_URL_STORAGE_KEY = "stellartrail_api_base_url";
const DEFAULT_API_BASE_URL = "https://api.example.invalid";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";
const API_REQUEST_TIMEOUT_MS = 15_000;
const WECHAT_LOGIN_TIMEOUT_MS = 5_000;

let loginPromise: Promise<string> | null = null;
let refreshPromise: Promise<string> | null = null;
let offlineCacheNoticePending = false;
interface ApiRequestOptions {
  method?: "GET" | "POST" | "PATCH" | "DELETE";
  data?: unknown;
  auth?: boolean;
  locale?: SkillLocale;
  cache?: boolean;
}

export interface WechatLoginProfile {
  nickname?: string | null;
  avatar_url?: string | null;
}

interface WechatLoginRequest {
  code: string;
  profile?: WechatLoginProfile;
}

interface ProfileUserResponse {
  user: WechatLoginResponse["user"];
}

export interface EmailVerificationCodeRequest {
  email: string;
}

export interface EmailVerificationCodeResponse {
  email: string;
  expires_at: string;
  debug_code?: string;
}

export interface EmailLoginCodeRequest {
  email: string;
}

export interface EmailLoginRequest {
  email: string;
  email_verification_code: string;
}

export interface PasswordResetCodeRequest {
  email: string;
}

export interface PasswordResetRequest {
  email: string;
  email_verification_code: string;
  password: string;
  confirm_password: string;
}

export interface BindEmailCodeRequest {
  email: string;
}

export interface BindEmailRequest {
  email: string;
  email_verification_code: string;
}

interface BindEmailResponse {
  user: WechatLoginResponse["user"];
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

export function isNotFoundApiError(error: unknown): boolean {
  return isApiResponseError(error) && error.statusCode === 404;
}

export function consumeOfflineCacheNotice(): string {
  if (!offlineCacheNoticePending) {
    return "";
  }
  offlineCacheNoticePending = false;
  return OFFLINE_CACHE_NOTICE;
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
  return (app.globalData?.apiBaseUrl ?? DEFAULT_API_BASE_URL).replace(
    /\/$/,
    "",
  );
}

export function getAssetsBaseUrl(): string {
  const app = getApp<{
    globalData?: {
      assetsBaseUrl?: string;
    };
  }>();
  return (app.globalData?.assetsBaseUrl ?? DEFAULT_ASSETS_BASE_URL).replace(
    /\/$/,
    "",
  );
}

export function resolveAssetUrl(pathOrUrl: string): string {
  if (/^https?:\/\//i.test(pathOrUrl)) {
    return pathOrUrl;
  }
  return `${getAssetsBaseUrl()}/${pathOrUrl.replace(/^\/+/, "")}`;
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

export async function getCurrentUser(): Promise<WechatLoginResponse["user"]> {
  const response = await requestJson<ProfileUserResponse>("/api/me/profile", {
    auth: true,
  });
  saveUser(response.user);
  return response.user;
}

export function sendBindEmailCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/me/email-binding-code", {
    method: "POST",
    auth: true,
    data: { email } satisfies BindEmailCodeRequest,
  });
}

export async function bindEmailToCurrentAccount(
  request: BindEmailRequest,
): Promise<WechatLoginResponse["user"]> {
  const response = await requestJson<BindEmailResponse>(
    "/api/me/email-binding",
    {
      method: "POST",
      auth: true,
      data: request,
    },
  );
  saveUser(response.user);
  return response.user;
}

export async function ensureAccessToken(): Promise<string> {
  const cached = wx.getStorageSync(TOKEN_STORAGE_KEY) as string | undefined;
  if (cached && (isOffline() || !shouldRefreshAccessToken())) {
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

export async function loginWithWechat(
  profile?: WechatLoginProfile,
): Promise<string> {
  if (!loginPromise) {
    loginPromise = runWechatLogin(profile).finally(() => {
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

export function sendEmailLoginCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/auth/email-login-code", {
    method: "POST",
    data: { email } satisfies EmailLoginCodeRequest,
  });
}

export async function loginWithEmailCode(
  request: EmailLoginRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/email-login",
    {
      method: "POST",
      data: request,
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export function sendPasswordResetCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/auth/password-reset-code", {
    method: "POST",
    data: { email } satisfies PasswordResetCodeRequest,
  });
}

export async function resetPassword(
  request: PasswordResetRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/password-reset",
    {
      method: "POST",
      data: request,
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export function createCaptcha(
  account: string,
): Promise<CaptchaChallengeResponse> {
  return requestJson("/api/auth/captcha", {
    method: "POST",
    data: { account } satisfies CaptchaChallengeRequest,
  });
}

async function runWechatLogin(profile?: WechatLoginProfile): Promise<string> {
  const code = await getWechatLoginCode();
  const normalizedProfile = normalizeWechatLoginProfile(profile);
  const response = await requestJson<WechatLoginResponse>(
    "/api/auth/wechat-login",
    {
      method: "POST",
      data: {
        code,
        ...(normalizedProfile ? { profile: normalizedProfile } : {}),
      } satisfies WechatLoginRequest,
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export async function uploadWechatAvatar(
  filePath: string,
): Promise<WechatLoginResponse["user"]> {
  if (isOffline()) {
    throw new OfflineWriteBlockedError();
  }
  const token = await ensureAccessToken();
  try {
    const response = await uploadWechatAvatarOnce(filePath, token);
    saveUser(response.user);
    return response.user;
  } catch (error) {
    if (isApiResponseError(error) && error.statusCode === 401) {
      const refreshedToken = await refreshAccessToken();
      const response = await uploadWechatAvatarOnce(filePath, refreshedToken);
      saveUser(response.user);
      return response.user;
    }
    throw error;
  }
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
  clearUserOfflineCaches();
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

export async function listGearAtlas(
  request: ListGearAtlasRequest = {},
): Promise<ListGearAtlasResponse> {
  return requestJson(`/api/gear-atlas${queryString(request)}`);
}

export async function getGearAtlasItem(
  id: string,
): Promise<GearAtlasPublicItem> {
  return requestJson(`/api/gear-atlas/${encodeURIComponent(id)}`);
}

export async function createGearAtlasSubmission(
  request: CreateGearAtlasSubmissionRequest,
): Promise<GearAtlasSubmission> {
  return requestJson("/api/me/gear-atlas-submissions", {
    method: "POST",
    data: request,
    auth: true,
  });
}

export async function submitGearToAtlas(
  id: string,
): Promise<GearAtlasSubmission> {
  return requestJson(
    `/api/me/gears/${encodeURIComponent(id)}/atlas-submission`,
    {
      method: "POST",
      auth: true,
    },
  );
}

export async function listMyGearAtlasSubmissions(
  request: { limit?: number; cursor?: string } = {},
): Promise<ListGearAtlasSubmissionsResponse> {
  return requestJson(`/api/me/gear-atlas-submissions${queryString(request)}`, {
    auth: true,
  });
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

export async function getGearSpecKeyRankings(
  category: GearCategory,
): Promise<GearSpecKeyRankingsResponse> {
  return requestJson(
    `/api/me/gears/spec-key-rankings${queryString({ category })}`,
    { auth: true },
  );
}

export async function getGearTagSuggestions(
  limit = 20,
): Promise<GearTagSuggestionsResponse> {
  return requestJson(`/api/me/gears/tag-suggestions${queryString({ limit })}`, {
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
  return requestJson(knotListPath(request), {
    locale,
  });
}

export async function getKnotDetail(
  id: string,
  locale: SkillLocale = "zh-CN",
): Promise<KnotDetail> {
  return requestJson(knotDetailPath(id), { locale });
}

export async function getKnotOfflineManifest(
  locale: SkillLocale = "zh-CN",
): Promise<KnotOfflineManifestResponse> {
  return requestJson("/api/skills/knots/offline-manifest", {
    locale,
    cache: false,
  });
}

export function knotListPath(request: ListKnotsRequest = {}): string {
  return `/api/skills/knots/list${queryString(request)}`;
}

export function knotDetailPath(id: string): string {
  return `/api/skills/knots/detail/${encodeURIComponent(id)}`;
}

export function getErrorMessage(error: unknown): string {
  if (isOfflineCacheMissError(error) || isOfflineWriteBlockedError(error)) {
    return error.message;
  }
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
  const method = options.method ?? "GET";
  if (method !== "GET" && isOffline()) {
    throw new OfflineWriteBlockedError();
  }
  const token = options.auth ? await ensureAccessToken() : undefined;
  const cacheDescriptor = offlineCacheDescriptor(path, options);
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
      method: method as any,
      data: options.data as any,
      header,
      timeout: API_REQUEST_TIMEOUT_MS,
      success: (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          if (cacheDescriptor) {
            writeOfflineCache(cacheDescriptor, response.data as T);
          }
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
        markNetworkFailure();
        if (cacheDescriptor) {
          const cached = readOfflineCache<T>(cacheDescriptor);
          if (cached) {
            offlineCacheNoticePending = true;
            resolve(cached.data);
            return;
          }
        }
        if (method !== "GET") {
          reject(new Error(requestFailureMessage(error.errMsg)));
          return;
        }
        reject(new OfflineCacheMissError());
      },
    });
  });
}

function offlineCacheDescriptor(
  path: string,
  options: ApiRequestOptions,
): OfflineCacheDescriptor | null {
  if (options.cache === false) {
    return null;
  }
  if ((options.method ?? "GET") !== "GET") {
    return null;
  }
  const locale = options.locale;
  if (isPublicCacheablePath(path)) {
    return {
      key: makeOfflineCacheKey(path, { locale }),
      scope: "public",
      ...(locale ? { locale } : {}),
    };
  }
  if (options.auth && isUserCacheablePath(path)) {
    const userId = getStoredUser()?.id;
    if (!userId) {
      return null;
    }
    return {
      key: makeOfflineCacheKey(path, { locale, userId }),
      scope: "user",
      userId,
      ...(locale ? { locale } : {}),
    };
  }
  return null;
}

function isPublicCacheablePath(path: string): boolean {
  return (
    path === "/api/skills" ||
    path.startsWith("/api/skills/") ||
    path === "/api/gear-templates" ||
    path.startsWith("/api/gear-templates/") ||
    path === "/api/gear-atlas" ||
    path.startsWith("/api/gear-atlas?") ||
    path.startsWith("/api/gear-atlas/")
  );
}

function isUserCacheablePath(path: string): boolean {
  return (
    path === "/api/me/gears" ||
    path.startsWith("/api/me/gears?") ||
    path.startsWith("/api/me/gears/") ||
    path === "/api/me/gear-atlas-submissions" ||
    path.startsWith("/api/me/gear-atlas-submissions?")
  );
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

function saveUser(user: WechatLoginResponse["user"]): void {
  wx.setStorageSync(USER_STORAGE_KEY, user);
}

function normalizeWechatLoginProfile(
  profile?: WechatLoginProfile,
): WechatLoginProfile | undefined {
  if (!profile) {
    return undefined;
  }
  const nickname = normalizeOptionalString(profile.nickname);
  const avatarUrl = normalizeOptionalString(profile.avatar_url);
  if (!nickname && !avatarUrl) {
    return undefined;
  }
  return {
    ...(nickname ? { nickname } : {}),
    ...(avatarUrl ? { avatar_url: avatarUrl } : {}),
  };
}

function normalizeOptionalString(value?: string | null): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }
  const trimmed = value.trim();
  return trimmed || undefined;
}

function uploadWechatAvatarOnce(
  filePath: string,
  token: string,
): Promise<ProfileUserResponse> {
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${getApiBaseUrl()}/api/me/profile/avatar`,
      filePath,
      name: "file",
      header: {
        authorization: `Bearer ${token}`,
      },
      success: (response) => {
        const data = parseUploadResponseData(response.data);
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(data as ProfileUserResponse);
          return;
        }
        reject(new ApiResponseError(response.statusCode, data));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || "头像上传失败，请稍后再试"));
      },
    });
  });
}

function parseUploadResponseData(data: string | object): unknown {
  if (typeof data !== "string") {
    return data;
  }
  if (!data) {
    return {};
  }
  try {
    return JSON.parse(data) as unknown;
  } catch {
    return { message: data };
  }
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
    let settled = false;
    const finish = (code: string) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      resolve(code);
    };
    const timer = setTimeout(() => {
      finish("local-dev-user");
    }, WECHAT_LOGIN_TIMEOUT_MS);

    wx.login({
      success: (result) => {
        finish(result.code || "local-dev-user");
      },
      fail: () => {
        finish("local-dev-user");
      },
    });
  });
}

function requestFailureMessage(errMsg?: string): string {
  if (errMsg && /timeout/i.test(errMsg)) {
    return "网络请求超时，请稍后再试";
  }
  return "网络请求失败，请检查网络后重试";
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
  return "服务暂时不可用，请稍后再试";
}

function isApiErrorBody(data: unknown): data is ApiErrorBody {
  return typeof data === "object" && data !== null;
}
