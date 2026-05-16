import { getThemeViewData, syncPageTheme } from "../../utils/theme";

Page({
  data: {
    title: "中国路线库",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },
});
