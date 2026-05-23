import type {
  AdminRoleResponse,
  AdminUserSelector,
  ApiUsageListRequest,
  ApiUsageListResponse,
  AppLocale,
  ListAdminFeedbackRequest,
  ListAdminFeedbackResponse,
  BindEmailCodeRequest,
  BindEmailRequest,
  BindEmailResponse,
  CaptchaChallengeRequest,
  CaptchaChallengeResponse,
  ClientKey,
  ClientVersion,
  ClientVersionRequest,
  ContentListResponse,
  CreateGearAtlasSubmissionRequest,
  CreateGearPackingListRequest,
  CreateGearRequest,
  EmailLoginCodeRequest,
  EmailLoginRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearAtlasPublicItem,
  GearAtlasSubmission,
  GearCategory,
  GearOverviewRequest,
  GearOverviewResponse,
  GearPackingListDetail,
  GearTemplate,
  GearItem,
  GearSpecKeyRankingsResponse,
  GearStatsResponse,
  GearTagSuggestionsResponse,
  HealthResponse,
  KnotDetail,
  KnotFiltersResponse,
  KnotListResponse,
  KnotMediaAssetId,
  KnotMediaUploadResponse,
  KnotOfflineManifestResponse,
  ListKnotsRequest,
  ListRoadmapRequest,
  ListRoadmapResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  ListGearAtlasRequest,
  ListGearAtlasResponse,
  ListGearAtlasSubmissionsRequest,
  ListGearAtlasSubmissionsResponse,
  ListGearPackingListsRequest,
  ListGearPackingListsResponse,
  ListClientVersionsRequest,
  ListClientVersionsResponse,
  ListGearsRequest,
  ListGearsResponse,
  MetaResponse,
  PasswordLoginRequest,
  PasswordResetCodeRequest,
  PasswordResetRequest,
  ProfileUserResponse,
  RefreshTokenRequest,
  RegisterRequest,
  RejectGearAtlasSubmissionRequest,
  RoadmapInteractionStatusResponse,
  RoadmapItem,
  RoadmapItemRequest,
  SkillCategoriesResponse,
  SkillLocale,
  UpdateGearAtlasSubmissionRequest,
  UpdateGearPackingItemRequest,
  UpdateGearPackingListRequest,
  UpdateGearRequest,
  WechatLoginRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

export interface ApiClientOptions {
  baseUrl: string;
  assetsBaseUrl?: string;
  fetcher?: typeof fetch;
  accessToken?: string;
  refreshToken?: string;
  onSessionRefresh?: (response: WechatLoginResponse) => void;
}

const API_PREFIX = "/api/v1";
const HEALTH_PATH = "/healthz";

export class StellarTrailApiError extends Error {
  readonly status: number;
  readonly code?: string;
  readonly body?: unknown;

  constructor(status: number, message: string, code?: string, body?: unknown) {
    super(message);
    this.name = "StellarTrailApiError";
    this.status = status;
    this.code = code;
    this.body = body;
  }
}

export interface AdminKnotMediaUploadInput {
  knotId: string;
  assetId: KnotMediaAssetId;
  mediaType?: KnotMediaAssetId;
  file: Blob;
  filename?: string;
  attribution?: string;
  licenseNote?: string;
  sourceName?: string;
  sourcePath?: string;
}

export class StellarTrailApiClient {
  private readonly baseUrl: string;
  private readonly assetsBaseUrl: string;
  private readonly fetcher: typeof fetch;
  private accessToken?: string;
  private refreshToken?: string;
  private refreshPromise?: Promise<WechatLoginResponse>;
  private onSessionRefresh?: (response: WechatLoginResponse) => void;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.assetsBaseUrl = (options.assetsBaseUrl ?? options.baseUrl).replace(
      /\/$/,
      "",
    );
    this.fetcher = options.fetcher ?? globalThis.fetch.bind(globalThis);
    this.accessToken = options.accessToken;
    this.refreshToken = options.refreshToken;
    this.onSessionRefresh = options.onSessionRefresh;
  }

  setAccessToken(accessToken?: string): void {
    this.accessToken = accessToken;
  }

  setRefreshToken(refreshToken?: string): void {
    this.refreshToken = refreshToken;
  }

  setSessionTokens(accessToken?: string, refreshToken?: string): void {
    this.accessToken = accessToken;
    this.refreshToken = refreshToken;
  }

  setSessionRefreshHandler(
    handler?: (response: WechatLoginResponse) => void,
  ): void {
    this.onSessionRefresh = handler;
  }

  async health(): Promise<HealthResponse> {
    return this.get("/healthz");
  }

  async meta(): Promise<MetaResponse> {
    return this.get("/meta");
  }

  async getCurrentClientVersion(clientKey: ClientKey): Promise<ClientVersion> {
    return this.get(
      `/client-versions/current${queryString({ client_key: clientKey })}`,
    );
  }

  async listClientVersions(
    clientKey: ClientKey,
    request: Pick<ListClientVersionsRequest, "limit" | "cursor"> = {},
  ): Promise<ListClientVersionsResponse> {
    return this.get(
      `/client-versions${queryString({ ...request, client_key: clientKey })}`,
    );
  }

  async listSkills(locale?: SkillLocale): Promise<SkillCategoriesResponse> {
    return this.get("/skills", false, locale);
  }

  async listKnotFilters(locale?: SkillLocale): Promise<KnotFiltersResponse> {
    return this.get("/skills/knots/filters", false, locale);
  }

  async listKnots(
    request: ListKnotsRequest = {},
    locale?: SkillLocale,
  ): Promise<KnotListResponse> {
    return this.get(`/skills/knots/list${queryString(request)}`, false, locale);
  }

  async getKnotDetail(id: string, locale?: SkillLocale): Promise<KnotDetail> {
    return this.get(
      `/skills/knots/detail/${encodeURIComponent(id)}`,
      false,
      locale,
    );
  }

  async getKnotOfflineManifest(
    locale?: SkillLocale,
  ): Promise<KnotOfflineManifestResponse> {
    return this.get("/skills/knots/offline-manifest", false, locale);
  }

  async listApiUsage(
    request: ApiUsageListRequest = {},
  ): Promise<ApiUsageListResponse> {
    return this.get(`/admin/api-usage${queryString(request)}`, true);
  }

  async grantAdmin(request: AdminUserSelector): Promise<AdminRoleResponse> {
    return this.post("/admin/admins", request, true);
  }

  async revokeAdmin(request: AdminUserSelector): Promise<void> {
    await this.request(
      `/admin/admins${queryString(request)}`,
      { method: "DELETE" },
      true,
    );
  }

  async listAdminFeedback(
    request: ListAdminFeedbackRequest = {},
  ): Promise<ListAdminFeedbackResponse> {
    return this.get(`/admin/feedback${queryString(request)}`, true);
  }

  async deleteAdminFeedback(id: string): Promise<void> {
    await this.request(
      `/admin/feedback/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async restoreAdminFeedback(id: string): Promise<void> {
    await this.request(
      `/admin/feedback/${encodeURIComponent(id)}/restore`,
      { method: "POST" },
      true,
    );
  }

  async listAdminClientVersions(
    request: ListClientVersionsRequest = {},
  ): Promise<ListClientVersionsResponse> {
    return this.get(`/admin/client-versions${queryString(request)}`, true);
  }

  async createAdminClientVersion(
    request: ClientVersionRequest,
  ): Promise<ClientVersion> {
    return this.post("/admin/client-versions", request, true);
  }

  async updateAdminClientVersion(
    id: string,
    request: ClientVersionRequest,
  ): Promise<ClientVersion> {
    return this.patch(
      `/admin/client-versions/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async listRoadmap(
    request: ListRoadmapRequest = {},
  ): Promise<ListRoadmapResponse> {
    return this.get(`/roadmap${queryString(request)}`);
  }

  async listMyRoadmap(
    request: ListRoadmapRequest = {},
  ): Promise<ListRoadmapResponse> {
    return this.get(`/me/roadmap${queryString(request)}`, true);
  }

  async voteRoadmapItem(id: string): Promise<RoadmapInteractionStatusResponse> {
    return this.putRoadmapInteraction(id, "vote");
  }

  async unvoteRoadmapItem(
    id: string,
  ): Promise<RoadmapInteractionStatusResponse> {
    return this.deleteRoadmapInteraction(id, "vote");
  }

  async subscribeRoadmapItem(
    id: string,
  ): Promise<RoadmapInteractionStatusResponse> {
    return this.putRoadmapInteraction(id, "subscription");
  }

  async unsubscribeRoadmapItem(
    id: string,
  ): Promise<RoadmapInteractionStatusResponse> {
    return this.deleteRoadmapInteraction(id, "subscription");
  }

  private async putRoadmapInteraction(
    id: string,
    action: "vote" | "subscription",
  ): Promise<RoadmapInteractionStatusResponse> {
    const response = await this.request(
      `/me/roadmap/${encodeURIComponent(id)}/${action}`,
      { method: "PUT" },
      true,
    );
    return response.json() as Promise<RoadmapInteractionStatusResponse>;
  }

  private async deleteRoadmapInteraction(
    id: string,
    action: "vote" | "subscription",
  ): Promise<RoadmapInteractionStatusResponse> {
    const response = await this.request(
      `/me/roadmap/${encodeURIComponent(id)}/${action}`,
      { method: "DELETE" },
      true,
    );
    return response.json() as Promise<RoadmapInteractionStatusResponse>;
  }

  async listAdminRoadmap(
    request: ListRoadmapRequest = {},
  ): Promise<ListRoadmapResponse> {
    return this.get(`/admin/roadmap${queryString(request)}`, true);
  }

  async createAdminRoadmapItem(
    request: RoadmapItemRequest,
  ): Promise<RoadmapItem> {
    return this.post("/admin/roadmap", request, true);
  }

  async updateAdminRoadmapItem(
    id: string,
    request: RoadmapItemRequest,
  ): Promise<RoadmapItem> {
    return this.patch(
      `/admin/roadmap/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async deleteAdminRoadmapItem(id: string): Promise<void> {
    await this.request(
      `/admin/roadmap/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async uploadKnotMedia(
    input: AdminKnotMediaUploadInput,
  ): Promise<KnotMediaUploadResponse> {
    const form = new FormData();
    form.set("media_type", input.mediaType ?? input.assetId);
    form.set("file", input.file, input.filename ?? input.assetId);
    if (input.attribution) form.set("attribution", input.attribution);
    if (input.licenseNote) form.set("license_note", input.licenseNote);
    if (input.sourceName) form.set("source_name", input.sourceName);
    if (input.sourcePath) form.set("source_path", input.sourcePath);
    const response = await this.request(
      `/admin/skills/knots/${encodeURIComponent(input.knotId)}/media/${encodeURIComponent(input.assetId)}`,
      { method: "PUT", body: form },
      true,
    );
    return response.json() as Promise<KnotMediaUploadResponse>;
  }

  resolveAssetUrl(pathOrUrl: string): string {
    if (/^https?:\/\//i.test(pathOrUrl)) {
      return pathOrUrl;
    }
    return `${this.assetsBaseUrl}/${pathOrUrl.replace(/^\/+/, "")}`;
  }

  async listGearTemplates(
    locale?: AppLocale,
  ): Promise<ContentListResponse<GearTemplate>> {
    return this.get("/gear-templates", false, locale);
  }

  async getGearTemplate(id: string, locale?: AppLocale): Promise<GearTemplate> {
    return this.get(`/gear-templates/${encodeURIComponent(id)}`, false, locale);
  }

  async listGearAtlas(
    request: ListGearAtlasRequest = {},
    locale?: AppLocale,
  ): Promise<ListGearAtlasResponse> {
    return this.get(`/gear-atlas${queryString(request)}`, false, locale);
  }

  async getGearAtlasItem(
    id: string,
    locale?: AppLocale,
  ): Promise<GearAtlasPublicItem> {
    return this.get(`/gear-atlas/${encodeURIComponent(id)}`, false, locale);
  }

  async createGearAtlasSubmission(
    request: CreateGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission> {
    return this.post("/me/gear-atlas-submissions", request, true);
  }

  async createGearAtlasSubmissionFromGear(
    gearId: string,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/me/gears/${encodeURIComponent(gearId)}/atlas-submission`,
      undefined,
      true,
    );
  }

  async listMyGearAtlasSubmissions(
    request: Pick<ListGearAtlasSubmissionsRequest, "limit" | "cursor"> = {},
  ): Promise<ListGearAtlasSubmissionsResponse> {
    return this.get(`/me/gear-atlas-submissions${queryString(request)}`, true);
  }

  async listAdminGearAtlasSubmissions(
    request: ListGearAtlasSubmissionsRequest = {},
  ): Promise<ListGearAtlasSubmissionsResponse> {
    return this.get(
      `/admin/gear-atlas-submissions${queryString(request)}`,
      true,
    );
  }

  async getAdminGearAtlasSubmission(id: string): Promise<GearAtlasSubmission> {
    return this.get(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      true,
    );
  }

  async updateAdminGearAtlasSubmission(
    id: string,
    request: UpdateGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission> {
    return this.patch(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async approveGearAtlasSubmission(id: string): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/approve`,
      undefined,
      true,
    );
  }

  async deleteAdminGearAtlasSubmission(id: string): Promise<void> {
    await this.request(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async restoreAdminGearAtlasSubmission(
    id: string,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/restore`,
      undefined,
      true,
    );
  }

  async rejectGearAtlasSubmission(
    id: string,
    request: RejectGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/reject`,
      request,
      true,
    );
  }

  async loginWithWechatCode(
    request: WechatLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/wechat-login",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async sendEmailVerificationCode(
    request: EmailVerificationCodeRequest,
  ): Promise<EmailVerificationCodeResponse> {
    return this.post<EmailVerificationCodeResponse>(
      "/auth/email-verification-code",
      request,
    );
  }

  async sendEmailLoginCode(
    request: EmailLoginCodeRequest,
  ): Promise<EmailVerificationCodeResponse> {
    return this.post<EmailVerificationCodeResponse>(
      "/auth/email-login-code",
      request,
    );
  }

  async loginWithEmailCode(
    request: EmailLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/email-login",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async sendPasswordResetCode(
    request: PasswordResetCodeRequest,
  ): Promise<EmailVerificationCodeResponse> {
    return this.post<EmailVerificationCodeResponse>(
      "/auth/password-reset-code",
      request,
    );
  }

  async resetPassword(
    request: PasswordResetRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/password-reset",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async sendBindEmailCode(
    request: BindEmailCodeRequest,
  ): Promise<EmailVerificationCodeResponse> {
    return this.post<EmailVerificationCodeResponse>(
      "/me/email-binding-code",
      request,
      true,
    );
  }

  async bindEmail(request: BindEmailRequest): Promise<BindEmailResponse> {
    return this.post<BindEmailResponse>("/me/email-binding", request, true);
  }

  async getProfile(): Promise<ProfileUserResponse> {
    return this.get<ProfileUserResponse>("/me/profile", true);
  }

  async uploadProfileAvatar(
    file: Blob,
    filename = "avatar.png",
  ): Promise<ProfileUserResponse> {
    const form = new FormData();
    form.set("file", file, filename);
    const response = await this.request(
      "/me/profile/avatar",
      { method: "PUT", body: form },
      true,
    );
    return response.json() as Promise<ProfileUserResponse>;
  }

  async createCaptcha(
    request: CaptchaChallengeRequest,
  ): Promise<CaptchaChallengeResponse> {
    return this.post<CaptchaChallengeResponse>("/auth/captcha", request);
  }

  async register(request: RegisterRequest): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/register",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async loginWithPassword(
    request: PasswordLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/login",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async refreshSession(
    request: RefreshTokenRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/auth/refresh",
      request,
    );
    this.applyLoginResponse(response, true);
    return response;
  }

  async listGearCategories(
    tab?: "available" | "history",
  ): Promise<GearCategoriesResponse> {
    return this.get(`/me/gears/categories${queryString({ tab })}`, true);
  }

  async getGearStats(
    tab?: "available" | "history",
  ): Promise<GearStatsResponse> {
    return this.get(`/me/gears/stats${queryString({ tab })}`, true);
  }

  async getGearOverview(
    request: GearOverviewRequest = {},
  ): Promise<GearOverviewResponse> {
    return this.get(`/me/gears/overview${queryString(request)}`, true);
  }

  async getGearSpecKeyRankings(
    category: GearCategory,
  ): Promise<GearSpecKeyRankingsResponse> {
    return this.get(
      `/me/gears/spec-key-rankings${queryString({ category })}`,
      true,
    );
  }

  async getGearTagSuggestions(limit = 20): Promise<GearTagSuggestionsResponse> {
    return this.get(`/me/gears/tag-suggestions${queryString({ limit })}`, true);
  }

  async listGears(request: ListGearsRequest = {}): Promise<ListGearsResponse> {
    return this.get(`/me/gears${queryString(request)}`, true);
  }

  async getGear(id: string): Promise<GearItem> {
    return this.get(`/me/gears/${encodeURIComponent(id)}`, true);
  }

  async createGear(request: CreateGearRequest): Promise<GearItem> {
    return this.post("/me/gears", request, true);
  }

  async updateGear(id: string, request: UpdateGearRequest): Promise<GearItem> {
    return this.patch(`/me/gears/${encodeURIComponent(id)}`, request, true);
  }

  async archiveGear(id: string): Promise<void> {
    await this.request(
      `/me/gears/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async deleteGear(id: string): Promise<void> {
    await this.request(
      `/me/gears/${encodeURIComponent(id)}/delete`,
      { method: "POST" },
      true,
    );
  }

  async undeleteGear(id: string): Promise<GearItem> {
    return this.post(
      `/me/gears/${encodeURIComponent(id)}/undelete`,
      undefined,
      true,
    );
  }

  async restoreGear(id: string): Promise<GearItem> {
    return this.post(
      `/me/gears/${encodeURIComponent(id)}/restore`,
      undefined,
      true,
    );
  }

  async exportGearsCsv(tab?: "available" | "history"): Promise<string> {
    const response = await this.request(
      `/me/gears/export${queryString({ tab, format: "csv" })}`,
      {},
      true,
    );
    return response.text();
  }

  async importGears(request: ImportGearsRequest): Promise<ImportGearsResponse> {
    return this.post("/me/gears/import", request, true);
  }

  async listGearPackingLists(
    request: ListGearPackingListsRequest = {},
  ): Promise<ListGearPackingListsResponse> {
    return this.get(`/me/packing-lists${queryString(request)}`, true);
  }

  async createGearPackingList(
    request: CreateGearPackingListRequest,
  ): Promise<GearPackingListDetail> {
    return this.post("/me/packing-lists", request, true);
  }

  async getGearPackingList(id: string): Promise<GearPackingListDetail> {
    return this.get(`/me/packing-lists/${encodeURIComponent(id)}`, true);
  }

  async updateGearPackingList(
    id: string,
    request: UpdateGearPackingListRequest,
  ): Promise<GearPackingListDetail> {
    return this.patch(
      `/me/packing-lists/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async deleteGearPackingList(id: string): Promise<void> {
    await this.request(
      `/me/packing-lists/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async addGearPackingItems(
    id: string,
    gearIds: string[],
  ): Promise<GearPackingListDetail> {
    return this.post(
      `/me/packing-lists/${encodeURIComponent(id)}/items`,
      { gear_ids: gearIds },
      true,
    );
  }

  async updateGearPackingItem(
    id: string,
    itemId: string,
    request: UpdateGearPackingItemRequest,
  ): Promise<GearPackingListDetail> {
    return this.patch(
      `/me/packing-lists/${encodeURIComponent(id)}/items/${encodeURIComponent(itemId)}`,
      request,
      true,
    );
  }

  async removeGearPackingItem(
    id: string,
    itemId: string,
  ): Promise<GearPackingListDetail> {
    const response = await this.request(
      `/me/packing-lists/${encodeURIComponent(id)}/items/${encodeURIComponent(itemId)}`,
      { method: "DELETE" },
      true,
    );
    return response.json() as Promise<GearPackingListDetail>;
  }

  private async get<T>(
    path: string,
    auth = false,
    locale?: AppLocale,
  ): Promise<T> {
    const response = await this.request(path, {}, auth, locale);
    return response.json() as Promise<T>;
  }

  private async post<T>(
    path: string,
    body?: unknown,
    auth = false,
  ): Promise<T> {
    const response = await this.request(
      path,
      {
        method: "POST",
        body: body === undefined ? undefined : JSON.stringify(body),
        headers:
          body === undefined
            ? undefined
            : { "content-type": "application/json" },
      },
      auth,
    );
    return response.json() as Promise<T>;
  }

  private async patch<T>(
    path: string,
    body: unknown,
    auth = false,
  ): Promise<T> {
    const response = await this.request(
      path,
      {
        method: "PATCH",
        body: JSON.stringify(body),
        headers: { "content-type": "application/json" },
      },
      auth,
    );
    return response.json() as Promise<T>;
  }

  private async request(
    path: string,
    init: RequestInit = {},
    auth = false,
    locale?: AppLocale,
  ): Promise<Response> {
    let response = await this.send(path, init, auth, locale);
    if (response.status === 401 && auth && this.refreshToken) {
      await this.refreshWithStoredToken();
      response = await this.send(path, init, auth, locale);
    }
    if (!response.ok) {
      throw await apiErrorFromResponse(response);
    }
    return response;
  }

  private async send(
    path: string,
    init: RequestInit,
    auth: boolean,
    locale?: AppLocale,
  ): Promise<Response> {
    const headers = new Headers(init.headers);
    if (locale) {
      headers.set("X-StellarTrail-Locale", locale);
    }
    if (auth) {
      if (!this.accessToken) {
        throw new Error("StellarTrail API request requires an access token");
      }
      headers.set("authorization", `Bearer ${this.accessToken}`);
    }
    return this.fetcher(`${this.baseUrl}${versionedApiPath(path)}`, {
      ...init,
      headers,
    });
  }

  private async refreshWithStoredToken(): Promise<void> {
    if (!this.refreshToken) {
      throw new Error("StellarTrail API request requires a refresh token");
    }
    if (!this.refreshPromise) {
      const refreshToken = this.refreshToken;
      this.refreshPromise = this.refreshSession({
        refresh_token: refreshToken,
      }).finally(() => {
        this.refreshPromise = undefined;
      });
    }
    await this.refreshPromise;
  }

  private applyLoginResponse(
    response: WechatLoginResponse,
    notify = false,
  ): void {
    this.accessToken = response.access_token;
    this.refreshToken = response.refresh_token;
    if (notify) {
      this.onSessionRefresh?.(response);
    }
  }
}

async function apiErrorFromResponse(
  response: Response,
): Promise<StellarTrailApiError> {
  let body: unknown;
  if (response.headers.get("content-type")?.includes("application/json")) {
    body = await response.json().catch(() => undefined);
  } else {
    body = await response.text().catch(() => undefined);
  }

  const code =
    isRecord(body) && typeof body.code === "string" ? body.code : undefined;
  const message =
    isRecord(body) && typeof body.message === "string"
      ? body.message
      : `StellarTrail API request failed: ${response.status}`;

  return new StellarTrailApiError(response.status, message, code, body);
}

function queryString(params: object): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== "") {
      search.set(key, String(value));
    }
  }
  const value = search.toString();
  return value ? `?${value}` : "";
}

function versionedApiPath(path: string): string {
  if (path === HEALTH_PATH || path.startsWith(`${API_PREFIX}/`)) {
    return path;
  }
  const normalized = path.startsWith("/") ? path : `/${path}`;
  return `${API_PREFIX}${normalized}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
