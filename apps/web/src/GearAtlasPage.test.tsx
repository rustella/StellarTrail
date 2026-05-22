import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  GearAtlasPublicItem,
  GearAtlasSubmission,
} from "@stellartrail/shared-types";

import GearAtlasPage from "./GearAtlasPage";
import type { WebGearApi } from "./api";
import type { WebSession } from "./session";

type GearAtlasApi = Pick<
  WebGearApi,
  "listGearAtlas" | "getGearAtlasItem" | "createGearAtlasSubmission"
>;

const session: WebSession = {
  accessToken: "token",
  expiresAt: "2026-06-01T00:00:00Z",
  refreshToken: "refresh",
  refreshExpiresAt: "2026-07-01T00:00:00Z",
  user: { id: "u1", username: "trail-user", email: null, nickname: null },
};

function buildItem(
  overrides: Partial<GearAtlasPublicItem> = {},
): GearAtlasPublicItem {
  return {
    id: "atlas-1",
    category: "lighting_system",
    category_label: "照明系统",
    name: "测试头灯",
    brand: "NITECORE",
    model: "NU25",
    description: "轻量头灯",
    weight_g: 86,
    official_price_cents: 19900,
    official_price_currency: "CNY",
    specs: { max_brightness: "450 lm" },
    approved_at: "2026-01-24T00:00:00Z",
    source_name: "8264",
    source_url: "https://example.test/gear",
    source_rating_score: 4.8,
    source_rating_count: 12,
    created_at: "2026-01-23T00:00:00Z",
    updated_at: "2026-01-24T00:00:00Z",
    ...overrides,
  };
}

function buildSubmission(
  overrides: Partial<GearAtlasSubmission> = {},
): GearAtlasSubmission {
  return {
    ...buildItem(overrides),
    source_type: "manual",
    source_user_gear_id: null,
    status: "pending",
    rejection_reason: null,
    reviewed_at: null,
    ...overrides,
  };
}

function buildApi(): GearAtlasApi {
  return {
    listGearAtlas: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [buildItem()],
    }),
    getGearAtlasItem: vi.fn().mockResolvedValue(buildItem()),
    createGearAtlasSubmission: vi.fn().mockResolvedValue(
      buildSubmission({
        id: "submitted-1",
        name: "新投稿装备",
      }),
    ),
  };
}

describe("GearAtlasPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    window.history.replaceState(null, "", "/");
  });

  it("loads public gear atlas without a signed-in session", async () => {
    const api = buildApi();

    render(<GearAtlasPage api={api} session={null} />);

    expect(
      await screen.findByRole("heading", { name: "装备图鉴" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("button", { name: /测试头灯/ }),
    ).toBeInTheDocument();
    expect(api.listGearAtlas).toHaveBeenCalledWith(
      { sort: "approved_at_desc", limit: 20 },
      "zh-CN",
    );
  });

  it("reloads the list when search, category, and sort filters change", async () => {
    const api = buildApi();
    render(<GearAtlasPage api={api} session={null} />);

    await screen.findByRole("button", { name: /测试头灯/ });
    fireEvent.change(screen.getByLabelText("搜索图鉴装备"), {
      target: { value: "头灯" },
    });
    fireEvent.click(screen.getByRole("button", { name: "搜索" }));

    await waitFor(() => {
      expect(api.listGearAtlas).toHaveBeenLastCalledWith(
        { sort: "approved_at_desc", limit: 20, q: "头灯" },
        "zh-CN",
      );
    });

    fireEvent.change(screen.getByLabelText("图鉴分类"), {
      target: { value: "lighting_system" },
    });

    await waitFor(() => {
      expect(api.listGearAtlas).toHaveBeenLastCalledWith(
        {
          sort: "approved_at_desc",
          limit: 20,
          category: "lighting_system",
          q: "头灯",
        },
        "zh-CN",
      );
    });

    fireEvent.change(screen.getByLabelText("图鉴排序"), {
      target: { value: "name_asc" },
    });

    await waitFor(() => {
      expect(api.listGearAtlas).toHaveBeenLastCalledWith(
        {
          sort: "name_asc",
          limit: 20,
          category: "lighting_system",
          q: "头灯",
        },
        "zh-CN",
      );
    });
  });

  it("appends more atlas items on scroll and keeps the load-more fallback", async () => {
    const api = buildApi();
    vi.mocked(api.listGearAtlas)
      .mockResolvedValueOnce({
        next_cursor: "20",
        items: [buildItem({ id: "atlas-page-1", name: "第一页图鉴装备" })],
      })
      .mockResolvedValueOnce({
        next_cursor: null,
        items: [buildItem({ id: "atlas-page-2", name: "第二页图鉴装备" })],
      });

    render(<GearAtlasPage api={api} session={null} />);

    expect(
      await screen.findByRole("button", { name: /第一页图鉴装备/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "加载更多" }),
    ).toBeInTheDocument();

    const list = screen.getByLabelText("装备图鉴列表");
    Object.defineProperty(list, "scrollHeight", {
      configurable: true,
      value: 1000,
    });
    Object.defineProperty(list, "clientHeight", {
      configurable: true,
      value: 500,
    });
    Object.defineProperty(list, "scrollTop", {
      configurable: true,
      value: 380,
    });
    fireEvent.scroll(list);

    expect(
      await screen.findByRole("button", { name: /第二页图鉴装备/ }),
    ).toBeInTheDocument();
    expect(api.listGearAtlas).toHaveBeenLastCalledWith(
      { sort: "approved_at_desc", limit: 20, cursor: "20" },
      "zh-CN",
    );
  });

  it("opens a public atlas detail drawer with specs and source summary", async () => {
    const api = buildApi();
    render(<GearAtlasPage api={api} session={null} />);

    fireEvent.click(await screen.findByRole("button", { name: /测试头灯/ }));

    expect(await screen.findByLabelText("图鉴详情")).toBeInTheDocument();
    expect(api.getGearAtlasItem).toHaveBeenCalledWith("atlas-1", "zh-CN");
    expect(screen.getByText("最大亮度")).toBeInTheDocument();
    expect(screen.getByText("450 lm")).toBeInTheDocument();
    expect(screen.getAllByText("8264 · 4.8 分 / 12 条")).toHaveLength(2);
    expect(screen.getByRole("link", { name: "打开来源" })).toHaveAttribute(
      "href",
      "https://example.test/gear",
    );
  });

  it("prompts for login instead of submitting when the user is anonymous", async () => {
    const api = buildApi();
    render(<GearAtlasPage api={api} session={null} />);

    fireEvent.click(screen.getByRole("button", { name: "投稿装备" }));

    expect(
      await screen.findByText("登录后可以把新装备投稿到图鉴审核。"),
    ).toBeInTheDocument();
    expect(api.createGearAtlasSubmission).not.toHaveBeenCalled();
  });

  it("submits only public atlas fields for a signed-in user", async () => {
    const api = buildApi();
    render(<GearAtlasPage api={api} session={session} />);

    fireEvent.click(screen.getByRole("button", { name: "投稿装备" }));
    fireEvent.change(screen.getByLabelText("装备名称 *"), {
      target: { value: "新投稿装备" },
    });
    fireEvent.change(screen.getByLabelText("品牌"), {
      target: { value: "BLACKDIAMOND" },
    });
    fireEvent.change(screen.getByLabelText("重量（g）"), {
      target: { value: "86" },
    });
    fireEvent.change(screen.getByLabelText("官方价格"), {
      target: { value: "699" },
    });
    fireEvent.change(screen.getByLabelText("容量"), {
      target: { value: "45" },
    });
    fireEvent.click(screen.getByRole("button", { name: "提交审核" }));

    await waitFor(() => {
      expect(api.createGearAtlasSubmission).toHaveBeenCalledWith({
        category: "backpack_system",
        name: "新投稿装备",
        brand: "BLACKDIAMOND",
        model: null,
        description: null,
        weight_g: 86,
        official_price_cents: 69900,
        official_price_currency: "CNY",
        specs: { capacity: "45 L" },
      });
    });
    expect(
      await screen.findByText("已提交审核，管理员通过后会进入公开图鉴。"),
    ).toBeInTheDocument();
  });
});
