import type { HealthResponse, MetaResponse } from '@stellartrail/shared-types';

export interface ApiClientOptions {
  baseUrl: string;
  fetcher?: typeof fetch;
}

export class StellarTrailApiClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, '');
    this.fetcher = options.fetcher ?? fetch;
  }

  async health(): Promise<HealthResponse> {
    return this.get('/healthz');
  }

  async meta(): Promise<MetaResponse> {
    return this.get('/api/meta');
  }

  private async get<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`);
    if (!response.ok) {
      throw new Error(`StellarTrail API request failed: ${response.status}`);
    }
    return response.json() as Promise<T>;
  }
}
