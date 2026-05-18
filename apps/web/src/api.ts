import { StellarTrailApiClient } from "@stellartrail/api-client";

import { getWebClientConfig } from "./client-config";
import type {
  CaptchaChallengeRequest,
  CaptchaChallengeResponse,
  CreateGearRequest,
  EmailLoginCodeRequest,
  EmailLoginRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearAtlasSubmission,
  GearItem,
  GearStatsResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  ListGearAtlasSubmissionsRequest,
  ListGearAtlasSubmissionsResponse,
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
  getGearStats(tab?: "available" | "history"): Promise<GearStatsResponse>;
  listGears(request?: ListGearsRequest): Promise<ListGearsResponse>;
  getGear(id: string): Promise<GearItem>;
  createGear(request: CreateGearRequest): Promise<GearItem>;
  updateGear(id: string, request: UpdateGearRequest): Promise<GearItem>;
  archiveGear(id: string): Promise<void>;
  restoreGear(id: string): Promise<GearItem>;
  exportGearsCsv(tab?: "available" | "history"): Promise<string>;
  importGears(request: ImportGearsRequest): Promise<ImportGearsResponse>;
  listAdminGearAtlasSubmissions(
    request?: ListGearAtlasSubmissionsRequest,
  ): Promise<ListGearAtlasSubmissionsResponse>;
  getAdminGearAtlasSubmission(id: string): Promise<GearAtlasSubmission>;
  approveGearAtlasSubmission(id: string): Promise<GearAtlasSubmission>;
  rejectGearAtlasSubmission(
    id: string,
    request?: RejectGearAtlasSubmissionRequest,
  ): Promise<GearAtlasSubmission>;
}

export function createWebGearApi(): WebGearApi {
  const config = getWebClientConfig();
  return new StellarTrailApiClient({
    baseUrl: config.apiBaseUrl,
    assetsBaseUrl: config.assetsBaseUrl,
  });
}
