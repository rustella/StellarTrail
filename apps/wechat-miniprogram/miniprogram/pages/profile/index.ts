import {
  getThemeViewData,
  syncPageTheme,
  togglePageTheme,
} from "../../utils/theme";

Page({
  data: {
    title: "我的寻径星野",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },

  toggleTheme() {
    togglePageTheme(this);
  },
});
