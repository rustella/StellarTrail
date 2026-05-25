import {
  applyThemeToSystem,
  loadThemePreference,
  type ThemeMode,
} from "./utils/theme";
import {
  loadClientConfig,
  type ClientDomainCandidate,
} from "./utils/client-config";
import { initNetworkState } from "./utils/network-state";

App<IAppOption>({
  onLaunch() {
    initNetworkState();
    const theme = loadThemePreference();
    const clientConfig = loadClientConfig();
    this.globalData.apiBaseUrl = clientConfig.apiBaseUrl;
    this.globalData.assetsBaseUrl = clientConfig.assetsBaseUrl;
    this.globalData.domainCandidates = clientConfig.domainCandidates;
    this.globalData.theme = theme;
    applyThemeToSystem(theme);
  },

  globalData: {
    apiBaseUrl: "https://api.example.invalid",
    assetsBaseUrl: "https://assets.example.invalid",
    domainCandidates: [],
    theme: "light",
  },
});

interface IAppOption {
  globalData: {
    apiBaseUrl: string;
    assetsBaseUrl: string;
    domainCandidates: ClientDomainCandidate[];
    theme: ThemeMode;
  };
}
