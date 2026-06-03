import { StellarTrailApiClient } from "@stellartrail/api-client";

import { getWebClientConfig } from "./client-config";
import type {
  AppLocale,
  CaptchaChallengeRequest,
  CaptchaChallengeResponse,
  ClientKey,
  ClientVersion,
  ClientVersionRequest,
  CreateGearAtlasSubmissionRequest,
  CreateGearRequest,
  EmailLoginCodeRequest,
  EmailLoginRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearAtlasPublicItem,
  GearAtlasSubmission,
  GearItem,
  GearStatsResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  ListGearAtlasRequest,
  ListGearAtlasResponse,
  ListGearAtlasSubmissionsRequest,
  ListGearAtlasSubmissionsResponse,
  ListClientVersionsRequest,
  ListClientVersionsResponse,
  ListAdminFeedbackRequest,
  ListAdminFeedbackResponse,
  KnotDetail,
  KnotFiltersResponse,
  KnotListResponse,
  ListGearsRequest,
  ListGearsResponse,
  ListKnotsRequest,
  MetaResponse,
  PasswordLoginRequest,
  PasswordResetCodeRequest,
  PasswordResetRequest,
  RegisterRequest,
  RejectGearAtlasSubmissionRequest,
  SkillCategoriesResponse,
  SkillLocale,
  UpdateGearAtlasSubmissionRequest,
  UpdateGearRequest,
  WechatLoginRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

export interface WebGearApi {
  setAccessToken(accessToken?: string): void;
  setSessionTokens(accessToken?: string, refreshToken?: string): void;
  setSessionRefreshHandler(
    handler?: (response: WechatLoginResponse) => void,
  ): void;
  resolveAssetUrl(pathOrUrl: string): string;
  meta(): Promise<MetaResponse>;
  getCurrentClientVersion(clientKey: ClientKey): Promise<ClientVersion>;
  listClientVersions(
    clientKey: ClientKey,
    request?: Pick<ListClientVersionsRequest, "limit" | "cursor">,
  ): Promise<ListClientVersionsResponse>;
  loginWithWechatCode(
    request: WechatLoginRequest,
  ): Promise<WechatLoginResponse>;
  sendEmailVerificationCode(
    request: EmailVerificationCodeRequest,
  ): Promise<EmailVerificationCodeResponse>;
  sendEmailLoginCode(
    request: EmailLoginCodeRequest,
  ): Promise<EmailVerificationCodeResponse>;
  loginWithEmailCode(request: EmailLoginRequest): Promise<WechatLoginResponse>;
  sendPasswordResetCode(
    request: PasswordResetCodeRequest,
  ): Promise<EmailVerificationCodeResponse>;
  resetPassword(request: PasswordResetRequest): Promise<WechatLoginResponse>;
  createCaptcha(
    request: CaptchaChallengeRequest,
  ): Promise<CaptchaChallengeResponse>;
  register(request: RegisterRequest): Promise<WechatLoginResponse>;
  loginWithPassword(
    request: PasswordLoginRequest,
  ): Promise<WechatLoginResponse>;
  listSkills(locale?: SkillLocale): Promise<SkillCategoriesResponse>;
  listKnotFilters(locale?: SkillLocale): Promise<KnotFiltersResponse>;
  listKnots(
    request?: ListKnotsRequest,
    locale?: SkillLocale,
  ): Promise<KnotListResponse>;
  getKnotDetail(id: string, locale?: SkillLocale): Promise<KnotDetail>;
  listGearCategories(
    tab?: "available" | "history",
  ): Promise<GearCategoriesResponse>;
  listGearAtlas(
    request?: ListGearAtlasRequest,
    locale?: AppLocale,
  ): Promise<ListGearAtlasResponse>;
  getGearAtlasItem(
    id: string,
    locale?: AppLocale,
  ): Promise<GearAtlasPublicItem>;
  createGearAtlasSubmission(
    request: CreateGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission>;
  createGearAtlasSubmissionFromGear(
    gearId: string,
  ): Promise<GearAtlasSubmission>;
  listMyGearAtlasSubmissions(
    request?: Pick<ListGearAtlasSubmissionsRequest, "limit" | "cursor">,
  ): Promise<ListGearAtlasSubmissionsResponse>;
  getGearStats(tab?: "available" | "history"): Promise<GearStatsResponse>;
  listGears(request?: ListGearsRequest): Promise<ListGearsResponse>;
  getGear(id: string): Promise<GearItem>;
  createGear(request: CreateGearRequest): Promise<GearItem>;
  updateGear(id: string, request: UpdateGearRequest): Promise<GearItem>;
  archiveGear(id: string): Promise<void>;
  deleteGear(id: string): Promise<void>;
  undeleteGear(id: string): Promise<GearItem>;
  restoreGear(id: string): Promise<GearItem>;
  exportGearsCsv(tab?: "available" | "history"): Promise<string>;
  importGears(request: ImportGearsRequest): Promise<ImportGearsResponse>;
  listAdminFeedback(
    request?: ListAdminFeedbackRequest,
  ): Promise<ListAdminFeedbackResponse>;
  deleteAdminFeedback(id: string): Promise<void>;
  restoreAdminFeedback(id: string): Promise<void>;
  listAdminClientVersions(
    request?: ListClientVersionsRequest,
  ): Promise<ListClientVersionsResponse>;
  createAdminClientVersion(
    request: ClientVersionRequest,
  ): Promise<ClientVersion>;
  updateAdminClientVersion(
    id: string,
    request: ClientVersionRequest,
  ): Promise<ClientVersion>;
  listAdminGearAtlasSubmissions(
    request?: ListGearAtlasSubmissionsRequest,
  ): Promise<ListGearAtlasSubmissionsResponse>;
  getAdminGearAtlasSubmission(id: string): Promise<GearAtlasSubmission>;
  updateAdminGearAtlasSubmission(
    id: string,
    request: UpdateGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission>;
  approveGearAtlasSubmission(id: string): Promise<GearAtlasSubmission>;
  deleteAdminGearAtlasSubmission(id: string): Promise<void>;
  restoreAdminGearAtlasSubmission(id: string): Promise<GearAtlasSubmission>;
  rejectGearAtlasSubmission(
    id: string,
    request: RejectGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission>;
}

export function createWebGearApi(): WebGearApi {
  const config = getWebClientConfig();
  return new StellarTrailApiClient({
    baseUrl: config.apiBaseUrl,
    assetsBaseUrl: config.assetsBaseUrl,
    clientIdentity: config.clientIdentity,
  });
}
