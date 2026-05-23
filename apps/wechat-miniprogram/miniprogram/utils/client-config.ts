export interface ClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
  domainCandidates?: ClientDomainCandidate[];
}

export interface ResolvedClientConfig extends ClientConfig {
  domainCandidates: ClientDomainCandidate[];
}

export interface ClientDomainCandidate {
  id: string;
  apiBaseUrl: string;
  assetsBaseUrl: string;
}

const DEFAULT_CONFIG: ResolvedClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
  domainCandidates: [
    {
      id: "stellartrail",
      apiBaseUrl: "https://api.example.invalid",
      assetsBaseUrl: "https://assets.example.invalid",
    },
    {
      id: "stellaris",
      apiBaseUrl: "https://api-alt1.example.invalid",
      assetsBaseUrl: "https://assets-alt1.example.invalid",
    },
    {
      id: "iwx",
      apiBaseUrl: "https://api-alt2.example.invalid",
      assetsBaseUrl: "https://assets-alt2.example.invalid",
    },
  ],
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
    domainCandidates: resolveDomainCandidates(local, apiBaseUrl, assetsBaseUrl),
  };
}

function resolveDomainCandidates(
  local: Partial<ClientConfig>,
  apiBaseUrl: string,
  assetsBaseUrl: string,
): ClientDomainCandidate[] {
  const configured = normalizeDomainCandidates(local.domainCandidates);
  if (configured.length > 0) {
    return configured;
  }
  if (local.apiBaseUrl && !isKnownProductionApiBaseUrl(apiBaseUrl)) {
    return [];
  }
  return DEFAULT_CONFIG.domainCandidates.map((candidate) => ({ ...candidate }));
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

function isKnownProductionApiBaseUrl(apiBaseUrl: string): boolean {
  return DEFAULT_CONFIG.domainCandidates.some(
    (candidate) =>
      normalizeBaseUrl(candidate.apiBaseUrl, DEFAULT_CONFIG.apiBaseUrl) ===
      apiBaseUrl,
  );
}
