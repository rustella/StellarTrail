import {
  applyThemeToSystem,
  loadThemePreference,
  type ThemeMode,
} from "./utils/theme";

App<IAppOption>({
  onLaunch() {
    const theme = loadThemePreference();
    this.globalData.theme = theme;
    applyThemeToSystem(theme);
  },

  globalData: {
    apiBaseUrl: "http://127.0.0.1:8080",
    theme: "light",
  },
});

interface IAppOption {
  globalData: {
    apiBaseUrl: string;
    theme: ThemeMode;
  };
}
