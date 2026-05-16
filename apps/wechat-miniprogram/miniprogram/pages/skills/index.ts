import { getThemeViewData, syncPageTheme } from "../../utils/theme";

Page({
  data: {
    title: "户外技能",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
  },
});
