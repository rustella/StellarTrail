import {
  applyThemeToSystem,
  loadThemePreference,
  type ThemeMode,
} from "./utils/theme";
import { loadClientConfig } from "./utils/client-config";

App<IAppOption>({
  onLaunch() {
    const theme = loadThemePreference();
    const clientConfig = loadClientConfig();
    this.globalData.apiBaseUrl = clientConfig.apiBaseUrl;
    this.globalData.assetsBaseUrl = clientConfig.assetsBaseUrl;
    this.globalData.theme = theme;
    applyThemeToSystem(theme);
  },

  globalData: {
    apiBaseUrl: "https://api.example.invalid",
    assetsBaseUrl: "https://assets.example.invalid",
    theme: "light",
  },
});

interface IAppOption {
  globalData: {
    apiBaseUrl: string;
    assetsBaseUrl: string;
    theme: ThemeMode;
  };
}
