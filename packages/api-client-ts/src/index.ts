import type {
  AdminRoleResponse,
  AdminUserSelector,
  ApiUsageListRequest,
  ApiUsageListResponse,
  AppLocale,
  AcceptKnotDisclaimerRequest,
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
  CreateTripInvitationResponse,
  CreateTripRequest,
  EmailLoginCodeRequest,
  EmailLoginRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearAtlasPublicItem,
  GearAtlasSubmission,
  GearCategory,
  GenerateGearAtlasLocalizationDraftRequest,
  GearOverviewRequest,
  GearOverviewResponse,
  GearPackingListDetail,
  GearTemplate,
  GearItem,
  GearSpecKeyRankingsResponse,
  GearStatsResponse,
  GearTab,
  GearTagSuggestionsResponse,
  HealthResponse,
  KnotDetail,
  KnotFiltersResponse,
  KnotListResponse,
  KnotMediaAssetId,
  KnotMediaUploadResponse,
  KnotOfflineManifestResponse,
  KnotDisclaimerResponse,
  ListKnotsRequest,
  ListRoadmapRequest,
  ListRoadmapResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  ImportTripPackingListRequest,
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
  ListTripsRequest,
  ListTripsResponse,
  ListTrailsResponse,
  MapAnnotation,
  MapAnnotationRequest,
  MapTrailLink,
  MapConfigResponse,
  OutdoorExperienceMapStateResponse,
  TripHomeHighlightResponse,
  MetaResponse,
  PasswordLoginRequest,
  PasswordResetCodeRequest,
  PasswordResetRequest,
  OutdoorProfileResponse,
  ListOutdoorExperiencesResponse,
  OutdoorExperience,
  OutdoorExperienceRequest,
  ProfileUserResponse,
  RefreshTokenRequest,
  RegisterRequest,
  RejectGearAtlasSubmissionRequest,
  RoadmapInteractionStatusResponse,
  RoadmapItem,
  RoadmapItemRequest,
  SkillCategoriesResponse,
  SkillLocale,
  Trail,
  TrailLinkRequest,
  TripsMapOverviewResponse,
  TripMapStateResponse,
  TripRecordCreateRequest,
  TripRecordPatchRequest,
  TripDetail,
  UpdateGearAtlasLocalizationRequest,
  UpdateMapAnnotationRequest,
  UpdateGearAtlasSubmissionRequest,
  UpdateGearPackingItemRequest,
  UpdateGearPackingListRequest,
  UpdateGearRequest,
  UpdateOutdoorProfileRequest,
  UpdateTripSectionsRequest,
  UpdateTrailRequest,
  UpdateTripRequest,
  WechatLoginRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

export interface ApiClientOptions {
  baseUrl: string;
  clientIdentity: string;
  assetsBaseUrl?: string;
  fetcher?: typeof fetch;
  accessToken?: string;
  refreshToken?: string;
  onSessionRefresh?: (response: WechatLoginResponse) => void;
  requestSignature?: ClientRequestSignatureConfig;
  nonceProvider?: () => string;
}

const API_PREFIX = "/api/v1";
const HEALTH_PATH = "/healthz";
const SIGNATURE_ALGORITHM = "STELLARTRAIL-HMAC-SHA256";
const SIGNING_FIELD_APP_ID = "app_id";
const SIGNING_FIELD_NONCE = "nonce";
const SIGNING_FIELD_SIGNATURE = "signature";

export interface ClientRequestSignatureConfig {
  app_id: string;
  app_secret: string;
}

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
  private readonly clientIdentity: string;
  private readonly fetcher: typeof fetch;
  private readonly requestSignature?: ClientRequestSignatureConfig;
  private readonly nonceProvider: () => string;
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
    this.clientIdentity = options.clientIdentity.trim();
    this.fetcher = options.fetcher ?? globalThis.fetch.bind(globalThis);
    this.requestSignature = normalizeRequestSignature(options.requestSignature);
    this.nonceProvider = options.nonceProvider ?? createSignatureNonce;
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

  async getKnotDisclaimer(): Promise<KnotDisclaimerResponse> {
    return this.get("/me/skills/knots/disclaimer", true);
  }

  async acceptKnotDisclaimer(
    request: AcceptKnotDisclaimerRequest = {},
  ): Promise<KnotDisclaimerResponse> {
    return this.post("/me/skills/knots/disclaimer/acceptance", request, true);
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
    const multipart = await buildMultipartBody({
      fields: [
        { name: "media_type", value: input.mediaType ?? input.assetId },
        ...(input.attribution
          ? [{ name: "attribution", value: input.attribution }]
          : []),
        ...(input.licenseNote
          ? [{ name: "license_note", value: input.licenseNote }]
          : []),
        ...(input.sourceName
          ? [{ name: "source_name", value: input.sourceName }]
          : []),
        ...(input.sourcePath
          ? [{ name: "source_path", value: input.sourcePath }]
          : []),
      ],
      file: input.file,
      filename: input.filename ?? input.assetId,
      contentType: knotMediaAssetContentType(input.assetId),
    });
    const response = await this.request(
      `/admin/skills/knots/${encodeURIComponent(input.knotId)}/media/${encodeURIComponent(input.assetId)}`,
      {
        method: "PUT",
        body: multipart.body,
        headers: { "content-type": multipart.contentType },
      },
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
    locale?: AppLocale,
  ): Promise<ListGearAtlasSubmissionsResponse> {
    return this.get(
      `/admin/gear-atlas-submissions${queryString(request)}`,
      true,
      locale,
    );
  }

  async getAdminGearAtlasSubmission(
    id: string,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.get(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      true,
      locale,
    );
  }

  async updateAdminGearAtlasSubmission(
    id: string,
    request: UpdateGearAtlasSubmissionRequest,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.patch(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      request,
      true,
      locale,
    );
  }

  async updateAdminGearAtlasLocalization(
    id: string,
    localization: AppLocale,
    request: UpdateGearAtlasLocalizationRequest,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.put(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/localizations/${encodeURIComponent(localization)}`,
      request,
      true,
      locale,
    );
  }

  async generateAdminGearAtlasLocalizationDraft(
    id: string,
    localization: AppLocale,
    request: GenerateGearAtlasLocalizationDraftRequest = {},
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/localizations/${encodeURIComponent(localization)}/generate-draft`,
      request,
      true,
      locale,
    );
  }

  async approveGearAtlasSubmission(
    id: string,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/approve`,
      undefined,
      true,
      locale,
    );
  }

  async deleteAdminGearAtlasSubmission(
    id: string,
    locale?: AppLocale,
  ): Promise<void> {
    await this.request(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
      locale,
    );
  }

  async restoreAdminGearAtlasSubmission(
    id: string,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/restore`,
      undefined,
      true,
      locale,
    );
  }

  async rejectGearAtlasSubmission(
    id: string,
    request: RejectGearAtlasSubmissionRequest,
    locale?: AppLocale,
  ): Promise<GearAtlasSubmission> {
    return this.post(
      `/admin/gear-atlas-submissions/${encodeURIComponent(id)}/reject`,
      request,
      true,
      locale,
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

  async getOutdoorProfile(): Promise<OutdoorProfileResponse> {
    return this.get<OutdoorProfileResponse>("/me/profile/outdoor", true);
  }

  async updateOutdoorProfile(
    request: UpdateOutdoorProfileRequest,
  ): Promise<OutdoorProfileResponse> {
    return this.patch<OutdoorProfileResponse>(
      "/me/profile/outdoor",
      request,
      true,
    );
  }

  async listOutdoorExperiences(): Promise<ListOutdoorExperiencesResponse> {
    return this.get<ListOutdoorExperiencesResponse>(
      "/me/outdoor-experiences",
      true,
    );
  }

  async createOutdoorExperience(
    request: OutdoorExperienceRequest,
  ): Promise<OutdoorExperience> {
    return this.post<OutdoorExperience>(
      "/me/outdoor-experiences",
      request,
      true,
    );
  }

  async getOutdoorExperience(id: string): Promise<OutdoorExperience> {
    return this.get<OutdoorExperience>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}`,
      true,
    );
  }

  async updateOutdoorExperience(
    id: string,
    request: OutdoorExperienceRequest,
  ): Promise<OutdoorExperience> {
    return this.patch<OutdoorExperience>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async deleteOutdoorExperience(id: string): Promise<void> {
    await this.request(
      `/me/outdoor-experiences/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async getMapConfig(): Promise<MapConfigResponse> {
    return this.get<MapConfigResponse>("/me/map/config", true);
  }

  async listTrails(): Promise<ListTrailsResponse> {
    return this.get<ListTrailsResponse>("/me/trails", true);
  }

  async uploadTrail(file: Blob, filename: string): Promise<Trail> {
    return this.uploadTrailForm<Trail>("/me/trails", file, filename);
  }

  async getTrail(id: string): Promise<Trail> {
    return this.get<Trail>(`/me/trails/${encodeURIComponent(id)}`, true);
  }

  async updateTrail(id: string, request: UpdateTrailRequest): Promise<Trail> {
    return this.patch<Trail>(
      `/me/trails/${encodeURIComponent(id)}`,
      request,
      true,
    );
  }

  async deleteTrail(id: string): Promise<void> {
    await this.request(
      `/me/trails/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async downloadTrailFile(id: string): Promise<Blob> {
    const response = await this.request(
      `/me/trails/${encodeURIComponent(id)}/file`,
      {},
      true,
    );
    return response.blob();
  }

  async getTripMap(id: string): Promise<TripMapStateResponse> {
    return this.get<TripMapStateResponse>(
      `/me/trips/${encodeURIComponent(id)}/map`,
      true,
    );
  }

  async getTripsMapOverview(): Promise<TripsMapOverviewResponse> {
    return this.get<TripsMapOverviewResponse>("/me/trips/map-overview", true);
  }

  async uploadTripTrail(
    id: string,
    file: Blob,
    filename: string,
  ): Promise<MapTrailLink> {
    return this.uploadTrailForm<MapTrailLink>(
      `/me/trips/${encodeURIComponent(id)}/trails`,
      file,
      filename,
    );
  }

  async linkTripTrail(
    id: string,
    request: TrailLinkRequest,
  ): Promise<MapTrailLink> {
    return this.post<MapTrailLink>(
      `/me/trips/${encodeURIComponent(id)}/trail-links`,
      request,
      true,
    );
  }

  async unlinkTripTrail(id: string, trailId: string): Promise<void> {
    await this.request(
      `/me/trips/${encodeURIComponent(id)}/trail-links/${encodeURIComponent(trailId)}`,
      { method: "DELETE" },
      true,
    );
  }

  async listTripMapAnnotations(id: string): Promise<MapAnnotation[]> {
    return this.get<MapAnnotation[]>(
      `/me/trips/${encodeURIComponent(id)}/map-annotations`,
      true,
    );
  }

  async createTripMapAnnotation(
    id: string,
    request: MapAnnotationRequest,
  ): Promise<MapAnnotation> {
    return this.post<MapAnnotation>(
      `/me/trips/${encodeURIComponent(id)}/map-annotations`,
      request,
      true,
    );
  }

  async updateTripMapAnnotation(
    id: string,
    annotationId: string,
    request: UpdateMapAnnotationRequest,
  ): Promise<MapAnnotation> {
    return this.patch<MapAnnotation>(
      `/me/trips/${encodeURIComponent(id)}/map-annotations/${encodeURIComponent(annotationId)}`,
      request,
      true,
    );
  }

  async deleteTripMapAnnotation(
    id: string,
    annotationId: string,
  ): Promise<void> {
    await this.request(
      `/me/trips/${encodeURIComponent(id)}/map-annotations/${encodeURIComponent(annotationId)}`,
      { method: "DELETE" },
      true,
    );
  }

  async getOutdoorExperienceMap(
    id: string,
  ): Promise<OutdoorExperienceMapStateResponse> {
    return this.get<OutdoorExperienceMapStateResponse>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/map`,
      true,
    );
  }

  async linkOutdoorExperienceTrail(
    id: string,
    request: TrailLinkRequest,
  ): Promise<MapTrailLink> {
    return this.post<MapTrailLink>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/trail-links`,
      request,
      true,
    );
  }

  async unlinkOutdoorExperienceTrail(
    id: string,
    trailId: string,
  ): Promise<void> {
    await this.request(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/trail-links/${encodeURIComponent(trailId)}`,
      { method: "DELETE" },
      true,
    );
  }

  async listOutdoorExperienceMapAnnotations(
    id: string,
  ): Promise<MapAnnotation[]> {
    return this.get<MapAnnotation[]>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/map-annotations`,
      true,
    );
  }

  async createOutdoorExperienceMapAnnotation(
    id: string,
    request: MapAnnotationRequest,
  ): Promise<MapAnnotation> {
    return this.post<MapAnnotation>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/map-annotations`,
      request,
      true,
    );
  }

  async updateOutdoorExperienceMapAnnotation(
    id: string,
    annotationId: string,
    request: UpdateMapAnnotationRequest,
  ): Promise<MapAnnotation> {
    return this.patch<MapAnnotation>(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/map-annotations/${encodeURIComponent(annotationId)}`,
      request,
      true,
    );
  }

  async deleteOutdoorExperienceMapAnnotation(
    id: string,
    annotationId: string,
  ): Promise<void> {
    await this.request(
      `/me/outdoor-experiences/${encodeURIComponent(id)}/map-annotations/${encodeURIComponent(annotationId)}`,
      { method: "DELETE" },
      true,
    );
  }

  async uploadProfileAvatar(
    file: Blob,
    filename = "avatar.png",
  ): Promise<ProfileUserResponse> {
    const multipart = await buildMultipartBody({
      fields: [],
      file,
      filename,
    });
    const response = await this.request(
      "/me/profile/avatar",
      {
        method: "PUT",
        body: multipart.body,
        headers: { "content-type": multipart.contentType },
      },
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

  async listGearCategories(tab?: GearTab): Promise<GearCategoriesResponse> {
    return this.get(`/me/gears/categories${queryString({ tab })}`, true);
  }

  async getGearStats(tab?: GearTab): Promise<GearStatsResponse> {
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

  async exportGearsCsv(tab?: GearTab): Promise<string> {
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

  async listTrips(request: ListTripsRequest = {}): Promise<ListTripsResponse> {
    return this.get(`/me/trips${queryString(request)}`, true);
  }

  async getTripHomeHighlight(
    today?: string,
  ): Promise<TripHomeHighlightResponse> {
    return this.get(`/me/trips/home-highlight${queryString({ today })}`, true);
  }

  async createTrip(request: CreateTripRequest): Promise<TripDetail> {
    return this.post("/me/trips", request, true);
  }

  async getTrip(id: string): Promise<TripDetail> {
    return this.get(`/me/trips/${encodeURIComponent(id)}`, true);
  }

  async updateTrip(
    id: string,
    request: UpdateTripRequest,
  ): Promise<TripDetail> {
    return this.patch(`/me/trips/${encodeURIComponent(id)}`, request, true);
  }

  async deleteTrip(id: string): Promise<void> {
    await this.request(
      `/me/trips/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async convertTripToOutdoorExperience(id: string): Promise<OutdoorExperience> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/convert-to-outdoor-experience`,
      undefined,
      true,
    );
  }

  async updateTripSections(
    id: string,
    request: UpdateTripSectionsRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/sections`,
      request,
      true,
    );
  }

  async createTripInvitation(
    id: string,
  ): Promise<CreateTripInvitationResponse> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/invitations`,
      undefined,
      true,
    );
  }

  async acceptTripInvitation(token: string): Promise<TripDetail> {
    return this.post(
      `/me/trip-invitations/${encodeURIComponent(token)}/accept`,
      undefined,
      true,
    );
  }

  async updateTripMember(
    id: string,
    memberId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/members/${encodeURIComponent(memberId)}`,
      request,
      true,
    );
  }

  async removeTripMember(id: string, memberId: string): Promise<TripDetail> {
    const response = await this.request(
      `/me/trips/${encodeURIComponent(id)}/members/${encodeURIComponent(memberId)}`,
      { method: "DELETE" },
      true,
    );
    return response.json() as Promise<TripDetail>;
  }

  async importTripPackingList(
    id: string,
    request: ImportTripPackingListRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/personal-gear/import-packing-list`,
      request,
      true,
    );
  }

  async createTripPersonalGearItem(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/personal-gear`,
      request,
      true,
    );
  }

  async updateTripPersonalGearItem(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/personal-gear/${encodeURIComponent(itemId)}`,
      request,
      true,
    );
  }

  async deleteTripPersonalGearItem(
    id: string,
    itemId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "personal-gear", itemId);
  }

  async createTripSharedGearDemand(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/shared-gear-demands`,
      request,
      true,
    );
  }

  async updateTripSharedGearDemand(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/shared-gear-demands/${encodeURIComponent(itemId)}`,
      request,
      true,
    );
  }

  async deleteTripSharedGearDemand(
    id: string,
    itemId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "shared-gear-demands", itemId);
  }

  async bindTripSharedGearDemandMyGear(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/shared-gear-demands/${encodeURIComponent(itemId)}/bind-my-gear`,
      request,
      true,
    );
  }

  async fillTripSharedGearDemandConcreteGear(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/shared-gear-demands/${encodeURIComponent(itemId)}/fill-concrete-gear`,
      request,
      true,
    );
  }

  async createTripItineraryDay(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/itinerary-days`,
      request,
      true,
    );
  }

  async updateTripItineraryDay(
    id: string,
    dayId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/itinerary-days/${encodeURIComponent(dayId)}`,
      request,
      true,
    );
  }

  async deleteTripItineraryDay(id: string, dayId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "itinerary-days", dayId);
  }

  async createTripItineraryTimeSlot(
    id: string,
    dayId: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/itinerary-days/${encodeURIComponent(dayId)}/time-slots`,
      request,
      true,
    );
  }

  async updateTripItineraryTimeSlot(
    id: string,
    dayId: string,
    slotId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/itinerary-days/${encodeURIComponent(dayId)}/time-slots/${encodeURIComponent(slotId)}`,
      request,
      true,
    );
  }

  async deleteTripItineraryTimeSlot(
    id: string,
    dayId: string,
    slotId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(
      id,
      `itinerary-days/${encodeURIComponent(dayId)}/time-slots`,
      slotId,
    );
  }

  async createTripRouteSegment(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/route-segments`,
      request,
      true,
    );
  }

  async updateTripRouteSegment(
    id: string,
    segmentId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/route-segments/${encodeURIComponent(segmentId)}`,
      request,
      true,
    );
  }

  async deleteTripRouteSegment(
    id: string,
    segmentId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "route-segments", segmentId);
  }

  async createTripSegmentAssignment(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "segment-assignments", request);
  }

  async updateTripSegmentAssignment(
    id: string,
    assignmentId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(
      id,
      "segment-assignments",
      assignmentId,
      request,
    );
  }

  async deleteTripSegmentAssignment(
    id: string,
    assignmentId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "segment-assignments", assignmentId);
  }

  async createTripFoodMeal(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/food-meals`,
      request,
      true,
    );
  }

  async updateTripFoodMeal(
    id: string,
    mealId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/food-meals/${encodeURIComponent(mealId)}`,
      request,
      true,
    );
  }

  async deleteTripFoodMeal(id: string, mealId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "food-meals", mealId);
  }

  async createTripFoodItem(
    id: string,
    mealId: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/food-meals/${encodeURIComponent(mealId)}/items`,
      request,
      true,
    );
  }

  async updateTripFoodItem(
    id: string,
    mealId: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/food-meals/${encodeURIComponent(mealId)}/items/${encodeURIComponent(itemId)}`,
      request,
      true,
    );
  }

  async deleteTripFoodItem(
    id: string,
    mealId: string,
    itemId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(
      id,
      `food-meals/${encodeURIComponent(mealId)}/items`,
      itemId,
    );
  }

  async createTripFoodSupply(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "food-supplies", request);
  }

  async updateTripFoodSupply(
    id: string,
    supplyId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(id, "food-supplies", supplyId, request);
  }

  async deleteTripFoodSupply(
    id: string,
    supplyId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "food-supplies", supplyId);
  }

  async createTripMedicalItem(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/medical-items`,
      request,
      true,
    );
  }

  async updateTripMedicalItem(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/medical-items/${encodeURIComponent(itemId)}`,
      request,
      true,
    );
  }

  async deleteTripMedicalItem(id: string, itemId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "medical-items", itemId);
  }

  async createTripSafetyRisk(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "safety-risks", request);
  }

  async updateTripSafetyRisk(
    id: string,
    riskId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(id, "safety-risks", riskId, request);
  }

  async deleteTripSafetyRisk(id: string, riskId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "safety-risks", riskId);
  }

  async createTripRescueContact(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "rescue-contacts", request);
  }

  async updateTripRescueContact(
    id: string,
    contactId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(id, "rescue-contacts", contactId, request);
  }

  async deleteTripRescueContact(
    id: string,
    contactId: string,
  ): Promise<TripDetail> {
    return this.deleteTripRecord(id, "rescue-contacts", contactId);
  }

  async createTripBudgetItem(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "budget-items", request);
  }

  async updateTripBudgetItem(
    id: string,
    itemId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(id, "budget-items", itemId, request);
  }

  async deleteTripBudgetItem(id: string, itemId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "budget-items", itemId);
  }

  async createTripGoalItem(
    id: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.createTripRecord(id, "goals", request);
  }

  async updateTripGoalItem(
    id: string,
    goalId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.updateTripRecord(id, "goals", goalId, request);
  }

  async deleteTripGoalItem(id: string, goalId: string): Promise<TripDetail> {
    return this.deleteTripRecord(id, "goals", goalId);
  }

  private async createTripRecord(
    id: string,
    collectionPath: string,
    request: TripRecordCreateRequest,
  ): Promise<TripDetail> {
    return this.post(
      `/me/trips/${encodeURIComponent(id)}/${collectionPath}`,
      request,
      true,
    );
  }

  private async updateTripRecord(
    id: string,
    collectionPath: string,
    recordId: string,
    request: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    return this.patch(
      `/me/trips/${encodeURIComponent(id)}/${collectionPath}/${encodeURIComponent(recordId)}`,
      request,
      true,
    );
  }

  private async deleteTripRecord(
    id: string,
    collectionPath: string,
    recordId: string,
  ): Promise<TripDetail> {
    const response = await this.request(
      `/me/trips/${encodeURIComponent(id)}/${collectionPath}/${encodeURIComponent(recordId)}`,
      { method: "DELETE" },
      true,
    );
    return response.json() as Promise<TripDetail>;
  }

  private async get<T>(
    path: string,
    auth = false,
    locale?: AppLocale,
  ): Promise<T> {
    const response = await this.request(path, {}, auth, locale);
    return response.json() as Promise<T>;
  }

  private async uploadTrailForm<T>(
    path: string,
    file: Blob,
    filename: string,
  ): Promise<T> {
    const form = new FormData();
    form.set("file", file, filename);
    const response = await this.request(
      path,
      { method: "POST", body: form },
      true,
    );
    return response.json() as Promise<T>;
  }

  private async post<T>(
    path: string,
    body?: unknown,
    auth = false,
    locale?: AppLocale,
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
      locale,
    );
    return response.json() as Promise<T>;
  }

  private async put<T>(
    path: string,
    body: unknown,
    auth = false,
    locale?: AppLocale,
  ): Promise<T> {
    const response = await this.request(
      path,
      {
        method: "PUT",
        body: JSON.stringify(body),
        headers: { "content-type": "application/json" },
      },
      auth,
      locale,
    );
    return response.json() as Promise<T>;
  }

  private async patch<T>(
    path: string,
    body: unknown,
    auth = false,
    locale?: AppLocale,
  ): Promise<T> {
    const response = await this.request(
      path,
      {
        method: "PATCH",
        body: JSON.stringify(body),
        headers: { "content-type": "application/json" },
      },
      auth,
      locale,
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
    headers.set("X-StellarTrail-Client", this.clientIdentity);
    if (locale) {
      headers.set("X-StellarTrail-Locale", locale);
    }
    if (auth) {
      if (!this.accessToken) {
        throw new Error("StellarTrail API request requires an access token");
      }
      headers.set("authorization", `Bearer ${this.accessToken}`);
    }
    const method = (init.method ?? "GET").toUpperCase();
    const request = await this.signRequestIfConfigured(
      method,
      versionedApiPath(path),
      init.body,
      headers,
    );
    return this.fetcher(`${this.baseUrl}${request.path}`, {
      ...init,
      body: request.body,
      headers,
    });
  }

  private async signRequestIfConfigured(
    method: string,
    path: string,
    body: BodyInit | null | undefined,
    headers: Headers,
  ): Promise<{ path: string; body: BodyInit | null | undefined }> {
    if (!this.requestSignature || !shouldSignRequest(method, path)) {
      return { path, body };
    }
    const parsed = parseRequestPath(path);
    if (isJsonRequest(headers) && body !== undefined && body !== null) {
      if (typeof body !== "string") {
        throw new Error("Signed JSON API requests must use a string body.");
      }
      const unsignedJson = parseSignedJsonBody(body);
      const nonce = this.nonceProvider().trim();
      const bodyHash = await sha256Hex(
        utf8Bytes(canonicalJsonWithoutSigningFields(unsignedJson)),
      );
      const signature = await hmacSha256Hex(
        this.requestSignature.app_secret,
        canonicalRequest(
          method,
          parsed.path,
          canonicalQuery(parsed.query),
          bodyHash,
          this.requestSignature.app_id,
          nonce,
        ),
      );
      return {
        path,
        body: JSON.stringify({
          ...unsignedJson,
          [SIGNING_FIELD_APP_ID]: this.requestSignature.app_id,
          [SIGNING_FIELD_NONCE]: nonce,
          [SIGNING_FIELD_SIGNATURE]: signature,
        }),
      };
    }

    const bodyHash = await sha256Hex(await requestBodyBytes(body));
    return {
      path: await this.signedQueryPath(method, parsed, bodyHash),
      body,
    };
  }

  private async signedQueryPath(
    method: string,
    parsed: ParsedRequestPath,
    bodyHash: string,
  ): Promise<string> {
    if (!this.requestSignature) {
      return buildRequestPath(parsed.path, parsed.query);
    }
    const nonce = this.nonceProvider().trim();
    const queryWithFields = queryWithSigningFields(parsed.query, {
      [SIGNING_FIELD_APP_ID]: this.requestSignature.app_id,
      [SIGNING_FIELD_NONCE]: nonce,
    });
    const signature = await hmacSha256Hex(
      this.requestSignature.app_secret,
      canonicalRequest(
        method,
        parsed.path,
        canonicalQuery(queryWithFields),
        bodyHash,
        this.requestSignature.app_id,
        nonce,
      ),
    );
    return buildRequestPath(
      parsed.path,
      queryWithSigningFields(queryWithFields, {
        [SIGNING_FIELD_SIGNATURE]: signature,
      }),
    );
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

interface ParsedRequestPath {
  path: string;
  query: string;
}

interface MultipartTextField {
  name: string;
  value: string;
}

interface MultipartBody {
  body: ArrayBuffer;
  contentType: string;
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

function normalizeRequestSignature(
  config: ClientRequestSignatureConfig | undefined,
): ClientRequestSignatureConfig | undefined {
  const app_id = config?.app_id?.trim();
  const app_secret = config?.app_secret?.trim();
  if (!app_id || !app_secret) {
    return undefined;
  }
  return { app_id, app_secret };
}

function shouldSignRequest(method: string, requestPath: string): boolean {
  if (method === "OPTIONS") {
    return false;
  }
  const path = parseRequestPath(requestPath).path;
  return (
    (path === API_PREFIX || path.startsWith(`${API_PREFIX}/`)) &&
    path !== `${API_PREFIX}/ping` &&
    path !== `${API_PREFIX}/echo` &&
    path !== HEALTH_PATH &&
    path !== "/ping" &&
    path !== "/echo"
  );
}

function parseRequestPath(requestPath: string): ParsedRequestPath {
  const [path, query = ""] = requestPath.split("?", 2);
  return { path, query };
}

function buildRequestPath(path: string, query: string): string {
  return query ? `${path}?${query}` : path;
}

function isJsonRequest(headers: Headers): boolean {
  const mediaType = headers
    .get("content-type")
    ?.split(";")[0]
    ?.trim()
    .toLowerCase();
  return (
    mediaType === "application/json" || Boolean(mediaType?.endsWith("+json"))
  );
}

function parseSignedJsonBody(body: string): Record<string, unknown> {
  const value = JSON.parse(body) as unknown;
  if (!isRecord(value) || Array.isArray(value)) {
    throw new Error("Signed JSON API requests must use an object body.");
  }
  return value;
}

function canonicalJsonWithoutSigningFields(
  value: Record<string, unknown>,
): string {
  const copy: Record<string, unknown> = {};
  for (const [key, item] of Object.entries(value)) {
    if (!isSigningField(key)) {
      copy[key] = item;
    }
  }
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

function queryWithSigningFields(
  query: string,
  fields: Partial<Record<string, string>>,
): string {
  const keysToReplace = new Set<string>();
  if (fields[SIGNING_FIELD_APP_ID]) {
    keysToReplace.add(SIGNING_FIELD_APP_ID);
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  if (fields[SIGNING_FIELD_NONCE]) {
    keysToReplace.add(SIGNING_FIELD_NONCE);
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  if (fields[SIGNING_FIELD_SIGNATURE]) {
    keysToReplace.add(SIGNING_FIELD_SIGNATURE);
  }
  const pairs = splitQueryPairs(query).filter(
    ([key]) => !keysToReplace.has(key),
  );
  for (const key of [
    SIGNING_FIELD_APP_ID,
    SIGNING_FIELD_NONCE,
    SIGNING_FIELD_SIGNATURE,
  ]) {
    const value = fields[key];
    if (value) {
      pairs.push([key, encodeURIComponent(value)]);
    }
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

function compareCanonicalText(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

function canonicalRequest(
  method: string,
  path: string,
  canonicalQueryString: string,
  bodyHash: string,
  appId: string,
  nonce: string,
): string {
  return [
    SIGNATURE_ALGORITHM,
    method,
    path,
    canonicalQueryString,
    bodyHash,
    appId,
    nonce,
  ].join("\n");
}

async function hmacSha256Hex(secret: string, message: string): Promise<string> {
  const crypto = requireWebCrypto();
  const key = await crypto.subtle.importKey(
    "raw",
    bufferSourceFromBytes(utf8Bytes(secret)),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const signature = await crypto.subtle.sign(
    "HMAC",
    key,
    bufferSourceFromBytes(utf8Bytes(message)),
  );
  return bytesToHex(new Uint8Array(signature));
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await requireWebCrypto().subtle.digest(
    "SHA-256",
    bufferSourceFromBytes(bytes),
  );
  return bytesToHex(new Uint8Array(digest));
}

function requireWebCrypto(): Crypto {
  const crypto = globalThis.crypto;
  if (!crypto?.subtle) {
    throw new Error("StellarTrail request signing requires Web Crypto.");
  }
  return crypto;
}

async function requestBodyBytes(
  body: BodyInit | null | undefined,
): Promise<Uint8Array> {
  if (body === undefined || body === null) {
    return new Uint8Array();
  }
  if (typeof body === "string") {
    return utf8Bytes(body);
  }
  if (body instanceof URLSearchParams) {
    return utf8Bytes(body.toString());
  }
  if (body instanceof Blob) {
    return new Uint8Array(await body.arrayBuffer());
  }
  if (body instanceof ArrayBuffer) {
    return new Uint8Array(body);
  }
  if (ArrayBuffer.isView(body)) {
    return new Uint8Array(
      new Uint8Array(body.buffer, body.byteOffset, body.byteLength),
    );
  }
  if (body instanceof FormData) {
    throw new Error("Signed multipart API requests must use a prepared body.");
  }
  throw new Error("Unsupported signed API request body.");
}

async function buildMultipartBody(input: {
  fields: MultipartTextField[];
  file: Blob;
  filename: string;
  contentType?: string;
}): Promise<MultipartBody> {
  const boundary = `StellarTrailBoundary${createSignatureNonce().replace(/-/g, "")}`;
  const contentType = multipartFileContentType(input);
  const chunks: Uint8Array[] = [];
  for (const field of input.fields) {
    chunks.push(
      utf8Bytes(
        `--${boundary}\r\nContent-Disposition: form-data; name="${escapeMultipartName(
          field.name,
        )}"\r\n\r\n${field.value}\r\n`,
      ),
    );
  }
  chunks.push(
    utf8Bytes(
      `--${boundary}\r\nContent-Disposition: form-data; name="file"; filename="${escapeMultipartName(
        input.filename,
      )}"\r\nContent-Type: ${contentType}\r\n\r\n`,
    ),
    new Uint8Array(await input.file.arrayBuffer()),
    utf8Bytes(`\r\n--${boundary}--\r\n`),
  );
  return {
    body: arrayBufferFromBytes(concatBytes(chunks)),
    contentType: `multipart/form-data; boundary=${boundary}`,
  };
}

function multipartFileContentType(input: {
  file: Blob;
  filename: string;
  contentType?: string;
}): string {
  const explicit = input.contentType?.trim().toLowerCase();
  if (explicit && isSafeMultipartContentType(explicit)) {
    return explicit;
  }
  const blobType = input.file.type.trim().toLowerCase();
  if (blobType && isSafeMultipartContentType(blobType)) {
    return blobType;
  }
  return contentTypeFromFilename(input.filename) ?? "application/octet-stream";
}

function knotMediaAssetContentType(assetId: KnotMediaAssetId): string {
  switch (assetId) {
    case "thumbnail":
    case "preview":
      return "image/webp";
    case "draw_gif":
    case "turntable_gif":
      return "image/gif";
    case "draw_mp4":
    case "turntable_mp4":
      return "video/mp4";
  }
}

function contentTypeFromFilename(filename: string): string | undefined {
  const extension = filename.split(".").pop()?.toLowerCase();
  switch (extension) {
    case "jpg":
    case "jpeg":
      return "image/jpeg";
    case "png":
      return "image/png";
    case "webp":
      return "image/webp";
    case "gif":
      return "image/gif";
    case "mp4":
      return "video/mp4";
    default:
      return undefined;
  }
}

function isSafeMultipartContentType(value: string): boolean {
  return /^[a-z0-9!#$&^_.+-]+\/[a-z0-9!#$&^_.+-]+$/.test(value);
}

function escapeMultipartName(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function concatBytes(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const output = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    for (const byte of chunk) {
      output[offset] = byte;
      offset += 1;
    }
  }
  return output;
}

function arrayBufferFromBytes(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer;
}

function bufferSourceFromBytes(bytes: Uint8Array): BufferSource {
  return new Uint8Array(bytes) as unknown as BufferSource;
}

function utf8Bytes(value: string): Uint8Array {
  return new TextEncoder().encode(value);
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join(
    "",
  );
}

let signatureNonceCounter = 0;

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
  const crypto = globalThis.crypto;
  if (typeof crypto?.getRandomValues !== "function") {
    return false;
  }
  try {
    const randomBytes = new Uint8Array(new ArrayBuffer(bytes.length));
    crypto.getRandomValues(randomBytes);
    for (let index = 0; index < randomBytes.length; index += 1) {
      bytes[index] = randomBytes[index];
    }
    return !isAllZeroBytes(randomBytes);
  } catch {
    return false;
  }
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

function isSigningField(key: string): boolean {
  return (
    key === SIGNING_FIELD_APP_ID ||
    key === SIGNING_FIELD_NONCE ||
    key === SIGNING_FIELD_SIGNATURE
  );
}
