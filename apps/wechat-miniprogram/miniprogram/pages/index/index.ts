import { getThemeViewData, syncPageTheme } from "../../utils/theme";

Page({
  data: {
    title: "寻径星野",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },
});
