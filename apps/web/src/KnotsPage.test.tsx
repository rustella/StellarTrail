import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import KnotsPage from "./KnotsPage";
import type { WebGearApi } from "./api";

const knotSummary = {
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

const secondKnotSummary = {
  ...knotSummary,
  id: "bowline-knot",
  slug: "dan-jie",
  title: "单结",
  summary: "快速形成固定绳圈。",
  difficulty: "leisure",
  categories: [
    { id: "basic-knots", slug: "ji-chu-sheng-jie", title: "基础绳结" },
  ],
  types: [{ id: "loop-knots", slug: "sheng-quan", title: "绳圈" }],
  media: [],
  href: "/api/skills/knots/detail/bowline-knot",
};

const knotDetail = {
  ...knotSummary,
  description: "适合风绳和营绳张力调节。",
  steps: ["将绳头绕过主绳。", "收紧后检查受力。"],
  locale: "zh-CN" as const,
};

type KnotsApi = Pick<WebGearApi, "listSkills" | "listKnots" | "getKnotDetail">;

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
          href: "/api/skills/knots/list",
        },
      ],
    }),
    listKnots: vi.fn().mockResolvedValue({
      locale: "zh-CN",
      items: [knotSummary],
      page: { offset: 0, limit: 24, next_offset: null },
    }),
    getKnotDetail: vi.fn().mockResolvedValue(knotDetail),
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
    expect(api.listKnots).toHaveBeenCalledWith(
      { offset: 0, limit: 24 },
      "zh-CN",
    );
    expect(screen.getByLabelText("绳结概览")).toHaveTextContent("绳结");
    expect(screen.getByText("可调节绳结")).toBeInTheDocument();
    expect(screen.getByText("露营绳结")).toBeInTheDocument();
    expect(screen.getByText("新手")).toBeInTheDocument();
    expect(screen.getByText("调节绳索上的张力。")).toBeInTheDocument();
    expect(screen.queryByText(/API|后端|接口|游客/)).not.toBeInTheDocument();
  });

  it("searches knots with the submitted keyword", async () => {
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

    fireEvent.change(screen.getByLabelText("搜索绳结"), {
      target: { value: "风绳" },
    });
    fireEvent.click(screen.getByRole("button", { name: "搜索" }));

    await waitFor(() => {
      expect(api.listKnots).toHaveBeenLastCalledWith(
        { offset: 0, limit: 24, q: "风绳" },
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

  it("opens a knot detail drawer with steps and media", async () => {
    const api = buildKnotsApi();

    render(<KnotsPage api={api} />);
    fireEvent.click(await screen.findByRole("button", { name: /可调节绳结/ }));

    expect(api.getKnotDetail).toHaveBeenCalledWith(
      "adjustable-grip-hitch-knot",
      "zh-CN",
    );
    expect(await screen.findByText("用途说明")).toBeInTheDocument();
    expect(screen.getByText("适合风绳和营绳张力调节。")).toBeInTheDocument();
    expect(screen.getByText("练习步骤")).toBeInTheDocument();
    expect(screen.getByText("将绳头绕过主绳。")).toBeInTheDocument();
    expect(screen.getByText("收紧后检查受力。")).toBeInTheDocument();
    expect(screen.getByText("演示素材")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "可调节绳结 演示素材" }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "关闭绳结详情" }));
    expect(screen.queryByText("用途说明")).not.toBeInTheDocument();
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

  it("shows an empty state when no knots match the search", async () => {
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

    fireEvent.change(screen.getByLabelText("搜索绳结"), {
      target: { value: "不存在" },
    });
    fireEvent.click(screen.getByRole("button", { name: "搜索" }));

    expect(await screen.findByText("没有找到相关绳结")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "加载更多" }),
    ).not.toBeInTheDocument();
  });
});
