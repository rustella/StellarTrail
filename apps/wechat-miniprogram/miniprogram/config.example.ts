import type { ClientConfig } from "./utils/client-config";

// Copy to config.ts for real mini program builds. config.ts is ignored by Git.
const config: ClientConfig = {
  apiBaseUrl: "https://api.example.invalid",
  assetsBaseUrl: "https://assets.example.invalid",
};

export default config;
