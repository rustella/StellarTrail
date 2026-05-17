import type {
  CaptchaChallengeRequest,
  CaptchaChallengeResponse,
  ContentListResponse,
  CreateGearRequest,
  EmailVerificationCodeRequest,
  EmailVerificationCodeResponse,
  GearCategoriesResponse,
  GearTemplate,
  GearItem,
  GearStatsResponse,
  HealthResponse,
  KnotDetail,
  KnotListResponse,
  ListKnotsRequest,
  ImportGearsRequest,
  ImportGearsResponse,
  MountainContent,
  ListGearsRequest,
  ListGearsResponse,
  MetaResponse,
  PasswordLoginRequest,
  RefreshTokenRequest,
  RegisterRequest,
  RouteContent,
  SkillCategoriesResponse,
  SkillLocale,
  UpdateGearRequest,
  WechatLoginRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

export interface ApiClientOptions {
  baseUrl: string;
  fetcher?: typeof fetch;
  accessToken?: string;
  refreshToken?: string;
  onSessionRefresh?: (response: WechatLoginResponse) => void;
}

export class StellarTrailApiClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private accessToken?: string;
  private refreshToken?: string;
  private refreshPromise?: Promise<WechatLoginResponse>;
  private onSessionRefresh?: (response: WechatLoginResponse) => void;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
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
    return this.get("/api/meta");
  }

  async listMountains(): Promise<ContentListResponse<MountainContent>> {
    return this.get("/api/mountains");
  }

  async getMountain(id: string): Promise<MountainContent> {
    return this.get(`/api/mountains/${encodeURIComponent(id)}`);
  }

  async listRoutes(): Promise<ContentListResponse<RouteContent>> {
    return this.get("/api/routes");
  }

  async getRoute(id: string): Promise<RouteContent> {
    return this.get(`/api/routes/${encodeURIComponent(id)}`);
  }

  async listSkills(locale?: SkillLocale): Promise<SkillCategoriesResponse> {
    return this.get("/api/skills", false, locale);
  }

  async listKnots(
    request: ListKnotsRequest = {},
    locale?: SkillLocale,
  ): Promise<KnotListResponse> {
    return this.get(
      `/api/skills/knots/list${queryString(request)}`,
      false,
      locale,
    );
  }

  async getKnotDetail(id: string, locale?: SkillLocale): Promise<KnotDetail> {
    return this.get(
      `/api/skills/knots/detail/${encodeURIComponent(id)}`,
      false,
      locale,
    );
  }

  async listGearTemplates(): Promise<ContentListResponse<GearTemplate>> {
    return this.get("/api/gear-templates");
  }

  async getGearTemplate(id: string): Promise<GearTemplate> {
    return this.get(`/api/gear-templates/${encodeURIComponent(id)}`);
  }

  async loginWithWechatCode(
    request: WechatLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/api/auth/wechat-login",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async sendEmailVerificationCode(
    request: EmailVerificationCodeRequest,
  ): Promise<EmailVerificationCodeResponse> {
    return this.post<EmailVerificationCodeResponse>(
      "/api/auth/email-verification-code",
      request,
    );
  }

  async createCaptcha(
    request: CaptchaChallengeRequest,
  ): Promise<CaptchaChallengeResponse> {
    return this.post<CaptchaChallengeResponse>("/api/auth/captcha", request);
  }

  async register(request: RegisterRequest): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/api/auth/register",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async loginWithPassword(
    request: PasswordLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/api/auth/login",
      request,
    );
    this.applyLoginResponse(response);
    return response;
  }

  async refreshSession(
    request: RefreshTokenRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/api/auth/refresh",
      request,
    );
    this.applyLoginResponse(response, true);
    return response;
  }

  async listGearCategories(
    tab?: "available" | "history",
  ): Promise<GearCategoriesResponse> {
    return this.get(`/api/me/gears/categories${queryString({ tab })}`, true);
  }

  async getGearStats(
    tab?: "available" | "history",
  ): Promise<GearStatsResponse> {
    return this.get(`/api/me/gears/stats${queryString({ tab })}`, true);
  }

  async listGears(request: ListGearsRequest = {}): Promise<ListGearsResponse> {
    return this.get(`/api/me/gears${queryString(request)}`, true);
  }

  async getGear(id: string): Promise<GearItem> {
    return this.get(`/api/me/gears/${encodeURIComponent(id)}`, true);
  }

  async createGear(request: CreateGearRequest): Promise<GearItem> {
    return this.post("/api/me/gears", request, true);
  }

  async updateGear(id: string, request: UpdateGearRequest): Promise<GearItem> {
    return this.patch(`/api/me/gears/${encodeURIComponent(id)}`, request, true);
  }

  async archiveGear(id: string): Promise<void> {
    await this.request(
      `/api/me/gears/${encodeURIComponent(id)}`,
      { method: "DELETE" },
      true,
    );
  }

  async restoreGear(id: string): Promise<GearItem> {
    return this.post(
      `/api/me/gears/${encodeURIComponent(id)}/restore`,
      undefined,
      true,
    );
  }

  async exportGearsCsv(tab?: "available" | "history"): Promise<string> {
    const response = await this.request(
      `/api/me/gears/export${queryString({ tab, format: "csv" })}`,
      {},
      true,
    );
    return response.text();
  }

  async importGears(request: ImportGearsRequest): Promise<ImportGearsResponse> {
    return this.post("/api/me/gears/import", request, true);
  }

  private async get<T>(
    path: string,
    auth = false,
    locale?: SkillLocale,
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
    locale?: SkillLocale,
  ): Promise<Response> {
    let response = await this.send(path, init, auth, locale);
    if (response.status === 401 && auth && this.refreshToken) {
      await this.refreshWithStoredToken();
      response = await this.send(path, init, auth, locale);
    }
    if (!response.ok) {
      throw new Error(`StellarTrail API request failed: ${response.status}`);
    }
    return response;
  }

  private async send(
    path: string,
    init: RequestInit,
    auth: boolean,
    locale?: SkillLocale,
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
    return this.fetcher(`${this.baseUrl}${path}`, {
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
