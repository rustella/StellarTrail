import { useCallback, useEffect, useRef, useState } from "react";
import type {
  KnotDetail,
  KnotMediaAsset,
  KnotSummary,
  ListKnotsRequest,
  SkillCategorySummary,
} from "@stellartrail/shared-types";

import type { WebGearApi } from "./api";

const KNOTS_LOCALE = "zh-CN";
const KNOTS_PAGE_SIZE = 24;

type KnotsApi = Pick<
  WebGearApi,
  "listSkills" | "listKnots" | "getKnotDetail" | "resolveAssetUrl"
>;

interface KnotsPageProps {
  api: KnotsApi;
}

export default function KnotsPage({ api }: KnotsPageProps) {
  const [categorySummary, setCategorySummary] =
    useState<SkillCategorySummary | null>(null);
  const [knots, setKnots] = useState<KnotSummary[]>([]);
  const [query, setQuery] = useState("");
  const [activeQuery, setActiveQuery] = useState("");
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

  const loadKnots = useCallback(
    async ({
      offset,
      q,
      append,
    }: {
      offset: number;
      q: string;
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
      try {
        const request: ListKnotsRequest = {
          offset,
          limit: KNOTS_PAGE_SIZE,
        };
        const trimmedQuery = q.trim();
        if (trimmedQuery) {
          request.q = trimmedQuery;
        }
        const response = await api.listKnots(request, KNOTS_LOCALE);
        if (requestId !== listRequestRef.current) {
          return;
        }
        setKnots((current) =>
          append ? [...current, ...response.items] : response.items,
        );
        setNextOffset(response.page.next_offset ?? null);
        setActiveQuery(trimmedQuery);
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
    void loadKnots({ offset: 0, q: "", append: false });
  }, [loadKnots, loadOverview]);

  function handleSearch(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void loadKnots({ offset: 0, q: query, append: false });
  }

  function handleRetry() {
    void loadKnots({ offset: 0, q: activeQuery, append: false });
  }

  function handleLoadMore() {
    if (nextOffset === null || loadingMore) {
      return;
    }
    void loadKnots({ offset: nextOffset, q: activeQuery, append: true });
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
          <small>含用途分类、难度和步骤演示</small>
        </div>
      </header>

      <form className="filter-panel knots-search" onSubmit={handleSearch}>
        <label htmlFor="knots-query">搜索绳结</label>
        <div className="filter-row">
          <input
            id="knots-query"
            aria-label="搜索绳结"
            value={query}
            placeholder="按用途、名称或场景搜索"
            onChange={(event) => setQuery(event.target.value)}
          />
          <button className="primary-button" type="submit" disabled={loading}>
            搜索
          </button>
        </div>
      </form>

      {error ? (
        <section className="empty-state knots-state" role="status">
          <h2>{error}</h2>
          <p>可以稍后再试，或换一个关键词重新查找。</p>
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
              <p>换个用途、场景或名称试试。</p>
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
            <span>{difficultyLabel(item.difficulty)}</span>
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
  const media = primaryMedia(item.media);
  const mediaUrl = media ? resolveAssetUrl(media.url) : undefined;
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
        <span>{difficultyLabel(item.difficulty)}</span>
        {item.categories.map((category) => (
          <span key={category.id}>{category.title}</span>
        ))}
      </div>
      {media ? (
        <figure className="knot-detail-media">
          <img src={mediaUrl} alt={`${item.title} 演示素材`} />
          {media.attribution || media.license_note ? (
            <figcaption>
              {[media.attribution, media.license_note]
                .filter(Boolean)
                .join(" · ")}
            </figcaption>
          ) : null}
        </figure>
      ) : null}
      <section>
        <h3>用途说明</h3>
        <p>{item.description || item.summary}</p>
      </section>
      <section>
        <h3>练习步骤</h3>
        {item.steps.length ? (
          <ol className="knot-steps">
            {item.steps.map((step, index) => (
              <li key={`${index}-${step}`}>{step}</li>
            ))}
          </ol>
        ) : (
          <p className="muted">暂时还没有分步说明。</p>
        )}
      </section>
      {item.media.length ? (
        <section>
          <h3>演示素材</h3>
          <div className="knot-media-list">
            {item.media.map((asset) => (
              <a
                key={assetKey(asset)}
                href={resolveAssetUrl(asset.url)}
                target="_blank"
                rel="noreferrer"
              >
                {mediaLabel(asset)}
              </a>
            ))}
          </div>
        </section>
      ) : null}
    </aside>
  );
}

function primaryMedia(media: KnotMediaAsset[]): KnotMediaAsset | undefined {
  return media.find((asset) => asset.mime_type.startsWith("image/"));
}

function difficultyLabel(value?: string | null): string {
  switch (value) {
    case "leisure":
      return "入门";
    case "beginner":
      return "新手";
    case "intermediate":
      return "进阶";
    case "advanced":
      return "熟练";
    case "technical":
      return "技术";
    default:
      return "常用";
  }
}

function mediaLabel(asset: KnotMediaAsset): string {
  if (asset.id === "thumbnail") return "缩略图";
  if (asset.id === "preview") return "预览图";
  if (asset.id === "draw_gif") return "打法动图";
  if (asset.id === "turntable_gif") return "旋转动图";
  if (asset.id === "draw_mp4") return "打法视频";
  if (asset.id === "turntable_mp4") return "旋转视频";
  return "素材";
}

function assetKey(asset: KnotMediaAsset): string {
  return `${asset.id}-${asset.url}`;
}
