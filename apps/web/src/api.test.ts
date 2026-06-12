import { afterEach, describe, expect, it, vi } from "vitest";

import {
  StellarTrailApiClient,
  StellarTrailApiError,
} from "@stellartrail/api-client";

import { createWebGearApi } from "./api";
import { buildClientIdentity } from "./client-config";

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
      const url = new URL(String(input), "https://example.test");
      expect(url.pathname).toBe("/api/v1/meta");
      expect(url.searchParams.get("app_id")).toBeTruthy();
      expect(url.searchParams.get("nonce")).toBeTruthy();
      expect(url.searchParams.get("signature")).toBeTruthy();
      expect(init).toBeDefined();
      expect(new Headers(init?.headers).get("X-StellarTrail-Client")).toBe(
        "web/0.1.0",
      );
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
      clientIdentity: string | null;
      body?: string;
    }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const headers = new Headers(init?.headers);
        const url = String(input);
        requests.push({
          url,
          authorization: headers.get("authorization"),
          clientIdentity: headers.get("X-StellarTrail-Client"),
          body: typeof init?.body === "string" ? init.body : undefined,
        });
        if (
          url === "/api/v1/me/gears/stats?tab=available" &&
          requests.length === 1
        ) {
          return new Response(JSON.stringify({ code: "unauthorized" }), {
            status: 401,
            headers: { "content-type": "application/json" },
          });
        }
        if (url === "/api/v1/auth/refresh") {
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
        if (url === "/api/v1/me/gears/stats?tab=available") {
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
      clientIdentity: "web/0.1.0",
      fetcher: fetcher as typeof fetch,
      accessToken: "access-old",
      refreshToken: "refresh-old",
      onSessionRefresh,
    });

    await expect(client.getGearStats("available")).resolves.toMatchObject({
      current_count: 0,
    });
    expect(requests.map((request) => request.url)).toEqual([
      "/api/v1/me/gears/stats?tab=available",
      "/api/v1/auth/refresh",
      "/api/v1/me/gears/stats?tab=available",
    ]);
    expect(requests[0].authorization).toBe("Bearer access-old");
    expect(requests[0].clientIdentity).toBe("web/0.1.0");
    expect(requests[1].body).toBe(
      JSON.stringify({ refresh_token: "refresh-old" }),
    );
    expect(requests[2].authorization).toBe("Bearer access-new");
    expect(requests[2].clientIdentity).toBe("web/0.1.0");
    expect(onSessionRefresh).toHaveBeenCalledWith(
      expect.objectContaining({
        access_token: "access-new",
        refresh_token: "refresh-new",
      }),
    );
  });

  it("surfaces the API error envelope message", async () => {
    const client = new StellarTrailApiClient({
      baseUrl: "",
      clientIdentity: "web/0.1.0",
      fetcher: vi.fn(
        async () =>
          new Response(
            JSON.stringify({
              code: "invalid_credentials",
              message: "用户名/邮箱或密码不正确",
            }),
            { status: 401, headers: { "content-type": "application/json" } },
          ),
      ) as typeof fetch,
    });

    try {
      await client.loginWithPassword({
        account: "stellarisw@qq.com",
        password: "wrong-password",
      });
      throw new Error("expected request to fail");
    } catch (error) {
      expect(error).toBeInstanceOf(StellarTrailApiError);
      expect(error).toMatchObject({
        status: 401,
        code: "invalid_credentials",
        message: "用户名/邮箱或密码不正确",
      });
    }
  });
});

describe("Web client configuration", () => {
  it("falls back to a valid web client identity when local config is invalid", () => {
    expect(buildClientIdentity("desktop", "0.1.0")).toBe("web/0.1.0");
    expect(buildClientIdentity("web", "")).toBe("web/0.1.0");
    expect(buildClientIdentity("web", "x".repeat(65))).toBe("web/0.1.0");
    expect(buildClientIdentity("web", "0.2.0")).toBe("web/0.2.0");
  });
});

describe("StellarTrailApiClient request signing", () => {
  it("signs admin atlas review GET requests with query parameters", async () => {
    const requests: Array<{ url: string; headers: Headers }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
        });
        return new Response(JSON.stringify({ items: [], next_cursor: null }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      },
    );
    const client = signedClient(fetcher as typeof fetch, ["nonce-get"]);

    await client.listAdminGearAtlasSubmissions(
      {
        status: "pending",
        deleted: "active",
        limit: 20,
      },
      "zh-CN",
    );

    expect(requests).toHaveLength(1);
    expect(requests[0].headers.get("authorization")).toBe(
      "Bearer access-token",
    );
    const url = new URL(`https://example.test${requests[0].url}`);
    expect(url.pathname).toBe("/api/v1/admin/gear-atlas-submissions");
    expect(url.searchParams.get("status")).toBe("pending");
    expect(url.searchParams.get("deleted")).toBe("active");
    expect(url.searchParams.get("limit")).toBe("20");
    expect(requests[0].headers.get("X-StellarTrail-Locale")).toBe("zh-CN");
    await expectValidSignature({
      url,
      method: "GET",
      bodyHash: await sha256Hex(new Uint8Array()),
      nonce: "nonce-get",
    });
  });

  it("signs JSON requests without mutating business fields", async () => {
    const requests: Array<{ url: string; body?: string }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          body: typeof init?.body === "string" ? init.body : undefined,
        });
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
      },
    );
    const client = signedClient(fetcher as typeof fetch, ["nonce-json"]);
    const request = {
      account: "test@example.com",
      password: "Password1",
    };

    await client.loginWithPassword(request);

    expect(request).toEqual({
      account: "test@example.com",
      password: "Password1",
    });
    const signedBody = JSON.parse(requests[0].body ?? "{}") as Record<
      string,
      string
    >;
    expect(signedBody).toMatchObject({
      account: "test@example.com",
      password: "Password1",
      app_id: "web-client",
      nonce: "nonce-json",
    });
    const url = new URL(`https://example.test${requests[0].url}`);
    await expectValidSignature({
      url,
      method: "POST",
      bodyHash: await sha256Hex(
        utf8Bytes(
          JSON.stringify({
            account: "test@example.com",
            password: "Password1",
          }),
        ),
      ),
      nonce: "nonce-json",
      signature: signedBody.signature,
    });
  });

  it("uses fresh signatures for refresh and retry requests", async () => {
    const requests: Array<{ url: string; body?: string }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        requests.push({
          url,
          body: typeof init?.body === "string" ? init.body : undefined,
        });
        if (
          url.startsWith("/api/v1/me/gears/stats?") &&
          requests.length === 1
        ) {
          return new Response(JSON.stringify({ code: "unauthorized" }), {
            status: 401,
            headers: { "content-type": "application/json" },
          });
        }
        if (url === "/api/v1/auth/refresh") {
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
      },
    );
    const client = signedClient(fetcher as typeof fetch, [
      "nonce-stats-old",
      "nonce-refresh",
      "nonce-stats-new",
    ]);

    await client.getGearStats("available");

    expect(requests).toHaveLength(3);
    expect(
      new URL(`https://example.test${requests[0].url}`).searchParams.get(
        "nonce",
      ),
    ).toBe("nonce-stats-old");
    expect(JSON.parse(requests[1].body ?? "{}").nonce).toBe("nonce-refresh");
    expect(
      new URL(`https://example.test${requests[2].url}`).searchParams.get(
        "nonce",
      ),
    ).toBe("nonce-stats-new");
  });

  it("signs multipart uploads with the raw multipart body hash", async () => {
    const requests: Array<{
      url: string;
      headers: Headers;
      body?: BodyInit | null;
    }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
          body: init?.body,
        });
        return new Response(
          JSON.stringify({
            user: { id: "u1", nickname: "测试用户", avatar_url: null },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = signedClient(fetcher as typeof fetch, ["nonce-upload"]);

    await client.uploadProfileAvatar(
      new Blob(["avatar-bytes"], { type: "image/png" }),
      "avatar.png",
    );

    expect(requests).toHaveLength(1);
    expect(requests[0].headers.get("content-type")).toMatch(
      /^multipart\/form-data; boundary=StellarTrailBoundary/,
    );
    expect(requests[0].body).toBeInstanceOf(ArrayBuffer);
    const body = new Uint8Array(requests[0].body as ArrayBuffer);
    const decoded = new TextDecoder().decode(body);
    expect(decoded).toContain('filename="avatar.png"');
    expect(decoded).toContain("Content-Type: image/png");
    const url = new URL(`https://example.test${requests[0].url}`);
    await expectValidSignature({
      url,
      method: "PUT",
      bodyHash: await sha256Hex(body),
      nonce: "nonce-upload",
    });
  });

  it("signs knot media uploads with the expected asset content type", async () => {
    const requests: Array<{
      url: string;
      headers: Headers;
      body?: BodyInit | null;
    }> = [];
    const fetcher = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        requests.push({
          url: String(input),
          headers: new Headers(init?.headers),
          body: init?.body,
        });
        return new Response(
          JSON.stringify({
            status: "uploaded",
            media: {
              id: "thumbnail",
              media_type: "thumbnail",
              mime_type: "image/webp",
              size_bytes: 10,
              url: "https://media.example.test/thumb.webp",
            },
          }),
          { status: 201, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = signedClient(fetcher as typeof fetch, ["nonce-knot-upload"]);

    await client.uploadKnotMedia({
      knotId: "adjustable-grip-hitch-knot",
      assetId: "thumbnail",
      file: new Blob(["webp-bytes"]),
      filename: "thumbnail.bin",
    });

    expect(requests).toHaveLength(1);
    expect(requests[0].headers.get("content-type")).toMatch(
      /^multipart\/form-data; boundary=StellarTrailBoundary/,
    );
    const body = new Uint8Array(requests[0].body as ArrayBuffer);
    const decoded = new TextDecoder().decode(body);
    expect(decoded).toContain('filename="thumbnail.bin"');
    expect(decoded).toContain("Content-Type: image/webp");
    const url = new URL(`https://example.test${requests[0].url}`);
    expect(url.pathname).toBe(
      "/api/v1/admin/skills/knots/adjustable-grip-hitch-knot/media/thumbnail",
    );
    await expectValidSignature({
      url,
      method: "PUT",
      bodyHash: await sha256Hex(body),
      nonce: "nonce-knot-upload",
    });
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
      clientIdentity: "web/0.1.0",
      fetcher: fetcher as typeof fetch,
      accessToken: "[REDACTED]",
    });

    await client.listKnots(
      {
        offset: 0,
        limit: 24,
        category: "camping-knots",
        q: "风绳",
      },
      "zh-CN",
    );

    expect(requests).toHaveLength(1);
    const url = new URL(`https://example.test${requests[0].url}`);
    expect(url.pathname).toBe("/api/v1/skills/knots/list");
    expect(url.searchParams.get("offset")).toBe("0");
    expect(url.searchParams.get("limit")).toBe("24");
    expect(url.searchParams.get("category")).toBe("camping-knots");
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
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = new StellarTrailApiClient({
      baseUrl: "",
      clientIdentity: "web/0.1.0",
      fetcher: fetcher as typeof fetch,
      accessToken: "[REDACTED]",
    });

    await client.listKnotFilters("zh-CN");

    expect(requests).toHaveLength(1);
    expect(requests[0].url).toBe("/api/v1/skills/knots/filters");
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
            aliases: [],
            description: null,
            steps: [],
            categories: [],
            types: [],
            media: [],
            href: "/api/v1/skills/knots/detail/adjustable%20grip",
            locale: "zh-CN",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    );
    const client = new StellarTrailApiClient({
      baseUrl: "",
      clientIdentity: "web/0.1.0",
      fetcher: fetcher as typeof fetch,
    });

    await client.getKnotDetail("adjustable grip", "zh-CN");

    expect(requests).toHaveLength(1);
    expect(requests[0].url).toBe(
      "/api/v1/skills/knots/detail/adjustable%20grip",
    );
    expect(requests[0].headers.get("X-StellarTrail-Locale")).toBe("zh-CN");
    expect(requests[0].headers.get("authorization")).toBeNull();
  });
});

function signedClient(
  fetcher: typeof fetch,
  nonces: string[],
): StellarTrailApiClient {
  let nonceIndex = 0;
  return new StellarTrailApiClient({
    baseUrl: "",
    clientIdentity: "web/0.1.0",
    fetcher,
    accessToken: "access-token",
    refreshToken: "refresh-token",
    requestSignature: {
      app_id: "web-client",
      app_secret: "web-secret",
    },
    nonceProvider: () => nonces[nonceIndex++] ?? `nonce-${nonceIndex}`,
  });
}

async function expectValidSignature(input: {
  url: URL;
  method: string;
  bodyHash: string;
  nonce: string;
  signature?: string | null;
}): Promise<void> {
  const appId = input.url.searchParams.get("app_id") ?? "web-client";
  const nonce = input.url.searchParams.get("nonce") ?? input.nonce;
  const signature =
    input.signature ?? input.url.searchParams.get("signature") ?? "";
  expect(appId).toBe("web-client");
  expect(nonce).toBe(input.nonce);
  expect(signature).toMatch(/^[0-9a-f]{64}$/);
  const expected = await hmacSha256Hex(
    "web-secret",
    [
      "STELLARTRAIL-HMAC-SHA256",
      input.method,
      input.url.pathname,
      canonicalQuery(input.url.search.slice(1)),
      input.bodyHash,
      appId,
      nonce,
    ].join("\n"),
  );
  expect(signature).toBe(expected);
}

function canonicalQuery(query: string): string {
  return query
    .split("&")
    .filter(Boolean)
    .map((pair) => {
      const [key, value = ""] = pair.split("=", 2);
      return [key, value] as const;
    })
    .filter(([key]) => key !== "signature")
    .sort(
      ([leftKey, leftValue], [rightKey, rightValue]) =>
        leftKey.localeCompare(rightKey) || leftValue.localeCompare(rightValue),
    )
    .map(([key, value]) => `${key}=${value}`)
    .join("&");
}

async function hmacSha256Hex(secret: string, message: string): Promise<string> {
  const key = await globalThis.crypto.subtle.importKey(
    "raw",
    bufferSourceFromBytes(utf8Bytes(secret)),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const signature = await globalThis.crypto.subtle.sign(
    "HMAC",
    key,
    bufferSourceFromBytes(utf8Bytes(message)),
  );
  return bytesToHex(new Uint8Array(signature));
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await globalThis.crypto.subtle.digest(
    "SHA-256",
    bufferSourceFromBytes(bytes),
  );
  return bytesToHex(new Uint8Array(digest));
}

function utf8Bytes(value: string): Uint8Array {
  return new TextEncoder().encode(value);
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join(
    "",
  );
}

function arrayBufferFromBytes(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer;
}

function bufferSourceFromBytes(bytes: Uint8Array): BufferSource {
  return new Uint8Array(bytes) as unknown as BufferSource;
}
