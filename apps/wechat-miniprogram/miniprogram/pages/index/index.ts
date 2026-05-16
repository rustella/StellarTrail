import { getThemeViewData, syncPageTheme } from "../../utils/theme";

Page({
  data: {
    title: "星径户外助手",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },
});
