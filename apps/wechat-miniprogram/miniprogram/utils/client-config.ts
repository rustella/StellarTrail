export interface ClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
}

const DEFAULT_CONFIG: ClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
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

export function loadClientConfig(): ClientConfig {
  const local = loadLocalConfig();
  return {
    apiBaseUrl: normalizeBaseUrl(local.apiBaseUrl, DEFAULT_CONFIG.apiBaseUrl),
    assetsBaseUrl: normalizeBaseUrl(
      local.assetsBaseUrl,
      DEFAULT_CONFIG.assetsBaseUrl,
    ),
  };
}
