export interface ClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
  domainCandidates?: ClientDomainCandidate[];
  requestSignature?: ClientRequestSignatureConfig;
}

export interface ResolvedClientConfig extends ClientConfig {
  domainCandidates: ClientDomainCandidate[];
  requestSignature?: ClientRequestSignatureConfig;
}

export interface ClientDomainCandidate {
  id: string;
  apiBaseUrl: string;
  assetsBaseUrl: string;
}

export interface ClientRequestSignatureConfig {
  app_id: string;
  app_secret: string;
}

const DEFAULT_CONFIG: ResolvedClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
  domainCandidates: [],
};

declare const require:
  | ((
      path: string,
    ) => { default?: Partial<ClientConfig> } & Partial<ClientConfig>)
  | undefined;

function normalizeBaseUrl(value: string | undefined, fallback: string): string {
  const trimmed = value?.trim();
  return (trimmed && trimmed.length > 0 ? trimmed : fallback).replace(
    /\/$/,
    "",
  );
}

function loadLocalConfig(): Partial<ClientConfig> {
  try {
    if (typeof require !== "function") {
      return {};
    }
    const module = require("../config");
    return module.default ?? module;
  } catch {
    return {};
  }
}

export function loadClientConfig(): ResolvedClientConfig {
  const local = loadLocalConfig();
  const apiBaseUrl = normalizeBaseUrl(
    local.apiBaseUrl,
    DEFAULT_CONFIG.apiBaseUrl,
  );
  const assetsBaseUrl = normalizeBaseUrl(
    local.assetsBaseUrl,
    DEFAULT_CONFIG.assetsBaseUrl,
  );
  return {
    apiBaseUrl,
    assetsBaseUrl,
    requestSignature: normalizeRequestSignature(local.requestSignature),
    domainCandidates: resolveDomainCandidates(local, apiBaseUrl, assetsBaseUrl),
  };
}

function resolveDomainCandidates(
  local: Partial<ClientConfig>,
  _apiBaseUrl: string,
  _assetsBaseUrl: string,
): ClientDomainCandidate[] {
  return normalizeDomainCandidates(local.domainCandidates);
}

function normalizeDomainCandidates(
  candidates: ClientDomainCandidate[] | undefined,
): ClientDomainCandidate[] {
  if (!Array.isArray(candidates)) {
    return [];
  }
  return candidates
    .map((candidate) => ({
      id: String(candidate.id || "").trim(),
      apiBaseUrl: normalizeBaseUrl(candidate.apiBaseUrl, ""),
      assetsBaseUrl: normalizeBaseUrl(candidate.assetsBaseUrl, ""),
    }))
    .filter(
      (candidate) =>
        candidate.id && candidate.apiBaseUrl && candidate.assetsBaseUrl,
    );
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
