import {
  getErrorMessage,
  getGearStats,
  listGears,
  hasAccessToken,
  listKnots,
} from "../../utils/api";
import {
  formatGearPrice,
  formatGearWeight,
  type GearStatsResponse,
  type GearSummary,
} from "../../utils/gear-utils";
import { mapSkillCard, type SkillCard } from "../../utils/skill-utils";
import { getThemeViewData, syncPageTheme } from "../../utils/theme";

interface QuickAction {
  id: string;
  icon: string;
  title: string;
  description: string;
  target: "gears" | "addGear" | "skills" | "profile";
}

interface HomeStatCard {
  label: string;
  value: string;
  hint: string;
}

interface HomeGearCard {
  id: string;
  name: string;
  brandModelText: string;
  weightText: string;
}

interface ChecklistItem {
  icon: string;
  title: string;
  description: string;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  archived_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};

const QUICK_ACTIONS: QuickAction[] = [
  {
    id: "gears",
    icon: "🎒",
    title: "装备库",
    description: "查看、搜索、归档装备",
    target: "gears",
  },
  {
    id: "addGear",
    icon: "➕",
    title: "添加装备",
    description: "快速记录新装备",
    target: "addGear",
  },
  {
    id: "skills",
    icon: "🧭",
    title: "户外技能",
    description: "绳结、天气、急救知识",
    target: "skills",
  },
  {
    id: "profile",
    icon: "⚙️",
    title: "个人设置",
    description: "黑夜模式与登录状态",
    target: "profile",
  },
];

const CHECKLIST_ITEMS: ChecklistItem[] = [
  {
    icon: "✓",
    title: "装备清点",
    description: "确认重量、状态和存放位置",
  },
  {
    icon: "☁",
    title: "天气与风险自查",
    description: "出发前关注降雨、温差和风力",
  },
  {
    icon: "✚",
    title: "急救与应急联系人",
    description: "复习急救知识并同步行程信息",
  },
];

Page({
  data: {
    title: "寻径星野",
    quickActions: QUICK_ACTIONS,
    checklistItems: CHECKLIST_ITEMS,
    gearLoading: false,
    gearError: "",
    skillLoading: false,
    skillError: "",
    gearStatCards: buildGearStatCards(EMPTY_STATS),
    recentGears: [] as HomeGearCard[],
    featuredSkills: [] as SkillCard[],
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    this.loadHomeDashboard();
  },

  onPullDownRefresh() {
    const tasks = [this.loadFeaturedSkills()];
    if (hasAccessToken()) {
      tasks.push(this.loadGearSummary());
    }
    Promise.all(tasks).finally(() => wx.stopPullDownRefresh());
  },

  loadHomeDashboard() {
    if (hasAccessToken()) {
      this.loadGearSummary();
    } else {
      this.setData({
        gearLoading: false,
        gearError: "登录后查看个人装备概览",
        gearStatCards: buildGearStatCards(EMPTY_STATS),
        recentGears: [] as HomeGearCard[],
      });
    }
    this.loadFeaturedSkills();
  },

  async loadGearSummary() {
    this.setData({ gearLoading: true, gearError: "" });
    try {
      const [stats, gears] = await Promise.all([
        getGearStats("available"),
        listGears({ tab: "available", limit: 2, sort: "created_at_desc" }),
      ]);
      this.setData({
        gearStatCards: buildGearStatCards(stats),
        recentGears: gears.items.map(mapHomeGearCard),
        gearLoading: false,
      });
    } catch (error) {
      this.setData({
        gearError: getErrorMessage(error),
        gearLoading: false,
        recentGears: [] as HomeGearCard[],
      });
    }
  },

  async loadFeaturedSkills() {
    this.setData({ skillLoading: true, skillError: "" });
    try {
      const response = await listKnots({ offset: 0, limit: 3 });
      this.setData({
        featuredSkills: response.items.slice(0, 3).map(mapSkillCard),
        skillLoading: false,
      });
    } catch (error) {
      this.setData({
        skillError: getErrorMessage(error),
        skillLoading: false,
        featuredSkills: [] as SkillCard[],
      });
    }
  },

  onQuickAction(event: WechatMiniprogram.BaseEvent) {
    const target = event.currentTarget.dataset.target as QuickAction["target"];
    if (target === "gears") {
      this.goGears();
      return;
    }
    if (target === "addGear") {
      this.goAddGear();
      return;
    }
    if (target === "skills") {
      this.goSkills();
      return;
    }
    if (target === "profile") {
      this.goProfile();
    }
  },

  goGears() {
    wx.switchTab({ url: "/pages/gears/index" });
  },

  goAddGear() {
    wx.navigateTo({ url: "/pages/gears/form/index" });
  },

  goSkills() {
    wx.switchTab({ url: "/pages/skills/index" });
  },

  goProfile() {
    wx.switchTab({ url: "/pages/profile/index" });
  },
});

function buildGearStatCards(stats: GearStatsResponse): HomeStatCard[] {
  return [
    {
      label: "可用装备",
      value: String(stats.current_count),
      hint: "当前可直接使用",
    },
    {
      label: "总重量",
      value: formatGearWeight(stats.total_weight_g),
      hint: "已记录装备重量",
    },
    {
      label: "装备估值",
      value: formatGearPrice(stats.total_value_cents),
      hint: "按购买价格汇总",
    },
  ];
}

function mapHomeGearCard(item: GearSummary): HomeGearCard {
  const brandModelText = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    id: item.id,
    name: item.name,
    brandModelText: brandModelText || "未记录品牌型号",
    weightText: formatGearWeight(item.weight_g),
  };
}
