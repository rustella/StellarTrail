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
  AcceptKnotDisclaimerRequest,
  KnotDetail,
  KnotDisclaimerResponse,
  KnotFiltersResponse,
  KnotListResponse,
  KnotOfflineManifestResponse,
  FavoriteKnotStatusResponse,
  ListFavoriteSkillsRequest,
  ListFavoriteSkillsResponse,
  ListKnotsRequest,
  ListSkillsResponse,
  SkillLocale,
} from "./skill-utils";
import type {
  CreateTripInvitationResponse,
  CreateTripRequest,
  ImportTripPackingListRequest,
  ListTripsResponse,
  ListOutdoorExperiencesResponse,
  OutdoorExperience,
  OutdoorExperienceRequest,
  TripRecordCreateRequest,
  TripRecordPatchRequest,
  TripHomeHighlightResponse,
  TripDetail,
  TripSummary,
  UpdateTripSectionsRequest,
  UpdateTripRequest,
} from "./trip-utils";
import type {
  ClientDomainCandidate,
  ClientRequestSignatureConfig,
} from "./client-config";
import { loadClientConfig } from "./client-config";
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
export type {
  ListOutdoorExperiencesResponse,
  OutdoorExperience,
  OutdoorExperienceRequest,
} from "./trip-utils";

const TOKEN_STORAGE_KEY = "stellartrail_access_token";
const ACCESS_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_access_token_expires_at";
const REFRESH_TOKEN_STORAGE_KEY = "stellartrail_refresh_token";
const REFRESH_TOKEN_EXPIRES_AT_STORAGE_KEY =
  "stellartrail_refresh_token_expires_at";
const USER_STORAGE_KEY = "stellartrail_user";
const KNOT_DISCLAIMER_ACCEPTANCE_STORAGE_KEY =
  "stellartrail_knot_disclaimer_acceptance_v1";
const API_BASE_URL_STORAGE_KEY = "stellartrail_api_base_url";
const ASSETS_BASE_URL_STORAGE_KEY = "stellartrail_assets_base_url";
const DEFAULT_API_BASE_URL = "https://api.example.invalid";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";
const API_PREFIX = "/api/v1";
const HEALTH_PATH = "/healthz";
const API_REQUEST_TIMEOUT_MS = 15_000;
const API_DOMAIN_HEALTH_TIMEOUT_MS = 3_000;
const WECHAT_LOGIN_TIMEOUT_MS = 5_000;
const SIGNATURE_ALGORITHM = "STELLARTRAIL-HMAC-SHA256";
const SIGNING_FIELD_APP_ID = "app_id";
const SIGNING_FIELD_NONCE = "nonce";
const SIGNING_FIELD_SIGNATURE = "signature";

let loginPromise: Promise<string> | null = null;
let refreshPromise: Promise<string> | null = null;
let domainProbePromise: Promise<void> | null = null;
let domainProbeCompleted = false;
let offlineCacheNoticePending = false;
let signatureNonceCounter = 0;
let cachedApiBaseUrl: string | null = null;
let cachedAssetsBaseUrl: string | null = null;
let cachedAccessToken: string | null | undefined;
let cachedAccessTokenExpiresAt: string | null | undefined;
let cachedRefreshToken: string | null | undefined;
let cachedUser: WechatLoginResponse["user"] | null | undefined;
const pendingGetRequests = new Map<string, Promise<unknown>>();
interface ApiRequestOptions {
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS";
  data?: unknown;
  auth?: boolean;
  locale?: SkillLocale;
  cache?: boolean;
}

interface SignedJsonRequest {
  path: string;
  data: unknown;
}

interface ParsedRequestPath {
  path: string;
  query: string;
}

interface MultipartTextField {
  name: string;
  value: string;
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

interface StoredKnotDisclaimerAcceptance {
  userId: string;
  version: string;
  acceptedAt: string;
}

export interface OutdoorProfile {
  user_id: string;
  outdoor_id?: string | null;
  real_name?: string | null;
  gender?: string | null;
  birth_date?: string | null;
  height_cm?: number | null;
  phone?: string | null;
  emergency_contact?: string | null;
  emergency_contact_relationship?: string | null;
  emergency_phone?: string | null;
  blood_type?: string | null;
  medical_history?: string | null;
  allergy_history?: string | null;
  medical_response_note?: string | null;
  diet_preference?: string | null;
  insurance_policy_no?: string | null;
  insurance_company_phone?: string | null;
  experience_note?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export type UpdateOutdoorProfileRequest = Partial<
  Omit<OutdoorProfile, "user_id" | "created_at" | "updated_at">
>;

export interface OutdoorProfileResponse {
  profile: OutdoorProfile;
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

export type ClientVersionReleaseNoteSectionKey =
  | "feature"
  | "bug_fix"
  | "notes";

export interface ClientVersionReleaseNoteSection {
  key: ClientVersionReleaseNoteSectionKey;
  title: string;
  items: string[];
}

export interface ClientVersion {
  id: string;
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes: string[];
  release_note_sections?: ClientVersionReleaseNoteSection[];
  status: "draft" | "published";
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListClientVersionsResponse {
  items: ClientVersion[];
  next_cursor?: string | null;
}

export type RoadmapStatus =
  | "planned"
  | "designing"
  | "building"
  | "preview"
  | "shipped";

export type RoadmapCategory =
  | "gear"
  | "skills"
  | "routes"
  | "offline"
  | "safety"
  | "community";

export interface RoadmapItem {
  id: string;
  client_key: ClientKey;
  title: string;
  summary: string;
  details?: string | null;
  category: RoadmapCategory | string;
  status: RoadmapStatus | string;
  priority: number;
  sort_order: number;
  is_published: boolean;
  vote_count: number;
  subscription_count: number;
  is_voted: boolean;
  is_subscribed: boolean;
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListRoadmapRequest {
  client_key?: ClientKey;
  status?: RoadmapStatus;
  limit?: number;
  cursor?: string;
}

export interface ListRoadmapResponse {
  items: RoadmapItem[];
  next_cursor?: string | null;
}

export type RoadmapInteractionStatusResponse = RoadmapItem;

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

function isRefreshAuthInvalidError(error: unknown): boolean {
  return (
    isApiResponseError(error) &&
    (error.statusCode === 401 || error.statusCode === 403)
  );
}

function isRefreshNetworkFailureError(error: unknown): boolean {
  return !isApiResponseError(error);
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

export function getOutdoorProfile(): Promise<OutdoorProfileResponse> {
  return requestJson("/api/v1/me/profile/outdoor", {
    auth: true,
  });
}

export function updateOutdoorProfile(
  data: UpdateOutdoorProfileRequest,
): Promise<OutdoorProfileResponse> {
  return requestJson("/api/v1/me/profile/outdoor", {
    method: "PATCH",
    auth: true,
    data,
  });
}

export async function listOutdoorExperiences(): Promise<ListOutdoorExperiencesResponse> {
  const response = await requestJson<ListOutdoorExperiencesResponse>(
    "/api/v1/me/outdoor-experiences",
    {
      auth: true,
    },
  );
  return {
    ...response,
    items: (response.items || []).map(normalizeOutdoorExperience),
  };
}

export async function createOutdoorExperience(
  request: OutdoorExperienceRequest,
): Promise<OutdoorExperience> {
  return normalizeOutdoorExperience(
    await requestJson("/api/v1/me/outdoor-experiences", {
      method: "POST",
      auth: true,
      data: request,
    }),
  );
}

export async function updateOutdoorExperience(
  id: string,
  request: OutdoorExperienceRequest,
): Promise<OutdoorExperience> {
  return normalizeOutdoorExperience(
    await requestJson(
      `/api/v1/me/outdoor-experiences/${encodeURIComponent(id)}`,
      {
        method: "PATCH",
        auth: true,
        data: request,
      },
    ),
  );
}

export function deleteOutdoorExperience(id: string): Promise<void> {
  return requestJson(
    `/api/v1/me/outdoor-experiences/${encodeURIComponent(id)}`,
    {
      method: "DELETE",
      auth: true,
    },
  );
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
    } catch (error) {
      if (isRefreshAuthInvalidError(error)) {
        clearLoginState();
      } else if (cached && isRefreshNetworkFailureError(error)) {
        return cached;
      }
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

export function updateWechatNickname(nickname: string): Promise<string> {
  const avatarUrl = getStoredUser()?.avatar_url;
  return loginWithWechat({
    nickname,
    ...(isUploadedProfileAvatarUrl(avatarUrl) ? { avatar_url: avatarUrl } : {}),
  });
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
  wx.removeStorageSync(KNOT_DISCLAIMER_ACCEPTANCE_STORAGE_KEY);
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

export async function listGearCategories(): Promise<GearCategoriesResponse> {
  return requestJson("/api/v1/me/gears/categories", {
    auth: true,
  });
}

export async function getGearStats(): Promise<GearStatsResponse> {
  return requestJson("/api/v1/me/gears/stats", {
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
  update:
    | boolean
    | {
        packed?: boolean;
        planned_quantity?: number | null;
        packed_quantity?: number | null;
      },
): Promise<GearPackingListDetail> {
  const data = typeof update === "boolean" ? { packed: update } : update;
  return requestJson(
    `/api/v1/me/packing-lists/${encodeURIComponent(id)}/items/${encodeURIComponent(itemId)}`,
    {
      method: "PATCH",
      data,
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

export async function listTrips(
  request: {
    limit?: number;
    cursor?: string;
    bucket?: string;
    trip_type?: string;
  } = {},
): Promise<ListTripsResponse> {
  const response = await requestJson<ListTripsResponse>(
    `/api/v1/me/trips${queryString(request)}`,
    {
      auth: true,
    },
  );
  return {
    ...response,
    items: (response.items || []).map(normalizeTripSummary),
  };
}

export async function getTripHomeHighlight(
  today: string,
): Promise<TripHomeHighlightResponse> {
  const response = await requestJson<TripHomeHighlightResponse>(
    `/api/v1/me/trips/home-highlight${queryString({ today })}`,
    {
      auth: true,
    },
  );
  if (!response.item) {
    return response;
  }
  const trip = normalizeTripSummary(response.item.trip || response.item.plan);
  return {
    ...response,
    item: {
      ...response.item,
      trip,
      plan: trip,
    },
  };
}

export async function createTrip(
  request: CreateTripRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson("/api/v1/me/trips", {
      method: "POST",
      data: request,
      auth: true,
    }),
  );
}

export async function getTrip(id: string): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(`/api/v1/me/trips/${encodeURIComponent(id)}`, {
      auth: true,
    }),
  );
}

export async function convertTripToOutdoorExperience(
  id: string,
): Promise<OutdoorExperience> {
  return normalizeOutdoorExperience(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/convert-to-outdoor-experience`,
      {
        method: "POST",
        auth: true,
      },
    ),
  );
}

export async function updateTrip(
  id: string,
  request: UpdateTripRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(`/api/v1/me/trips/${encodeURIComponent(id)}`, {
      method: "PATCH",
      data: request,
      auth: true,
    }),
  );
}

export async function deleteTrip(id: string): Promise<void> {
  await requestJson<void>(`/api/v1/me/trips/${encodeURIComponent(id)}`, {
    method: "DELETE",
    auth: true,
  });
}

export async function updateTripSections(
  id: string,
  request: UpdateTripSectionsRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(`/api/v1/me/trips/${encodeURIComponent(id)}/sections`, {
      method: "PATCH",
      data: request,
      auth: true,
    }),
  );
}

export async function createTripInvitation(
  id: string,
): Promise<CreateTripInvitationResponse> {
  return requestJson(`/api/v1/me/trips/${encodeURIComponent(id)}/invitations`, {
    method: "POST",
    auth: true,
  });
}

export async function acceptTripInvitation(token: string): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trip-invitations/${encodeURIComponent(token)}/accept`,
      {
        method: "POST",
        auth: true,
      },
    ),
  );
}

export async function updateTripMember(
  id: string,
  memberId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/members/${encodeURIComponent(memberId)}`,
      {
        method: "PATCH",
        data: request,
        auth: true,
      },
    ),
  );
}

export async function removeTripMember(
  id: string,
  memberId: string,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/members/${encodeURIComponent(memberId)}`,
      {
        method: "DELETE",
        auth: true,
      },
    ),
  );
}

export async function importTripPackingList(
  id: string,
  request: ImportTripPackingListRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/personal-gear/import-packing-list`,
      {
        method: "POST",
        data: request,
        auth: true,
      },
    ),
  );
}

export function createTripPersonalGearItem(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "personal-gear", request);
}

export function updateTripPersonalGearItem(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "personal-gear", itemId, request);
}

export function deleteTripPersonalGearItem(
  id: string,
  itemId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "personal-gear", itemId);
}

export function createTripSharedGearDemand(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "shared-gear-demands", request);
}

export function updateTripSharedGearDemand(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "shared-gear-demands", itemId, request);
}

export function deleteTripSharedGearDemand(
  id: string,
  itemId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "shared-gear-demands", itemId);
}

export async function bindTripSharedGearDemandMyGear(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/shared-gear-demands/${encodeURIComponent(itemId)}/bind-my-gear`,
      {
        method: "POST",
        data: request,
        auth: true,
      },
    ),
  );
}

export async function fillTripSharedGearDemandConcreteGear(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/shared-gear-demands/${encodeURIComponent(itemId)}/fill-concrete-gear`,
      {
        method: "POST",
        data: request,
        auth: true,
      },
    ),
  );
}

export function createTripItineraryDay(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "itinerary-days", request);
}

export function updateTripItineraryDay(
  id: string,
  dayId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "itinerary-days", dayId, request);
}

export function deleteTripItineraryDay(
  id: string,
  dayId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "itinerary-days", dayId);
}

export function createTripItineraryTimeSlot(
  id: string,
  dayId: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(
    id,
    `itinerary-days/${encodeURIComponent(dayId)}/time-slots`,
    request,
  );
}

export function updateTripItineraryTimeSlot(
  id: string,
  dayId: string,
  slotId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(
    id,
    `itinerary-days/${encodeURIComponent(dayId)}/time-slots`,
    slotId,
    request,
  );
}

export function deleteTripItineraryTimeSlot(
  id: string,
  dayId: string,
  slotId: string,
): Promise<TripDetail> {
  return deleteTripRecord(
    id,
    `itinerary-days/${encodeURIComponent(dayId)}/time-slots`,
    slotId,
  );
}

export function createTripRouteSegment(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "route-segments", request);
}

export function updateTripRouteSegment(
  id: string,
  segmentId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "route-segments", segmentId, request);
}

export function deleteTripRouteSegment(
  id: string,
  segmentId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "route-segments", segmentId);
}

export function createTripSegmentAssignment(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "segment-assignments", request);
}

export function updateTripSegmentAssignment(
  id: string,
  assignmentId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "segment-assignments", assignmentId, request);
}

export function deleteTripSegmentAssignment(
  id: string,
  assignmentId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "segment-assignments", assignmentId);
}

export function updateTripFoodMeal(
  id: string,
  mealId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "food-meals", mealId, request);
}

export function createTripFoodMeal(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "food-meals", request);
}

export function deleteTripFoodMeal(
  id: string,
  mealId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "food-meals", mealId);
}

export function createTripFoodItem(
  id: string,
  mealId: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(
    id,
    `food-meals/${encodeURIComponent(mealId)}/items`,
    request,
  );
}

export function updateTripFoodItem(
  id: string,
  mealId: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(
    id,
    `food-meals/${encodeURIComponent(mealId)}/items`,
    itemId,
    request,
  );
}

export function deleteTripFoodItem(
  id: string,
  mealId: string,
  itemId: string,
): Promise<TripDetail> {
  return deleteTripRecord(
    id,
    `food-meals/${encodeURIComponent(mealId)}/items`,
    itemId,
  );
}

export function createTripFoodSupply(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "food-supplies", request);
}

export function updateTripFoodSupply(
  id: string,
  supplyId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "food-supplies", supplyId, request);
}

export function deleteTripFoodSupply(
  id: string,
  supplyId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "food-supplies", supplyId);
}

export function createTripMedicalItem(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "medical-items", request);
}

export function updateTripMedicalItem(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "medical-items", itemId, request);
}

export function deleteTripMedicalItem(
  id: string,
  itemId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "medical-items", itemId);
}

export function createTripSafetyRisk(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "safety-risks", request);
}

export function updateTripSafetyRisk(
  id: string,
  riskId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "safety-risks", riskId, request);
}

export function deleteTripSafetyRisk(
  id: string,
  riskId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "safety-risks", riskId);
}

export function createTripRescueContact(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "rescue-contacts", request);
}

export function updateTripRescueContact(
  id: string,
  contactId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "rescue-contacts", contactId, request);
}

export function deleteTripRescueContact(
  id: string,
  contactId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "rescue-contacts", contactId);
}

export function createTripBudgetItem(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "budget-items", request);
}

export function updateTripBudgetItem(
  id: string,
  itemId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "budget-items", itemId, request);
}

export function deleteTripBudgetItem(
  id: string,
  itemId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "budget-items", itemId);
}

export function createTripGoalItem(
  id: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return createTripRecord(id, "goals", request);
}

export function updateTripGoalItem(
  id: string,
  goalId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return updateTripRecord(id, "goals", goalId, request);
}

export function deleteTripGoalItem(
  id: string,
  goalId: string,
): Promise<TripDetail> {
  return deleteTripRecord(id, "goals", goalId);
}

async function createTripRecord(
  id: string,
  collectionPath: string,
  request: TripRecordCreateRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/${collectionPath}`,
      {
        method: "POST",
        data: request,
        auth: true,
      },
    ),
  );
}

async function updateTripRecord(
  id: string,
  collectionPath: string,
  recordId: string,
  request: TripRecordPatchRequest,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/${collectionPath}/${encodeURIComponent(recordId)}`,
      {
        method: "PATCH",
        data: request,
        auth: true,
      },
    ),
  );
}

async function deleteTripRecord(
  id: string,
  collectionPath: string,
  recordId: string,
): Promise<TripDetail> {
  return normalizeTripDetail(
    await requestJson(
      `/api/v1/me/trips/${encodeURIComponent(id)}/${collectionPath}/${encodeURIComponent(recordId)}`,
      {
        method: "DELETE",
        auth: true,
      },
    ),
  );
}

function normalizeTripSummary(value: any): TripSummary {
  const source = value || {};
  const title = source.title ?? source.name ?? "";
  const dayCount = Number(source.day_count ?? source.itinerary_day_count ?? 0);
  return {
    ...source,
    trip_type: source.trip_type ?? "team",
    title,
    name: title,
    enabled_sections: source.enabled_sections ?? [],
    route_use_slope_adjustment: !!source.route_use_slope_adjustment,
    route_use_high_altitude_adjustment:
      !!source.route_use_high_altitude_adjustment,
    day_count: dayCount,
    itinerary_day_count: dayCount,
    time_bucket: source.time_bucket ?? "undated",
    days_until_start: source.days_until_start ?? null,
    days_until_end: source.days_until_end ?? null,
    member_count: Number(source.member_count ?? 1),
    readiness: source.readiness ?? {
      missing_count: 0,
      missing_labels: [],
      completion_percent: 0,
    },
    outdoor_experience_id: source.outdoor_experience_id ?? null,
    field_versions: source.field_versions ?? {},
    is_deleted: !!source.is_deleted,
    created_at: source.created_at ?? "",
    updated_at: source.updated_at ?? "",
  };
}

function normalizeOutdoorExperience(value: any): OutdoorExperience {
  const source = value || {};
  return {
    id: String(source.id ?? ""),
    user_id: String(source.user_id ?? ""),
    source_trip_id: source.source_trip_id ?? null,
    trip_type: source.trip_type ?? "team",
    title: String(source.title ?? ""),
    start_date: source.start_date ?? null,
    end_date: source.end_date ?? null,
    day_count: normalizeOptionalNumber(source.day_count),
    companion_count: normalizeOptionalNumber(source.companion_count),
    route_summary: source.route_summary ?? null,
    gear_summary: source.gear_summary ?? null,
    food_summary: source.food_summary ?? null,
    budget_summary: source.budget_summary ?? null,
    notes: source.notes ?? null,
    created_at: source.created_at ?? "",
    updated_at: source.updated_at ?? "",
  };
}

function normalizeOptionalNumber(value: unknown): number | null {
  if (value === null || value === undefined || value === "") {
    return null;
  }
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : null;
}

function normalizeTripDetail(value: any): TripDetail {
  const source = value || {};
  const trip = normalizeTripSummary(source.trip ?? source.plan);
  const members = (source.members || []).map((member: any) => ({
    ...member,
    trip_id: member.trip_id ?? member.plan_id ?? trip.id,
    plan_id: member.plan_id ?? member.trip_id ?? trip.id,
  }));
  const personalGear = source.personal_gear ?? source.personal_gear_items ?? [];
  const sharedGear = (
    source.shared_gear_demands ??
    source.shared_gear_items ??
    []
  ).map((item: any) => ({
    ...item,
    template_key: item.template_key ?? item.slot_key ?? null,
    demand_name: item.demand_name ?? item.slot_name ?? null,
    slot_key: item.slot_key ?? item.template_key ?? null,
    slot_name: item.slot_name ?? item.demand_name ?? item.name ?? null,
  }));
  const sharedTemplates = (source.shared_gear_demand_templates || []).map(
    (item: any) => ({
      ...item,
      template_key: item.template_key ?? item.slot_key,
      demand_name: item.demand_name ?? item.slot_name,
      slot_key: item.slot_key ?? item.template_key,
      slot_name: item.slot_name ?? item.demand_name,
    }),
  );
  const weightSummaries =
    source.weight_summaries ?? source.gear_weight_summaries ?? [];
  return {
    ...source,
    trip,
    plan: trip,
    sections: source.sections ?? trip.enabled_sections,
    members,
    personal_gear: personalGear,
    personal_gear_items: personalGear,
    shared_gear_demands: sharedGear,
    shared_gear_items: sharedGear,
    shared_gear_demand_templates: sharedTemplates,
    weight_summaries: weightSummaries,
    gear_weight_summaries: weightSummaries,
  } as TripDetail;
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

export async function deleteGear(id: string): Promise<void> {
  await requestJson<void>(`/api/v1/me/gears/${encodeURIComponent(id)}`, {
    method: "DELETE",
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

export async function getKnotDisclaimer(): Promise<KnotDisclaimerResponse> {
  const response = await requestJson<KnotDisclaimerResponse>(
    "/api/v1/me/skills/knots/disclaimer",
    {
      auth: true,
    },
  );
  rememberKnotDisclaimerAcceptance(response);
  return response;
}

export function hasLocalKnotDisclaimerAcceptance(): boolean {
  const userId = getStoredUser()?.id;
  if (!userId) {
    return false;
  }
  const acceptance = readStoredKnotDisclaimerAcceptance();
  return Boolean(
    acceptance &&
    acceptance.userId === userId &&
    typeof acceptance.version === "string" &&
    acceptance.version.trim(),
  );
}

export async function acceptKnotDisclaimer(
  request: AcceptKnotDisclaimerRequest = {},
): Promise<KnotDisclaimerResponse> {
  const response = await requestJson<KnotDisclaimerResponse>(
    "/api/v1/me/skills/knots/disclaimer/acceptance",
    {
      method: "POST",
      auth: true,
      cache: false,
      data: request,
    },
  );
  rememberKnotDisclaimerAcceptance(response);
  return response;
}

function rememberKnotDisclaimerAcceptance(
  response: KnotDisclaimerResponse,
): void {
  if (!response.accepted || !response.version) {
    return;
  }
  const userId = getStoredUser()?.id;
  if (!userId) {
    return;
  }
  const acceptance: StoredKnotDisclaimerAcceptance = {
    userId,
    version: response.version,
    acceptedAt: response.accepted_at ?? new Date().toISOString(),
  };
  try {
    wx.setStorageSync(KNOT_DISCLAIMER_ACCEPTANCE_STORAGE_KEY, acceptance);
  } catch {
    // Local acceptance only improves offline UX; online server state remains authoritative.
  }
}

function readStoredKnotDisclaimerAcceptance(): StoredKnotDisclaimerAcceptance | null {
  try {
    const stored = wx.getStorageSync(KNOT_DISCLAIMER_ACCEPTANCE_STORAGE_KEY) as
      | StoredKnotDisclaimerAcceptance
      | undefined;
    if (
      !stored ||
      typeof stored.userId !== "string" ||
      typeof stored.version !== "string" ||
      typeof stored.acceptedAt !== "string"
    ) {
      return null;
    }
    return stored;
  } catch {
    return null;
  }
}

export async function listFavoriteSkills(
  request: ListFavoriteSkillsRequest = {},
  locale: SkillLocale = "zh-CN",
): Promise<ListFavoriteSkillsResponse> {
  return requestJson(`/api/v1/me/skills/favorites${queryString(request)}`, {
    auth: true,
    locale,
  });
}

export async function getFavoriteKnotStatus(
  id: string,
): Promise<FavoriteKnotStatusResponse> {
  return requestJson(
    `/api/v1/me/skills/favorites/knots/${encodeURIComponent(id)}`,
    { auth: true, cache: false },
  );
}

export async function favoriteKnot(
  id: string,
): Promise<FavoriteKnotStatusResponse> {
  return requestJson(
    `/api/v1/me/skills/favorites/knots/${encodeURIComponent(id)}`,
    { method: "PUT", auth: true, cache: false },
  );
}

export async function unfavoriteKnot(
  id: string,
): Promise<FavoriteKnotStatusResponse> {
  return requestJson(
    `/api/v1/me/skills/favorites/knots/${encodeURIComponent(id)}`,
    { method: "DELETE", auth: true, cache: false },
  );
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

export async function listRoadmap(
  request: ListRoadmapRequest = {},
): Promise<ListRoadmapResponse> {
  return requestJson(`/api/v1/roadmap${queryString(request)}`);
}

export async function listMyRoadmap(
  request: ListRoadmapRequest = {},
): Promise<ListRoadmapResponse> {
  return requestJson(`/api/v1/me/roadmap${queryString(request)}`, {
    auth: true,
  });
}

export async function voteRoadmapItem(
  id: string,
): Promise<RoadmapInteractionStatusResponse> {
  return requestJson(`/api/v1/me/roadmap/${encodeURIComponent(id)}/vote`, {
    method: "PUT",
    auth: true,
    cache: false,
  });
}

export async function unvoteRoadmapItem(
  id: string,
): Promise<RoadmapInteractionStatusResponse> {
  return requestJson(`/api/v1/me/roadmap/${encodeURIComponent(id)}/vote`, {
    method: "DELETE",
    auth: true,
    cache: false,
  });
}

export async function subscribeRoadmapItem(
  id: string,
): Promise<RoadmapInteractionStatusResponse> {
  return requestJson(
    `/api/v1/me/roadmap/${encodeURIComponent(id)}/subscription`,
    {
      method: "PUT",
      auth: true,
      cache: false,
    },
  );
}

export async function unsubscribeRoadmapItem(
  id: string,
): Promise<RoadmapInteractionStatusResponse> {
  return requestJson(
    `/api/v1/me/roadmap/${encodeURIComponent(id)}/subscription`,
    {
      method: "DELETE",
      auth: true,
      cache: false,
    },
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
  const header: Record<string, string> = clientIdentityHeader();
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
  const signedRequest = signJsonRequestIfConfigured(
    method,
    requestPath,
    options.data,
  );

  return new Promise<T>((resolve, reject) => {
    wx.request({
      url: `${apiBaseUrl}${signedRequest.path}`,
      method: method as any,
      data: signedRequest.data as any,
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
              if (isRefreshAuthInvalidError(error)) {
                clearLoginState();
                reject(new LoginRequiredError("登录状态已过期，请重新登录"));
                return;
              }
              if (cacheDescriptor && isRefreshNetworkFailureError(error)) {
                const cached = readOfflineCache<T>(cacheDescriptor);
                if (cached) {
                  offlineCacheNoticePending = true;
                  resolve(cached.data);
                  return;
                }
              }
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
      header: clientIdentityHeader(),
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

function getRequestSignatureConfig(): ClientRequestSignatureConfig | undefined {
  const app = getApp<{
    globalData?: {
      requestSignature?: ClientRequestSignatureConfig;
    };
  }>();
  const app_id = app.globalData?.requestSignature?.app_id?.trim();
  const app_secret = app.globalData?.requestSignature?.app_secret?.trim();
  if (!app_id || !app_secret) {
    return undefined;
  }
  return { app_id, app_secret };
}

function clientIdentityHeader(): Record<string, string> {
  return {
    "X-StellarTrail-Client": getClientIdentity(),
  };
}

function getClientIdentity(): string {
  const app = getApp<{
    globalData?: {
      clientIdentity?: string;
    };
  }>();
  const clientIdentity = app.globalData?.clientIdentity?.trim();
  if (clientIdentity) {
    return clientIdentity;
  }
  return loadClientConfig().clientIdentity;
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
    path === "/api/v1/roadmap" ||
    path.startsWith("/api/v1/roadmap?") ||
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
    path === "/api/v1/me/outdoor-experiences" ||
    path.startsWith("/api/v1/me/outdoor-experiences?") ||
    path === "/api/v1/me/trips/home-highlight" ||
    path.startsWith("/api/v1/me/trips/home-highlight?") ||
    path === "/api/v1/me/skills/knots/disclaimer" ||
    path === "/api/v1/me/roadmap" ||
    path.startsWith("/api/v1/me/roadmap?") ||
    path === "/api/v1/me/gear-atlas-submissions" ||
    path.startsWith("/api/v1/me/gear-atlas-submissions?") ||
    path === "/api/v1/me/skills/favorites" ||
    path.startsWith("/api/v1/me/skills/favorites?") ||
    path.startsWith("/api/v1/me/skills/favorites/knots/")
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

function isUploadedProfileAvatarUrl(
  avatarUrl?: string | null,
): avatarUrl is string {
  return (
    typeof avatarUrl === "string" &&
    /\/users\/[^/?#]+\/avatar\//.test(avatarUrl)
  );
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
  const requestSignature = getRequestSignatureConfig();
  if (requestSignature) {
    return uploadSignedMultipartFile<ProfileUserResponse>({
      apiBaseUrl,
      requestPath,
      filePath,
      token,
      fields: [],
      failureMessage: "头像上传失败，请稍后再试",
      requestSignature,
    });
  }
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${apiBaseUrl}${requestPath}`,
      filePath,
      name: "file",
      header: {
        ...clientIdentityHeader(),
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
  const requestSignature = getRequestSignatureConfig();
  if (requestSignature) {
    return uploadSignedMultipartFile<UploadImageResponse>({
      apiBaseUrl,
      requestPath,
      filePath,
      token,
      fields: [{ name: "purpose", value: "feedback" }],
      failureMessage: "图片上传失败，请稍后再试",
      requestSignature,
    });
  }
  return new Promise((resolve, reject) => {
    wx.uploadFile({
      url: `${apiBaseUrl}${requestPath}`,
      filePath,
      name: "file",
      formData: {
        purpose: "feedback",
      },
      header: {
        ...clientIdentityHeader(),
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

function signJsonRequestIfConfigured(
  method: ApiRequestOptions["method"],
  requestPath: string,
  data: unknown,
): SignedJsonRequest {
  const requestSignature = getRequestSignatureConfig();
  if (!requestSignature || !shouldSignRequest(method, requestPath)) {
    return { path: requestPath, data };
  }
  if (data === undefined) {
    return {
      path: signQueryRequest(method, requestPath, sha256Hex(new Uint8Array())),
      data,
    };
  }
  if (!isPlainRecord(data)) {
    throw new Error("Signed JSON API requests must use an object body.");
  }
  const bodyHash = sha256Hex(
    utf8Bytes(canonicalJsonWithoutSigningFields(data)),
  );
  const parsed = parseRequestPath(requestPath);
  const nonce = createSignatureNonce();
  const signature = hmacSha256Hex(
    requestSignature.app_secret,
    canonicalRequest(
      method,
      parsed.path,
      canonicalQuery(parsed.query),
      bodyHash,
      requestSignature.app_id,
      nonce,
    ),
  );
  return {
    path: requestPath,
    data: {
      ...(data as Record<string, unknown>),
      app_id: requestSignature.app_id,
      nonce,
      signature,
    },
  };
}

function signQueryRequest(
  method: ApiRequestOptions["method"],
  requestPath: string,
  bodyHash: string,
  requestSignature = getRequestSignatureConfig(),
): string {
  if (!requestSignature) {
    return requestPath;
  }
  const parsed = parseRequestPath(requestPath);
  const nonce = createSignatureNonce();
  const queryWithFields = queryWithSigningFields(parsed.query, {
    app_id: requestSignature.app_id,
    nonce,
  });
  const signature = hmacSha256Hex(
    requestSignature.app_secret,
    canonicalRequest(
      method,
      parsed.path,
      canonicalQuery(queryWithFields),
      bodyHash,
      requestSignature.app_id,
      nonce,
    ),
  );
  return buildRequestPath(
    parsed.path,
    queryWithSigningFields(queryWithFields, { signature }),
  );
}

function shouldSignRequest(
  method: ApiRequestOptions["method"],
  requestPath: string,
): boolean {
  if (method === undefined) {
    method = "GET";
  }
  if (method === "OPTIONS") {
    return false;
  }
  const path = parseRequestPath(requestPath).path;
  return (
    path.startsWith(`${API_PREFIX}/`) &&
    path !== `${API_PREFIX}/ping` &&
    path !== `${API_PREFIX}/echo` &&
    path !== HEALTH_PATH
  );
}

function parseRequestPath(requestPath: string): ParsedRequestPath {
  const [path, query = ""] = requestPath.split("?", 2);
  return { path, query };
}

function buildRequestPath(path: string, query: string): string {
  return query ? `${path}?${query}` : path;
}

function queryWithSigningFields(
  query: string,
  fields: Partial<Record<"app_id" | "nonce" | "signature", string>>,
): string {
  const keysToReplace = new Set<string>();
  if (fields.app_id) {
    keysToReplace.add(SIGNING_FIELD_APP_ID);
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  if (fields.nonce) {
    keysToReplace.add(SIGNING_FIELD_NONCE);
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  if (fields.signature) {
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  const pairs = splitQueryPairs(query).filter(
    ([key]) => !keysToReplace.has(key),
  );
  if (fields.app_id) {
    pairs.push([SIGNING_FIELD_APP_ID, encodeURIComponent(fields.app_id)]);
  }
  if (fields.nonce) {
    pairs.push([SIGNING_FIELD_NONCE, encodeURIComponent(fields.nonce)]);
  }
  if (fields.signature) {
    pairs.push([SIGNING_FIELD_SIGNATURE, encodeURIComponent(fields.signature)]);
  }
  return pairs.map(([key, value]) => `${key}=${value}`).join("&");
}

function canonicalQuery(query: string): string {
  return splitQueryPairs(query)
    .filter(([key]) => key !== SIGNING_FIELD_SIGNATURE)
    .sort(
      ([leftKey, leftValue], [rightKey, rightValue]) =>
        compareCanonicalText(leftKey, rightKey) ||
        compareCanonicalText(leftValue, rightValue),
    )
    .map(([key, value]) => `${key}=${value}`)
    .join("&");
}

function compareCanonicalText(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

function splitQueryPairs(query: string): Array<[string, string]> {
  if (!query) {
    return [];
  }
  return query
    .split("&")
    .filter(Boolean)
    .map((pair) => {
      const [key, value = ""] = pair.split("=", 2);
      return [key, value];
    });
}

function canonicalRequest(
  method: ApiRequestOptions["method"],
  path: string,
  canonicalQueryString: string,
  bodyHash: string,
  appId: string,
  nonce: string,
): string {
  return [
    SIGNATURE_ALGORITHM,
    method ?? "GET",
    path,
    canonicalQueryString,
    bodyHash,
    appId,
    nonce,
  ].join("\n");
}

async function uploadSignedMultipartFile<T>(options: {
  apiBaseUrl: string;
  requestPath: string;
  filePath: string;
  token: string;
  fields: MultipartTextField[];
  failureMessage: string;
  requestSignature: ClientRequestSignatureConfig;
}): Promise<T> {
  const fileBytes = await readLocalFileBytes(options.filePath);
  const boundary = `StellarTrailBoundary${createSignatureNonce().replace(/-/g, "")}`;
  const body = buildMultipartBody({
    boundary,
    fields: options.fields,
    filePath: options.filePath,
    fileBytes,
  });
  const signedPath = signQueryRequest(
    "POST",
    options.requestPath,
    sha256Hex(body),
    options.requestSignature,
  );
  return new Promise((resolve, reject) => {
    wx.request({
      url: `${options.apiBaseUrl}${signedPath}`,
      method: "POST" as any,
      data: body.buffer as any,
      header: {
        ...clientIdentityHeader(),
        authorization: `Bearer ${options.token}`,
        "content-type": `multipart/form-data; boundary=${boundary}`,
      },
      timeout: API_REQUEST_TIMEOUT_MS,
      success: (response) => {
        const data = parseUploadResponseData(response.data as string | object);
        if (response.statusCode >= 200 && response.statusCode < 300) {
          resolve(data as T);
          return;
        }
        reject(new ApiResponseError(response.statusCode, data));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || options.failureMessage));
      },
    });
  });
}

function readLocalFileBytes(filePath: string): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    wx.getFileSystemManager().readFile({
      filePath,
      success: (result) => {
        if (typeof result.data === "string") {
          resolve(utf8Bytes(result.data));
          return;
        }
        resolve(new Uint8Array(result.data));
      },
      fail: (error) => {
        reject(new Error(error.errMsg || "读取文件失败"));
      },
    });
  });
}

function buildMultipartBody(input: {
  boundary: string;
  fields: MultipartTextField[];
  filePath: string;
  fileBytes: Uint8Array;
}): Uint8Array {
  const parts: Uint8Array[] = [];
  input.fields.forEach((field) => {
    parts.push(
      utf8Bytes(
        `--${input.boundary}\r\nContent-Disposition: form-data; name="${escapeMultipartName(
          field.name,
        )}"\r\n\r\n${field.value}\r\n`,
      ),
    );
  });
  parts.push(
    utf8Bytes(
      `--${input.boundary}\r\nContent-Disposition: form-data; name="file"; filename="${escapeMultipartName(
        fileNameFromPath(input.filePath),
      )}"\r\nContent-Type: application/octet-stream\r\n\r\n`,
    ),
    input.fileBytes,
    utf8Bytes(`\r\n--${input.boundary}--\r\n`),
  );
  return concatBytes(parts);
}

function escapeMultipartName(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function fileNameFromPath(filePath: string): string {
  return filePath.split(/[\\/]/).filter(Boolean).pop() || "upload.bin";
}

function canonicalJsonWithoutSigningFields(
  value: Record<string, unknown>,
): string {
  const copy: Record<string, unknown> = {};
  Object.keys(value).forEach((key) => {
    if (
      key !== SIGNING_FIELD_APP_ID &&
      key !== SIGNING_FIELD_NONCE &&
      key !== SIGNING_FIELD_SIGNATURE
    ) {
      copy[key] = value[key];
    }
  });
  return canonicalJson(copy);
}

function canonicalJson(value: unknown, inArray = false): string {
  if (value === null || typeof value === "boolean") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    return JSON.stringify(Number.isFinite(value) ? value : null);
  }
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  if (
    value === undefined ||
    typeof value === "function" ||
    typeof value === "symbol"
  ) {
    return inArray ? "null" : "";
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => canonicalJson(item, true)).join(",")}]`;
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    const entries = Object.keys(record)
      .sort()
      .map((key) => {
        const item = canonicalJson(record[key]);
        return item ? `${JSON.stringify(key)}:${item}` : "";
      })
      .filter(Boolean);
    return `{${entries.join(",")}}`;
  }
  return "null";
}

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function createSignatureNonce(): string {
  const timestamp = Date.now();
  signatureNonceCounter = (signatureNonceCounter + 1) >>> 0;
  return `${timestamp.toString(36)}-${randomHex(
    16,
    timestamp,
    signatureNonceCounter,
  )}`;
}

function randomHex(
  byteLength: number,
  timestamp: number,
  counter: number,
): string {
  const bytes = new Uint8Array(byteLength);
  if (!fillRandomBytes(bytes)) {
    fillPseudoRandomBytes(bytes);
  }
  mixNonceEntropy(bytes, timestamp, counter);
  return bytesToHex(bytes);
}

function fillRandomBytes(bytes: Uint8Array): boolean {
  const globalCrypto = (globalThis as unknown as {
    crypto?: { getRandomValues?: (array: Uint8Array) => Uint8Array };
  }).crypto;
  if (typeof globalCrypto?.getRandomValues === "function") {
    try {
      globalCrypto.getRandomValues(bytes);
      if (!isAllZeroBytes(bytes)) {
        return true;
      }
    } catch {
      // Fall through to the Mini Program API and finally Math.random below.
    }
  }

  const wxWithRandom = wx as unknown as {
    getRandomValues?: (array: Uint8Array) => Uint8Array;
  };
  if (typeof wxWithRandom.getRandomValues === "function") {
    try {
      wxWithRandom.getRandomValues(bytes);
      if (!isAllZeroBytes(bytes)) {
        return true;
      }
    } catch {
      // Some WeChat runtimes expose wx.getRandomValues with a different shape.
    }
  }

  return false;
}

function fillPseudoRandomBytes(bytes: Uint8Array): void {
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Math.floor(Math.random() * 256);
  }
}

function mixNonceEntropy(
  bytes: Uint8Array,
  timestamp: number,
  counter: number,
): void {
  let timestampValue = Math.max(0, Math.floor(timestamp));
  for (let index = 0; index < Math.min(8, bytes.length); index += 1) {
    bytes[index] ^= timestampValue & 0xff;
    timestampValue = Math.floor(timestampValue / 256);
  }

  let counterValue = counter >>> 0;
  for (let index = 0; index < Math.min(4, bytes.length); index += 1) {
    const targetIndex = bytes.length - 1 - index;
    bytes[targetIndex] ^= counterValue & 0xff;
    counterValue >>>= 8;
  }

  if (isAllZeroBytes(bytes) && bytes.length > 0) {
    bytes[0] = 1;
  }
}

function isAllZeroBytes(bytes: Uint8Array): boolean {
  return bytes.every((byte) => byte === 0);
}

function hmacSha256Hex(secret: string, message: string): string {
  let key = utf8Bytes(secret);
  if (key.length > 64) {
    key = sha256(key);
  }
  const innerPad = new Uint8Array(64);
  const outerPad = new Uint8Array(64);
  innerPad.fill(0x36);
  outerPad.fill(0x5c);
  key.forEach((byte, index) => {
    innerPad[index] ^= byte;
    outerPad[index] ^= byte;
  });
  return bytesToHex(
    sha256(
      concatBytes([
        outerPad,
        sha256(concatBytes([innerPad, utf8Bytes(message)])),
      ]),
    ),
  );
}

function sha256Hex(bytes: Uint8Array): string {
  return bytesToHex(sha256(bytes));
}

function sha256(bytes: Uint8Array): Uint8Array {
  const constants = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ];
  const hash = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
    0x1f83d9ab, 0x5be0cd19,
  ];
  const bitLength = bytes.length * 8;
  const paddedLength = Math.ceil((bytes.length + 9) / 64) * 64;
  const padded = new Uint8Array(paddedLength);
  padded.set(bytes);
  padded[bytes.length] = 0x80;
  const view = new DataView(padded.buffer);
  view.setUint32(paddedLength - 4, bitLength, false);
  const words = new Array<number>(64);
  for (let offset = 0; offset < padded.length; offset += 64) {
    for (let index = 0; index < 16; index += 1) {
      words[index] = view.getUint32(offset + index * 4, false);
    }
    for (let index = 16; index < 64; index += 1) {
      const s0 =
        rotateRight(words[index - 15], 7) ^
        rotateRight(words[index - 15], 18) ^
        (words[index - 15] >>> 3);
      const s1 =
        rotateRight(words[index - 2], 17) ^
        rotateRight(words[index - 2], 19) ^
        (words[index - 2] >>> 10);
      words[index] = (words[index - 16] + s0 + words[index - 7] + s1) >>> 0;
    }
    let [a, b, c, d, e, f, g, h] = hash;
    for (let index = 0; index < 64; index += 1) {
      const s1 = rotateRight(e, 6) ^ rotateRight(e, 11) ^ rotateRight(e, 25);
      const ch = (e & f) ^ (~e & g);
      const temp1 = (h + s1 + ch + constants[index] + words[index]) >>> 0;
      const s0 = rotateRight(a, 2) ^ rotateRight(a, 13) ^ rotateRight(a, 22);
      const maj = (a & b) ^ (a & c) ^ (b & c);
      const temp2 = (s0 + maj) >>> 0;
      h = g;
      g = f;
      f = e;
      e = (d + temp1) >>> 0;
      d = c;
      c = b;
      b = a;
      a = (temp1 + temp2) >>> 0;
    }
    hash[0] = (hash[0] + a) >>> 0;
    hash[1] = (hash[1] + b) >>> 0;
    hash[2] = (hash[2] + c) >>> 0;
    hash[3] = (hash[3] + d) >>> 0;
    hash[4] = (hash[4] + e) >>> 0;
    hash[5] = (hash[5] + f) >>> 0;
    hash[6] = (hash[6] + g) >>> 0;
    hash[7] = (hash[7] + h) >>> 0;
  }
  const output = new Uint8Array(32);
  const outputView = new DataView(output.buffer);
  hash.forEach((word, index) => outputView.setUint32(index * 4, word, false));
  return output;
}

function rotateRight(value: number, shift: number): number {
  return (value >>> shift) | (value << (32 - shift));
}

function utf8Bytes(value: string): Uint8Array {
  const bytes: number[] = [];
  for (let index = 0; index < value.length; index += 1) {
    let codePoint = value.codePointAt(index) ?? 0;
    if (codePoint > 0xffff) {
      index += 1;
    }
    if (codePoint <= 0x7f) {
      bytes.push(codePoint);
    } else if (codePoint <= 0x7ff) {
      bytes.push(0xc0 | (codePoint >> 6), 0x80 | (codePoint & 0x3f));
    } else if (codePoint <= 0xffff) {
      bytes.push(
        0xe0 | (codePoint >> 12),
        0x80 | ((codePoint >> 6) & 0x3f),
        0x80 | (codePoint & 0x3f),
      );
    } else {
      bytes.push(
        0xf0 | (codePoint >> 18),
        0x80 | ((codePoint >> 12) & 0x3f),
        0x80 | ((codePoint >> 6) & 0x3f),
        0x80 | (codePoint & 0x3f),
      );
    }
  }
  return new Uint8Array(bytes);
}

function concatBytes(parts: Uint8Array[]): Uint8Array {
  const length = parts.reduce((sum, part) => sum + part.length, 0);
  const output = new Uint8Array(length);
  let offset = 0;
  parts.forEach((part) => {
    output.set(part, offset);
    offset += part.length;
  });
  return output;
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
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
  const configuredCode = getConfiguredWechatLoginCode();
  if (configuredCode) {
    return Promise.resolve(configuredCode);
  }
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

function getConfiguredWechatLoginCode(): string | undefined {
  const app = getApp<{
    globalData?: {
      wechatLoginCode?: string;
    };
  }>();
  const code = app.globalData?.wechatLoginCode?.trim();
  return code || undefined;
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
