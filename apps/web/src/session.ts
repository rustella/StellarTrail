export interface WebSessionUser {
  id: string;
  username?: string | null;
  email?: string | null;
  nickname?: string | null;
  avatar_url?: string | null;
}

export interface WebSession {
  accessToken: string;
  expiresAt: string;
  refreshToken: string;
  refreshExpiresAt: string;
  user: WebSessionUser;
}

export const SESSION_STORAGE_KEY = "stellartrail.web.session";

export function loadSession(): WebSession | null {
  const raw = localStorage.getItem(SESSION_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const value = JSON.parse(raw) as Partial<WebSession>;
    if (
      !value.accessToken ||
      !value.expiresAt ||
      !value.refreshToken ||
      !value.refreshExpiresAt ||
      !value.user?.id
    ) {
      clearSession();
      return null;
    }
    return {
      accessToken: value.accessToken,
      expiresAt: value.expiresAt,
      refreshToken: value.refreshToken,
      refreshExpiresAt: value.refreshExpiresAt,
      user: value.user,
    };
  } catch {
    clearSession();
    return null;
  }
}

export function saveSession(session: WebSession): void {
  localStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(session));
}

export function clearSession(): void {
  localStorage.removeItem(SESSION_STORAGE_KEY);
}
