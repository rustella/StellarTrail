import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {
  CreateGearRequest,
  GearCategory,
  GearCategoryFilter,
  GearItem,
  GearSort,
  GearStatsResponse,
  GearStatus,
  GearSummary,
  GearTab,
  MetaResponse,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

import { createWebGearApi, type WebGearApi } from "./api";
import {
  CATEGORY_OPTIONS,
  SORT_OPTIONS,
  STATUS_OPTIONS,
  categoryLabel,
  statusLabel,
} from "./gear-options";
import {
  formatCompactCurrency,
  formatCurrency,
  formatDate,
  formatWeight,
  fromPriceCents,
  joinGearName,
  toPriceCents,
} from "./formatters";
import {
  clearSession,
  loadSession,
  saveSession,
  type WebSession,
} from "./session";
import "./styles.css";

type GearCategoryFilterId = "all" | GearCategory;
type GearStatusFilter = "" | GearStatus;
type ViewMode = "table" | "cards";
type ThemeMode = "light" | "dark";
type FormMode = "create" | "edit";
type AuthMode = "wechat" | "password" | "register";

interface PasswordLoginState {
  account: string;
  password: string;
  captchaTicket: string;
  captchaAnswer: string;
}

interface RegisterFormState {
  username: string;
  email: string;
  password: string;
  confirmPassword: string;
  emailVerificationCode: string;
}

interface CaptchaState {
  ticket: string;
  imageSvg: string;
  debugAnswer?: string;
}

interface AppProps {
  client?: WebGearApi;
}

interface GearFormState {
  category: GearCategory;
  name: string;
  brand: string;
  model: string;
  color: string;
  material: string;
  capacity: string;
  size: string;
  description: string;
  weightG: string;
  warmthIndex: string;
  waterproofIndex: string;
  purchaseDate: string;
  purchasePrice: string;
  expiryOrWarrantyDate: string;
  purchaseLocation: string;
  status: GearStatus;
  storageLocation: string;
  tags: string;
  shareEnabled: boolean;
  notes: string;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  archived_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};

const emptyForm: GearFormState = {
  category: "backpack_system",
  name: "",
  brand: "",
  model: "",
  color: "",
  material: "",
  capacity: "",
  size: "",
  description: "",
  weightG: "",
  warmthIndex: "",
  waterproofIndex: "",
  purchaseDate: "",
  purchasePrice: "",
  expiryOrWarrantyDate: "",
  purchaseLocation: "",
  status: "available",
  storageLocation: "",
  tags: "",
  shareEnabled: false,
  notes: "",
};

const THEME_STORAGE_KEY = "stellartrail.web.theme";

const emptyPasswordLogin: PasswordLoginState = {
  account: "",
  password: "",
  captchaTicket: "",
  captchaAnswer: "",
};

const emptyRegisterForm: RegisterFormState = {
  username: "",
  email: "",
  password: "",
  confirmPassword: "",
  emailVerificationCode: "",
};

export default function App({ client }: AppProps) {
  const [api] = useState<WebGearApi>(() => client ?? createWebGearApi());
  const [session, setSession] = useState<WebSession | null>(() =>
    loadSession(),
  );
  const [meta, setMeta] = useState<MetaResponse | null>(null);
  const [tab, setTab] = useState<GearTab>("available");
  const [category, setCategory] = useState<GearCategoryFilterId>("all");
  const [status, setStatus] = useState<GearStatusFilter>("");
  const [sort, setSort] = useState<GearSort>("created_at_desc");
  const [query, setQuery] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("table");
  const [theme, setTheme] = useState<ThemeMode>(() => loadThemePreference());
  const [authMode, setAuthMode] = useState<AuthMode>("wechat");
  const [passwordLogin, setPasswordLogin] =
    useState<PasswordLoginState>(emptyPasswordLogin);
  const [registerForm, setRegisterForm] =
    useState<RegisterFormState>(emptyRegisterForm);
  const [captcha, setCaptcha] = useState<CaptchaState | null>(null);
  const [emailCodeNotice, setEmailCodeNotice] = useState<string | null>(null);
  const [categories, setCategories] = useState<GearCategoryFilter[]>([
    { id: "all", label: "全部装备", count: 0 },
  ]);
  const [stats, setStats] = useState<GearStatsResponse>(EMPTY_STATS);
  const [gears, setGears] = useState<GearSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loginCode, setLoginCode] = useState("web-local-user");
  const [formMode, setFormMode] = useState<FormMode>("create");
  const [formGearId, setFormGearId] = useState<string | null>(null);
  const [form, setForm] = useState<GearFormState>(emptyForm);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [detail, setDetail] = useState<GearItem | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    api.setAccessToken(session?.accessToken);
  }, [api, session?.accessToken]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  const listRequest = useMemo(
    () => ({
      tab,
      category: category === "all" ? undefined : category,
      status: status || undefined,
      q: query.trim() || undefined,
      sort,
      limit: 50,
    }),
    [category, query, sort, status, tab],
  );

  const loadDashboard = useCallback(async () => {
    if (!session) {
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const [metaResponse, categoryResponse, statsResponse, listResponse] =
        await Promise.all([
          api.meta(),
          api.listGearCategories(tab),
          api.getGearStats(tab),
          api.listGears(listRequest),
        ]);
      setMeta(metaResponse);
      setCategories(
        categoryResponse.items.length
          ? categoryResponse.items
          : [{ id: "all", label: "全部装备", count: 0 }],
      );
      setStats(statsResponse);
      setGears(listResponse.items);
    } catch (err) {
      const message = errorMessage(err);
      setError(message);
      if (message.includes("401")) {
        clearSession();
        setSession(null);
      }
    } finally {
      setLoading(false);
    }
  }, [api, listRequest, session, tab]);

  useEffect(() => {
    void loadDashboard();
  }, [loadDashboard]);

  function completeLogin(response: WechatLoginResponse) {
    const nextSession: WebSession = {
      accessToken: response.access_token,
      expiresAt: response.expires_at,
      user: response.user,
    };
    saveSession(nextSession);
    api.setAccessToken(response.access_token);
    setSession(nextSession);
  }

  function switchAuthMode(mode: AuthMode) {
    setAuthMode(mode);
    setError(null);
    setCaptcha(null);
    setEmailCodeNotice(null);
  }

  async function handleLogin(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const response = await api.loginWithWechatCode({
        code: loginCode.trim() || "web-local-user",
        profile: { nickname: "Web 本地用户", avatar_url: null },
      });
      completeLogin(response);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function handlePasswordLogin(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const account = passwordLogin.account.trim();
    if (!account || !passwordLogin.password) {
      setError("请填写用户名或邮箱和密码");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const response = await api.loginWithPassword({
        account,
        password: passwordLogin.password,
        captcha_ticket: passwordLogin.captchaTicket.trim() || undefined,
        captcha_answer: passwordLogin.captchaAnswer.trim() || undefined,
      });
      completeLogin(response);
    } catch (err) {
      const message = errorMessage(err);
      if (message.includes("428")) {
        try {
          await loadCaptcha(account);
          setError("多次登录失败，请输入验证码后重试");
        } catch (captchaErr) {
          setError(errorMessage(captchaErr));
        }
      } else {
        setError(message);
      }
    } finally {
      setSubmitting(false);
    }
  }

  async function handleSendEmailCode() {
    const email = registerForm.email.trim();
    if (!email) {
      setError("请先填写邮箱");
      return;
    }
    setSubmitting(true);
    setError(null);
    setEmailCodeNotice(null);
    try {
      const response = await api.sendEmailVerificationCode({ email });
      setEmailCodeNotice(
        response.debug_code
          ? `本地验证码：${response.debug_code}`
          : `验证码已发送至 ${response.email}`,
      );
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleRegister(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (registerForm.password !== registerForm.confirmPassword) {
      setError("两次输入的密码不一致");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const response = await api.register({
        username: registerForm.username.trim(),
        email: registerForm.email.trim(),
        password: registerForm.password,
        confirm_password: registerForm.confirmPassword,
        email_verification_code: registerForm.emailVerificationCode.trim(),
      });
      completeLogin(response);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function loadCaptcha(account: string) {
    const response = await api.createCaptcha({ account });
    setCaptcha({
      ticket: response.captcha_ticket,
      imageSvg: response.image_svg,
      debugAnswer: response.debug_answer,
    });
    setPasswordLogin((current) => ({
      ...current,
      captchaTicket: response.captcha_ticket,
      captchaAnswer: "",
    }));
  }

  async function handleRefreshCaptcha() {
    const account = passwordLogin.account.trim();
    if (!account) {
      setError("请先填写用户名或邮箱");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await loadCaptcha(account);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  function handleLogout() {
    clearSession();
    api.setAccessToken(undefined);
    setSession(null);
    setGears([]);
    setDetail(null);
  }

  function toggleTheme() {
    setTheme((current) => (current === "dark" ? "light" : "dark"));
  }

  function openCreateForm() {
    setFormMode("create");
    setFormGearId(null);
    setForm(emptyForm);
    setIsFormOpen(true);
  }

  async function openEditForm(id: string) {
    setSubmitting(true);
    setError(null);
    try {
      const item = await api.getGear(id);
      setFormMode("edit");
      setFormGearId(id);
      setForm(formFromGear(item));
      setIsFormOpen(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function openDetail(id: string) {
    setSubmitting(true);
    setError(null);
    try {
      setDetail(await api.getGear(id));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function submitForm(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const payload = formToPayload(form);
    if (!payload.name.trim()) {
      setError("请填写装备名称");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      if (formMode === "edit" && formGearId) {
        await api.updateGear(formGearId, payload);
      } else {
        await api.createGear(payload);
      }
      setIsFormOpen(false);
      await loadDashboard();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function archiveGear(id: string) {
    if (!window.confirm("确认将该装备移入历史装备吗？")) {
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await api.archiveGear(id);
      await loadDashboard();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function restoreGear(id: string) {
    setSubmitting(true);
    setError(null);
    try {
      await api.restoreGear(id);
      await loadDashboard();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function exportCsv() {
    setSubmitting(true);
    setError(null);
    try {
      const csv = await api.exportGearsCsv(tab);
      const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = `stellartrail-gears-${tab}.csv`;
      link.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function importJson(file: File | undefined) {
    if (!file) {
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const parsed = JSON.parse(await file.text()) as unknown;
      const items = Array.isArray(parsed)
        ? parsed
        : isRecord(parsed) && Array.isArray(parsed.items)
          ? parsed.items
          : [];
      const result = await api.importGears({
        dry_run: false,
        items: items as CreateGearRequest[],
      });
      setError(
        `导入完成：新增 ${result.created_count} 件，失败 ${result.failed_count} 件`,
      );
      await loadDashboard();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  }

  if (!session) {
    return (
      <main className="login-page">
        <section className="login-card">
          <div className="login-card-top">
            <p className="eyebrow">StellarTrail · 寻径星野</p>
            <ThemeToggle theme={theme} onToggle={toggleTheme} />
          </div>
          <h1>{authTitle(authMode)}</h1>
          <p className="muted">
            {authMode === "wechat"
              ? "使用微信 code 登录，或切换到账号密码登录 / 注册新账号。"
              : authMode === "password"
                ? "使用后端账号密码登录；多次失败时按提示输入图形验证码。"
                : "通过邮箱验证码创建账号，注册成功后会自动进入个人装备库。"}
          </p>
          <div className="auth-tabs" role="group" aria-label="登录方式">
            <button
              type="button"
              className={authMode === "wechat" ? "active" : ""}
              onClick={() => switchAuthMode("wechat")}
            >
              微信登录
            </button>
            <button
              type="button"
              className={authMode === "password" ? "active" : ""}
              onClick={() => switchAuthMode("password")}
            >
              账号登录
            </button>
            <button
              type="button"
              className={authMode === "register" ? "active" : ""}
              onClick={() => switchAuthMode("register")}
            >
              注册账号
            </button>
          </div>

          {authMode === "wechat" ? (
            <form className="auth-form" onSubmit={handleLogin}>
              <label htmlFor="login-code">Mock code</label>
              <input
                id="login-code"
                value={loginCode}
                onChange={(event) => setLoginCode(event.target.value)}
              />
              <button
                type="submit"
                className="primary-button"
                disabled={submitting}
              >
                {submitting ? "登录中…" : "进入装备库"}
              </button>
            </form>
          ) : null}

          {authMode === "password" ? (
            <form className="auth-form" onSubmit={handlePasswordLogin}>
              <label htmlFor="login-account">用户名或邮箱</label>
              <input
                id="login-account"
                value={passwordLogin.account}
                autoComplete="username"
                onChange={(event) =>
                  setPasswordLogin((current) => ({
                    ...current,
                    account: event.target.value,
                  }))
                }
              />
              <label htmlFor="login-password">密码</label>
              <input
                id="login-password"
                type="password"
                value={passwordLogin.password}
                autoComplete="current-password"
                onChange={(event) =>
                  setPasswordLogin((current) => ({
                    ...current,
                    password: event.target.value,
                  }))
                }
              />
              {captcha ? (
                <div className="captcha-panel">
                  <img
                    src={`data:image/svg+xml;utf8,${encodeURIComponent(captcha.imageSvg)}`}
                    alt="验证码"
                  />
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => void handleRefreshCaptcha()}
                    disabled={submitting}
                  >
                    换一张
                  </button>
                  {captcha.debugAnswer ? (
                    <p className="helper-text">
                      本地验证码答案：{captcha.debugAnswer}
                    </p>
                  ) : null}
                </div>
              ) : null}
              {captcha ? (
                <label htmlFor="login-captcha-answer">验证码</label>
              ) : null}
              {captcha ? (
                <input
                  id="login-captcha-answer"
                  value={passwordLogin.captchaAnswer}
                  onChange={(event) =>
                    setPasswordLogin((current) => ({
                      ...current,
                      captchaAnswer: event.target.value,
                    }))
                  }
                />
              ) : null}
              <button
                type="submit"
                className="primary-button"
                disabled={submitting}
              >
                {submitting ? "登录中…" : "账号密码登录"}
              </button>
            </form>
          ) : null}

          {authMode === "register" ? (
            <form className="auth-form" onSubmit={handleRegister}>
              <label htmlFor="register-username">用户名</label>
              <input
                id="register-username"
                value={registerForm.username}
                autoComplete="username"
                onChange={(event) =>
                  setRegisterForm((current) => ({
                    ...current,
                    username: event.target.value,
                  }))
                }
              />
              <label htmlFor="register-email">邮箱</label>
              <input
                id="register-email"
                type="email"
                value={registerForm.email}
                autoComplete="email"
                onChange={(event) =>
                  setRegisterForm((current) => ({
                    ...current,
                    email: event.target.value,
                  }))
                }
              />
              <label htmlFor="register-password">密码</label>
              <input
                id="register-password"
                type="password"
                value={registerForm.password}
                autoComplete="new-password"
                onChange={(event) =>
                  setRegisterForm((current) => ({
                    ...current,
                    password: event.target.value,
                  }))
                }
              />
              <label htmlFor="register-confirm-password">确认密码</label>
              <input
                id="register-confirm-password"
                type="password"
                value={registerForm.confirmPassword}
                autoComplete="new-password"
                onChange={(event) =>
                  setRegisterForm((current) => ({
                    ...current,
                    confirmPassword: event.target.value,
                  }))
                }
              />
              <button
                type="button"
                className="secondary-button"
                onClick={() => void handleSendEmailCode()}
                disabled={submitting}
              >
                发送邮箱验证码
              </button>
              {emailCodeNotice ? (
                <p className="helper-text">{emailCodeNotice}</p>
              ) : null}
              <label htmlFor="register-email-code">邮箱验证码</label>
              <input
                id="register-email-code"
                value={registerForm.emailVerificationCode}
                inputMode="numeric"
                onChange={(event) =>
                  setRegisterForm((current) => ({
                    ...current,
                    emailVerificationCode: event.target.value,
                  }))
                }
              />
              <button
                type="submit"
                className="primary-button"
                disabled={submitting}
              >
                {submitting ? "注册中…" : "注册并进入装备库"}
              </button>
            </form>
          ) : null}
          {error ? <p className="error-text">{error}</p> : null}
        </section>
      </main>
    );
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-block">
          <div className="brand-logo">星</div>
          <div>
            <strong>寻径星野</strong>
            <span>StellarTrail</span>
          </div>
        </div>
        <nav aria-label="主导航">
          <a className="active" href="#gear">
            装备库
          </a>
          <span>路线清单 · 待接入</span>
          <span>户外技能 · 待接入</span>
        </nav>
        <div
          className="sidebar-global-actions"
          role="group"
          aria-label="全局设置"
        >
          <ThemeToggle theme={theme} onToggle={toggleTheme} />
        </div>
        <div className="sidebar-footer">
          <span>{displayUserName(session)}</span>
          <button className="ghost-button" onClick={handleLogout}>
            退出
          </button>
        </div>
      </aside>

      <main className="dashboard" id="gear">
        <header className="page-header">
          <div>
            <p className="eyebrow">
              {meta ? `${meta.env} · ${meta.database_kind}` : "local · api"}
            </p>
            <h1>装备管理</h1>
            <p className="muted">
              管理您的户外装备库，追踪装备状态、重量和价值。
            </p>
          </div>
          <div className="toolbar">
            <button
              className={
                viewMode === "table" ? "segmented active" : "segmented"
              }
              onClick={() => setViewMode("table")}
            >
              表格视图
            </button>
            <button
              className={
                viewMode === "cards" ? "segmented active" : "segmented"
              }
              onClick={() => setViewMode("cards")}
            >
              卡片视图
            </button>
            <button
              className="secondary-button"
              onClick={() => void loadDashboard()}
              disabled={loading}
            >
              刷新
            </button>
            <button
              className="secondary-button"
              onClick={() => fileInputRef.current?.click()}
              disabled={submitting}
            >
              导入
            </button>
            <button
              className="secondary-button"
              onClick={() => void exportCsv()}
              disabled={submitting}
            >
              导出
            </button>
            <button className="primary-button" onClick={openCreateForm}>
              添加装备
            </button>
            <input
              ref={fileInputRef}
              className="hidden-input"
              type="file"
              accept="application/json,.json"
              onChange={(event) =>
                void importJson(event.currentTarget.files?.[0])
              }
            />
          </div>
        </header>

        {error ? (
          <div className="notice" role="status">
            {error}
          </div>
        ) : null}

        <section className="tabs" aria-label="装备状态分组">
          <button
            className={tab === "available" ? "active" : ""}
            onClick={() => setTab("available")}
          >
            可用装备
          </button>
          <button
            className={tab === "history" ? "active" : ""}
            onClick={() => setTab("history")}
          >
            历史装备
          </button>
        </section>

        <section className="stats-grid" aria-label="装备统计">
          <StatCard
            label="当前装备数量"
            value={`${stats.current_count} 件`}
            hint={`历史 ${stats.archived_count} 件`}
          />
          <StatCard
            label="装备价值"
            value={formatCurrency(stats.total_value_cents)}
            hint={formatCompactCurrency(stats.total_value_cents)}
          />
          <StatCard
            label="总重量"
            value={formatWeight(stats.total_weight_g)}
            hint="用于路线打包估算"
          />
        </section>

        <section className="filter-panel">
          <div className="category-chips" aria-label="分类筛选">
            {categories.map((item) => (
              <button
                key={item.id}
                className={category === item.id ? "chip active" : "chip"}
                onClick={() => setCategory(item.id)}
              >
                {item.label}
                <span>{item.count}</span>
              </button>
            ))}
          </div>
          <div className="filter-row">
            <input
              aria-label="搜索装备"
              placeholder="搜索装备名称、品牌、型号"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <select
              aria-label="状态筛选"
              value={status}
              onChange={(event) =>
                setStatus(event.target.value as GearStatusFilter)
              }
            >
              <option value="">全部状态</option>
              {STATUS_OPTIONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
            <select
              aria-label="排序"
              value={sort}
              onChange={(event) => setSort(event.target.value as GearSort)}
            >
              {SORT_OPTIONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </div>
        </section>

        <section className="content-card" aria-busy={loading}>
          {viewMode === "table" ? (
            <GearTable
              items={gears}
              tab={tab}
              onOpen={openDetail}
              onEdit={openEditForm}
              onArchive={archiveGear}
              onRestore={restoreGear}
            />
          ) : (
            <GearCards
              items={gears}
              tab={tab}
              onOpen={openDetail}
              onEdit={openEditForm}
              onArchive={archiveGear}
              onRestore={restoreGear}
            />
          )}
          {!loading && gears.length === 0 ? (
            <EmptyState onCreate={openCreateForm} />
          ) : null}
        </section>
      </main>

      {isFormOpen ? (
        <GearFormModal
          form={form}
          mode={formMode}
          submitting={submitting}
          onClose={() => setIsFormOpen(false)}
          onSubmit={submitForm}
          onChange={setForm}
        />
      ) : null}
      {detail ? (
        <GearDetailDrawer
          item={detail}
          onClose={() => setDetail(null)}
          onEdit={() => void openEditForm(detail.id)}
        />
      ) : null}
    </div>
  );
}

function StatCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint: string;
}) {
  return (
    <article className="stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{hint}</small>
    </article>
  );
}

function ThemeToggle({
  theme,
  onToggle,
}: {
  theme: ThemeMode;
  onToggle(): void;
}) {
  const isDark = theme === "dark";
  return (
    <button
      type="button"
      className="theme-toggle secondary-button"
      onClick={onToggle}
      aria-pressed={isDark}
      aria-label={isDark ? "切换到白天模式" : "切换到黑夜模式"}
    >
      <span aria-hidden="true">{isDark ? "☀️" : "🌙"}</span>
      {isDark ? "白天模式" : "黑夜模式"}
    </button>
  );
}

function GearTable({
  items,
  tab,
  onOpen,
  onEdit,
  onArchive,
  onRestore,
}: GearListProps) {
  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>装备名称</th>
            <th>分类</th>
            <th>状态</th>
            <th>重量</th>
            <th>价格</th>
            <th>购买日期</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {items.map((item) => (
            <tr key={item.id}>
              <td>
                <button
                  className="link-button"
                  onClick={() => void onOpen(item.id)}
                >
                  {joinGearName(item)}
                </button>
                <small>
                  {item.brand || item.model
                    ? [item.brand, item.model].filter(Boolean).join(" · ")
                    : "未填写品牌型号"}
                </small>
              </td>
              <td>{item.category_label || categoryLabel(item.category)}</td>
              <td>
                <span className={`status-pill status-${item.status}`}>
                  {item.status_label || statusLabel(item.status)}
                </span>
              </td>
              <td>{formatWeight(item.weight_g)}</td>
              <td>{formatCurrency(item.purchase_price_cents)}</td>
              <td>{formatDate(item.purchase_date)}</td>
              <td className="actions">
                <button onClick={() => void onOpen(item.id)}>查看</button>
                <button onClick={() => void onEdit(item.id)}>编辑</button>
                {tab === "history" ? (
                  <button onClick={() => void onRestore(item.id)}>恢复</button>
                ) : (
                  <button onClick={() => void onArchive(item.id)}>归档</button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

interface GearListProps {
  items: GearSummary[];
  tab: GearTab;
  onOpen(id: string): Promise<void> | void;
  onEdit(id: string): Promise<void> | void;
  onArchive(id: string): Promise<void> | void;
  onRestore(id: string): Promise<void> | void;
}

function GearCards({
  items,
  tab,
  onOpen,
  onEdit,
  onArchive,
  onRestore,
}: GearListProps) {
  return (
    <div className="gear-cards">
      {items.map((item) => (
        <article className="gear-card" key={item.id}>
          <div>
            <span className="category-text">
              {item.category_label || categoryLabel(item.category)}
            </span>
            <h3>{joinGearName(item)}</h3>
          </div>
          <span className={`status-pill status-${item.status}`}>
            {item.status_label || statusLabel(item.status)}
          </span>
          <dl>
            <div>
              <dt>重量</dt>
              <dd>{formatWeight(item.weight_g)}</dd>
            </div>
            <div>
              <dt>价格</dt>
              <dd>{formatCurrency(item.purchase_price_cents)}</dd>
            </div>
            <div>
              <dt>购买日期</dt>
              <dd>{formatDate(item.purchase_date)}</dd>
            </div>
          </dl>
          <div className="actions">
            <button onClick={() => void onOpen(item.id)}>查看</button>
            <button onClick={() => void onEdit(item.id)}>编辑</button>
            {tab === "history" ? (
              <button onClick={() => void onRestore(item.id)}>恢复</button>
            ) : (
              <button onClick={() => void onArchive(item.id)}>归档</button>
            )}
          </div>
        </article>
      ))}
    </div>
  );
}

function EmptyState({ onCreate }: { onCreate(): void }) {
  return (
    <div className="empty-state">
      <h2>还没有装备</h2>
      <p>先添加常用户外装备，后续可用于路线打包清单对比。</p>
      <button className="primary-button" onClick={onCreate}>
        添加第一件装备
      </button>
    </div>
  );
}

function GearFormModal({
  form,
  mode,
  submitting,
  onClose,
  onSubmit,
  onChange,
}: {
  form: GearFormState;
  mode: FormMode;
  submitting: boolean;
  onClose(): void;
  onSubmit(event: React.FormEvent<HTMLFormElement>): void;
  onChange(next: GearFormState): void;
}) {
  const update = <K extends keyof GearFormState>(
    key: K,
    value: GearFormState[K],
  ) => onChange({ ...form, [key]: value });
  return (
    <div className="modal-backdrop" role="presentation">
      <form
        className="gear-modal"
        onSubmit={onSubmit}
        aria-label={mode === "create" ? "添加装备" : "编辑装备"}
      >
        <header>
          <div>
            <h2>{mode === "create" ? "添加装备" : "编辑装备"}</h2>
            <p>填写装备基础信息、性能指标、购买信息和存放位置。</p>
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

        <fieldset>
          <legend>基本信息</legend>
          <label>
            分类 *
            <select
              value={form.category}
              onChange={(event) =>
                update("category", event.target.value as GearCategory)
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
          <label>
            颜色
            <input
              value={form.color}
              onChange={(event) => update("color", event.target.value)}
            />
          </label>
          <label>
            材质
            <input
              value={form.material}
              onChange={(event) => update("material", event.target.value)}
            />
          </label>
          <label className="full-width">
            装备描述
            <textarea
              value={form.description}
              onChange={(event) => update("description", event.target.value)}
              maxLength={100}
            />
          </label>
        </fieldset>

        <fieldset>
          <legend>性能指标</legend>
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
            容量
            <input
              value={form.capacity}
              onChange={(event) => update("capacity", event.target.value)}
            />
          </label>
          <label>
            尺寸
            <input
              value={form.size}
              onChange={(event) => update("size", event.target.value)}
            />
          </label>
          <label>
            保暖指数
            <input
              value={form.warmthIndex}
              onChange={(event) => update("warmthIndex", event.target.value)}
            />
          </label>
          <label>
            防水指数
            <input
              value={form.waterproofIndex}
              onChange={(event) =>
                update("waterproofIndex", event.target.value)
              }
            />
          </label>
        </fieldset>

        <fieldset>
          <legend>购买与状态</legend>
          <label>
            购买日期
            <input
              type="date"
              value={form.purchaseDate}
              onChange={(event) => update("purchaseDate", event.target.value)}
            />
          </label>
          <label>
            价格（元）
            <input
              type="number"
              min="0"
              step="0.01"
              value={form.purchasePrice}
              onChange={(event) => update("purchasePrice", event.target.value)}
            />
          </label>
          <label>
            过期/过保日期
            <input
              type="date"
              value={form.expiryOrWarrantyDate}
              onChange={(event) =>
                update("expiryOrWarrantyDate", event.target.value)
              }
            />
          </label>
          <label>
            购买地点
            <input
              value={form.purchaseLocation}
              onChange={(event) =>
                update("purchaseLocation", event.target.value)
              }
            />
          </label>
          <label>
            当前状态
            <select
              value={form.status}
              onChange={(event) =>
                update("status", event.target.value as GearStatus)
              }
            >
              {STATUS_OPTIONS.map((item) => (
                <option key={item.value} value={item.value}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>
          <label>
            存放位置
            <input
              value={form.storageLocation}
              onChange={(event) =>
                update("storageLocation", event.target.value)
              }
            />
          </label>
          <label className="full-width">
            标签
            <input
              placeholder="用英文逗号或中文逗号分隔"
              value={form.tags}
              onChange={(event) => update("tags", event.target.value)}
            />
          </label>
          <label className="checkbox full-width">
            <input
              type="checkbox"
              checked={form.shareEnabled}
              onChange={(event) => update("shareEnabled", event.target.checked)}
            />{" "}
            共享基础信息到公共装备库（不共享购买、存放和备注信息）
          </label>
          <label className="full-width">
            备注
            <textarea
              value={form.notes}
              onChange={(event) => update("notes", event.target.value)}
              maxLength={100}
            />
          </label>
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
            {submitting ? "保存中…" : "保存装备"}
          </button>
        </footer>
      </form>
    </div>
  );
}

function GearDetailDrawer({
  item,
  onClose,
  onEdit,
}: {
  item: GearItem;
  onClose(): void;
  onEdit(): void;
}) {
  return (
    <aside className="detail-drawer" aria-label="装备详情">
      <button className="icon-button" onClick={onClose} aria-label="关闭详情">
        ×
      </button>
      <p className="eyebrow">{categoryLabel(item.category)}</p>
      <h2>{joinGearName(item)}</h2>
      <span className={`status-pill status-${item.status}`}>
        {statusLabel(item.status)}
      </span>
      <dl>
        <div>
          <dt>重量</dt>
          <dd>{formatWeight(item.weight_g)}</dd>
        </div>
        <div>
          <dt>价格</dt>
          <dd>{formatCurrency(item.purchase_price_cents)}</dd>
        </div>
        <div>
          <dt>购买日期</dt>
          <dd>{formatDate(item.purchase_date)}</dd>
        </div>
        <div>
          <dt>存放位置</dt>
          <dd>{item.storage_location || "—"}</dd>
        </div>
        <div>
          <dt>容量/尺寸</dt>
          <dd>
            {[item.capacity, item.size].filter(Boolean).join(" · ") || "—"}
          </dd>
        </div>
        <div>
          <dt>标签</dt>
          <dd>{item.tags.length ? item.tags.join("、") : "—"}</dd>
        </div>
        <div>
          <dt>描述</dt>
          <dd>{item.description || "—"}</dd>
        </div>
        <div>
          <dt>备注</dt>
          <dd>{item.notes || "—"}</dd>
        </div>
      </dl>
      <button className="primary-button" onClick={onEdit}>
        编辑装备
      </button>
    </aside>
  );
}

function formToPayload(form: GearFormState): CreateGearRequest {
  return {
    category: form.category,
    name: form.name.trim(),
    brand: optional(form.brand),
    model: optional(form.model),
    color: optional(form.color),
    material: optional(form.material),
    capacity: optional(form.capacity),
    size: optional(form.size),
    description: optional(form.description),
    weight_g: optionalNumber(form.weightG),
    warmth_index: optional(form.warmthIndex),
    waterproof_index: optional(form.waterproofIndex),
    purchase_date: optional(form.purchaseDate),
    purchase_price_cents: toPriceCents(form.purchasePrice),
    expiry_or_warranty_date: optional(form.expiryOrWarrantyDate),
    purchase_location: optional(form.purchaseLocation),
    status: form.status,
    storage_location: optional(form.storageLocation),
    tags: form.tags
      .split(/[,，]/)
      .map((tag) => tag.trim())
      .filter(Boolean),
    share_enabled: form.shareEnabled,
    notes: optional(form.notes),
  };
}

function formFromGear(item: GearItem): GearFormState {
  return {
    category: item.category,
    name: item.name,
    brand: item.brand ?? "",
    model: item.model ?? "",
    color: item.color ?? "",
    material: item.material ?? "",
    capacity: item.capacity ?? "",
    size: item.size ?? "",
    description: item.description ?? "",
    weightG:
      item.weight_g === null || item.weight_g === undefined
        ? ""
        : String(item.weight_g),
    warmthIndex: item.warmth_index ?? "",
    waterproofIndex: item.waterproof_index ?? "",
    purchaseDate: item.purchase_date ?? "",
    purchasePrice: fromPriceCents(item.purchase_price_cents),
    expiryOrWarrantyDate: item.expiry_or_warranty_date ?? "",
    purchaseLocation: item.purchase_location ?? "",
    status: item.status,
    storageLocation: item.storage_location ?? "",
    tags: item.tags.join("，"),
    shareEnabled: item.share_enabled,
    notes: item.notes ?? "",
  };
}

function optional(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function optionalNumber(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
}

function authTitle(mode: AuthMode): string {
  if (mode === "password") {
    return "账号密码登录";
  }
  if (mode === "register") {
    return "注册账号";
  }
  return "本地开发登录";
}

function displayUserName(session: WebSession): string {
  return (
    session.user.nickname ??
    session.user.username ??
    session.user.email ??
    "本地用户"
  );
}

function loadThemePreference(): ThemeMode {
  return localStorage.getItem(THEME_STORAGE_KEY) === "dark" ? "dark" : "light";
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : "请求失败，请稍后重试";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
