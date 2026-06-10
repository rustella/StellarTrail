import {
  applyThemeToSystem,
  loadThemePreference,
  type ThemeMode,
} from "./utils/theme";
import {
  loadClientConfig,
  type ClientDomainCandidate,
  type ClientRequestSignatureConfig,
} from "./utils/client-config";
import { loadAppLocale, type AppLocale } from "./utils/locale";
import { initNetworkState } from "./utils/network-state";

App<IAppOption>({
  onLaunch() {
    initNetworkState();
    const theme = loadThemePreference();
    const appLocale = loadAppLocale();
    const clientConfig = loadClientConfig();
    this.globalData.apiBaseUrl = clientConfig.apiBaseUrl;
    this.globalData.assetsBaseUrl = clientConfig.assetsBaseUrl;
    this.globalData.clientIdentity = clientConfig.clientIdentity;
    this.globalData.wechatLoginCode = clientConfig.wechatLoginCode;
    this.globalData.domainCandidates = clientConfig.domainCandidates;
    this.globalData.requestSignature = clientConfig.requestSignature;
    this.globalData.theme = theme;
    this.globalData.appLocale = appLocale;
    applyThemeToSystem(theme);
  },

  globalData: {
    apiBaseUrl: "https://api.example.invalid",
    assetsBaseUrl: "https://assets.example.invalid",
    clientIdentity: "wechat/0.2.2",
    wechatLoginCode: undefined,
    domainCandidates: [],
    requestSignature: undefined,
    theme: "light",
    appLocale: "zh-CN",
  },
});

interface IAppOption {
  globalData: {
    apiBaseUrl: string;
    assetsBaseUrl: string;
    clientIdentity: string;
    wechatLoginCode?: string;
    domainCandidates: ClientDomainCandidate[];
    requestSignature?: ClientRequestSignatureConfig;
    theme: ThemeMode;
    appLocale: AppLocale;
  };
}
