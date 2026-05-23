import type {
  CreateGearRequest,
  CreateGearAtlasSubmissionRequest,
  CreateGearPackingListRequest,
  GearCategoriesResponse,
  GearAtlasPublicItem,
  GearAtlasSubmission,
  GearCategory,
  GearItem,
  GearOverviewResponse,
  GearPackingListDetail,
  GearSpecKeyRankingsResponse,
  GearStatsResponse,
  GearTagSuggestionsResponse,
  GearTemplate,
  ListGearTemplatesResponse,
  ListGearAtlasRequest,
  ListGearAtlasResponse,
  ListGearAtlasSubmissionsResponse,
  ListGearPackingListsResponse,
  ListGearsRequest,
  ListGearsResponse,
  UpdateGearPackingListRequest,
  UpdateGearRequest,
  WechatLoginResponse,
} from "./gear-utils";
import type {
  KnotDetail,
  KnotFiltersResponse,
  KnotListResponse,
  KnotOfflineManifestResponse,
  ListKnotsRequest,
  ListSkillsResponse,
  SkillLocale,
} from "./skill-utils";
import type { ClientDomainCandidate } from "./client-config";
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
const ASSETS_BASE_URL_STORAGE_KEY = "stellartrail_assets_base_url";
const DEFAULT_API_BASE_URL = "https://api.example.invalid";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";
const API_PREFIX = "/api/v1";
const HEALTH_PATH = "/healthz";
const API_REQUEST_TIMEOUT_MS = 15_000;
const API_DOMAIN_HEALTH_TIMEOUT_MS = 3_000;
const WECHAT_LOGIN_TIMEOUT_MS = 5_000;

let loginPromise: Promise<string> | null = null;
let refreshPromise: Promise<string> | null = null;
let domainProbePromise: Promise<void> | null = null;
let domainProbeCompleted = false;
let offlineCacheNoticePending = false;
let cachedApiBaseUrl: string | null = null;
let cachedAssetsBaseUrl: string | null = null;
let cachedAccessToken: string | null | undefined;
let cachedAccessTokenExpiresAt: string | null | undefined;
let cachedRefreshToken: string | null | undefined;
let cachedUser: WechatLoginResponse["user"] | null | undefined;
const pendingGetRequests = new Map<string, Promise<unknown>>();
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

export type FeedbackCategory =
  | "suggestion"
  | "bug"
  | "content_correction"
  | "other";

export interface CreateFeedbackRequest {
  category: FeedbackCategory;
  content: string;
  contact?: string | null;
  page?: string | null;
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
  image_ids?: string[];
}

export interface FeedbackResponse {
  id: string;
  category: string;
  content: string;
  contact?: string | null;
  page?: string | null;
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
  status: string;
  images: unknown[];
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface UploadImageResponse {
  id: string;
  purpose: string;
  original_filename: string;
  image_type: string;
  content_type: string;
  size_bytes: number;
  sha256: string;
  download_url: string;
  is_deleted: boolean;
  created_at: string;
}

export type ClientKey =
  | "wechat_miniprogram"
  | "web"
  | "android"
  | "ios"
  | "macos";

export interface ClientVersion {
  id: string;
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes: string[];
  status: "draft" | "published";
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListClientVersionsResponse {
  items: ClientVersion[];
  next_cursor?: string | null;
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
  if (cachedApiBaseUrl) {
    return cachedApiBaseUrl;
  }
  const stored = wx.getStorageSync(API_BASE_URL_STORAGE_KEY) as
    | string
    | undefined;
  if (stored) {
    const normalized = normalizeStoredApiBaseUrl(stored);
    if (normalized) {
      cachedApiBaseUrl = normalized;
      return normalized;
    }
    wx.removeStorageSync(API_BASE_URL_STORAGE_KEY);
  }
  const app = getApp<{
    globalData?: {
      apiBaseUrl?: string;
    };
  }>();
  cachedApiBaseUrl = (
    app.globalData?.apiBaseUrl ?? DEFAULT_API_BASE_URL
  ).replace(/\/$/, "");
  return cachedApiBaseUrl;
}

export function getAssetsBaseUrl(): string {
  if (cachedAssetsBaseUrl) {
    return cachedAssetsBaseUrl;
  }
  const stored = wx.getStorageSync(ASSETS_BASE_URL_STORAGE_KEY) as
    | string
    | undefined;
  if (stored) {
    const normalized = normalizeStoredBaseUrl(stored);
    if (normalized) {
      cachedAssetsBaseUrl = normalized;
      return normalized;
    }
    wx.removeStorageSync(ASSETS_BASE_URL_STORAGE_KEY);
  }
  const app = getApp<{
    globalData?: {
      assetsBaseUrl?: string;
    };
  }>();
  cachedAssetsBaseUrl = (
    app.globalData?.assetsBaseUrl ?? DEFAULT_ASSETS_BASE_URL
  ).replace(/\/$/, "");
  return cachedAssetsBaseUrl;
}

export function resolveAssetUrl(pathOrUrl: string): string {
  if (/^https?:\/\//i.test(pathOrUrl)) {
    return normalizeKnownAssetUrl(pathOrUrl);
  }
  return `${getAssetsBaseUrl()}/${pathOrUrl.replace(/^\/+/, "")}`;
}

export function setApiBaseUrl(baseUrl: string): void {
  cachedApiBaseUrl = baseUrl.replace(/\/$/, "");
  domainProbeCompleted = true;
  wx.setStorageSync(API_BASE_URL_STORAGE_KEY, cachedApiBaseUrl);
}

function normalizeStoredApiBaseUrl(baseUrl: string): string | null {
  return normalizeStoredBaseUrl(baseUrl);
}

function normalizeStoredBaseUrl(baseUrl: string): string | null {
  const normalized = baseUrl.trim().replace(/\/$/, "");
  const match = normalized.match(/^https?:\/\/([^/:?#]+)(?::\d+)?(?:[/?#]|$)/i);
  if (!match) {
    return null;
  }
  const hostname = match[1].toLowerCase();
  if (hostname.endsWith(".local")) {
    return null;
  }
  return normalized;
}

function normalizeKnownAssetUrl(url: string): string {
  const parsed = parseUrlParts(url);
  if (!parsed || !isKnownAssetsHost(parsed.hostname)) {
    return url;
  }
  return `${getAssetsBaseUrl()}${parsed.path}`;
}

function isKnownAssetsHost(hostname: string): boolean {
  if (parseUrlParts(getAssetsBaseUrl())?.hostname === hostname) {
    return true;
  }
  return getDomainCandidates().some((candidate) => {
    const parsed = parseUrlParts(candidate.assetsBaseUrl);
    return parsed?.hostname === hostname;
  });
}

function parseUrlParts(url: string): { hostname: string; path: string } | null {
  const match = url.match(/^https?:\/\/([^/:?#]+)(?::\d+)?([/?#].*)?$/i);
  if (!match) {
    return null;
  }
  return {
    hostname: match[1].toLowerCase(),
    path: match[2] || "",
  };
}

export function hasAccessToken(): boolean {
  return Boolean(readAccessToken());
}

export function getStoredUser(): WechatLoginResponse["user"] | null {
  if (cachedUser !== undefined) {
    return cachedUser ? normalizeUserAssets(cachedUser) : cachedUser;
  }
  const stored =
    (wx.getStorageSync(USER_STORAGE_KEY) as
      | WechatLoginResponse["user"]
      | undefined) ?? null;
  cachedUser = stored ? normalizeUserAssets(stored) : null;
  return cachedUser;
}

export async function getCurrentUser(): Promise<WechatLoginResponse["user"]> {
  const response = await requestJson<ProfileUserResponse>(
    "/api/v1/me/profile",
    {
      auth: true,
    },
  );
  saveUser(response.user);
  return response.user;
}

export function sendBindEmailCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/v1/me/email-binding-code", {
    method: "POST",
    auth: true,
    data: { email } satisfies BindEmailCodeRequest,
  });
}

export async function bindEmailToCurrentAccount(
  request: BindEmailRequest,
): Promise<WechatLoginResponse["user"]> {
  const response = await requestJson<BindEmailResponse>(
    "/api/v1/me/email-binding",
    {
      method: "POST",
      auth: true,
      data: request,
    },
  );
  saveUser(response.user);
  return response.user;
}

export function createFeedback(
  request: CreateFeedbackRequest,
): Promise<FeedbackResponse> {
  return requestJson("/api/v1/me/feedback", {
    method: "POST",
    auth: true,
    data: request,
  });
}

export async function uploadFeedbackImage(
  filePath: string,
): Promise<UploadImageResponse> {
  if (isOffline()) {
    throw new OfflineWriteBlockedError();
  }
  const token = await ensureAccessToken();
  try {
    return await uploadFeedbackImageOnce(filePath, token);
  } catch (error) {
    if (isApiResponseError(error) && error.statusCode === 401) {
      const refreshedToken = await refreshAccessToken();
      return uploadFeedbackImageOnce(filePath, refreshedToken);
    }
    throw error;
  }
}

export async function ensureAccessToken(): Promise<string> {
  const cached = readAccessToken();
  if (cached && (isOffline() || !shouldRefreshAccessToken())) {
    return cached;
  }
  if (readRefreshToken()) {
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
  const response = await requestJson<WechatLoginResponse>(
    "/api/v1/auth/login",
    {
      method: "POST",
      data: normalizePasswordLoginRequest(request),
    },
  );
  saveLoginResponse(response);
  return response.access_token;
}

export async function registerWithPassword(
  request: RegisterRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/v1/auth/register",
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
  return requestJson("/api/v1/auth/email-verification-code", {
    method: "POST",
    data: { email } satisfies EmailVerificationCodeRequest,
  });
}

export function sendEmailLoginCode(
  email: string,
): Promise<EmailVerificationCodeResponse> {
  return requestJson("/api/v1/auth/email-login-code", {
    method: "POST",
    data: { email } satisfies EmailLoginCodeRequest,
  });
}

export async function loginWithEmailCode(
  request: EmailLoginRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/v1/auth/email-login",
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
  return requestJson("/api/v1/auth/password-reset-code", {
    method: "POST",
    data: { email } satisfies PasswordResetCodeRequest,
  });
}

export async function resetPassword(
  request: PasswordResetRequest,
): Promise<string> {
  const response = await requestJson<WechatLoginResponse>(
    "/api/v1/auth/password-reset",
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
  return requestJson("/api/v1/auth/captcha", {
    method: "POST",
    data: { account } satisfies CaptchaChallengeRequest,
  });
}

async function runWechatLogin(profile?: WechatLoginProfile): Promise<string> {
  const code = await getWechatLoginCode();
  const normalizedProfile = normalizeWechatLoginProfile(profile);
  const response = await requestJson<WechatLoginResponse>(
    "/api/v1/auth/wechat-login",
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
  cachedAccessToken = null;
  cachedAccessTokenExpiresAt = null;
  cachedRefreshToken = null;
  cachedUser = null;
  pendingGetRequests.clear();
  wx.removeStorageSync(TOKEN_STORAGE_KEY);
  wx.removeStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY);
  wx.removeStorageSync(REFRESH_TOKEN_STORAGE_KEY);
  wx.removeStorageSync(REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY);
  wx.removeStorageSync(USER_STORAGE_KEY);
}

export async function listGearTemplates(): Promise<ListGearTemplatesResponse> {
  return requestJson("/api/v1/gear-templates");
}

export async function getGearTemplate(id: string): Promise<GearTemplate> {
  return requestJson(`/api/v1/gear-templates/${encodeURIComponent(id)}`);
}

export async function listGearAtlas(
  request: ListGearAtlasRequest = {},
): Promise<ListGearAtlasResponse> {
  return requestJson(`/api/v1/gear-atlas${queryString(request)}`);
}

export async function getGearAtlasItem(
  id: string,
): Promise<GearAtlasPublicItem> {
  return requestJson(`/api/v1/gear-atlas/${encodeURIComponent(id)}`);
}

export async function createGearAtlasSubmission(
  request: CreateGearAtlasSubmissionRequest,
): Promise<GearAtlasSubmission> {
  return requestJson("/api/v1/me/gear-atlas-submissions", {
    method: "POST",
    data: request,
    auth: true,
  });
}

export async function submitGearToAtlas(
  id: string,
): Promise<GearAtlasSubmission> {
  return requestJson(
    `/api/v1/me/gears/${encodeURIComponent(id)}/atlas-submission`,
    {
      method: "POST",
      auth: true,
    },
  );
}

export async function listMyGearAtlasSubmissions(
  request: { limit?: number; cursor?: string } = {},
): Promise<ListGearAtlasSubmissionsResponse> {
  return requestJson(
    `/api/v1/me/gear-atlas-submissions${queryString(request)}`,
    {
      auth: true,
    },
  );
}

export async function listGearCategories(
  tab: "available" | "history",
): Promise<GearCategoriesResponse> {
  return requestJson(`/api/v1/me/gears/categories${queryString({ tab })}`, {
    auth: true,
  });
}

export async function getGearStats(
  tab: "available" | "history",
): Promise<GearStatsResponse> {
  return requestJson(`/api/v1/me/gears/stats${queryString({ tab })}`, {
    auth: true,
  });
}

export async function getGearSpecKeyRankings(
  category: GearCategory,
): Promise<GearSpecKeyRankingsResponse> {
  return requestJson(
    `/api/v1/me/gears/spec-key-rankings${queryString({ category })}`,
    { auth: true },
  );
}

export async function getGearOverview(request: {
  tab?: "available" | "history";
  limit?: number;
  sort?: string;
}): Promise<GearOverviewResponse> {
  return requestJson(`/api/v1/me/gears/overview${queryString(request)}`, {
    auth: true,
  });
}

export async function getGearTagSuggestions(
  limit = 20,
): Promise<GearTagSuggestionsResponse> {
  return requestJson(
    `/api/v1/me/gears/tag-suggestions${queryString({ limit })}`,
    {
      auth: true,
    },
  );
}

export async function listGears(
  request: ListGearsRequest,
): Promise<ListGearsResponse> {
  return requestJson(`/api/v1/me/gears${queryString(request)}`, { auth: true });
}

export async function listGearPackingLists(
  request: { limit?: number; cursor?: string } = {},
): Promise<ListGearPackingListsResponse> {
  return requestJson(`/api/v1/me/packing-lists${queryString(request)}`, {
    auth: true,
  });
}

export async function createGearPackingList(
  request: CreateGearPackingListRequest,
): Promise<GearPackingListDetail> {
  return requestJson("/api/v1/me/packing-lists", {
    method: "POST",
    data: request,
    auth: true,
  });
}

export async function getGearPackingList(
  id: string,
): Promise<GearPackingListDetail> {
  return requestJson(`/api/v1/me/packing-lists/${encodeURIComponent(id)}`, {
    auth: true,
  });
}

export async function updateGearPackingList(
  id: string,
  request: UpdateGearPackingListRequest,
): Promise<GearPackingListDetail> {
  return requestJson(`/api/v1/me/packing-lists/${encodeURIComponent(id)}`, {
    method: "PATCH",
    data: request,
    auth: true,
  });
}

export async function deleteGearPackingList(id: string): Promise<void> {
  await requestJson<void>(
    `/api/v1/me/packing-lists/${encodeURIComponent(id)}`,
    {
      method: "DELETE",
      auth: true,
    },
  );
}

export async function addGearPackingItems(
  id: string,
  gearIds: string[],
): Promise<GearPackingListDetail> {
  return requestJson(
    `/api/v1/me/packing-lists/${encodeURIComponent(id)}/items`,
    {
      method: "POST",
      data: { gear_ids: gearIds },
      auth: true,
    },
  );
}

export async function updateGearPackingItem(
  id: string,
  itemId: string,
  packed: boolean,
): Promise<GearPackingListDetail> {
  return requestJson(
    `/api/v1/me/packing-lists/${encodeURIComponent(id)}/items/${encodeURIComponent(itemId)}`,
    {
      method: "PATCH",
      data: { packed },
      auth: true,
    },
  );
}

export async function removeGearPackingItem(
  id: string,
  itemId: string,
): Promise<GearPackingListDetail> {
  return requestJson(
    `/api/v1/me/packing-lists/${encodeURIComponent(id)}/items/${encodeURIComponent(itemId)}`,
    {
      method: "DELETE",
      auth: true,
    },
  );
}

export async function getGear(id: string): Promise<GearItem> {
  return requestJson(`/api/v1/me/gears/${encodeURIComponent(id)}`, {
    auth: true,
  });
}

export async function createGear(
  request: CreateGearRequest,
): Promise<GearItem> {
  return requestJson("/api/v1/me/gears", {
    method: "POST",
    data: request,
    auth: true,
  });
}

export async function updateGear(
  id: string,
  request: UpdateGearRequest,
): Promise<GearItem> {
  return requestJson(`/api/v1/me/gears/${encodeURIComponent(id)}`, {
    method: "PATCH",
    data: request,
    auth: true,
  });
}

export async function archiveGear(id: string): Promise<void> {
  await requestJson<void>(`/api/v1/me/gears/${encodeURIComponent(id)}`, {
    method: "DELETE",
    auth: true,
  });
}

export async function deleteGear(id: string): Promise<void> {
  await requestJson<void>(`/api/v1/me/gears/${encodeURIComponent(id)}/delete`, {
    method: "POST",
    auth: true,
  });
}

export async function undeleteGear(id: string): Promise<GearItem> {
  return requestJson(`/api/v1/me/gears/${encodeURIComponent(id)}/undelete`, {
    method: "POST",
    auth: true,
  });
}

export async function restoreGear(id: string): Promise<GearItem> {
  return requestJson(`/api/v1/me/gears/${encodeURIComponent(id)}/restore`, {
    method: "POST",
    auth: true,
  });
}

export async function listSkills(
  locale: SkillLocale = "zh-CN",
): Promise<ListSkillsResponse> {
  return requestJson("/api/v1/skills", { locale });
}

export async function listKnots(
  request: ListKnotsRequest = {},
  locale: SkillLocale = "zh-CN",
): Promise<KnotListResponse> {
  return requestJson(knotListPath(request), {
    locale,
  });
}

export async function getKnotFilters(
  locale: SkillLocale = "zh-CN",
): Promise<KnotFiltersResponse> {
  return requestJson("/api/v1/skills/knots/filters", { locale });
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
  return requestJson("/api/v1/skills/knots/offline-manifest", {
    locale,
    cache: false,
  });
}

export async function listClientVersions(
  clientKey: ClientKey,
  options: { limit?: number; cursor?: string } = {},
): Promise<ListClientVersionsResponse> {
  return requestJson(
    `/api/v1/client-versions${queryString({
      ...options,
      client_key: clientKey,
    })}`,
    { cache: false },
  );
}

export function knotListPath(request: ListKnotsRequest = {}): string {
  return `/api/v1/skills/knots/list${queryString(request)}`;
}

export function knotDetailPath(id: string): string {
  return `/api/v1/skills/knots/detail/${encodeURIComponent(id)}`;
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
  const requestPath = versionedApiPath(path);
  const method = options.method ?? "GET";
  if (method === "GET" && !didRetryUnauthorized) {
    const key = inFlightGetKey(requestPath, options);
    const pending = pendingGetRequests.get(key) as Promise<T> | undefined;
    if (pending) {
      return pending;
    }
    const request = requestJsonOnce<T>(
      requestPath,
      options,
      didRetryUnauthorized,
    );
    pendingGetRequests.set(key, request);
    return request.finally(() => {
      if (pendingGetRequests.get(key) === request) {
        pendingGetRequests.delete(key);
      }
    });
  }
  return requestJsonOnce(path, options, didRetryUnauthorized);
}

async function requestJsonOnce<T>(
  path: string,
  options: ApiRequestOptions = {},
  didRetryUnauthorized = false,
): Promise<T> {
  const requestPath = versionedApiPath(path);
  const method = options.method ?? "GET";
  if (method !== "GET" && isOffline()) {
    throw new OfflineWriteBlockedError();
  }
  const token = options.auth ? await ensureAccessToken() : undefined;
  const cacheDescriptor = offlineCacheDescriptor(requestPath, options);
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
  const apiBaseUrl = await getApiBaseUrlForRequest(requestPath);

  return new Promise<T>((resolve, reject) => {
    wx.request({
      url: `${apiBaseUrl}${requestPath}`,
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

function inFlightGetKey(path: string, options: ApiRequestOptions): string {
  const authScope = options.auth ? (getStoredUser()?.id ?? "auth") : "public";
  return ["GET", path, options.locale ?? "", authScope].join("|");
}

async function getApiBaseUrlForRequest(path: string): Promise<string> {
  if (path !== HEALTH_PATH) {
    await ensureProductionDomainSelected();
  }
  return getApiBaseUrl();
}

async function ensureProductionDomainSelected(): Promise<void> {
  if (domainProbeCompleted || isOffline()) {
    return;
  }
  const candidates = getDomainCandidates();
  if (candidates.length === 0) {
    domainProbeCompleted = true;
    return;
  }
  if (!domainProbePromise) {
    domainProbePromise = probeProductionDomains(candidates).finally(() => {
      domainProbePromise = null;
    });
  }
  await domainProbePromise;
}

async function probeProductionDomains(
  candidates: ClientDomainCandidate[],
): Promise<void> {
  for (const candidate of candidates) {
    const apiBaseUrl = normalizeStoredBaseUrl(candidate.apiBaseUrl);
    const assetsBaseUrl = normalizeStoredBaseUrl(candidate.assetsBaseUrl);
    if (!apiBaseUrl || !assetsBaseUrl) {
      continue;
    }
    const healthy = await probeHealthz(apiBaseUrl);
    if (healthy) {
      setSelectedClientBaseUrls(apiBaseUrl, assetsBaseUrl);
      domainProbeCompleted = true;
      return;
    }
  }
  const fallback = candidates[0];
  setSelectedClientBaseUrls(
    normalizeStoredBaseUrl(fallback.apiBaseUrl) ?? DEFAULT_API_BASE_URL,
    normalizeStoredBaseUrl(fallback.assetsBaseUrl) ?? DEFAULT_ASSETS_BASE_URL,
  );
  domainProbeCompleted = true;
}

function probeHealthz(apiBaseUrl: string): Promise<boolean> {
  return new Promise((resolve) => {
    wx.request({
      url: `${apiBaseUrl}${HEALTH_PATH}`,
      method: "GET" as any,
      timeout: API_DOMAIN_HEALTH_TIMEOUT_MS,
      success: (response) => {
        resolve(response.statusCode >= 200 && response.statusCode < 300);
      },
      fail: () => {
        resolve(false);
      },
    });
  });
}

function getDomainCandidates(): ClientDomainCandidate[] {
  const app = getApp<{
    globalData?: {
      domainCandidates?: ClientDomainCandidate[];
    };
  }>();
  const candidates = app.globalData?.domainCandidates;
  if (!Array.isArray(candidates)) {
    return [];
  }
  return candidates;
}

function setSelectedClientBaseUrls(
  apiBaseUrl: string,
  assetsBaseUrl: string,
): void {
  cachedApiBaseUrl = apiBaseUrl.replace(/\/$/, "");
  cachedAssetsBaseUrl = assetsBaseUrl.replace(/\/$/, "");
  const app = getApp<{
    globalData?: {
      apiBaseUrl?: string;
      assetsBaseUrl?: string;
    };
  }>();
  if (app.globalData) {
    app.globalData.apiBaseUrl = cachedApiBaseUrl;
    app.globalData.assetsBaseUrl = cachedAssetsBaseUrl;
  }
  wx.setStorageSync(API_BASE_URL_STORAGE_KEY, cachedApiBaseUrl);
  wx.setStorageSync(ASSETS_BASE_URL_STORAGE_KEY, cachedAssetsBaseUrl);
}

function versionedApiPath(path: string): string {
  if (path === HEALTH_PATH || path.startsWith(`${API_PREFIX}/`)) {
    return path;
  }
  const normalized = path.startsWith("/") ? path : `/${path}`;
  return `${API_PREFIX}${normalized}`;
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
    path === "/api/v1/skills" ||
    path.startsWith("/api/v1/skills/") ||
    path === "/api/v1/gear-templates" ||
    path.startsWith("/api/v1/gear-templates/") ||
    path === "/api/v1/gear-atlas" ||
    path.startsWith("/api/v1/gear-atlas?") ||
    path.startsWith("/api/v1/gear-atlas/")
  );
}

function isUserCacheablePath(path: string): boolean {
  return (
    path === "/api/v1/me/gears" ||
    path.startsWith("/api/v1/me/gears?") ||
    path.startsWith("/api/v1/me/gears/") ||
    path === "/api/v1/me/packing-lists" ||
    path.startsWith("/api/v1/me/packing-lists?") ||
    path.startsWith("/api/v1/me/packing-lists/") ||
    path === "/api/v1/me/gear-atlas-submissions" ||
    path.startsWith("/api/v1/me/gear-atlas-submissions?")
  );
}

function saveLoginResponse(response: WechatLoginResponse): void {
  const user = normalizeUserAssets(response.user);
  cachedAccessToken = response.access_token;
  cachedAccessTokenExpiresAt = response.expires_at;
  cachedRefreshToken = response.refresh_token;
  cachedUser = user;
  wx.setStorageSync(TOKEN_STORAGE_KEY, response.access_token);
  wx.setStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY, response.expires_at);
  wx.setStorageSync(REFRESH_TOKEN_STORAGE_KEY, response.refresh_token);
  wx.setStorageSync(
    REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY,
    response.refresh_expires_at,
  );
  wx.setStorageSync(USER_STORAGE_KEY, user);
}

function saveUser(user: WechatLoginResponse["user"]): void {
  const normalized = normalizeUserAssets(user);
  cachedUser = normalized;
  wx.setStorageSync(USER_STORAGE_KEY, normalized);
}

function normalizeUserAssets(
  user: WechatLoginResponse["user"],
): WechatLoginResponse["user"] {
  if (!user.avatar_url) {
    return user;
  }
  return {
    ...user,
    avatar_url: normalizeKnownAssetUrl(user.avatar_url),
  };
}

function readAccessToken(): string | undefined {
  if (cachedAccessToken !== undefined) {
    return cachedAccessToken ?? undefined;
  }
  cachedAccessToken =
    (wx.getStorageSync(TOKEN_STORAGE_KEY) as string | undefined) ?? null;
  return cachedAccessToken ?? undefined;
}

function readRefreshToken(): string | undefined {
  if (cachedRefreshToken !== undefined) {
    return cachedRefreshToken ?? undefined;
  }
  cachedRefreshToken =
    (wx.getStorageSync(REFRESH_TOKEN_STORAGE_KEY) as string | undefined) ??
    null;
  return cachedRefreshToken ?? undefined;
}

function readAccessTokenExpiresAt(): string | undefined {
  if (cachedAccessTokenExpiresAt !== undefined) {
    return cachedAccessTokenExpiresAt ?? undefined;
  }
  cachedAccessTokenExpiresAt =
    (wx.getStorageSync(ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY) as
      | string
      | undefined) ?? null;
  return cachedAccessTokenExpiresAt ?? undefined;
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

async function uploadWechatAvatarOnce(
  filePath: string,
  token: string,
): Promise<ProfileUserResponse> {
  const requestPath = versionedApiPath("/me/profile/avatar");
  const apiBaseUrl = await getApiBaseUrlForRequest(requestPath);
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${apiBaseUrl}${requestPath}`,
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

async function uploadFeedbackImageOnce(
  filePath: string,
  token: string,
): Promise<UploadImageResponse> {
  const requestPath = versionedApiPath("/me/uploads");
  const apiBaseUrl = await getApiBaseUrlForRequest(requestPath);
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${apiBaseUrl}${requestPath}`,
      filePath,
      name: "file",
      formData: {
        purpose: "feedback",
      },
      header: {
        authorization: `Bearer ${token}`,
      },
      success: (response) => {
        const data = parseUploadResponseData(response.data);
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(data as UploadImageResponse);
          return;
        }
        reject(new ApiResponseError(response.statusCode, data));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || "图片上传失败，请稍后再试"));
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
  const refreshToken = readRefreshToken();
  if (!refreshToken) {
    throw new Error("登录已过期，请重新登录");
  }
  const response = await requestJson<WechatLoginResponse>("/auth/refresh", {
    method: "POST",
    data: { refresh_token: refreshToken },
  });
  saveLoginResponse(response);
  return response.access_token;
}

function shouldRefreshAccessToken(): boolean {
  const expiresAt = readAccessTokenExpiresAt();
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
  if (errMsg && /(合法域名|domain list|url not in domain)/i.test(errMsg)) {
    return "服务连接配置异常，请稍后再试";
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
