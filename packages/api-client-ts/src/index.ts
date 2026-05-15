import type {
  CreateGearRequest,
  GearCategoriesResponse,
  GearItem,
  GearStatsResponse,
  HealthResponse,
  ImportGearsRequest,
  ImportGearsResponse,
  ListGearsRequest,
  ListGearsResponse,
  MetaResponse,
  UpdateGearRequest,
  WechatLoginRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

export interface ApiClientOptions {
  baseUrl: string;
  fetcher?: typeof fetch;
  accessToken?: string;
}

export class StellarTrailApiClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private accessToken?: string;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.fetcher = options.fetcher ?? fetch;
    this.accessToken = options.accessToken;
  }

  setAccessToken(accessToken?: string): void {
    this.accessToken = accessToken;
  }

  async health(): Promise<HealthResponse> {
    return this.get("/healthz");
  }

  async meta(): Promise<MetaResponse> {
    return this.get("/api/meta");
  }

  async loginWithWechatCode(
    request: WechatLoginRequest,
  ): Promise<WechatLoginResponse> {
    const response = await this.post<WechatLoginResponse>(
      "/api/auth/wechat-login",
      request,
    );
    this.accessToken = response.access_token;
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

  private async get<T>(path: string, auth = false): Promise<T> {
    const response = await this.request(path, {}, auth);
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
  ): Promise<Response> {
    const headers = new Headers(init.headers);
    if (auth) {
      if (!this.accessToken) {
        throw new Error("StellarTrail API request requires an access token");
      }
      headers.set("authorization", `Bearer ${this.accessToken}`);
    }
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      ...init,
      headers,
    });
    if (!response.ok) {
      throw new Error(`StellarTrail API request failed: ${response.status}`);
    }
    return response;
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
