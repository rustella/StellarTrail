import { useCallback, useEffect, useRef, useState } from "react";
import type {
  KnotDetail,
  KnotFilterOption,
  KnotFiltersResponse,
  KnotMediaAsset,
  KnotSummary,
  ListKnotsRequest,
  SkillCategorySummary,
} from "@stellartrail/shared-types";

import type { WebGearApi } from "./api";

const KNOTS_LOCALE = "zh-CN";
const KNOTS_PAGE_SIZE = 24;
const EMPTY_KNOT_FILTERS: KnotFiltersResponse = {
  locale: KNOTS_LOCALE,
  categories: [],
};

type KnotsApi = Pick<
  WebGearApi,
  | "listSkills"
  | "listKnotFilters"
  | "listKnots"
  | "getKnotDetail"
  | "resolveAssetUrl"
>;

interface ActiveKnotFilters {
  category: string;
}

interface KnotsPageProps {
  api: KnotsApi;
}

export default function KnotsPage({ api }: KnotsPageProps) {
  const [categorySummary, setCategorySummary] =
    useState<SkillCategorySummary | null>(null);
  const [knotFilters, setKnotFilters] =
    useState<KnotFiltersResponse>(EMPTY_KNOT_FILTERS);
  const [filtersLoading, setFiltersLoading] = useState(true);
  const [knots, setKnots] = useState<KnotSummary[]>([]);
  const [selectedCategory, setSelectedCategory] = useState("");
  const [activeFilters, setActiveFilters] = useState<ActiveKnotFilters>({
    category: "",
  });
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [detail, setDetail] = useState<KnotDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const listRequestRef = useRef(0);
  const detailRequestRef = useRef(0);

  const loadOverview = useCallback(async () => {
    try {
      const response = await api.listSkills(KNOTS_LOCALE);
      const summary = response.items.find(
        (item) => item.slug === "knots" || item.id === "knots",
      );
      setCategorySummary(summary ?? response.items[0] ?? null);
    } catch {
      setCategorySummary(null);
    }
  }, [api]);

  const loadKnotFilters = useCallback(async () => {
    setFiltersLoading(true);
    try {
      const response = await api.listKnotFilters(KNOTS_LOCALE);
      setKnotFilters(response);
    } catch {
      setKnotFilters(EMPTY_KNOT_FILTERS);
    } finally {
      setFiltersLoading(false);
    }
  }, [api]);

  const loadKnots = useCallback(
    async ({
      offset,
      category,
      append,
    }: {
      offset: number;
      category: string;
      append: boolean;
    }) => {
      const requestId = listRequestRef.current + 1;
      listRequestRef.current = requestId;
      if (append) {
        setLoadingMore(true);
      } else {
        setLoading(true);
        setError(null);
      }
      const normalizedCategory = category.trim();
      try {
        const request: ListKnotsRequest = {
          offset,
          limit: KNOTS_PAGE_SIZE,
        };
        if (normalizedCategory) {
          request.category = normalizedCategory;
        }
        const response = await api.listKnots(request, KNOTS_LOCALE);
        if (requestId !== listRequestRef.current) {
          return;
        }
        setKnots((current) =>
          append ? [...current, ...response.items] : response.items,
        );
        setNextOffset(response.page.next_offset ?? null);
        setActiveFilters({ category: normalizedCategory });
      } catch {
        if (requestId !== listRequestRef.current) {
          return;
        }
        if (!append) {
          setKnots([]);
          setNextOffset(null);
        }
        setError("绳结内容暂时没加载出来");
      } finally {
        if (requestId === listRequestRef.current) {
          setLoading(false);
          setLoadingMore(false);
        }
      }
    },
    [api],
  );

  useEffect(() => {
    void loadOverview();
    void loadKnotFilters();
    void loadKnots({
      offset: 0,
      category: "",
      append: false,
    });
  }, [loadKnots, loadKnotFilters, loadOverview]);

  function handleCategoryChange(event: React.ChangeEvent<HTMLSelectElement>) {
    const category = event.target.value;
    setSelectedCategory(category);
    void loadKnots({
      offset: 0,
      category,
      append: false,
    });
  }

  function handleRetry() {
    void loadKnots({ offset: 0, ...activeFilters, append: false });
  }

  function handleLoadMore() {
    if (nextOffset === null || loadingMore) {
      return;
    }
    void loadKnots({ offset: nextOffset, ...activeFilters, append: true });
  }

  async function openDetail(id: string) {
    const requestId = detailRequestRef.current + 1;
    detailRequestRef.current = requestId;
    setDetailLoading(true);
    setDetailError(null);
    try {
      const response = await api.getKnotDetail(id, KNOTS_LOCALE);
      if (requestId !== detailRequestRef.current) {
        return;
      }
      setDetail(response);
    } catch {
      if (requestId === detailRequestRef.current) {
        setDetailError("绳结详情暂时没加载出来");
      }
    } finally {
      if (requestId === detailRequestRef.current) {
        setDetailLoading(false);
      }
    }
  }

  function closeDetail() {
    detailRequestRef.current += 1;
    setDetail(null);
    setDetailError(null);
    setDetailLoading(false);
  }

  const totalText = categorySummary
    ? `${categorySummary.item_count} 个绳结`
    : `${knots.length} 个绳结`;

  return (
    <section className="knots-page" aria-labelledby="knots-title">
      <header className="page-header knots-hero">
        <div>
          <p className="eyebrow">Outdoor Skills</p>
          <h1 id="knots-title">绳结</h1>
          <p className="muted">
            {categorySummary?.summary ??
              "学习露营、登山和日常户外场景常用绳结，按用途快速找到练习方法。"}
          </p>
        </div>
        <div className="knots-summary-card" aria-label="绳结概览">
          <span>{categorySummary?.title ?? "绳结"}</span>
          <strong>{totalText}</strong>
          <small>含用途分类、类型和步骤演示</small>
        </div>
      </header>

      <div className="filter-panel knots-filter-panel" aria-label="筛选绳结">
        <div className="knot-filter-copy">
          <span>按用途分类快速筛选</span>
          <strong>选择绳结用途</strong>
          <p>按露营、基础等用途查看绳结。</p>
        </div>
        <label
          className="knot-category-select-card"
          htmlFor="knots-category-filter"
        >
          <span>用途分类</span>
          <select
            id="knots-category-filter"
            className="knot-category-select"
            aria-label="用途分类"
            value={selectedCategory}
            onChange={handleCategoryChange}
            disabled={filtersLoading}
          >
            <option value="">全部用途</option>
            {knotFilters.categories.map((option) => (
              <option key={option.id} value={option.id}>
                {filterOptionLabel(option)}
              </option>
            ))}
          </select>
        </label>
      </div>

      {error ? (
        <section className="empty-state knots-state" role="status">
          <h2>{error}</h2>
          <p>可以稍后再试，或换一个用途分类重新查找。</p>
          <button
            className="primary-button"
            type="button"
            onClick={handleRetry}
          >
            重试
          </button>
        </section>
      ) : null}

      {!error ? (
        <section className="content-card knots-content" aria-busy={loading}>
          {loading ? <p className="muted">正在整理绳结内容…</p> : null}
          {!loading && knots.length === 0 ? (
            <div className="empty-state knots-state">
              <h2>没有找到相关绳结</h2>
              <p>换个用途分类试试。</p>
            </div>
          ) : null}
          <div className="knot-grid">
            {knots.map((item) => (
              <KnotCard
                key={item.id}
                item={item}
                onOpen={openDetail}
                resolveAssetUrl={(url) => api.resolveAssetUrl(url)}
              />
            ))}
          </div>
          {nextOffset !== null ? (
            <div className="load-more-row">
              <button
                className="secondary-button"
                type="button"
                onClick={handleLoadMore}
                disabled={loadingMore}
              >
                {loadingMore ? "加载中…" : "加载更多"}
              </button>
            </div>
          ) : null}
        </section>
      ) : null}

      {detailError ? (
        <div className="notice" role="status">
          {detailError}
        </div>
      ) : null}
      {detailLoading ? (
        <div className="notice" role="status">
          正在打开绳结详情…
        </div>
      ) : null}
      {detail ? (
        <KnotDetailDrawer
          item={detail}
          onClose={closeDetail}
          resolveAssetUrl={(url) => api.resolveAssetUrl(url)}
        />
      ) : null}
    </section>
  );
}

function filterOptionLabel(option: KnotFilterOption): string {
  return `${option.title}（${option.count}）`;
}

function KnotCard({
  item,
  onOpen,
  resolveAssetUrl,
}: {
  item: KnotSummary;
  onOpen(id: string): Promise<void> | void;
  resolveAssetUrl(url: string): string;
}) {
  const media = primaryMedia(item.media);
  const mediaUrl = media ? resolveAssetUrl(media.url) : undefined;
  return (
    <article className="knot-card">
      <button
        type="button"
        className="knot-card-button"
        onClick={() => void onOpen(item.id)}
        aria-label={`查看${item.title}`}
      >
        {media ? (
          <img src={mediaUrl} alt="" loading="lazy" />
        ) : (
          <span className="knot-media-placeholder" aria-hidden="true">
            ⛓
          </span>
        )}
        <span className="knot-card-body">
          <span className="knot-card-meta">
            {item.categories[0] ? (
              <span>{item.categories[0].title}</span>
            ) : null}
          </span>
          <strong>{item.title}</strong>
          <span>{item.summary}</span>
          <span className="knot-tags">
            {item.types.slice(0, 2).map((type) => (
              <span key={type.id}>{type.title}</span>
            ))}
          </span>
        </span>
      </button>
    </article>
  );
}

function KnotDetailDrawer({
  item,
  onClose,
  resolveAssetUrl,
}: {
  item: KnotDetail;
  onClose(): void;
  resolveAssetUrl(url: string): string;
}) {
  const mediaOptions = buildDetailMediaOptions(item.media);
  const defaultMediaKey = mediaOptions[0]?.key ?? "";
  const [selectedMediaKey, setSelectedMediaKey] = useState(defaultMediaKey);

  useEffect(() => {
    setSelectedMediaKey(defaultMediaKey);
  }, [defaultMediaKey, item.id]);

  const selectedMedia =
    mediaOptions.find((option) => option.key === selectedMediaKey) ??
    mediaOptions[0];
  const selectedMediaUrl = selectedMedia
    ? resolveAssetUrl(selectedMedia.asset.url)
    : undefined;
  const posterMedia = mediaOptions.find((option) => option.key === "photo");
  const posterUrl = posterMedia
    ? resolveAssetUrl(posterMedia.asset.url)
    : undefined;

  return (
    <aside className="detail-drawer knot-detail-drawer" aria-label="绳结详情">
      <button
        className="icon-button"
        type="button"
        onClick={onClose}
        aria-label="关闭绳结详情"
      >
        ×
      </button>
      <p className="eyebrow">绳结练习</p>
      <h2>{item.title}</h2>
      <div className="knot-card-meta">
        {item.categories.map((category) => (
          <span key={category.id}>{category.title}</span>
        ))}
      </div>
      {selectedMedia && selectedMediaUrl ? (
        <figure className="knot-detail-media">
          <div
            className="knot-media-switcher"
            role="group"
            aria-label="选择查看方式"
          >
            {mediaOptions.map((option) => (
              <button
                key={option.key}
                type="button"
                aria-pressed={option.key === selectedMedia.key}
                onClick={() => setSelectedMediaKey(option.key)}
              >
                {option.label}
              </button>
            ))}
          </div>
          {selectedMedia.kind === "video" ? (
            <video
              aria-label={`${item.title} ${selectedMedia.label}`}
              src={selectedMediaUrl}
              poster={posterUrl}
              controls
              loop
              muted
              playsInline
              preload="metadata"
            />
          ) : (
            <img
              src={selectedMediaUrl}
              alt={`${item.title} ${selectedMedia.label}`}
            />
          )}
        </figure>
      ) : null}
      <section>
        <h3>用途说明</h3>
        <p>{item.description || item.summary}</p>
      </section>
    </aside>
  );
}

function primaryMedia(media: KnotMediaAsset[]): KnotMediaAsset | undefined {
  return (
    selectPreferredMedia(media, ["thumbnail", "preview"]) ?? staticImage(media)
  );
}

type DetailMediaOption = {
  key: "photo" | "draw" | "turntable";
  label: string;
  kind: "image" | "video";
  asset: KnotMediaAsset;
};

function buildDetailMediaOptions(media: KnotMediaAsset[]): DetailMediaOption[] {
  const options: DetailMediaOption[] = [];
  const photo =
    selectPreferredMedia(media, ["preview", "thumbnail"]) ?? staticImage(media);
  if (photo) {
    options.push({
      key: "photo",
      label: "高清图",
      kind: "image",
      asset: photo,
    });
  }

  const draw = selectPreferredMedia(media, ["draw_mp4", "draw_gif"]);
  if (draw) {
    options.push({
      key: "draw",
      label: "打法动图",
      kind: mediaKind(draw),
      asset: draw,
    });
  }

  const turntable = selectPreferredMedia(media, [
    "turntable_mp4",
    "turntable_gif",
  ]);
  if (turntable) {
    options.push({
      key: "turntable",
      label: "旋转动图",
      kind: mediaKind(turntable),
      asset: turntable,
    });
  }

  return options;
}

function selectPreferredMedia(
  media: KnotMediaAsset[],
  orderedIds: string[],
): KnotMediaAsset | undefined {
  for (const id of orderedIds) {
    const match = media.find(
      (asset) => asset.id === id || asset.media_type === id,
    );
    if (match) return match;
  }
  return undefined;
}

function staticImage(media: KnotMediaAsset[]): KnotMediaAsset | undefined {
  return media.find(
    (asset) =>
      asset.mime_type.startsWith("image/") && asset.mime_type !== "image/gif",
  );
}

function mediaKind(asset: KnotMediaAsset): "image" | "video" {
  return asset.mime_type.startsWith("video/") || asset.id.endsWith("_mp4")
    ? "video"
    : "image";
}
