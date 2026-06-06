import type { ClientConfig } from "./utils/client-config";

// Copy to config.ts for real mini program builds. config.ts is ignored by Git.
const config: ClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
  clientIdentity: {
    client: "wechat",
    version: "0.2.2",
  },
  // Optional for local mock-login test environments.
  // wechatLoginCode: "local-dev-user",
  requestSignature: {
    app_id: "example-client-id",
    app_secret: "example-client-secret",
  },
  domainCandidates: [
    {
      id: "primary",
      apiBaseUrl: "https://api.example.invalid",
      assetsBaseUrl: "https://assets.example.invalid",
    },
    {
      id: "backup",
      apiBaseUrl: "https://api-backup.example.invalid",
      assetsBaseUrl: "https://assets-backup.example.invalid",
    },
  ],
};

export default config;
