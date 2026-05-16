import { getErrorMessage, listSkills } from "../../utils/api";
import { mapSkillCard, type SkillCard } from "../../utils/skill-utils";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";

Page({
  data: {
    title: "户外技能",
    skills: [] as SkillCard[],
    loading: false,
    error: "",
    ...getThemeViewData(),
  },

  onLoad() {
    this.loadSkills();
  },

  onShow() {
    syncPageTheme(this);
  },

  onPullDownRefresh() {
    this.loadSkills().finally(() => wx.stopPullDownRefresh());
  },

  async loadSkills() {
    this.setData({ loading: true, error: "" });
    try {
      const response = await listSkills();
      this.setData({
        skills: response.items.map(mapSkillCard),
        loading: false,
      });
    } catch (error) {
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        skills: [] as SkillCard[],
      });
    }
  },
});
