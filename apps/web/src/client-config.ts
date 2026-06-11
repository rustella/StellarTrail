export interface WebClientConfig {
  apiBaseUrl: string;
  assetsBaseUrl: string;
  clientIdentity: string;
  requestSignature?: WebRequestSignatureConfig;
}

export interface WebRequestSignatureConfig {
  app_id: string;
  app_secret: string;
}

const DEFAULT_API_BASE_URL = "";
const DEFAULT_ASSETS_BASE_URL = "https://assets.example.invalid";
const LOCAL_DEV_API_BASE_URL = "";
const WEB_CLIENT = "web";
const SUPPORTED_CLIENTS = new Set(["web", "wechat", "android", "ios", "mac"]);
const MAX_CLIENT_VERSION_LEN = 64;

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
  const normalizedClient = normalizeClientName(client);
  const normalizedVersion = normalizeClientVersion(
    version,
    __STELLARTRAIL_WEB_CLIENT_VERSION__,
  );
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
    requestSignature: normalizeRequestSignature({
      app_id: import.meta.env.VITE_STELLARTRAIL_REQUEST_SIGNATURE_APP_ID,
      app_secret: import.meta.env
        .VITE_STELLARTRAIL_REQUEST_SIGNATURE_APP_SECRET,
    }),
  };
}

function normalizeClientName(client: string): string {
  const normalized = client.trim();
  return SUPPORTED_CLIENTS.has(normalized) ? normalized : WEB_CLIENT;
}

function normalizeClientVersion(version: string, fallback: string): string {
  const normalized = version.trim();
  if (isValidClientVersion(normalized)) {
    return normalized;
  }
  const fallbackVersion = fallback.trim();
  return isValidClientVersion(fallbackVersion) ? fallbackVersion : "0.1.0";
}

function isValidClientVersion(version: string): boolean {
  return (
    version.length > 0 &&
    version.length <= MAX_CLIENT_VERSION_LEN &&
    !/[\/\u0000-\u001f\u007f]/.test(version)
  );
}

function normalizeRequestSignature(
  config: Partial<WebRequestSignatureConfig>,
): WebRequestSignatureConfig | undefined {
  const app_id = config.app_id?.trim();
  const app_secret = config.app_secret?.trim();
  if (!app_id || !app_secret) {
    return undefined;
  }
  return { app_id, app_secret };
}
