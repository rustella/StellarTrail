import { afterEach, describe, expect, it } from "vitest";

import { clearSession, loadSession, saveSession } from "./session";

describe("web session", () => {
  afterEach(() => {
    localStorage.clear();
  });

  it("round trips a login session through localStorage", () => {
    saveSession({
      accessToken: "token-123",
      expiresAt: "2026-06-01T00:00:00Z",
      user: { id: "u1", nickname: "测试用户", avatar_url: null },
    });

    expect(loadSession()).toEqual({
      accessToken: "token-123",
      expiresAt: "2026-06-01T00:00:00Z",
      user: { id: "u1", nickname: "测试用户", avatar_url: null },
    });
  });

  it("clears invalid or missing session data", () => {
    localStorage.setItem("stellartrail.web.session", "not-json");
    expect(loadSession()).toBeNull();
    expect(localStorage.getItem("stellartrail.web.session")).toBeNull();

    saveSession({
      accessToken: "token-123",
      expiresAt: "2026-06-01T00:00:00Z",
      user: { id: "u1" },
    });
    clearSession();
    expect(loadSession()).toBeNull();
  });
});
