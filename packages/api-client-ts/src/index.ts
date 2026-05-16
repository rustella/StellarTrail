import type {
  AppLocale,
  HealthResponse,
  KnotDetail,
  KnotListResponse,
  MetaResponse,
  SkillCategoriesResponse,
} from "@stellartrail/shared-types";

export interface ApiClientOptions {
  baseUrl: string;
  fetcher?: typeof fetch;
  locale?: AppLocale;
}

export interface KnotListOptions {
  offset?: number;
  limit?: number;
  category?: string;
  q?: string;
  locale?: AppLocale;
}

export class StellarTrailApiClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private locale?: AppLocale;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.fetcher = options.fetcher ?? fetch;
    this.locale = options.locale;
  }

  setLocale(locale?: AppLocale): void {
    this.locale = locale;
  }

  async health(): Promise<HealthResponse> {
    return this.get("/healthz");
  }

  async meta(): Promise<MetaResponse> {
    return this.get("/api/meta");
  }

  async skills(locale?: AppLocale): Promise<SkillCategoriesResponse> {
    return this.get("/api/skills", locale);
  }

  async knotsList(options: KnotListOptions = {}): Promise<KnotListResponse> {
    const search = new URLSearchParams();
    if (options.offset !== undefined) {
      search.set("offset", String(options.offset));
    }
    if (options.limit !== undefined) {
      search.set("limit", String(options.limit));
    }
    if (options.category) {
      search.set("category", options.category);
    }
    if (options.q) {
      search.set("q", options.q);
    }
    const query = search.toString();
    return this.get(
      `/api/skills/knots/list${query ? `?${query}` : ""}`,
      options.locale,
    );
  }

  async knotDetail(id: string, locale?: AppLocale): Promise<KnotDetail> {
    return this.get(
      `/api/skills/knots/detail/${encodeURIComponent(id)}`,
      locale,
    );
  }

  private async get<T>(path: string, localeOverride?: AppLocale): Promise<T> {
    const headers = new Headers();
    const locale = localeOverride ?? this.locale;
    if (locale) {
      headers.set("X-StellarTrail-Locale", locale);
    }

    const response = await this.fetcher(`${this.baseUrl}${path}`, { headers });
    if (!response.ok) {
      throw new Error(`StellarTrail API request failed: ${response.status}`);
    }
    return response.json() as Promise<T>;
  }
}
