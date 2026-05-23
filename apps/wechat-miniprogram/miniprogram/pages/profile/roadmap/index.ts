import {
  consumeOfflineCacheNotice,
  getErrorMessage,
  hasAccessToken,
  isLoginRequiredError,
  listMyRoadmap,
  listRoadmap,
  subscribeRoadmapItem,
  type RoadmapItem,
  type RoadmapStatus,
  unsubscribeRoadmapItem,
  unvoteRoadmapItem,
  voteRoadmapItem,
} from "../../../utils/api-roadmap";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  requireLoginForAction,
  showLoginPrompt,
} from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";

type RoadmapStatusFilter = "all" | RoadmapStatus;

interface RoadmapViewItem extends RoadmapItem {
  categoryText: string;
  statusText: string;
  statusClass: string;
  voteText: string;
  subscriptionText: string;
  actionLoading: boolean;
}

const STATUS_FILTERS: Array<{ id: RoadmapStatusFilter; label: string }> = [
  { id: "all", label: "全部状态" },
  { id: "planned", label: "已规划" },
  { id: "designing", label: "设计中" },
  { id: "building", label: "开发中" },
  { id: "preview", label: "预览中" },
  { id: "shipped", label: "已上线" },
];

Page({
  data: {
    items: [] as RoadmapViewItem[],
    statusFilters: STATUS_FILTERS,
    statusFilterLabels: STATUS_FILTERS.map((item) => item.label),
    selectedStatus: "all" as RoadmapStatusFilter,
    selectedStatusIndex: 0,
    loading: false,
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  onLoad() {
    this.loadRoadmap();
  },

  onShow() {
    syncPageTheme(this);
  },

  onPullDownRefresh() {
    this.loadRoadmap().finally(() => wx.stopPullDownRefresh());
  },

  onStatusFilterChange(event: WechatMiniprogram.BaseEvent) {
    const index = Number((event as any).detail.value || 0);
    const selected = STATUS_FILTERS[index] ?? STATUS_FILTERS[0];
    this.setData({
      selectedStatus: selected.id,
      selectedStatusIndex: index,
    });
    this.loadRoadmap();
  },

  async loadRoadmap() {
    const request = roadmapRequest(this.data.selectedStatus);
    this.setData({ loading: true, error: "", offlineNotice: "" });
    try {
      const response = hasAccessToken()
        ? await listMyRoadmap(request)
        : await listRoadmap(request);
      this.setData({
        items: response.items.map(mapRoadmapItem),
        loading: false,
        offlineNotice: consumeOfflineCacheNotice(),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        const response = await listRoadmap(request);
        this.setData({
          items: response.items.map(mapRoadmapItem),
          loading: false,
          offlineNotice: consumeOfflineCacheNotice(),
        });
        return;
      }
      this.setData({
        error: getErrorMessage(error),
        loading: false,
        items: [] as RoadmapViewItem[],
      });
    }
  },

  async toggleVote(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以给你关心的功能投票。",
        redirectUrl: "/pages/profile/roadmap/index",
      })
    ) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const item = this.data.items.find((entry) => entry.id === id);
    if (!item || item.actionLoading) {
      return;
    }
    this.setItemLoading(id, true);
    try {
      const updated = item.is_voted
        ? await unvoteRoadmapItem(id)
        : await voteRoadmapItem(id);
      this.mergeUpdatedItem(updated);
    } catch (error) {
      this.handleActionError(error);
    } finally {
      this.setItemLoading(id, false);
    }
  },

  async toggleSubscription(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    if (
      !requireLoginForAction(this, {
        message: "登录后可以订阅功能进度，后续在站内查看更新。",
        redirectUrl: "/pages/profile/roadmap/index",
      })
    ) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const item = this.data.items.find((entry) => entry.id === id);
    if (!item || item.actionLoading) {
      return;
    }
    this.setItemLoading(id, true);
    try {
      const updated = item.is_subscribed
        ? await unsubscribeRoadmapItem(id)
        : await subscribeRoadmapItem(id);
      this.mergeUpdatedItem(updated);
    } catch (error) {
      this.handleActionError(error);
    } finally {
      this.setItemLoading(id, false);
    }
  },

  setItemLoading(id: string, actionLoading: boolean) {
    this.setData({
      items: this.data.items.map((item) =>
        item.id === id ? { ...item, actionLoading } : item,
      ),
    });
  },

  mergeUpdatedItem(updated: RoadmapItem) {
    this.setData({
      items: this.data.items.map((item) =>
        item.id === updated.id
          ? { ...mapRoadmapItem(updated), actionLoading: false }
          : item,
      ),
    });
  },

  handleActionError(error: unknown) {
    if (isLoginRequiredError(error)) {
      showLoginPrompt(this, {
        message: "登录状态已过期，请重新登录后继续。",
        redirectUrl: "/pages/profile/roadmap/index",
      });
      return;
    }
    wx.showToast({ title: getErrorMessage(error), icon: "none" });
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function roadmapRequest(selectedStatus: RoadmapStatusFilter) {
  return {
    client_key: "wechat_miniprogram" as const,
    status: selectedStatus === "all" ? undefined : selectedStatus,
    limit: 50,
  };
}

function mapRoadmapItem(item: RoadmapItem): RoadmapViewItem {
  return {
    ...item,
    categoryText: categoryText(item.category),
    statusText: statusText(item.status),
    statusClass: statusClass(item.status),
    voteText: `${item.vote_count} 票`,
    subscriptionText: item.is_subscribed ? "已订阅" : "订阅",
    actionLoading: false,
  };
}

function categoryText(category: string): string {
  const labels: Record<string, string> = {
    gear: "装备",
    skills: "技能",
    routes: "路线",
    offline: "离线",
    safety: "安全",
    community: "社区",
  };
  return labels[category] ?? "规划";
}

function statusText(status: string): string {
  const labels: Record<string, string> = {
    planned: "已规划",
    designing: "设计中",
    building: "开发中",
    preview: "预览中",
    shipped: "已上线",
  };
  return labels[status] ?? "规划中";
}

function statusClass(status: string): string {
  return `status-${status || "planned"}`;
}
