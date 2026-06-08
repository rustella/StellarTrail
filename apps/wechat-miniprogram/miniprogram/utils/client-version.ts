import { loadClientConfig } from "./client-config";

type SemanticVersionTuple = [number, number, number];

interface ClientVersionLike {
  version: string;
}

interface ResolveWechatClientVersionOptions {
  appClientIdentity?: string | null;
  configClientIdentity?: string | null;
  nativeVersion?: string | null;
}

const SEMANTIC_VERSION_PATTERN = /^(\d+)\.(\d+)\.(\d+)$/;
const WECHAT_CLIENT_KEY = "wechat";

export function getCurrentWechatClientVersion(): string | undefined {
  return resolveWechatClientVersion({
    appClientIdentity: safeAppClientIdentity(),
    configClientIdentity: loadClientConfig().clientIdentity,
    nativeVersion: safeNativeMiniProgramVersion(),
  });
}

export function resolveWechatClientVersion(
  options: ResolveWechatClientVersionOptions,
): string | undefined {
  return (
    parseWechatClientIdentityVersion(options.appClientIdentity) ||
    parseWechatClientIdentityVersion(options.configClientIdentity) ||
    normalizeSemanticVersion(options.nativeVersion)
  );
}

export function filterClientVersionsForCurrentVersion<
  T extends ClientVersionLike,
>(versions: T[], currentVersion = getCurrentWechatClientVersion()): T[] {
  const maxVersion = normalizeSemanticVersion(currentVersion);
  if (!maxVersion) {
    return versions;
  }
  return versions.filter((version) =>
    isSemanticVersionLessThanOrEqual(version.version, maxVersion),
  );
}

export function isSemanticVersionLessThanOrEqual(
  version: string,
  maxVersion: string,
): boolean {
  const comparison = compareSemanticVersions(version, maxVersion);
  return comparison !== undefined && comparison <= 0;
}

export function compareSemanticVersions(
  left: string,
  right: string,
): number | undefined {
  const leftParts = parseSemanticVersion(left);
  const rightParts = parseSemanticVersion(right);
  if (!leftParts || !rightParts) {
    return undefined;
  }
  for (let index = 0; index < leftParts.length; index += 1) {
    const difference = leftParts[index] - rightParts[index];
    if (difference !== 0) {
      return difference;
    }
  }
  return 0;
}

export function parseSemanticVersion(
  version?: string | null,
): SemanticVersionTuple | undefined {
  const normalized = normalizeSemanticVersion(version);
  if (!normalized) {
    return undefined;
  }
  const match = normalized.match(SEMANTIC_VERSION_PATTERN);
  if (!match) {
    return undefined;
  }
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

function normalizeSemanticVersion(version?: string | null): string | undefined {
  const normalized = version?.trim();
  if (!normalized || !SEMANTIC_VERSION_PATTERN.test(normalized)) {
    return undefined;
  }
  return normalized;
}

function parseWechatClientIdentityVersion(
  clientIdentity?: string | null,
): string | undefined {
  const normalized = clientIdentity?.trim();
  if (!normalized) {
    return undefined;
  }
  const [client, version, ...rest] = normalized.split("/");
  if (rest.length > 0 || client.trim().toLowerCase() !== WECHAT_CLIENT_KEY) {
    return undefined;
  }
  return normalizeSemanticVersion(version);
}

function safeAppClientIdentity(): string | undefined {
  try {
    const app = getApp() as {
      globalData?: {
        clientIdentity?: string;
      };
    };
    return app.globalData?.clientIdentity;
  } catch {
    return undefined;
  }
}

function safeNativeMiniProgramVersion(): string | undefined {
  try {
    return wx.getAccountInfoSync().miniProgram.version;
  } catch {
    return undefined;
  }
}
