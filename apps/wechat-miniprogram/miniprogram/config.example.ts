import type { ClientConfig } from "./utils/client-config";

// Copy to config.ts for real mini program builds. config.ts is ignored by Git.
const config: ClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
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
