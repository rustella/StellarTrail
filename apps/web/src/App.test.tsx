import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import type { WebGearApi } from "./api";

const sampleKnotSummary = {
  id: "adjustable-grip-hitch-knot",
  slug: "ke-tiao-jie-sheng-jie",
  title: "可调节绳结",
  summary: "调节绳索上的张力。",
  difficulty: "beginner",
  categories: [
    { id: "camping-knots", slug: "lu-ying-sheng-jie", title: "露营绳结" },
  ],
  types: [{ id: "hitch-knots", slug: "jie-sheng", title: "系结" }],
  media: [
    {
      id: "thumbnail",
      media_type: "thumbnail",
      url: "https://cdn.example.com/knots/thumb.webp",
      mime_type: "image/webp",
      width: 640,
      height: 360,
      size_bytes: 12345,
      attribution: "Knots3D",
      license_note: null,
    },
  ],
  href: "/api/skills/knots/detail/adjustable-grip-hitch-knot",
};

const sampleKnotDetail = {
  ...sampleKnotSummary,
  description: "适合风绳和营绳张力调节。",
  steps: ["将绳头绕过主绳。", "收紧后检查受力。"],
  locale: "zh-CN" as const,
};

function buildClient(): WebGearApi {
  return {
    setAccessToken: vi.fn(),
    setSessionTokens: vi.fn(),
    setSessionRefreshHandler: vi.fn(),
    resolveAssetUrl: vi.fn((pathOrUrl: string) =>
      pathOrUrl.startsWith("/")
        ? `https://assets.example.test${pathOrUrl}`
        : pathOrUrl,
    ),
    meta: vi.fn().mockResolvedValue({
      name: "StellarTrail",
      env: "local",
      database_kind: "sqlite",
    }),
    loginWithWechatCode: vi.fn().mockResolvedValue({
      access_token: "token-123",
      expires_at: "2026-06-01T00:00:00Z",
      refresh_token: "refresh-123",
      refresh_expires_at: "2026-07-01T00:00:00Z",
      user: { id: "u1", nickname: "测试用户", avatar_url: null },
    }),
    loginWithPassword: vi.fn().mockResolvedValue({
      access_token: "token-password",
      expires_at: "2026-06-01T00:00:00Z",
      refresh_token: "refresh-password",
      refresh_expires_at: "2026-07-01T00:00:00Z",
      user: {
        id: "u2",
        username: "trail-user",
        email: "trail@example.com",
        nickname: null,
        avatar_url: null,
      },
    }),
    sendEmailVerificationCode: vi.fn().mockResolvedValue({
      email: "new@example.com",
      expires_at: "2026-05-17T00:10:00Z",
      debug_code: "123456",
    }),
    sendEmailLoginCode: vi.fn().mockResolvedValue({
      email: "trail@example.com",
      expires_at: "2026-05-17T00:10:00Z",
      debug_code: "654321",
    }),
    loginWithEmailCode: vi.fn().mockResolvedValue({
      access_token: "token-email",
      expires_at: "2026-06-01T00:00:00Z",
      refresh_token: "refresh-email",
      refresh_expires_at: "2026-07-01T00:00:00Z",
      user: {
        id: "u4",
        username: "trail-user",
        email: "trail@example.com",
        nickname: null,
        avatar_url: null,
      },
    }),
    sendPasswordResetCode: vi.fn().mockResolvedValue({
      email: "trail@example.com",
      expires_at: "2026-05-17T00:10:00Z",
      debug_code: "987654",
    }),
    resetPassword: vi.fn().mockResolvedValue({
      access_token: "token-reset",
      expires_at: "2026-06-01T00:00:00Z",
      refresh_token: "refresh-reset",
      refresh_expires_at: "2026-07-01T00:00:00Z",
      user: {
        id: "u5",
        username: "trail-user",
        email: "trail@example.com",
        nickname: null,
        avatar_url: null,
      },
    }),
    createCaptcha: vi.fn().mockResolvedValue({
      captcha_ticket: "captcha-ticket",
      captcha_type: "image",
      image_svg: '<svg xmlns="http://www.w3.org/2000/svg"></svg>',
      expires_at: "2026-05-17T00:05:00Z",
      debug_answer: "ABCD",
    }),
    register: vi.fn().mockResolvedValue({
      access_token: "token-register",
      expires_at: "2026-06-01T00:00:00Z",
      refresh_token: "refresh-register",
      refresh_expires_at: "2026-07-01T00:00:00Z",
      user: {
        id: "u3",
        username: "new-user",
        email: "new@example.com",
        nickname: null,
        avatar_url: null,
      },
    }),
    listSkills: vi.fn().mockResolvedValue({
      items: [
        {
          id: "knots",
          slug: "knots",
          title: "绳结",
          summary: "户外、露营、钓鱼、航海等场景常用绳结技能。",
          item_count: 1,
          href: "/api/skills/knots/list",
        },
      ],
    }),
    listKnotFilters: vi.fn().mockResolvedValue({
      locale: "zh-CN",
      categories: [
        {
          id: "camping-knots",
          slug: "lu-ying-sheng-jie",
          title: "露营绳结",
          count: 1,
        },
      ],
      difficulties: [],
    }),
    listKnots: vi.fn().mockResolvedValue({
      locale: "zh-CN",
      items: [sampleKnotSummary],
      page: { offset: 0, limit: 24, next_offset: null },
    }),
    getKnotDetail: vi.fn().mockResolvedValue(sampleKnotDetail),
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
    listAdminFeedback: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [
        {
          id: "feedback-1",
          user: {
            id: "u-feedback",
            username: "trail_user",
            email: "trail@example.test",
            nickname: "寻径用户",
            avatar_url: null,
          },
          category: "bug",
          content: "装备详情页图片没有显示",
          contact: "trail@example.test",
          page: "/pages/gears/detail/index?id=gear-1",
          client_platform: "wechat_miniprogram",
          client_version: "0.1.0",
          device_model: "iPhone 15",
          status: "open",
          images: [
            {
              id: "upload-1",
              purpose: "feedback",
              original_filename: "screen.png",
              image_type: "png",
              content_type: "image/png",
              size_bytes: 67,
              sha256: "hash",
              download_url: "/api/admin/feedback-images/upload-1",
              created_at: "2026-01-23T00:00:00Z",
            },
          ],
          created_at: "2026-01-23T00:00:00Z",
          updated_at: "2026-01-23T00:00:00Z",
        },
      ],
    }),
    listAdminGearAtlasSubmissions: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [
        {
          id: "atlas-1",
          category: "electronics_system",
          category_label: "电子系统",
          name: "SUMMIT 20000 超薄充电宝",
          brand: "NITECORE",
          model: "SUMMIT 20000",
          description: "冬季徒步备用电源",
          weight_g: 315,
          official_price_cents: 69900,
          official_price_currency: "CNY",
          specs: { battery_capacity: "20000 mAh" },
          approved_at: null,
          source_type: "user_gear",
          source_user_gear_id: "gear-1",
          status: "pending",
          rejection_reason: null,
          reviewed_at: null,
          created_at: "2026-01-23T00:00:00Z",
          updated_at: "2026-01-23T00:00:00Z",
        },
      ],
    }),
    getAdminGearAtlasSubmission: vi.fn().mockResolvedValue({
      id: "atlas-1",
      category: "electronics_system",
      category_label: "电子系统",
      name: "SUMMIT 20000 超薄充电宝",
      brand: "NITECORE",
      model: "SUMMIT 20000",
      description: "冬季徒步备用电源",
      weight_g: 315,
      official_price_cents: 69900,
      official_price_currency: "CNY",
      specs: { battery_capacity: "20000 mAh" },
      approved_at: null,
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      status: "pending",
      rejection_reason: null,
      reviewed_at: null,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-23T00:00:00Z",
    }),
    approveGearAtlasSubmission: vi.fn().mockResolvedValue({
      id: "atlas-1",
      category: "electronics_system",
      category_label: "电子系统",
      name: "SUMMIT 20000 超薄充电宝",
      brand: "NITECORE",
      model: "SUMMIT 20000",
      description: "冬季徒步备用电源",
      weight_g: 315,
      official_price_cents: 69900,
      official_price_currency: "CNY",
      specs: { battery_capacity: "20000 mAh" },
      approved_at: "2026-01-24T00:00:00Z",
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      status: "approved",
      rejection_reason: null,
      reviewed_at: "2026-01-24T00:00:00Z",
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-24T00:00:00Z",
    }),
    rejectGearAtlasSubmission: vi.fn().mockResolvedValue({
      id: "atlas-1",
      category: "electronics_system",
      category_label: "电子系统",
      name: "SUMMIT 20000 超薄充电宝",
      brand: "NITECORE",
      model: "SUMMIT 20000",
      description: "冬季徒步备用电源",
      weight_g: 315,
      official_price_cents: 69900,
      official_price_currency: "CNY",
      specs: { battery_capacity: "20000 mAh" },
      approved_at: null,
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      status: "rejected",
      rejection_reason: "信息不足",
      reviewed_at: "2026-01-24T00:00:00Z",
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-24T00:00:00Z",
    }),
  };
}

function deferred<T>() {
  let resolve: (value: T) => void = () => {};
  const promise = new Promise<T>((nextResolve) => {
    resolve = nextResolve;
  });
  return { promise, resolve };
}

describe("App", () => {
  afterEach(() => {
    localStorage.clear();
    window.history.replaceState(null, "", "/");
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

  it("hides environment and database diagnostics from the gear dashboard header", async () => {
    const client = buildClient();
    vi.mocked(client.meta).mockResolvedValue({
      name: "StellarTrail",
      env: "production",
      database_kind: "postgres",
    });
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));

    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();
    await waitFor(() => expect(client.listGears).toHaveBeenCalled());
    expect(client.meta).not.toHaveBeenCalled();
    expect(screen.queryByText(/production/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/postgres/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/local\s*·\s*api/i)).not.toBeInTheDocument();
  });

  it("keeps outdoor skills expanded as the second navigation group", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    const navigation = await screen.findByRole("navigation", {
      name: "主导航",
    });

    expect(
      Array.from(navigation.children).map((item) => item.textContent?.trim()),
    ).toEqual(["装备库", "户外技能绳结", "路线清单 · 待接入"]);
    expect(screen.getByRole("button", { name: /户外技能/ })).toHaveAttribute(
      "aria-expanded",
      "true",
    );
    expect(screen.getByRole("button", { name: "绳结" })).toBeInTheDocument();
    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    expect(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    ).toHaveAttribute("aria-expanded", "false");
    expect(
      within(adminNavigation).queryByRole("button", { name: "装备图鉴审核" }),
    ).not.toBeInTheDocument();
  });

  it("renders the sidebar brand as a separated bilingual wordmark", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    const wordmark = await screen.findByLabelText("寻径星野 StellarTrail");

    expect(wordmark).toHaveClass("brand-wordmark");
    expect(within(wordmark).getByText("寻径星野")).toHaveClass(
      "brand-wordmark-cn",
    );
    expect(within(wordmark).getByText("StellarTrail")).toHaveClass(
      "brand-wordmark-en",
    );
  });

  it("uses the optimized product icon in the sidebar brand", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    const icon = await screen.findByRole("img", {
      name: "寻径星野产品图标",
    });

    expect(icon).toHaveClass("brand-logo-image");
    expect(icon).toHaveAttribute("src", "/app-icon.png");
    expect(screen.queryByText("星")).not.toBeInTheDocument();
  });

  it("opens the knots page from the expanded outdoor skills group", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();

    const outdoorSkills = screen.getByRole("button", { name: /户外技能/ });
    expect(outdoorSkills).toHaveAttribute("aria-expanded", "true");
    expect(client.listKnots).not.toHaveBeenCalled();

    const knotsNavItem = screen.getByRole("button", { name: "绳结" });
    fireEvent.click(knotsNavItem);

    expect(
      await screen.findByRole("heading", { name: "绳结" }),
    ).toBeInTheDocument();
    expect(knotsNavItem).toHaveAttribute("aria-current", "page");
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();
    expect(client.listKnots).toHaveBeenCalledWith(
      { offset: 0, limit: 24 },
      "zh-CN",
    );

    fireEvent.click(screen.getByRole("button", { name: "装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();
  });

  it("opens the gear atlas review queue for administrators", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();

    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    fireEvent.click(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    );
    fireEvent.click(
      within(adminNavigation).getByRole("button", { name: "装备图鉴审核" }),
    );

    expect(
      await screen.findByRole("heading", { name: "装备图鉴审核" }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe("/admin");
    expect((await screen.findAllByText("待审核")).length).toBeGreaterThan(0);
    expect(
      screen.getAllByText((_, element) =>
        Boolean(element?.textContent?.includes("个人装备生成")),
      ).length,
    ).toBeGreaterThan(0);
    expect(client.listAdminGearAtlasSubmissions).toHaveBeenCalledWith({
      status: "pending",
      limit: 50,
    });
  });

  it("opens the admin feedback page from the collapsed administrator group", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();

    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    fireEvent.click(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    );
    fireEvent.click(
      within(adminNavigation).getByRole("button", { name: "反馈信息" }),
    );

    expect(
      await screen.findByRole("heading", { name: "反馈信息" }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe("/admin/feedback");
    expect(
      await screen.findByText("装备详情页图片没有显示"),
    ).toBeInTheDocument();
    expect(screen.getByText("screen.png")).toBeInTheDocument();
    expect(client.listAdminFeedback).toHaveBeenCalledWith({
      status: "open",
      limit: 50,
    });
  });

  it("opens the admin review queue directly from the /admin URL", async () => {
    window.history.replaceState(null, "", "/admin");
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));

    expect(
      await screen.findByRole("heading", { name: "装备图鉴审核" }),
    ).toBeInTheDocument();
    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    expect(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(
      within(adminNavigation).getByRole("button", { name: "装备图鉴审核" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      within(adminNavigation).getByRole("button", { name: "反馈信息" }),
    ).toBeInTheDocument();
    expect(client.listAdminGearAtlasSubmissions).toHaveBeenCalledWith({
      status: "pending",
      limit: 50,
    });
  });

  it("opens the feedback page directly from the /admin/feedback URL with expanded admin nav", async () => {
    window.history.replaceState(null, "", "/admin/feedback");
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));

    expect(
      await screen.findByRole("heading", { name: "反馈信息" }),
    ).toBeInTheDocument();
    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    expect(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(
      within(adminNavigation).getByRole("button", { name: "反馈信息" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      within(adminNavigation).getByRole("button", { name: "装备图鉴审核" }),
    ).toBeInTheDocument();
    expect(client.listAdminFeedback).toHaveBeenCalledWith({
      status: "open",
      limit: 50,
    });
  });

  it("toggles the outdoor skills group without opening knots", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();

    const outdoorSkills = screen.getByRole("button", { name: /户外技能/ });
    expect(outdoorSkills).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: "绳结" })).toBeInTheDocument();

    fireEvent.click(outdoorSkills);
    expect(outdoorSkills).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByRole("button", { name: "绳结" }),
    ).not.toBeInTheDocument();
    expect(client.listKnots).not.toHaveBeenCalled();

    fireEvent.click(outdoorSkills);
    expect(outdoorSkills).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: "绳结" })).toBeInTheDocument();
    expect(client.listKnots).not.toHaveBeenCalled();
  });

  it("clears previous dashboard totals before loading a newly registered empty account", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(await screen.findByText("2 件")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "退出" }));

    const categoriesRequest =
      deferred<Awaited<ReturnType<WebGearApi["listGearCategories"]>>>();
    const statsRequest =
      deferred<Awaited<ReturnType<WebGearApi["getGearStats"]>>>();
    const listRequest =
      deferred<Awaited<ReturnType<WebGearApi["listGears"]>>>();
    vi.mocked(client.listGearCategories).mockReturnValueOnce(
      categoriesRequest.promise,
    );
    vi.mocked(client.getGearStats).mockReturnValueOnce(statsRequest.promise);
    vi.mocked(client.listGears).mockReturnValueOnce(listRequest.promise);

    fireEvent.click(screen.getByRole("button", { name: "账号登录" }));
    fireEvent.click(screen.getByRole("button", { name: "注册账号" }));
    fireEvent.change(screen.getByLabelText("用户名"), {
      target: { value: "New-User" },
    });
    fireEvent.change(screen.getByLabelText("邮箱"), {
      target: { value: "new@example.com" },
    });
    fireEvent.change(screen.getByLabelText("密码"), {
      target: { value: "strong-password" },
    });
    fireEvent.change(screen.getByLabelText("确认密码"), {
      target: { value: "strong-password" },
    });
    fireEvent.click(screen.getByRole("button", { name: "发送邮箱验证码" }));
    expect(await screen.findByText("本地验证码：123456")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("邮箱验证码"), {
      target: { value: "123456" },
    });
    fireEvent.click(screen.getByRole("button", { name: "注册并登录" }));

    await screen.findByRole("heading", { name: "装备管理" });
    expect(screen.getByText("0 件")).toBeInTheDocument();
    expect(screen.queryByText("2 件")).not.toBeInTheDocument();
    expect(
      within(screen.getByLabelText("分类筛选")).getByText("0"),
    ).toBeInTheDocument();

    await waitFor(() => expect(client.getGearStats).toHaveBeenCalledTimes(2));
    categoriesRequest.resolve({
      items: [{ id: "all", label: "全部装备", count: 0 }],
    });
    statsRequest.resolve({
      current_count: 0,
      archived_count: 0,
      total_value_cents: 0,
      total_weight_g: 0,
      by_category: [],
      by_status: [],
    });
    listRequest.resolve({ items: [], next_cursor: null });
    expect(await screen.findByText("还没有装备")).toBeInTheDocument();
  });

  it("keeps registration outside the top auth method switch", () => {
    const client = buildClient();
    render(<App client={client} />);

    const authMethods = screen.getByRole("group", { name: "登录方式" });
    expect(within(authMethods).getAllByRole("button")).toHaveLength(3);
    expect(
      within(authMethods).getByRole("button", { name: "微信登录" }),
    ).toBeInTheDocument();
    expect(
      within(authMethods).getByRole("button", { name: "账号登录" }),
    ).toBeInTheDocument();
    expect(
      within(authMethods).getByRole("button", { name: "邮箱验证码" }),
    ).toBeInTheDocument();
    expect(
      within(authMethods).queryByRole("button", { name: "注册账号" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "注册账号" }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      within(authMethods).getByRole("button", { name: "账号登录" }),
    );

    expect(screen.getByRole("button", { name: "登录" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "注册账号" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "忘记密码" }),
    ).toBeInTheDocument();
    expect(
      within(authMethods).queryByRole("button", { name: "注册账号" }),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "注册账号" }));

    expect(
      screen.queryByRole("group", { name: "登录方式" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "微信登录" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "账号登录" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "已有账号？返回登录" }));

    const restoredAuthMethods = screen.getByRole("group", { name: "登录方式" });
    expect(within(restoredAuthMethods).getAllByRole("button")).toHaveLength(3);
    expect(
      within(restoredAuthMethods).getByRole("button", { name: "微信登录" }),
    ).toBeInTheDocument();
    expect(
      within(restoredAuthMethods).getByRole("button", { name: "账号登录" }),
    ).toBeInTheDocument();
    expect(
      within(restoredAuthMethods).getByRole("button", { name: "邮箱验证码" }),
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

  it("shows the theme switch in the global app shell and persists the preference", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });

    const sidebar = screen.getByRole("complementary");
    const toolbar = document.querySelector(".toolbar");
    expect(toolbar).not.toBeNull();
    expect(
      within(toolbar as HTMLElement).queryByRole("button", {
        name: "切换到黑夜模式",
      }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      within(sidebar).getByRole("button", { name: "切换到黑夜模式" }),
    );

    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(localStorage.getItem("stellartrail.web.theme")).toBe("dark");
    expect(
      within(sidebar).getByRole("button", { name: "切换到白天模式" }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(
      within(sidebar).getByRole("button", { name: "切换到白天模式" }),
    );

    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    expect(localStorage.getItem("stellartrail.web.theme")).toBe("light");
  });

  it("logs in with an account password credential and renders the gear dashboard", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "账号登录" }));
    fireEvent.change(screen.getByLabelText("用户名或邮箱"), {
      target: { value: "trail@example.com" },
    });
    fireEvent.change(screen.getByLabelText("密码"), {
      target: { value: "correct-password" },
    });
    fireEvent.click(screen.getByRole("button", { name: "登录" }));

    await screen.findByRole("heading", { name: "装备管理" });
    expect(client.loginWithPassword).toHaveBeenCalledWith({
      account: "trail@example.com",
      password: "correct-password",
    });
    expect(localStorage.getItem("stellartrail.web.session")).toContain(
      "token-password",
    );
  });

  it("logs in with an email verification code", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "邮箱验证码" }));
    fireEvent.change(screen.getByLabelText("邮箱"), {
      target: { value: "trail@example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "获取邮箱验证码" }));
    expect(await screen.findByText("本地验证码：654321")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("邮箱验证码"), {
      target: { value: "654321" },
    });
    fireEvent.click(screen.getByRole("button", { name: "邮箱验证码登录" }));

    await screen.findByRole("heading", { name: "装备管理" });
    expect(client.sendEmailLoginCode).toHaveBeenCalledWith({
      email: "trail@example.com",
    });
    expect(client.loginWithEmailCode).toHaveBeenCalledWith({
      email: "trail@example.com",
      email_verification_code: "654321",
    });
    expect(localStorage.getItem("stellartrail.web.session")).toContain(
      "token-email",
    );
  });

  it("resets password with an email code and stores the new session", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "账号登录" }));
    fireEvent.click(screen.getByRole("button", { name: "忘记密码" }));
    fireEvent.change(screen.getByLabelText("邮箱"), {
      target: { value: "trail@example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "获取邮箱验证码" }));
    expect(await screen.findByText("本地验证码：987654")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("邮箱验证码"), {
      target: { value: "987654" },
    });
    fireEvent.change(screen.getByLabelText("新密码"), {
      target: { value: "new-strong-password" },
    });
    fireEvent.change(screen.getByLabelText("确认新密码"), {
      target: { value: "new-strong-password" },
    });
    fireEvent.click(screen.getByRole("button", { name: "重设密码并登录" }));

    await screen.findByRole("heading", { name: "装备管理" });
    expect(client.sendPasswordResetCode).toHaveBeenCalledWith({
      email: "trail@example.com",
    });
    expect(client.resetPassword).toHaveBeenCalledWith({
      email: "trail@example.com",
      email_verification_code: "987654",
      password: "new-strong-password",
      confirm_password: "new-strong-password",
    });
    expect(localStorage.getItem("stellartrail.web.session")).toContain(
      "token-reset",
    );
  });

  it("registers a password account with the registration session response", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "账号登录" }));
    fireEvent.click(screen.getByRole("button", { name: "注册账号" }));
    fireEvent.change(screen.getByLabelText("用户名"), {
      target: { value: "New-User" },
    });
    fireEvent.change(screen.getByLabelText("邮箱"), {
      target: { value: "new@example.com" },
    });
    fireEvent.change(screen.getByLabelText("密码"), {
      target: { value: "strong-password" },
    });
    fireEvent.change(screen.getByLabelText("确认密码"), {
      target: { value: "strong-password" },
    });

    fireEvent.click(screen.getByRole("button", { name: "发送邮箱验证码" }));
    expect(await screen.findByText("本地验证码：123456")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("邮箱验证码"), {
      target: { value: "123456" },
    });
    fireEvent.click(screen.getByRole("button", { name: "注册并登录" }));

    await screen.findByRole("heading", { name: "装备管理" });
    expect(client.sendEmailVerificationCode).toHaveBeenCalledWith({
      email: "new@example.com",
    });
    expect(client.register).toHaveBeenCalledWith({
      username: "New-User",
      email: "new@example.com",
      password: "strong-password",
      confirm_password: "strong-password",
      email_verification_code: "123456",
    });
    expect(client.loginWithPassword).not.toHaveBeenCalled();
    expect(localStorage.getItem("stellartrail.web.session")).toContain(
      "token-register",
    );
    expect(localStorage.getItem("stellartrail.web.session")).not.toContain(
      "token-password",
    );
  });
});
