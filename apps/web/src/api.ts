import { StellarTrailApiClient } from "@stellartrail/api-client";
import type {
  CaptchaChallengeRequest,
  CaptchaChallengeResponse,
  CreateGearRequest,
  EmailLoginCodeRequest,
  EmailLoginRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearItem,
  GearStatsResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  KnotDetail,
  KnotListResponse,
  ListGearsRequest,
  ListGearsResponse,
  ListKnotsRequest,
  MetaResponse,
  PasswordLoginRequest,
  PasswordResetCodeRequest,
  PasswordResetRequest,
  RegisterRequest,
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
}

export function createWebGearApi(): WebGearApi {
  return new StellarTrailApiClient({
    baseUrl: import.meta.env.VITE_STELLARTRAIL_API_BASE_URL ?? "",
  });
}
