import { afterEach, describe, expect, it, vi } from "vitest";

import { StellarTrailApiClient } from "@stellartrail/api-client";

import { createWebGearApi } from "./api";

describe("createWebGearApi", () => {
  const originalFetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it("binds the browser fetch function before the API client stores it", async () => {
    globalThis.fetch = vi.fn(function (
      this: typeof globalThis,
      input: RequestInfo | URL,
      init?: RequestInit,
    ) {
      if (this !== globalThis) {
        throw new TypeError("Illegal invocation");
      }
      expect(input).toBe("https://api.example.invalid/api/meta");
      expect(init).toBeDefined();
      return Promise.resolve(
        new Response(
          JSON.stringify({
            name: "StellarTrail",
            env: "local",
            database_kind: "sqlite",
          }),
          {
            status: 200,
            headers: { "content-type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    await expect(createWebGearApi().meta()).resolves.toEqual({
      name: "StellarTrail",
      env: "local",
      database_kind: "sqlite",
    });
  });
  it("refreshes an authenticated request once when the access token expires", async () => {
    const onSessionRefresh = vi.fn();
    const requests: Array<{
      url: string;
      authorization: string | null;
      body?: string;
    }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const headers = new Headers(init?.headers);
        const url = String(input);
        requests.push({
          url,
          authorization: headers.get("authorization"),
          body: typeof init?.body === "string" ? init.body : undefined,
        });
        if (
          url === "/api/me/gears/stats?tab=available" &&
          requests.length === 1
        ) {
          return new Response(JSON.stringify({ code: "unauthorized" }), {
            status: 401,
            headers: { "content-type": "application/json" },
          });
        }
        if (url === "/api/auth/refresh") {
          return new Response(
            JSON.stringify({
              access_token: "access-new",
              expires_at: "2026-06-01T02:00:00Z",
              refresh_token: "refresh-new",
              refresh_expires_at: "2026-07-01T00:00:00Z",
              user: { id: "u1", nickname: "测试用户", avatar_url: null },
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        if (url === "/api/me/gears/stats?tab=available") {
          return new Response(
            JSON.stringify({
              current_count: 0,
              archived_count: 0,
              total_value_cents: 0,
              total_weight_g: 0,
              by_category: [],
              by_status: [],
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response("not found", { status: 404 });
      },
    );

    const client = new StellarTrailApiClient({
      baseUrl: "",
      fetcher: fetcher as typeof fetch,
      accessToken: "access-old",
      refreshToken: "refresh-old",
      onSessionRefresh,
    });

    await expect(client.getGearStats("available")).resolves.toMatchObject({
      current_count: 0,
    });
    expect(requests.map((request) => request.url)).toEqual([
      "/api/me/gears/stats?tab=available",
      "/api/auth/refresh",
      "/api/me/gears/stats?tab=available",
    ]);
    expect(requests[0].authorization).toBe("Bearer access-old");
    expect(requests[1].body).toBe(
      JSON.stringify({ refresh_token: "refresh-old" }),
    );
    expect(requests[2].authorization).toBe("Bearer access-new");
    expect(onSessionRefresh).toHaveBeenCalledWith(
      expect.objectContaining({
        access_token: "access-new",
        refresh_token: "refresh-new",
      }),
    );
  });
});

describe("StellarTrailApiClient public knot requests", () => {
  it("lists knots with zh-CN locale header without authorization", async () => {
    const requests: Array<{ url: string; headers: Headers }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
        });
        return new Response(
          JSON.stringify({
            locale: "zh-CN",
            items: [],
            page: { offset: 0, limit: 24, next_offset: null },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = new StellarTrailApiClient({
      baseUrl: "",
      fetcher: fetcher as typeof fetch,
      accessToken: "[REDACTED]",
    });

    await client.listKnots(
      {
        offset: 0,
        limit: 24,
        category: "camping-knots",
        difficulty: "beginner",
        q: "风绳",
      },
      "zh-CN",
    );

    expect(requests).toHaveLength(1);
    const url = new URL(`https://example.test${requests[0].url}`);
    expect(url.pathname).toBe("/api/skills/knots/list");
    expect(url.searchParams.get("offset")).toBe("0");
    expect(url.searchParams.get("limit")).toBe("24");
    expect(url.searchParams.get("category")).toBe("camping-knots");
    expect(url.searchParams.get("difficulty")).toBe("beginner");
    expect(url.searchParams.get("q")).toBe("风绳");
    expect(requests[0].headers.get("X-StellarTrail-Locale")).toBe("zh-CN");
    expect(requests[0].headers.get("authorization")).toBeNull();
  });

  it("lists knot filter options with zh-CN locale header without authorization", async () => {
    const requests: Array<{ url: string; headers: Headers }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
        });
        return new Response(
          JSON.stringify({
            locale: "zh-CN",
            categories: [],
            difficulties: [],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = new StellarTrailApiClient({
      baseUrl: "",
      fetcher: fetcher as typeof fetch,
      accessToken: "[REDACTED]",
    });

    await client.listKnotFilters("zh-CN");

    expect(requests).toHaveLength(1);
    expect(requests[0].url).toBe("/api/skills/knots/filters");
    expect(requests[0].headers.get("X-StellarTrail-Locale")).toBe("zh-CN");
    expect(requests[0].headers.get("authorization")).toBeNull();
  });

  it("gets knot detail with an encoded id and locale header", async () => {
    const requests: Array<{ url: string; headers: Headers }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
        });
        return new Response(
          JSON.stringify({
            id: "adjustable grip",
            slug: "adjustable-grip",
            title: "可调节绳结",
            summary: "调节绳索上的张力。",
            description: null,
            steps: [],
            difficulty: null,
            categories: [],
            types: [],
            media: [],
            href: "/api/skills/knots/detail/adjustable%20grip",
            locale: "zh-CN",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = new StellarTrailApiClient({
      baseUrl: "",
      fetcher: fetcher as typeof fetch,
    });

    await client.getKnotDetail("adjustable grip", "zh-CN");

    expect(requests).toHaveLength(1);
    expect(requests[0].url).toBe("/api/skills/knots/detail/adjustable%20grip");
    expect(requests[0].headers.get("X-StellarTrail-Locale")).toBe("zh-CN");
    expect(requests[0].headers.get("authorization")).toBeNull();
  });
});
