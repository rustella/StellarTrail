export interface WebClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
}

const DEFAULT_API_BASE_URL = "https://api.example.invalid";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";

export function normalizeBaseUrl(
  value: string | undefined,
  fallback: string,
): string {
  const trimmed = value?.trim();
  return (trimmed && trimmed.length > 0 ? trimmed : fallback).replace(
    /\/$/,
    "",
  );
}

export function getWebClientConfig(): WebClientConfig {
  return {
    apiBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_STELLARTRAIL_API_BASE_URL,
      DEFAULT_API_BASE_URL,
    ),
    assetsBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_STELLARTRAIL_ASSETS_BASE_URL,
      DEFAULT_ASSETS_BASE_URL,
    ),
  };
}
