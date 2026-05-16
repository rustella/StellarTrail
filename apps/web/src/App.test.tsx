import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import type { WebGearApi } from "./api";

function buildClient(): WebGearApi {
  return {
    setAccessToken: vi.fn(),
    meta: vi.fn().mockResolvedValue({
      name: "StellarTrail",
      env: "local",
      database_kind: "sqlite",
    }),
    loginWithWechatCode: vi.fn().mockResolvedValue({
      access_token: "token-123",
      expires_at: "2026-06-01T00:00:00Z",
      user: { id: "u1", nickname: "测试用户", avatar_url: null },
    }),
    listGearCategories: vi.fn().mockResolvedValue({
      items: [
        { id: "all", label: "全部装备", count: 2 },
        { id: "electronics_system", label: "电子系统", count: 1 },
      ],
    }),
    getGearStats: vi.fn().mockResolvedValue({
      current_count: 2,
      archived_count: 1,
      total_value_cents: 3106442,
      total_weight_g: 16085,
      by_category: [],
      by_status: [],
    }),
    listGears: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [
        {
          id: "gear-1",
          category: "electronics_system",
          category_label: "电子系统",
          name: "SUMMIT 20000 超薄充电宝",
          brand: "NITECORE",
          model: "SUMMIT 20000",
          status: "available",
          status_label: "可用",
          weight_g: 315,
          purchase_price_cents: 63900,
          purchase_date: "2026-01-22",
          created_at: "2026-01-23T00:00:00Z",
          updated_at: "2026-01-23T00:00:00Z",
        },
        {
          id: "gear-2",
          category: "backpack_system",
          category_label: "背负系统",
          name: "轻量背包",
          brand: null,
          model: null,
          status: "maintenance",
          status_label: "保养中",
          weight_g: 860,
          purchase_price_cents: null,
          purchase_date: null,
          created_at: "2026-01-24T00:00:00Z",
          updated_at: "2026-01-24T00:00:00Z",
        },
      ],
    }),
    getGear: vi.fn().mockResolvedValue({
      id: "gear-1",
      user_id: "u1",
      category: "electronics_system",
      name: "SUMMIT 20000 超薄充电宝",
      brand: "NITECORE",
      model: "SUMMIT 20000",
      color: null,
      material: null,
      capacity: "20000mAh",
      size: null,
      description: "冬季徒步备用电源",
      weight_g: 315,
      warmth_index: null,
      waterproof_index: null,
      purchase_date: "2026-01-22",
      purchase_price_cents: 63900,
      expiry_or_warranty_date: null,
      purchase_location: "京东",
      status: "available",
      storage_location: "装备柜 A1",
      tags: ["冬季", "电子"],
      share_enabled: false,
      share_status: "not_shared",
      notes: "冷天备用",
      archived_at: null,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-23T00:00:00Z",
    }),
    createGear: vi.fn().mockResolvedValue({ id: "gear-new" }),
    updateGear: vi.fn().mockResolvedValue({ id: "gear-1" }),
    archiveGear: vi.fn().mockResolvedValue(undefined),
    restoreGear: vi.fn().mockResolvedValue({ id: "gear-1" }),
    exportGearsCsv: vi.fn().mockResolvedValue("name\nSUMMIT"),
    importGears: vi.fn().mockResolvedValue({
      created_count: 1,
      updated_count: 0,
      failed_count: 0,
      errors: [],
    }),
  };
}

describe("App", () => {
  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("logs in with local mock code and renders the gear dashboard", async () => {
    const client = buildClient();
    render(<App client={client} />);

    expect(
      screen.getByRole("heading", { name: "本地开发登录" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));

    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();
    expect(screen.getByText("当前装备数量")).toBeInTheDocument();
    expect(await screen.findByText("2 件")).toBeInTheDocument();
    expect(screen.getByText("¥31,064.42")).toBeInTheDocument();
    expect(screen.getByText("全部装备")).toBeInTheDocument();
    expect(
      screen.getByText("NITECORE SUMMIT 20000 超薄充电宝 · SUMMIT 20000"),
    ).toBeInTheDocument();
  });

  it("submits a minimal add gear form through the API client", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });
    await screen.findByText("NITECORE SUMMIT 20000 超薄充电宝 · SUMMIT 20000");

    fireEvent.click(screen.getByRole("button", { name: "添加装备" }));
    fireEvent.change(screen.getByLabelText("装备名称 *"), {
      target: { value: "测试头灯" },
    });
    fireEvent.change(screen.getByLabelText("品牌"), {
      target: { value: "Black Diamond" },
    });
    fireEvent.change(screen.getByLabelText("重量（g）"), {
      target: { value: "86" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存装备" }));

    await waitFor(() => {
      expect(client.createGear).toHaveBeenCalledWith(
        expect.objectContaining({
          category: "backpack_system",
          name: "测试头灯",
          brand: "Black Diamond",
          weight_g: 86,
          share_enabled: false,
        }),
      );
    });
    expect(client.listGears).toHaveBeenCalledTimes(2);
  });

  it("toggles dark mode and persists the theme preference", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });

    fireEvent.click(screen.getByRole("button", { name: "切换到黑夜模式" }));

    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(localStorage.getItem("stellartrail.web.theme")).toBe("dark");
    expect(
      screen.getByRole("button", { name: "切换到白天模式" }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "切换到白天模式" }));

    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    expect(localStorage.getItem("stellartrail.web.theme")).toBe("light");
  });
});
