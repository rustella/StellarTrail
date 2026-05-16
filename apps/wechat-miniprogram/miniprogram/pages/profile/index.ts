import {
  getThemeViewData,
  syncPageTheme,
  togglePageTheme,
} from "../../utils/theme";

Page({
  data: {
    title: "我的 StellarTrail",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },

  toggleTheme() {
    togglePageTheme(this);
  },
});
