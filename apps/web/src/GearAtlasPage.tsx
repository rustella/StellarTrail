import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import type {
  AppLocale,
  CreateGearAtlasSubmissionRequest,
  GearAtlasPublicItem,
  GearAtlasSort,
  GearCategory,
  GearCurrency,
  GearSpecs,
  ListGearAtlasRequest,
} from "@stellartrail/shared-types";

import type { WebGearApi } from "./api";
import {
  combineSpecValue,
  getGearAtlasSpecFieldViews,
  normalizeSpecsForCategory,
  specLabel,
} from "./gear-atlas-utils";
import { CATEGORY_OPTIONS, categoryLabel } from "./gear-options";
import { formatDate, formatWeight, joinGearName } from "./formatters";
import type { WebSession } from "./session";

const ATLAS_LOCALE: AppLocale = "zh-CN";
const ATLAS_PAGE_SIZE = 20;

const ATLAS_SORT_OPTIONS: Array<{ value: GearAtlasSort; label: string }> = [
  { value: "approved_at_desc", label: "最近收录" },
  { value: "name_asc", label: "名称 A-Z" },
  { value: "weight_desc", label: "重量由高到低" },
  { value: "official_price_desc", label: "官方价由高到低" },
];

const CURRENCY_OPTIONS: GearCurrency[] = ["CNY", "USD", "EUR", "JPY", "HKD"];

type AtlasCategoryFilterId = "all" | GearCategory;

type GearAtlasApi = Pick<
  WebGearApi,
  "listGearAtlas" | "getGearAtlasItem" | "createGearAtlasSubmission"
>;

interface GearAtlasPageProps {
  api: GearAtlasApi;
  session: WebSession | null;
  initialDetailId?: string | null;
}

interface AtlasSubmissionFormState {
  category: GearCategory;
  name: string;
  brand: string;
  model: string;
  description: string;
  weightG: string;
  officialPrice: string;
  officialPriceCurrency: GearCurrency;
  specs: GearSpecs;
}

const emptyAtlasForm: AtlasSubmissionFormState = {
  category: "backpack_system",
  name: "",
  brand: "",
  model: "",
  description: "",
  weightG: "",
  officialPrice: "",
  officialPriceCurrency: "CNY",
  specs: {},
};

export default function GearAtlasPage({
  api,
  session,
  initialDetailId,
}: GearAtlasPageProps) {
  const [category, setCategory] = useState<AtlasCategoryFilterId>("all");
  const [query, setQuery] = useState("");
  const [activeQuery, setActiveQuery] = useState("");
  const [sort, setSort] = useState<GearAtlasSort>("approved_at_desc");
  const [items, setItems] = useState<GearAtlasPublicItem[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [detail, setDetail] = useState<GearAtlasPublicItem | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [isSubmitOpen, setIsSubmitOpen] = useState(false);
  const [submitForm, setSubmitForm] =
    useState<AtlasSubmissionFormState>(emptyAtlasForm);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const listRequestRef = useRef(0);
  const loadMoreInFlightRef = useRef(false);
  const detailRequestRef = useRef(0);

  const listRequest = useMemo(() => {
    const request: ListGearAtlasRequest = {
      sort,
      limit: ATLAS_PAGE_SIZE,
    };
    if (category !== "all") {
      request.category = category;
    }
    if (activeQuery) {
      request.q = activeQuery;
    }
    return request;
  }, [activeQuery, category, sort]);

  const loadItems = useCallback(
    async (
      options: { cursor?: string | null; append?: boolean } = {},
    ): Promise<void> => {
      const append = Boolean(options.append && options.cursor);
      if (append) {
        if (loadMoreInFlightRef.current) {
          return;
        }
        loadMoreInFlightRef.current = true;
        setLoadingMore(true);
      } else {
        listRequestRef.current += 1;
        setLoading(true);
        setNextCursor(null);
      }
      const requestId = listRequestRef.current;
      setError(null);
      try {
        const response = await api.listGearAtlas(
          {
            ...listRequest,
            cursor: options.cursor || undefined,
          },
          ATLAS_LOCALE,
        );
        if (requestId !== listRequestRef.current) {
          return;
        }
        setNextCursor(response.next_cursor ?? null);
        setItems((current) =>
          append ? mergeAtlasItems(current, response.items) : response.items,
        );
      } catch (err) {
        if (requestId !== listRequestRef.current) {
          return;
        }
        if (!append) {
          setItems([]);
          setNextCursor(null);
        }
        setError(errorMessage(err));
      } finally {
        if (append) {
          loadMoreInFlightRef.current = false;
          setLoadingMore(false);
        } else if (requestId === listRequestRef.current) {
          setLoading(false);
        }
      }
    },
    [api, listRequest],
  );

  const openDetail = useCallback(
    async (
      id: string,
      options: { updatePath?: boolean } = {},
    ): Promise<void> => {
      const requestId = detailRequestRef.current + 1;
      detailRequestRef.current = requestId;
      setDetailLoading(true);
      setDetailError(null);
      try {
        const response = await api.getGearAtlasItem(id, ATLAS_LOCALE);
        if (requestId !== detailRequestRef.current) {
          return;
        }
        setDetail(response);
        if (options.updatePath !== false) {
          const nextPath = `/gear-atlas/${encodeURIComponent(id)}`;
          if (window.location.pathname !== nextPath) {
            window.history.pushState(null, "", nextPath);
          }
        }
      } catch (err) {
        if (requestId === detailRequestRef.current) {
          setDetailError(errorMessage(err));
        }
      } finally {
        if (requestId === detailRequestRef.current) {
          setDetailLoading(false);
        }
      }
    },
    [api],
  );

  useEffect(() => {
    void loadItems();
  }, [loadItems]);

  useEffect(() => {
    if (initialDetailId) {
      void openDetail(initialDetailId, { updatePath: false });
      return;
    }
    detailRequestRef.current += 1;
    setDetail(null);
    setDetailError(null);
    setDetailLoading(false);
  }, [initialDetailId, openDetail]);

  function handleSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextQuery = query.trim();
    if (nextQuery === activeQuery) {
      void loadItems();
      return;
    }
    setActiveQuery(nextQuery);
  }

  function clearSearch() {
    setQuery("");
    setActiveQuery("");
  }

  function handleListScroll(event: React.UIEvent<HTMLElement>) {
    const list = event.currentTarget;
    const distanceToBottom =
      list.scrollHeight - list.scrollTop - list.clientHeight;
    if (distanceToBottom < 160 && nextCursor && !loading && !loadingMore) {
      void loadItems({ cursor: nextCursor, append: true });
    }
  }

  function closeDetail() {
    detailRequestRef.current += 1;
    setDetail(null);
    setDetailError(null);
    setDetailLoading(false);
    if (window.location.pathname.startsWith("/gear-atlas/")) {
      window.history.pushState(null, "", "/gear-atlas");
    }
  }

  function openSubmitForm() {
    if (!session) {
      setNotice("登录后可以把新装备投稿到图鉴审核。");
      return;
    }
    setSubmitError(null);
    setNotice(null);
    setIsSubmitOpen(true);
  }

  async function submitAtlasForm(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!session) {
      setNotice("登录后可以把新装备投稿到图鉴审核。");
      return;
    }
    let payload: CreateGearAtlasSubmissionRequest;
    try {
      payload = atlasFormToPayload(submitForm);
    } catch (err) {
      setSubmitError(errorMessage(err));
      return;
    }
    setSubmitting(true);
    setSubmitError(null);
    try {
      await api.createGearAtlasSubmission(payload);
      setSubmitForm(emptyAtlasForm);
      setIsSubmitOpen(false);
      setNotice("已提交审核，管理员通过后会进入公开图鉴。");
    } catch (err) {
      setSubmitError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  const specFields = useMemo(
    () => getGearAtlasSpecFieldViews(submitForm.category, submitForm.specs),
    [submitForm.category, submitForm.specs],
  );

  return (
    <section className="gear-atlas-page" aria-labelledby="gear-atlas-title">
      <header className="page-header gear-atlas-hero">
        <div>
          <p className="eyebrow">Gear Atlas</p>
          <h1 id="gear-atlas-title">装备图鉴</h1>
          <p className="muted">
            浏览已审核收录的市面装备，按分类、品牌和关键参数快速查找参考信息。
          </p>
        </div>
        <div className="toolbar">
          <button
            type="button"
            className="secondary-button"
            onClick={() => void loadItems()}
            disabled={loading}
          >
            刷新
          </button>
          <button
            type="button"
            className="primary-button"
            onClick={openSubmitForm}
          >
            投稿装备
          </button>
        </div>
      </header>

      {notice ? (
        <div className="notice" role="status">
          {notice}
        </div>
      ) : null}

      <section className="filter-panel gear-atlas-filter">
        <form className="gear-atlas-search" onSubmit={handleSearch}>
          <input
            aria-label="搜索图鉴装备"
            placeholder="搜索装备名、品牌、型号"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <button type="submit" className="primary-button">
            搜索
          </button>
          {query || activeQuery ? (
            <button
              type="button"
              className="secondary-button"
              onClick={clearSearch}
            >
              清除
            </button>
          ) : null}
        </form>
        <div className="filter-row">
          <select
            aria-label="图鉴分类"
            value={category}
            onChange={(event) =>
              setCategory(event.target.value as AtlasCategoryFilterId)
            }
          >
            <option value="all">全部分类</option>
            {CATEGORY_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </select>
          <select
            aria-label="图鉴排序"
            value={sort}
            onChange={(event) => setSort(event.target.value as GearAtlasSort)}
          >
            {ATLAS_SORT_OPTIONS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </select>
        </div>
      </section>

      {error ? (
        <section className="empty-state atlas-state" role="status">
          <h2>装备图鉴暂时没加载出来</h2>
          <p>{error}</p>
          <button
            type="button"
            className="primary-button"
            onClick={() => void loadItems()}
          >
            重试
          </button>
        </section>
      ) : null}

      {!error ? (
        <section
          className="gear-atlas-results"
          aria-label="装备图鉴列表"
          aria-busy={loading || loadingMore}
          onScroll={handleListScroll}
        >
          {loading ? <p className="muted">正在加载装备图鉴...</p> : null}
          {!loading && items.length === 0 ? (
            <div className="empty-state atlas-state">
              <h2>还没有收录装备</h2>
              <p>可以先提交一件装备，审核通过后会展示在这里。</p>
              <button
                type="button"
                className="primary-button"
                onClick={openSubmitForm}
              >
                投稿装备
              </button>
            </div>
          ) : null}
          <div className="gear-atlas-grid">
            {items.map((item) => (
              <AtlasCard key={item.id} item={item} onOpen={openDetail} />
            ))}
          </div>
          {items.length > 0 && nextCursor ? (
            <div className="load-more-row">
              <button
                type="button"
                className="secondary-button"
                disabled={loadingMore}
                onClick={() =>
                  void loadItems({ cursor: nextCursor, append: true })
                }
              >
                {loadingMore ? "加载中..." : "加载更多"}
              </button>
            </div>
          ) : null}
          {loadingMore ? (
            <p className="muted atlas-loading-more" role="status">
              正在加载更多装备...
            </p>
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
          正在打开图鉴详情...
        </div>
      ) : null}
      {detail ? (
        <AtlasDetailDrawer item={detail} onClose={closeDetail} />
      ) : null}
      {isSubmitOpen ? (
        <AtlasSubmitModal
          form={submitForm}
          specFields={specFields}
          submitting={submitting}
          error={submitError}
          onClose={() => setIsSubmitOpen(false)}
          onSubmit={submitAtlasForm}
          onChange={setSubmitForm}
        />
      ) : null}
    </section>
  );
}

function AtlasCard({
  item,
  onOpen,
}: {
  item: GearAtlasPublicItem;
  onOpen(id: string): Promise<void> | void;
}) {
  return (
    <article className="atlas-public-card">
      <button
        type="button"
        className="atlas-public-card-button"
        onClick={() => void onOpen(item.id)}
        aria-label={`查看${joinGearName(item)}`}
      >
        <span className="atlas-card-topline">
          <span className="category-text">
            {item.category_label || categoryLabel(item.category)}
          </span>
          <span className="status-pill">已收录</span>
        </span>
        <strong>{joinGearName(item)}</strong>
        <span className="atlas-card-meta">{brandModelText(item)}</span>
        <span className="atlas-card-desc">
          {item.description || "暂无描述"}
        </span>
        <span className="atlas-card-metrics">
          <span>
            <b>{formatWeight(item.weight_g)}</b>
            <small>重量</small>
          </span>
          <span>
            <b>
              {formatAtlasPrice(
                item.official_price_cents,
                item.official_price_currency,
              )}
            </b>
            <small>官方价</small>
          </span>
        </span>
        <span className="atlas-source-text">{sourceSummary(item)}</span>
      </button>
    </article>
  );
}

function AtlasDetailDrawer({
  item,
  onClose,
}: {
  item: GearAtlasPublicItem;
  onClose(): void;
}) {
  const specs = Object.entries(item.specs ?? {});
  return (
    <aside className="detail-drawer atlas-detail-drawer" aria-label="图鉴详情">
      <button
        className="icon-button"
        type="button"
        onClick={onClose}
        aria-label="关闭图鉴详情"
      >
        ×
      </button>
      <p className="eyebrow">
        {item.category_label || categoryLabel(item.category)}
      </p>
      <h2>{joinGearName(item)}</h2>
      <span className="status-pill">已收录</span>
      <section className="atlas-detail-section">
        <h3>公开信息</h3>
        <dl>
          <div>
            <dt>品牌型号</dt>
            <dd>{brandModelText(item)}</dd>
          </div>
          <div>
            <dt>描述</dt>
            <dd>{item.description || "—"}</dd>
          </div>
          <div>
            <dt>重量</dt>
            <dd>{formatWeight(item.weight_g)}</dd>
          </div>
          <div>
            <dt>官方价格</dt>
            <dd>
              {formatAtlasPrice(
                item.official_price_cents,
                item.official_price_currency,
              )}
            </dd>
          </div>
          <div>
            <dt>收录时间</dt>
            <dd>{formatDate(item.approved_at)}</dd>
          </div>
        </dl>
      </section>
      <section className="atlas-detail-section">
        <h3>分类参数</h3>
        {specs.length ? (
          <dl>
            {specs.map(([key, value]) => (
              <div key={key}>
                <dt>{specLabel(item.category, key)}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>
        ) : (
          <p className="muted">暂无分类参数。</p>
        )}
      </section>
      <section className="atlas-detail-section">
        <h3>来源摘要</h3>
        <dl>
          <div>
            <dt>来源</dt>
            <dd>{sourceSummary(item)}</dd>
          </div>
          {item.source_url ? (
            <div>
              <dt>来源链接</dt>
              <dd>
                <a href={item.source_url} target="_blank" rel="noreferrer">
                  打开来源
                </a>
              </dd>
            </div>
          ) : null}
          <div>
            <dt>更新时间</dt>
            <dd>{formatDate(item.updated_at)}</dd>
          </div>
        </dl>
      </section>
    </aside>
  );
}

function AtlasSubmitModal({
  form,
  specFields,
  submitting,
  error,
  onClose,
  onSubmit,
  onChange,
}: {
  form: AtlasSubmissionFormState;
  specFields: ReturnType<typeof getGearAtlasSpecFieldViews>;
  submitting: boolean;
  error: string | null;
  onClose(): void;
  onSubmit(event: FormEvent<HTMLFormElement>): void;
  onChange(next: AtlasSubmissionFormState): void;
}) {
  const update = <K extends keyof AtlasSubmissionFormState>(
    key: K,
    value: AtlasSubmissionFormState[K],
  ) => onChange({ ...form, [key]: value });
  const updateSpecValue = (key: string, value: string, unit?: string) => {
    onChange({
      ...form,
      specs: {
        ...form.specs,
        [key]: combineSpecValue(value, unit),
      },
    });
  };
  return (
    <div className="modal-backdrop" role="presentation">
      <form
        className="gear-modal atlas-submit-modal"
        onSubmit={onSubmit}
        aria-label="投稿装备图鉴"
      >
        <header>
          <div>
            <h2>投稿到装备图鉴</h2>
            <p>只填写适合公开展示的信息，审核通过后会出现在装备图鉴。</p>
          </div>
          <button
            type="button"
            className="icon-button"
            onClick={onClose}
            aria-label="关闭"
          >
            ×
          </button>
        </header>
        {error ? (
          <div className="notice" role="status">
            {error}
          </div>
        ) : null}
        <fieldset>
          <legend>基本信息</legend>
          <label>
            装备分类 *
            <select
              value={form.category}
              onChange={(event) =>
                onChange({
                  ...form,
                  category: event.target.value as GearCategory,
                  specs: {},
                })
              }
            >
              {CATEGORY_OPTIONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label>
            装备名称 *
            <input
              required
              value={form.name}
              onChange={(event) => update("name", event.target.value)}
              maxLength={80}
            />
          </label>
          <label>
            品牌
            <input
              value={form.brand}
              onChange={(event) => update("brand", event.target.value)}
            />
          </label>
          <label>
            型号
            <input
              value={form.model}
              onChange={(event) => update("model", event.target.value)}
            />
          </label>
          <label className="full-width">
            装备描述
            <textarea
              value={form.description}
              onChange={(event) => update("description", event.target.value)}
              maxLength={200}
            />
          </label>
        </fieldset>
        <fieldset>
          <legend>可公开信息</legend>
          <label>
            重量（g）
            <input
              type="number"
              min="0"
              value={form.weightG}
              onChange={(event) => update("weightG", event.target.value)}
            />
          </label>
          <label>
            官方价格
            <input
              type="number"
              min="0"
              step="0.01"
              value={form.officialPrice}
              onChange={(event) => update("officialPrice", event.target.value)}
            />
          </label>
          <label>
            价格币种
            <select
              value={form.officialPriceCurrency}
              onChange={(event) =>
                update(
                  "officialPriceCurrency",
                  event.target.value as GearCurrency,
                )
              }
            >
              {CURRENCY_OPTIONS.map((currency) => (
                <option key={currency} value={currency}>
                  {currency}
                </option>
              ))}
            </select>
          </label>
          {specFields.map((field) => (
            <div className="atlas-spec-input" key={field.key}>
              <label htmlFor={`atlas-spec-${field.key}`}>{field.label}</label>
              <div className="atlas-spec-input-row">
                {field.choiceOnly ? (
                  <span
                    className={
                      field.unitLabel ? "choice-value" : "choice-value muted"
                    }
                  >
                    {field.unitLabel || field.placeholder}
                  </span>
                ) : (
                  <input
                    id={`atlas-spec-${field.key}`}
                    type={field.inputType === "number" ? "number" : "text"}
                    value={field.valueText}
                    placeholder={field.placeholder}
                    onChange={(event) =>
                      updateSpecValue(
                        field.key,
                        event.target.value,
                        field.unitLabel,
                      )
                    }
                  />
                )}
                {field.units?.length ? (
                  <select
                    aria-label={`${field.label}单位`}
                    value={field.unitIndex}
                    onChange={(event) => {
                      const unitIndex = Number(event.target.value || 0);
                      const unit = field.units?.[unitIndex] ?? "";
                      updateSpecValue(
                        field.key,
                        field.choiceOnly ? "" : field.valueText,
                        unit,
                      );
                    }}
                  >
                    {field.unitLabels.map((label, index) => (
                      <option key={`${field.key}-${label}`} value={index}>
                        {label || "无单位"}
                      </option>
                    ))}
                  </select>
                ) : null}
              </div>
            </div>
          ))}
        </fieldset>
        <footer>
          <button type="button" className="secondary-button" onClick={onClose}>
            取消
          </button>
          <button
            type="submit"
            className="primary-button"
            disabled={submitting}
          >
            {submitting ? "提交中..." : "提交审核"}
          </button>
        </footer>
      </form>
    </div>
  );
}

function atlasFormToPayload(
  form: AtlasSubmissionFormState,
): CreateGearAtlasSubmissionRequest {
  const name = form.name.trim();
  if (!name) {
    throw new Error("装备名称不能为空");
  }
  return {
    category: form.category,
    name,
    brand: optionalText(form.brand),
    model: optionalText(form.model),
    description: optionalText(form.description),
    weight_g: optionalWholeNumber(form.weightG, "重量必须是非负整数"),
    official_price_cents: optionalPriceCents(form.officialPrice),
    official_price_currency: form.officialPrice.trim()
      ? form.officialPriceCurrency
      : null,
    specs: normalizeSpecsForCategory(form.category, form.specs),
  };
}

function mergeAtlasItems(
  current: GearAtlasPublicItem[],
  next: GearAtlasPublicItem[],
): GearAtlasPublicItem[] {
  const existingIds = new Set(current.map((item) => item.id));
  return [
    ...current,
    ...next.filter((item) => {
      if (existingIds.has(item.id)) {
        return false;
      }
      existingIds.add(item.id);
      return true;
    }),
  ];
}

function brandModelText(item: GearAtlasPublicItem): string {
  return (
    [item.brand, item.model].filter(Boolean).join(" · ") || "未填写品牌型号"
  );
}

function sourceSummary(item: GearAtlasPublicItem): string {
  const source =
    item.source_name || (item.source_url ? "外部来源" : "社区投稿");
  if (
    item.source_rating_score !== undefined &&
    item.source_rating_score !== null
  ) {
    const rating = `${item.source_rating_score.toFixed(1)} 分`;
    const count =
      item.source_rating_count !== undefined &&
      item.source_rating_count !== null
        ? ` / ${item.source_rating_count} 条`
        : "";
    return `${source} · ${rating}${count}`;
  }
  return source;
}

function formatAtlasPrice(
  cents?: number | null,
  currency?: string | null,
): string {
  if (cents === undefined || cents === null) {
    return "—";
  }
  const normalizedCurrency = currency || "CNY";
  const amount = cents / 100;
  if (normalizedCurrency === "CNY") {
    return `¥${amount.toLocaleString("zh-CN", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })}`;
  }
  return `${normalizedCurrency} ${amount.toLocaleString("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

function optionalText(value: string): string | null {
  const text = value.trim();
  return text ? text : null;
}

function optionalWholeNumber(value: string, message: string): number | null {
  const text = value.trim();
  if (!text) {
    return null;
  }
  const parsed = Number(text);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(message);
  }
  return parsed;
}

function optionalPriceCents(value: string): number | null {
  const text = value.trim();
  if (!text) {
    return null;
  }
  const parsed = Number(text);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error("官方价格必须是非负数字");
  }
  return Math.round(parsed * 100);
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : "请求失败，请稍后重试";
}
