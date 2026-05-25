import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import KnotsPage from "./KnotsPage";
import type { WebGearApi } from "./api";

const knotSummary = {
  id: "adjustable-grip-hitch-knot",
  slug: "ke-tiao-jie-sheng-jie",
  title: "可调节绳结",
  summary: "调节绳索上的张力。",
  aliases: ["可调节活结", "考利可调节套结"],
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

const secondKnotSummary = {
  ...knotSummary,
  id: "bowline-knot",
  slug: "dan-jie",
  title: "单结",
  summary: "快速形成固定绳圈。",
  aliases: ["布林结"],
  categories: [
    { id: "basic-knots", slug: "ji-chu-sheng-jie", title: "基础绳结" },
  ],
  types: [{ id: "loop-knots", slug: "sheng-quan", title: "绳圈" }],
  media: [],
  href: "/api/v1/skills/knots/detail/bowline-knot",
};

const knotDetail = {
  ...knotSummary,
  description: "适合风绳和营绳张力调节。",
  steps: ["将绳头绕过主绳。", "收紧后检查受力。"],
  locale: "zh-CN" as const,
};

const richKnotDetail = {
  ...knotDetail,
  media: [
    {
      id: "thumbnail",
      media_type: "thumbnail",
      url: "https://cdn.example.com/knots/thumb.webp",
      mime_type: "image/webp",
      width: 640,
      height: 360,
      size_bytes: 12345,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
    {
      id: "preview",
      media_type: "preview",
      url: "https://cdn.example.com/knots/preview-hd.webp",
      mime_type: "image/webp",
      width: 1600,
      height: 900,
      size_bytes: 56789,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
    {
      id: "draw_mp4",
      media_type: "draw_mp4",
      url: "https://cdn.example.com/knots/draw.mp4",
      mime_type: "video/mp4",
      width: 1280,
      height: 720,
      size_bytes: 34567,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
    {
      id: "draw_gif",
      media_type: "draw_gif",
      url: "https://cdn.example.com/knots/draw.gif",
      mime_type: "image/gif",
      width: 800,
      height: 450,
      size_bytes: 45678,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
    {
      id: "turntable_mp4",
      media_type: "turntable_mp4",
      url: "https://cdn.example.com/knots/turntable.mp4",
      mime_type: "video/mp4",
      width: 1280,
      height: 720,
      size_bytes: 45670,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
    {
      id: "turntable_gif",
      media_type: "turntable_gif",
      url: "https://cdn.example.com/knots/turntable.gif",
      mime_type: "image/gif",
      width: 800,
      height: 450,
      size_bytes: 56780,
      attribution: "Knots 3D",
      license_note: "Use only after authorization is confirmed.",
    },
  ],
};

const gifFallbackKnotDetail = {
  ...richKnotDetail,
  media: richKnotDetail.media.filter(
    (asset) => asset.id !== "draw_mp4" && asset.id !== "turntable_mp4",
  ),
};

const secondRichKnotDetail = {
  ...richKnotDetail,
  ...secondKnotSummary,
  description: "适合快速制作固定绳圈。",
  steps: ["绕出绳圈。", "穿回并收紧。"],
  locale: "zh-CN" as const,
  media: richKnotDetail.media.map((asset) => ({
    ...asset,
    url: asset.url.replace("/knots/", "/knots/second-"),
  })),
};

type KnotsApi = Pick<
  WebGearApi,
  | "listSkills"
  | "listKnotFilters"
  | "listKnots"
  | "getKnotDetail"
  | "resolveAssetUrl"
>;

function buildKnotsApi(): KnotsApi {
  return {
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
        {
          id: "basic-knots",
          slug: "ji-chu-sheng-jie",
          title: "基础绳结",
          count: 1,
        },
      ],
    }),
    listKnots: vi.fn().mockResolvedValue({
      locale: "zh-CN",
      items: [knotSummary],
      page: { offset: 0, limit: 24, next_offset: null },
    }),
    getKnotDetail: vi.fn().mockResolvedValue(knotDetail),
    resolveAssetUrl: vi.fn((pathOrUrl: string) =>
      pathOrUrl.startsWith("/")
        ? `https://assets.example.test${pathOrUrl}`
        : pathOrUrl,
    ),
  };
}

describe("KnotsPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders the outdoor skills knots list from public skill APIs", async () => {
    const api = buildKnotsApi();

    render(<KnotsPage api={api} />);

    expect(
      await screen.findByRole("heading", { name: "绳结" }),
    ).toBeInTheDocument();
    expect(api.listSkills).toHaveBeenCalledWith("zh-CN");
    expect(api.listKnotFilters).toHaveBeenCalledWith("zh-CN");
    expect(api.listKnots).toHaveBeenCalledWith(
      { offset: 0, limit: 24 },
      "zh-CN",
    );
    expect(screen.getByLabelText("绳结概览")).toHaveTextContent("绳结");
    expect(screen.getByText("可调节绳结")).toBeInTheDocument();
    expect(screen.getByText("露营绳结")).toBeInTheDocument();
    expect(screen.queryByText("新手")).not.toBeInTheDocument();
    expect(screen.getByText("调节绳索上的张力。")).toBeInTheDocument();
    expect(screen.queryByText(/API|后端|接口|游客/)).not.toBeInTheDocument();
  });

  it("renders only the usage category filter control", async () => {
    const api = buildKnotsApi();

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    const categorySelect = screen.getByLabelText("用途分类");
    expect(categorySelect).toHaveClass("knot-category-select");
    expect(categorySelect).toHaveDisplayValue("全部用途");
    expect(screen.getByText("按用途分类快速筛选")).toBeInTheDocument();
    expect(screen.queryByLabelText("搜索绳结")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("难度")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "搜索" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "清空筛选" }),
    ).not.toBeInTheDocument();
  });

  it("filters knots by usage category", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [secondKnotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("用途分类"), {
      target: { value: "basic-knots" },
    });

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 0, limit: 24, category: "basic-knots" },
        "zh-CN",
      );
    });
    expect(await screen.findByText("单结")).toBeInTheDocument();
    expect(screen.queryByText("可调节绳结")).not.toBeInTheDocument();
  });

  it("loads the next knots page and appends results", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: 24 },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [secondKnotSummary],
        page: { offset: 24, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "加载更多" }));

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 24, limit: 24 },
        "zh-CN",
      );
    });
    expect(await screen.findByText("单结")).toBeInTheDocument();
    expect(screen.getByText("可调节绳结")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "加载更多" }),
    ).not.toBeInTheDocument();
  });

  it("preserves the selected usage category when loading more", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [secondKnotSummary],
        page: { offset: 0, limit: 24, next_offset: 24 },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 24, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("用途分类"), {
      target: { value: "camping-knots" },
    });

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 0, limit: 24, category: "camping-knots" },
        "zh-CN",
      );
    });

    fireEvent.click(await screen.findByRole("button", { name: "加载更多" }));

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 24, limit: 24, category: "camping-knots" },
        "zh-CN",
      );
    });
  });

  it("clears the usage category by selecting all usages", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [secondKnotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    const categorySelect = screen.getByLabelText("用途分类");
    fireEvent.change(categorySelect, { target: { value: "basic-knots" } });

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 0, limit: 24, category: "basic-knots" },
        "zh-CN",
      );
    });

    fireEvent.change(categorySelect, { target: { value: "" } });

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 0, limit: 24 },
        "zh-CN",
      );
    });
    expect(categorySelect).toHaveValue("");
  });

  it("opens a knot detail drawer with steps and high-resolution media first", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.getKnotDetail).mockResolvedValue(richKnotDetail);

    render(<KnotsPage api={api} />);
    fireEvent.click(await screen.findByRole("button", { name: /可调节绳结/ }));

    expect(api.getKnotDetail).toHaveBeenCalledWith(
      "adjustable-grip-hitch-knot",
      "zh-CN",
    );
    expect(await screen.findByText("用途说明")).toBeInTheDocument();
    expect(screen.getByText("适合风绳和营绳张力调节。")).toBeInTheDocument();
    expect(screen.queryByText("练习步骤")).not.toBeInTheDocument();
    expect(screen.queryByText("将绳头绕过主绳。")).not.toBeInTheDocument();
    expect(screen.queryByText("收紧后检查受力。")).not.toBeInTheDocument();

    const defaultImage = screen.getByRole("img", {
      name: "可调节绳结 高清图",
    });
    expect(defaultImage.getAttribute("src")).toContain("preview-hd.webp");
    expect(defaultImage.getAttribute("src")).not.toContain("draw.gif");

    expect(screen.getByRole("button", { name: "高清图" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    fireEvent.click(screen.getByRole("button", { name: "打法动图" }));
    const drawMotion = screen.getByLabelText("可调节绳结 打法动图");
    expect(drawMotion.tagName).toBe("VIDEO");
    expect(drawMotion.getAttribute("src")).toContain("draw.mp4");
    expect(drawMotion.getAttribute("src")).not.toContain("draw.gif");

    fireEvent.click(screen.getByRole("button", { name: "旋转动图" }));
    const turntableMotion = screen.getByLabelText("可调节绳结 旋转动图");
    expect(turntableMotion.tagName).toBe("VIDEO");
    expect(turntableMotion.getAttribute("src")).toContain("turntable.mp4");
    expect(turntableMotion.getAttribute("src")).not.toContain("turntable.gif");

    expect(screen.queryByText("演示素材")).not.toBeInTheDocument();
    expect(screen.queryByText("Knots 3D")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Use only after authorization is confirmed."),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "关闭绳结详情" }));
    expect(screen.queryByText("用途说明")).not.toBeInTheDocument();
  });

  it("falls back to gif motion when mp4 assets are unavailable", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.getKnotDetail).mockResolvedValue(gifFallbackKnotDetail);

    render(<KnotsPage api={api} />);
    fireEvent.click(await screen.findByRole("button", { name: /可调节绳结/ }));

    const defaultImage = await screen.findByRole("img", {
      name: "可调节绳结 高清图",
    });
    expect(defaultImage.getAttribute("src")).toContain("preview-hd.webp");

    fireEvent.click(screen.getByRole("button", { name: "打法动图" }));
    const drawMotion = screen.getByRole("img", {
      name: "可调节绳结 打法动图",
    });
    expect(drawMotion.getAttribute("src")).toContain("draw.gif");

    fireEvent.click(screen.getByRole("button", { name: "旋转动图" }));
    const turntableMotion = screen.getByRole("img", {
      name: "可调节绳结 旋转动图",
    });
    expect(turntableMotion.getAttribute("src")).toContain("turntable.gif");
  });

  it("resets to the high-resolution image when opening another knot", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots).mockResolvedValue({
      locale: "zh-CN",
      items: [knotSummary, secondKnotSummary],
      page: { offset: 0, limit: 24, next_offset: null },
    });
    vi.mocked(api.getKnotDetail)
      .mockResolvedValueOnce(richKnotDetail)
      .mockResolvedValueOnce(secondRichKnotDetail);

    render(<KnotsPage api={api} />);
    fireEvent.click(await screen.findByRole("button", { name: /可调节绳结/ }));

    fireEvent.click(await screen.findByRole("button", { name: "打法动图" }));
    expect(screen.getByLabelText("可调节绳结 打法动图").tagName).toBe("VIDEO");

    fireEvent.click(screen.getByRole("button", { name: /单结/ }));

    const secondDefaultImage = await screen.findByRole("img", {
      name: "单结 高清图",
    });
    expect(secondDefaultImage.getAttribute("src")).toContain(
      "second-preview-hd.webp",
    );
    expect(screen.queryByLabelText("单结 打法动图")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "高清图" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("shows a retry action when knots fail to load", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockRejectedValueOnce(new Error("network failed"))
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);

    expect(
      await screen.findByText("绳结内容暂时没加载出来"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重试" }));

    await waitFor(() => expect(api.listKnots).toHaveBeenCalledTimes(2));
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();
  });

  it("shows an empty state when no knots match the usage category", async () => {
    const api = buildKnotsApi();
    vi.mocked(api.listKnots)
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [knotSummary],
        page: { offset: 0, limit: 24, next_offset: null },
      })
      .mockResolvedValueOnce({
        locale: "zh-CN",
        items: [],
        page: { offset: 0, limit: 24, next_offset: null },
      });

    render(<KnotsPage api={api} />);
    expect(await screen.findByText("可调节绳结")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("用途分类"), {
      target: { value: "basic-knots" },
    });

    expect(await screen.findByText("没有找到相关绳结")).toBeInTheDocument();
    expect(screen.getByText("换个用途分类试试。")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "加载更多" }),
    ).not.toBeInTheDocument();
  });
});
