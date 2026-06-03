import { readFileSync } from "node:fs";
import { fileURLToPath, URL } from "node:url";

import react from "@vitejs/plugin-react";
import { loadEnv } from "vite";
import { defineConfig } from "vitest/config";

const DEFAULT_API_PROXY_TARGET = "https://api.example.invalid";
const envDir = fileURLToPath(new URL(".", import.meta.url));
const packageJson = JSON.parse(
  readFileSync(new URL("package.json", import.meta.url), "utf8"),
) as { version?: string };

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, envDir, "");
  const apiProxyTarget =
    env.VITE_STELLARTRAIL_API_PROXY_TARGET?.trim() ||
    env.VITE_STELLARTRAIL_API_BASE_URL?.trim() ||
    DEFAULT_API_PROXY_TARGET;

  return {
    plugins: [react()],
    define: {
      __STELLARTRAIL_WEB_CLIENT_VERSION__: JSON.stringify(
        packageJson.version ?? "0.1.0",
      ),
    },
    resolve: {
      alias: {
        "@stellartrail/api-client": fileURLToPath(
          new URL("../../packages/api-client-ts/src/index.ts", import.meta.url),
        ),
        "@stellartrail/shared-types": fileURLToPath(
          new URL("../../packages/shared-types/src/index.ts", import.meta.url),
        ),
      },
    },
    server: {
      host: "127.0.0.1",
      port: 5173,
      proxy: {
        "/api/v1": {
          target: apiProxyTarget,
          changeOrigin: true,
          secure: true,
        },
        "/healthz": {
          target: apiProxyTarget,
          changeOrigin: true,
          secure: true,
        },
      },
    },
    test: {
      environment: "jsdom",
      setupFiles: ["src/test/setup.ts"],
    },
  };
});
