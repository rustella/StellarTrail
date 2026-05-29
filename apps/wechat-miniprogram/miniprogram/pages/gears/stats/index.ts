import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  getGearStats,
  hasAccessToken,
  isLoginRequiredError,
} from "../../../utils/api-gears";
import {
  formatGearPrice,
  formatGearWeight,
  type GearStatsResponse,
} from "../../../utils/gear-display";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  loginPageUrl,
  openLoginPageFromPrompt,
  showLoginPrompt,
} from "../../../utils/auth-prompt";

const echarts = require("../../../vendor/echarts.simple.min");

interface KpiCard {
  label: string;
  value: string;
}

interface ChartRow {
  label: string;
  value: number;
  valueText: string;
  percentageText: string;
}

interface ChartComponent {
  init(
    callback: (canvas: any, width: number, height: number, dpr: number) => any,
  ): void;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};
const CHART_COLORS = [
  "#0f766e",
  "#22c55e",
  "#38bdf8",
  "#f59e0b",
  "#a78bfa",
  "#f97316",
  "#14b8a6",
  "#64748b",
];

let statsRequestSeq = 0;

Page({
  data: {
    echarts,
    chartEc: {
      lazyLoad: true,
    },
    isLoggedIn: hasAccessToken(),
    loading: false,
    error: "",
    offlineNotice: "",
    stats: EMPTY_STATS,
    kpiCards: buildKpiCards(EMPTY_STATS),
    categoryCountRows: [] as ChartRow[],
    categoryCountChartRows: [] as ChartRow[],
    categoryWeightRows: [] as ChartRow[],
    categoryWeightChartRows: [] as ChartRow[],
    categoryValueRows: [] as ChartRow[],
    categoryValueChartRows: [] as ChartRow[],
    statusCountRows: [] as ChartRow[],
    statusCountChartRows: [] as ChartRow[],
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad() {
    this.loadStats();
  },

  onShow() {
    syncPageTheme(this);
  },

  onPullDownRefresh() {
    this.loadStats().finally(() => wx.stopPullDownRefresh());
  },

  async loadStats() {
    const isLoggedIn = hasAccessToken();
    this.setData({ isLoggedIn, error: "" });
    if (!isLoggedIn) {
      this.disposeCharts();
      this.setData({
        loading: false,
        stats: EMPTY_STATS,
        kpiCards: buildKpiCards(EMPTY_STATS),
        ...emptyChartData(),
      });
      return;
    }

    const requestSeq = ++statsRequestSeq;
    this.setData({ loading: true, error: "" });
    try {
      const stats = await getGearStats();
      if (requestSeq !== statsRequestSeq) {
        return;
      }
      const chartData = buildChartData(stats);
      const offlineNotice = consumeOfflineCacheNotice();
      this.disposeCharts();
      this.setData({
        stats,
        kpiCards: buildKpiCards(stats),
        ...chartData,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
      wx.nextTick(() => this.renderCharts());
    } catch (error) {
      if (requestSeq !== statsRequestSeq) {
        return;
      }
      this.disposeCharts();
      if (isLoginRequiredError(error)) {
        this.setData({
          isLoggedIn: false,
          error: "",
          loading: false,
          ...emptyChartData(),
        });
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看装备统计。",
          redirectUrl: "/pages/gears/stats/index",
        });
        return;
      }
      this.setData({ error: getErrorMessage(error), ...emptyChartData() });
    } finally {
      if (requestSeq === statsRequestSeq) {
        this.setData({ loading: false });
      }
    }
  },

  renderCharts() {
    this.renderChart(
      "categoryCountChart",
      "分类数量占比",
      this.data.categoryCountChartRows,
    );
    this.renderChart(
      "categoryWeightChart",
      "分类重量占比",
      this.data.categoryWeightChartRows,
    );
    this.renderChart(
      "categoryValueChart",
      "分类估值占比",
      this.data.categoryValueChartRows,
    );
    this.renderChart(
      "statusCountChart",
      "状态数量占比",
      this.data.statusCountChartRows,
    );
  },

  renderChart(componentId: string, title: string, rows: ChartRow[]) {
    if (!rows.length) {
      return;
    }
    const component = this.selectComponent(
      `#${componentId}`,
    ) as unknown as ChartComponent | null;
    if (!component || typeof component.init !== "function") {
      return;
    }
    component.init((canvas, width, height, dpr) => {
      const chart = (echarts as any).init(canvas, null, {
        width,
        height,
        devicePixelRatio: dpr,
      });
      canvas.setChart(chart);
      chart.setOption({
        color: CHART_COLORS,
        tooltip: {
          trigger: "item",
          confine: true,
          formatter(params: any) {
            return `${params.name}: ${params.data.valueText} (${params.percent}%)`;
          },
        },
        series: [
          {
            name: title,
            type: "pie",
            radius: ["48%", "72%"],
            center: ["50%", "50%"],
            avoidLabelOverlap: true,
            label: {
              color: "#64748b",
              formatter: "{b}",
            },
            labelLine: {
              lineStyle: {
                color: "#94a3b8",
              },
            },
            data: rows.map((row) => ({
              name: row.label,
              value: row.value,
              valueText: row.valueText,
            })),
          },
        ],
      });
      return chart;
    });
  },

  disposeCharts() {
    for (const componentId of [
      "categoryCountChart",
      "categoryWeightChart",
      "categoryValueChart",
      "statusCountChart",
    ]) {
      const component = this.selectComponent(`#${componentId}`) as unknown as {
        chart?: { dispose?: () => void };
      } | null;
      component?.chart?.dispose?.();
    }
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/gears/stats/index") });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function buildKpiCards(stats: GearStatsResponse): KpiCard[] {
  const categoryCount = stats.by_category.filter(
    (item) => item.count > 0,
  ).length;
  return [
    { label: "装备数量", value: String(stats.current_count) },
    { label: "分类数", value: String(categoryCount) },
    { label: "总重量", value: formatGearWeight(stats.total_weight_g) },
    { label: "估值", value: formatGearPrice(stats.total_value_cents) },
  ];
}

function buildChartData(stats: GearStatsResponse) {
  const categoryCountChartRows = buildRows(
    stats.by_category.map((item) => ({
      label: item.label,
      value: item.count,
      valueText: `${item.count} 件`,
    })),
  );
  const categoryWeightChartRows = buildRows(
    stats.by_category.map((item) => ({
      label: item.label,
      value: item.total_weight_g ?? 0,
      valueText: formatGearWeight(item.total_weight_g ?? 0),
    })),
  );
  const categoryValueChartRows = buildRows(
    stats.by_category.map((item) => ({
      label: item.label,
      value: item.total_value_cents ?? 0,
      valueText: formatGearPrice(item.total_value_cents ?? 0),
    })),
  );
  const statusCountChartRows = buildRows(
    stats.by_status.map((item) => ({
      label: item.label,
      value: item.count,
      valueText: `${item.count} 件`,
    })),
  );

  return {
    categoryCountChartRows,
    categoryCountRows: categoryCountChartRows.slice(0, 5),
    categoryWeightChartRows,
    categoryWeightRows: categoryWeightChartRows.slice(0, 5),
    categoryValueChartRows,
    categoryValueRows: categoryValueChartRows.slice(0, 5),
    statusCountChartRows,
    statusCountRows: statusCountChartRows.slice(0, 5),
  };
}

function buildRows(
  items: Array<{ label: string; value: number; valueText: string }>,
): ChartRow[] {
  const positiveItems = items
    .filter((item) => item.value > 0)
    .sort((a, b) => b.value - a.value);
  const total = positiveItems.reduce((sum, item) => sum + item.value, 0);
  if (total <= 0) {
    return [];
  }
  return positiveItems.map((item) => ({
    ...item,
    percentageText: formatPercentage(item.value / total),
  }));
}

function emptyChartData() {
  return {
    categoryCountRows: [] as ChartRow[],
    categoryCountChartRows: [] as ChartRow[],
    categoryWeightRows: [] as ChartRow[],
    categoryWeightChartRows: [] as ChartRow[],
    categoryValueRows: [] as ChartRow[],
    categoryValueChartRows: [] as ChartRow[],
    statusCountRows: [] as ChartRow[],
    statusCountChartRows: [] as ChartRow[],
  };
}

function formatPercentage(value: number): string {
  return `${(value * 100).toFixed(1).replace(/\.0$/, "")}%`;
}
