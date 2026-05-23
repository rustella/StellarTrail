import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import type {
  ClientVersion,
  GearAtlasPublicItem,
  GearAtlasSubmission,
} from "@stellartrail/shared-types";
import { afterEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import type { WebGearApi } from "./api";

const sampleKnotSummary = {
  id: "adjustable-grip-hitch-knot",
  slug: "ke-tiao-jie-sheng-jie",
  title: "可调节绳结",
  summary: "调节绳索上的张力。",
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
  href: "/api/v1/skills/knots/detail/adjustable-grip-hitch-knot",
};

const sampleKnotDetail = {
  ...sampleKnotSummary,
  description: "适合风绳和营绳张力调节。",
  steps: ["将绳头绕过主绳。", "收紧后检查受力。"],
  locale: "zh-CN" as const,
};

function buildAtlasSubmission(
  overrides: Partial<GearAtlasSubmission> = {},
): GearAtlasSubmission {
  return {
    id: "atlas-fixture",
    category: "electronics_system",
    category_label: "电子系统",
    name: "测试装备",
    brand: null,
    model: null,
    description: "公开字段",
    weight_g: 315,
    official_price_cents: 69900,
    official_price_currency: "CNY",
    variants: [],
    specs: { battery_capacity: "20000 mAh" },
    approved_at: null,
    source_type: "external_import",
    source_user_gear_id: null,
    status: "pending",
    rejection_reason: null,
    review_changes: [],
    reviewed_at: null,
    is_deleted: false,
    created_at: "2026-01-23T00:00:00Z",
    updated_at: "2026-01-23T00:00:00Z",
    ...overrides,
  };
}

function buildAtlasPublicItem(
  overrides: Partial<GearAtlasPublicItem> = {},
): GearAtlasPublicItem {
  return buildAtlasSubmission(overrides);
}

function buildClientVersion(
  overrides: Partial<ClientVersion> = {},
): ClientVersion {
  return {
    id: "client-version-1",
    client_key: "wechat_miniprogram",
    version: "0.1.0",
    title: "0.1.0 初始版本",
    release_notes: ["装备库上线", "绳结离线缓存"],
    status: "published",
    published_at: "2026-05-23T00:00:00Z",
    created_at: "2026-05-23T00:00:00Z",
    updated_at: "2026-05-23T00:00:00Z",
    ...overrides,
  };
}

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
    getCurrentClientVersion: vi.fn().mockResolvedValue(buildClientVersion()),
    listClientVersions: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [buildClientVersion()],
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
          href: "/api/v1/skills/knots/list",
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
          is_deleted: false,
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
          is_deleted: false,
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
      atlas_item_id: null,
      selected_variant_key: "standard",
      selected_variant_label: "标准版",
      specs: { battery_capacity: "20000 mAh" },
      tags: ["冬季", "电子"],
      share_enabled: false,
      share_status: "not_shared",
      notes: "冷天备用",
      archived_at: null,
      is_deleted: false,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-23T00:00:00Z",
    }),
    createGear: vi.fn().mockResolvedValue({ id: "gear-new" }),
    updateGear: vi.fn().mockResolvedValue({ id: "gear-1" }),
    archiveGear: vi.fn().mockResolvedValue(undefined),
    deleteGear: vi.fn().mockResolvedValue(undefined),
    undeleteGear: vi
      .fn()
      .mockResolvedValue({ id: "gear-1", is_deleted: false }),
    restoreGear: vi.fn().mockResolvedValue({ id: "gear-1" }),
    exportGearsCsv: vi.fn().mockResolvedValue("name\nSUMMIT"),
    importGears: vi.fn().mockResolvedValue({
      created_count: 1,
      updated_count: 0,
      failed_count: 0,
      errors: [],
    }),
    listGearAtlas: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [
        buildAtlasPublicItem({
          id: "atlas-public-1",
          name: "已收录头灯",
          brand: "NITECORE",
          model: "NU25",
          category: "lighting_system",
          category_label: "照明系统",
          approved_at: "2026-01-24T00:00:00Z",
        }),
      ],
    }),
    getGearAtlasItem: vi.fn().mockResolvedValue(
      buildAtlasPublicItem({
        id: "atlas-public-1",
        name: "已收录头灯",
        brand: "NITECORE",
        model: "NU25",
        category: "lighting_system",
        category_label: "照明系统",
        approved_at: "2026-01-24T00:00:00Z",
      }),
    ),
    createGearAtlasSubmission: vi.fn().mockResolvedValue(
      buildAtlasSubmission({
        id: "atlas-submitted",
        name: "新投稿装备",
        status: "pending",
      }),
    ),
    createGearAtlasSubmissionFromGear: vi.fn().mockResolvedValue(
      buildAtlasSubmission({
        id: "atlas-from-gear",
        source_user_gear_id: "gear-1",
        status: "pending",
      }),
    ),
    listMyGearAtlasSubmissions: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [],
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
              download_url: "/api/v1/admin/feedback-images/upload-1",
              is_deleted: false,
              created_at: "2026-01-23T00:00:00Z",
            },
          ],
          is_deleted: false,
          created_at: "2026-01-23T00:00:00Z",
          updated_at: "2026-01-23T00:00:00Z",
        },
      ],
    }),
    listAdminClientVersions: vi.fn().mockResolvedValue({
      next_cursor: null,
      items: [buildClientVersion()],
    }),
    createAdminClientVersion: vi.fn().mockImplementation((request) =>
      Promise.resolve(
        buildClientVersion({
          id: "client-version-created",
          ...request,
          published_at:
            request.status === "published" ? "2026-05-23T01:00:00Z" : null,
        }),
      ),
    ),
    updateAdminClientVersion: vi.fn().mockImplementation((id, request) =>
      Promise.resolve(
        buildClientVersion({
          id,
          ...request,
          published_at:
            request.status === "published" ? "2026-05-23T02:00:00Z" : null,
        }),
      ),
    ),
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
          variants: [],
          specs: { battery_capacity: "20000 mAh" },
          approved_at: null,
          source_type: "user_gear",
          source_user_gear_id: "gear-1",
          source_name: "8264",
          source_url: "https://example.test/gear",
          source_rating_score: 4.8,
          source_rating_count: 12,
          status: "pending",
          rejection_reason: null,
          reviewed_at: null,
          is_deleted: false,
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
      variants: [],
      specs: { battery_capacity: "20000 mAh" },
      approved_at: null,
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      source_name: "8264",
      source_url: "https://example.test/gear",
      source_rating_score: 4.8,
      source_rating_count: 12,
      status: "pending",
      rejection_reason: null,
      reviewed_at: null,
      is_deleted: false,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-23T00:00:00Z",
    }),
    updateAdminGearAtlasSubmission: vi.fn().mockImplementation((_, request) =>
      Promise.resolve(
        buildAtlasSubmission({
          id: "atlas-1",
          ...request,
        }),
      ),
    ),
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
      variants: [],
      specs: { battery_capacity: "20000 mAh" },
      approved_at: "2026-01-24T00:00:00Z",
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      status: "approved",
      rejection_reason: null,
      reviewed_at: "2026-01-24T00:00:00Z",
      is_deleted: false,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-24T00:00:00Z",
    }),
    deleteAdminFeedback: vi.fn().mockResolvedValue(undefined),
    restoreAdminFeedback: vi.fn().mockResolvedValue(undefined),
    deleteAdminGearAtlasSubmission: vi.fn().mockResolvedValue(undefined),
    restoreAdminGearAtlasSubmission: vi.fn().mockResolvedValue(
      buildAtlasSubmission({
        id: "atlas-1",
        is_deleted: false,
      }),
    ),
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
      variants: [],
      specs: { battery_capacity: "20000 mAh" },
      approved_at: null,
      source_type: "user_gear",
      source_user_gear_id: "gear-1",
      status: "rejected",
      rejection_reason: "信息不足",
      reviewed_at: "2026-01-24T00:00:00Z",
      is_deleted: false,
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
    ).toEqual(["装备库", "装备图鉴", "户外技能绳结", "路线清单 · 待接入"]);
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

  it("opens the public gear atlas directly without requiring login", async () => {
    window.history.replaceState(null, "", "/gear-atlas");
    const client = buildClient();
    render(<App client={client} />);

    expect(
      await screen.findByRole("heading", { name: "装备图鉴" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "本地开发登录" }),
    ).not.toBeInTheDocument();
    expect(
      await screen.findByRole("button", { name: /已收录头灯/ }),
    ).toBeInTheDocument();
    expect(client.listGearAtlas).toHaveBeenCalledWith(
      { sort: "approved_at_desc", limit: 20 },
      "zh-CN",
    );
  });

  it("opens the public gear atlas from the signed-in app shell", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    expect(
      await screen.findByRole("heading", { name: "装备管理" }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "装备图鉴" }));

    expect(
      await screen.findByRole("heading", { name: "装备图鉴" }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe("/gear-atlas");
    expect(client.listGearAtlas).toHaveBeenCalledWith(
      { sort: "approved_at_desc", limit: 20 },
      "zh-CN",
    );
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
    fireEvent.click(screen.getByRole("button", { name: /SUMMIT 20000/ }));
    expect(
      await screen.findByText("8264 · 4.8 分 / 12 条"),
    ).toBeInTheDocument();
    expect(screen.getByText("https://example.test/gear")).toBeInTheDocument();
    expect(client.listAdminGearAtlasSubmissions).toHaveBeenCalledWith({
      status: "pending",
      deleted: "active",
      limit: 20,
    });
  });

  it("shows localized spec names in the admin atlas review detail", async () => {
    const client = buildClient();
    vi.mocked(client.listAdminGearAtlasSubmissions).mockResolvedValueOnce({
      next_cursor: null,
      items: [
        buildAtlasSubmission({
          id: "sleep-atlas",
          category: "sleep_system",
          category_label: "睡眠系统",
          name: "超轻羽绒睡袋",
          specs: {
            fill_weight: "700g",
            filling: "FP700+ 90% 白鹅绒",
            material: "15D 460T 超细尼龙",
            temperature_or_r_value: "0~-10度",
          },
          variants: [{ key: "m-75-195", label: "M 75*195" }],
        }),
      ],
    });
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

    expect(await screen.findByText("填充重量")).toBeInTheDocument();
    expect(screen.getByText("填充物")).toBeInTheDocument();
    expect(screen.getByText("材质")).toBeInTheDocument();
    expect(screen.getByText("可选尺寸")).toBeInTheDocument();
    expect(await screen.findByDisplayValue("M 75*195")).toBeInTheDocument();
    expect(screen.getByText("温标/R 值")).toBeInTheDocument();
    expect(screen.queryByText("fill_weight")).not.toBeInTheDocument();
    expect(screen.queryByText("filling")).not.toBeInTheDocument();
    expect(screen.queryByText("material")).not.toBeInTheDocument();
    expect(screen.queryByText("size")).not.toBeInTheDocument();
    expect(
      screen.queryByText("temperature_or_r_value"),
    ).not.toBeInTheDocument();

    fireEvent.change(screen.getByDisplayValue("M 75*195"), {
      target: { value: "M 75*196" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存字段" }));
    await waitFor(() => {
      expect(client.updateAdminGearAtlasSubmission).toHaveBeenCalledWith(
        "sleep-atlas",
        expect.objectContaining({
          variants: expect.arrayContaining([
            expect.objectContaining({ key: "m-75-196", label: "M 75*196" }),
          ]),
        }),
      );
    });
  });

  it("requires a rejection reason in the admin atlas review detail", async () => {
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
    fireEvent.click(screen.getByRole("button", { name: "拒绝" }));

    expect(client.rejectGearAtlasSubmission).not.toHaveBeenCalled();

    const reasonInput = screen.getByPlaceholderText(
      "请说明需要用户修改的字段或原因",
    ) as HTMLTextAreaElement;
    const valueSetter = Object.getOwnPropertyDescriptor(
      window.HTMLTextAreaElement.prototype,
      "value",
    )?.set;
    valueSetter?.call(reasonInput, "品牌型号不完整");
    fireEvent.input(reasonInput);
    fireEvent.click(screen.getByRole("button", { name: "拒绝" }));

    await waitFor(() => {
      expect(client.rejectGearAtlasSubmission).toHaveBeenCalledWith("atlas-1", {
        reason: "品牌型号不完整",
      });
    });
  });

  it("saves dirty admin edits before approving an atlas submission", async () => {
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
    fireEvent.change(
      await screen.findByDisplayValue("SUMMIT 20000 超薄充电宝"),
      {
        target: { value: "SUMMIT 20000 超薄充电宝 修订版" },
      },
    );
    fireEvent.click(screen.getByRole("button", { name: "保存并通过" }));

    await waitFor(() => {
      expect(client.updateAdminGearAtlasSubmission).toHaveBeenCalledWith(
        "atlas-1",
        expect.objectContaining({
          name: "SUMMIT 20000 超薄充电宝 修订版",
        }),
      );
      expect(client.approveGearAtlasSubmission).toHaveBeenCalledWith("atlas-1");
    });
    expect(
      vi.mocked(client.updateAdminGearAtlasSubmission).mock
        .invocationCallOrder[0],
    ).toBeLessThan(
      vi.mocked(client.approveGearAtlasSubmission).mock.invocationCallOrder[0],
    );
  });

  it("loads additional atlas review submissions when the list is scrolled", async () => {
    const client = buildClient();
    vi.mocked(client.listAdminGearAtlasSubmissions)
      .mockResolvedValueOnce({
        next_cursor: "20",
        items: [
          buildAtlasSubmission({
            id: "atlas-page-1",
            name: "第一页审核装备",
          }),
        ],
      })
      .mockResolvedValueOnce({
        next_cursor: null,
        items: [
          buildAtlasSubmission({
            id: "atlas-page-2",
            name: "第二页审核装备",
          }),
        ],
      });
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

    expect(await screen.findAllByText("第一页审核装备")).toHaveLength(2);
    const reviewList = screen.getByLabelText("图鉴审核投稿列表");
    Object.defineProperty(reviewList, "scrollHeight", {
      configurable: true,
      value: 1000,
    });
    Object.defineProperty(reviewList, "clientHeight", {
      configurable: true,
      value: 400,
    });
    Object.defineProperty(reviewList, "scrollTop", {
      configurable: true,
      value: 560,
    });

    fireEvent.scroll(reviewList);

    expect(await screen.findByText("第二页审核装备")).toBeInTheDocument();
    expect(client.listAdminGearAtlasSubmissions).toHaveBeenLastCalledWith({
      status: "pending",
      deleted: "active",
      limit: 20,
      cursor: "20",
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
      deleted: "active",
      limit: 50,
    });
  });

  it("opens the admin client versions page and saves a release", async () => {
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
      within(adminNavigation).getByRole("button", { name: "版本信息" }),
    );

    expect(
      await screen.findByRole("heading", { name: "版本信息" }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe("/admin/client-versions");
    expect(client.listAdminClientVersions).toHaveBeenCalledWith({
      client_key: "wechat_miniprogram",
      status: undefined,
      limit: 50,
    });
    expect(await screen.findByText("0.1.0 初始版本")).toBeInTheDocument();

    fireEvent.change(screen.getByPlaceholderText("例如 0.1.0"), {
      target: { value: "0.2.0" },
    });
    fireEvent.change(screen.getByPlaceholderText("例如 0.1.0 初始版本"), {
      target: { value: "0.2.0 装备图鉴更新" },
    });
    fireEvent.change(screen.getByPlaceholderText("每行一条更新内容"), {
      target: { value: "新增版本信息\n优化绳结离线缓存" },
    });
    fireEvent.click(screen.getByRole("button", { name: "创建版本" }));

    await waitFor(() => {
      expect(client.createAdminClientVersion).toHaveBeenCalledWith({
        client_key: "wechat_miniprogram",
        version: "0.2.0",
        title: "0.2.0 装备图鉴更新",
        release_notes: ["新增版本信息", "优化绳结离线缓存"],
        status: "draft",
      });
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
    expect(
      within(adminNavigation).getByRole("button", { name: "版本信息" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(client.listAdminGearAtlasSubmissions).toHaveBeenCalledWith({
        status: "pending",
        deleted: "active",
        limit: 20,
      });
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
    expect(
      within(adminNavigation).getByRole("button", { name: "版本信息" }),
    ).toBeInTheDocument();
    expect(client.listAdminFeedback).toHaveBeenCalledWith({
      status: "open",
      deleted: "active",
      limit: 50,
    });
  });

  it("opens the client versions page directly from the /admin/client-versions URL", async () => {
    window.history.replaceState(null, "", "/admin/client-versions");
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));

    expect(
      await screen.findByRole("heading", { name: "版本信息" }),
    ).toBeInTheDocument();
    const adminNavigation = screen.getByRole("navigation", {
      name: "管理员导航",
    });
    expect(
      within(adminNavigation).getByRole("button", { name: "管理员后台" }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(
      within(adminNavigation).getByRole("button", { name: "版本信息" }),
    ).toHaveAttribute("aria-current", "page");
    expect(client.listAdminClientVersions).toHaveBeenCalledWith({
      client_key: "wechat_miniprogram",
      status: undefined,
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

  it("submits a personal gear item to the atlas review queue from detail", async () => {
    const client = buildClient();
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });
    await screen.findByText("NITECORE SUMMIT 20000 超薄充电宝 · SUMMIT 20000");
    fireEvent.click(screen.getAllByRole("button", { name: "查看" })[0]);

    expect(await screen.findByLabelText("装备详情")).toBeInTheDocument();
    expect(screen.getByText("我的尺寸")).toBeInTheDocument();
    expect(screen.getByText("标准版")).toBeInTheDocument();
    expect(
      screen.getByText("关联或投稿到图鉴后可查看该装备可选尺寸"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "投稿到图鉴" }));

    await waitFor(() => {
      expect(client.createGearAtlasSubmissionFromGear).toHaveBeenCalledWith(
        "gear-1",
      );
    });
    expect(await screen.findByText("已提交图鉴审核")).toBeInTheDocument();
  });

  it("shows admin atlas review changes in personal gear detail", async () => {
    const client = buildClient();
    vi.mocked(client.listMyGearAtlasSubmissions).mockResolvedValueOnce({
      next_cursor: null,
      items: [
        buildAtlasSubmission({
          id: "atlas-approved",
          source_user_gear_id: "gear-1",
          status: "approved",
          review_changes: [
            {
              field: "name",
              label: "名称",
              before: "旧名称",
              after: "新名称",
            },
          ],
        }),
      ],
    });
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });
    await screen.findByText("NITECORE SUMMIT 20000 超薄充电宝 · SUMMIT 20000");
    fireEvent.click(screen.getAllByRole("button", { name: "查看" })[0]);

    expect(await screen.findByText("图鉴投稿：已通过")).toBeInTheDocument();
    expect(
      screen.getByText("管理员调整：名称 从 旧名称 改为 新名称"),
    ).toBeInTheDocument();
  });

  it("shows atlas variants with the selected personal size when gear is linked", async () => {
    const client = buildClient();
    vi.mocked(client.getGear).mockResolvedValueOnce({
      id: "gear-1",
      user_id: "u1",
      category: "sleep_system",
      name: "超轻羽绒睡袋",
      brand: "BLACKICE",
      model: "G700",
      description: "冬季备用",
      weight_g: 1010,
      purchase_date: "2026-01-22",
      purchase_price_cents: 90000,
      purchase_location: "京东",
      status: "available",
      storage_location: "装备柜 A1",
      atlas_item_id: "atlas-public-1",
      selected_variant_key: "m-75-195",
      selected_variant_label: "M 75*195",
      specs: {},
      tags: ["冬季"],
      share_enabled: false,
      share_status: "not_shared",
      notes: null,
      archived_at: null,
      is_deleted: false,
      created_at: "2026-01-23T00:00:00Z",
      updated_at: "2026-01-23T00:00:00Z",
    });
    vi.mocked(client.getGearAtlasItem).mockResolvedValueOnce(
      buildAtlasPublicItem({
        id: "atlas-public-1",
        variants: [
          { key: "m-75-195", label: "M 75*195" },
          { key: "l-80-205", label: "L 80*205" },
        ],
      }),
    );
    render(<App client={client} />);

    fireEvent.click(screen.getByRole("button", { name: "进入装备库" }));
    await screen.findByRole("heading", { name: "装备管理" });
    const viewButtons = await screen.findAllByRole("button", { name: "查看" });
    fireEvent.click(viewButtons[0]);

    expect(await screen.findByLabelText("装备详情")).toBeInTheDocument();
    expect(client.getGearAtlasItem).toHaveBeenCalledWith(
      "atlas-public-1",
      "zh-CN",
    );
    expect(screen.getByText("我的尺寸")).toBeInTheDocument();
    expect(screen.getAllByText("M 75*195").length).toBeGreaterThan(0);
    expect(screen.getByText(/L 80\*205/)).toBeInTheDocument();
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
