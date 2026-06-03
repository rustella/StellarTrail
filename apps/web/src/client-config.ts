export interface WebClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
  clientIdentity: string;
}

const DEFAULT_API_BASE_URL = "";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";
const LOCAL_DEV_API_BASE_URL = "";
const WEB_CLIENT = "web";

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

export function buildClientIdentity(client: string, version: string): string {
  const normalizedClient = client.trim() || WEB_CLIENT;
  const normalizedVersion =
    version.trim() || __STELLARTRAIL_WEB_CLIENT_VERSION__;
  return `${normalizedClient}/${normalizedVersion}`;
}

export function getWebClientConfig(): WebClientConfig {
  const apiFallback = import.meta.env.DEV
    ? LOCAL_DEV_API_BASE_URL
    : DEFAULT_API_BASE_URL;
  return {
    apiBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_STELLARTRAIL_API_BASE_URL,
      apiFallback,
    ),
    assetsBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_STELLARTRAIL_ASSETS_BASE_URL,
      DEFAULT_ASSETS_BASE_URL,
    ),
    clientIdentity: buildClientIdentity(
      WEB_CLIENT,
      import.meta.env.VITE_STELLARTRAIL_CLIENT_VERSION ??
        __STELLARTRAIL_WEB_CLIENT_VERSION__,
    ),
  };
}
