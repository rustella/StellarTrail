import { afterEach, describe, expect, it, vi } from "vitest";

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
      expect(input).toBe("/api/meta");
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
});
