import type { ClientConfig } from "./utils/client-config";

// Copy to config.ts for real mini program builds. config.ts is ignored by Git.
const config: ClientConfig = {
  apiBaseUrl: "https://api.stellartrail.cn",
  assetsBaseUrl: "https://assets.stellartrail.cn",
};

export default config;
