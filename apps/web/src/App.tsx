import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StellarTrailApiError } from "@stellartrail/api-client";
import type {
  AdminFeedbackItem,
  ClientKey,
  ClientVersion,
  ClientVersionReleaseNoteSection,
  ClientVersionRequest,
  ClientVersionStatus,
  CreateGearRequest,
  DeletedFilter,
  GearAtlasPublicItem,
  GearCategory,
  GearCategoryFilter,
  GearCurrency,
  GearSpecs,
  GearAtlasStatus,
  GearAtlasSubmission,
  GearItem,
  GearSort,
  GearStatsResponse,
  GearStatus,
  GearSummary,
  GearTab,
  GearVariant,
  UpdateGearAtlasSubmissionRequest,
  WechatLoginResponse,
} from "@stellartrail/shared-types";

import { createWebGearApi, type WebGearApi } from "./api";
import GearAtlasPage from "./GearAtlasPage";
import {
  combineSpecValue,
  getGearAtlasSpecFieldViews,
  normalizeSpecsForCategory,
  normalizeVariants,
  variantKeyFromLabel,
  variantSummary,
} from "./gear-atlas-utils";
import KnotsPage from "./KnotsPage";
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
type AuthMode = "wechat" | "password" | "email" | "reset" | "register";
type ActivePage =
  | "gear"
  | "gearAtlas"
  | "atlasReview"
  | "adminFeedback"
  | "clientVersions"
  | "knots";
type FeedbackStatusFilter = "" | "open";
type ClientVersionStatusFilter = "" | ClientVersionStatus;

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

interface EmailCodeLoginState {
  email: string;
  emailVerificationCode: string;
}

interface ClientVersionFormState {
  id: string | null;
  clientKey: ClientKey;
  version: string;
  title: string;
  featureNotesText: string;
  bugFixNotesText: string;
  notesText: string;
  status: ClientVersionStatus;
}

interface PasswordResetFormState {
  email: string;
  emailVerificationCode: string;
  password: string;
  confirmPassword: string;
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
  atlasItemId: string;
  selectedVariantKey: string;
  selectedVariantLabel: string;
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
  notes: string;
}

interface AtlasReviewFormState {
  category: GearCategory;
  name: string;
  brand: string;
  model: string;
  description: string;
  weightG: string;
  officialPrice: string;
  officialPriceCurrency: GearCurrency;
  variants: GearVariant[];
  specs: GearSpecs;
}

const EMPTY_STATS: GearStatsResponse = {
  current_count: 0,
  archived_count: 0,
  total_value_cents: 0,
  total_weight_g: 0,
  by_category: [],
  by_status: [],
};

const EMPTY_CATEGORIES: GearCategoryFilter[] = [
  { id: "all", label: "全部装备", count: 0 },
];

const CURRENCY_OPTIONS: GearCurrency[] = ["CNY", "USD", "EUR", "JPY", "HKD"];

const emptyForm: GearFormState = {
  category: "backpack_system",
  name: "",
  brand: "",
  model: "",
  color: "",
  material: "",
  capacity: "",
  atlasItemId: "",
  selectedVariantKey: "",
  selectedVariantLabel: "",
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
  notes: "",
};

const THEME_STORAGE_KEY = "stellartrail.web.theme";
const ATLAS_REVIEW_PAGE_SIZE = 20;

const CLIENT_VERSION_CLIENT_OPTIONS: Array<{ id: ClientKey; label: string }> = [
  { id: "wechat_miniprogram", label: "微信小程序" },
  { id: "web", label: "Web App" },
  { id: "android", label: "Android" },
  { id: "ios", label: "iOS" },
  { id: "macos", label: "macOS" },
];

const CLIENT_VERSION_STATUS_OPTIONS: Array<{
  id: ClientVersionStatus;
  label: string;
}> = [
  { id: "draft", label: "草稿" },
  { id: "published", label: "已发布" },
];

const emptyClientVersionForm: ClientVersionFormState = {
  id: null,
  clientKey: "wechat_miniprogram",
  version: "",
  title: "",
  featureNotesText: "",
  bugFixNotesText: "",
  notesText: "",
  status: "draft",
};

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

const emptyEmailCodeLogin: EmailCodeLoginState = {
  email: "",
  emailVerificationCode: "",
};

const emptyPasswordResetForm: PasswordResetFormState = {
  email: "",
  emailVerificationCode: "",
  password: "",
  confirmPassword: "",
};

export default function App({ client }: AppProps) {
  const [api] = useState<WebGearApi>(() => client ?? createWebGearApi());
  const [session, setSession] = useState<WebSession | null>(() =>
    loadSession(),
  );
  const [tab, setTab] = useState<GearTab>("available");
  const [category, setCategory] = useState<GearCategoryFilterId>("all");
  const [status, setStatus] = useState<GearStatusFilter>("");
  const [sort, setSort] = useState<GearSort>("created_at_desc");
  const [query, setQuery] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("table");
  const [theme, setTheme] = useState<ThemeMode>(() => loadThemePreference());
  const [authMode, setAuthMode] = useState<AuthMode>("wechat");
  const [activePage, setActivePage] = useState<ActivePage>(() =>
    activePageFromPath(window.location.pathname),
  );
  const [outdoorSkillsOpen, setOutdoorSkillsOpen] = useState(true);
  const [adminNavOpen, setAdminNavOpen] = useState(() =>
    isAdminPage(activePageFromPath(window.location.pathname)),
  );
  const [passwordLogin, setPasswordLogin] =
    useState<PasswordLoginState>(emptyPasswordLogin);
  const [registerForm, setRegisterForm] =
    useState<RegisterFormState>(emptyRegisterForm);
  const [emailLoginForm, setEmailLoginForm] =
    useState<EmailCodeLoginState>(emptyEmailCodeLogin);
  const [passwordResetForm, setPasswordResetForm] =
    useState<PasswordResetFormState>(emptyPasswordResetForm);
  const [captcha, setCaptcha] = useState<CaptchaState | null>(null);
  const [emailCodeNotice, setEmailCodeNotice] = useState<string | null>(null);
  const [categories, setCategories] =
    useState<GearCategoryFilter[]>(EMPTY_CATEGORIES);
  const [stats, setStats] = useState<GearStatsResponse>(EMPTY_STATS);
  const [gears, setGears] = useState<GearSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loginCode, setLoginCode] = useState("web-local-user");
  const [formMode, setFormMode] = useState<FormMode>("create");
  const [formGearId, setFormGearId] = useState<string | null>(null);
  const [form, setForm] = useState<GearFormState>(emptyForm);
  const [formAtlasItem, setFormAtlasItem] =
    useState<GearAtlasPublicItem | null>(null);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [detail, setDetail] = useState<GearItem | null>(null);
  const [detailAtlasItem, setDetailAtlasItem] =
    useState<GearAtlasPublicItem | null>(null);
  const [detailAtlasSubmission, setDetailAtlasSubmission] =
    useState<GearAtlasSubmission | null>(null);
  const [atlasStatus, setAtlasStatus] = useState<"" | GearAtlasStatus>(
    "pending",
  );
  const [atlasDeleted, setAtlasDeleted] = useState<DeletedFilter>("active");
  const [atlasSubmissions, setAtlasSubmissions] = useState<
    GearAtlasSubmission[]
  >([]);
  const [atlasDetail, setAtlasDetail] = useState<GearAtlasSubmission | null>(
    null,
  );
  const [atlasNextCursor, setAtlasNextCursor] = useState<string | null>(null);
  const [atlasLoading, setAtlasLoading] = useState(false);
  const [atlasLoadingMore, setAtlasLoadingMore] = useState(false);
  const [atlasError, setAtlasError] = useState<string | null>(null);
  const [feedbackStatus, setFeedbackStatus] =
    useState<FeedbackStatusFilter>("open");
  const [feedbackDeleted, setFeedbackDeleted] =
    useState<DeletedFilter>("active");
  const [adminFeedback, setAdminFeedback] = useState<AdminFeedbackItem[]>([]);
  const [feedbackLoading, setFeedbackLoading] = useState(false);
  const [feedbackError, setFeedbackError] = useState<string | null>(null);
  const [clientVersionClientKey, setClientVersionClientKey] =
    useState<ClientKey>("wechat_miniprogram");
  const [clientVersionStatus, setClientVersionStatus] =
    useState<ClientVersionStatusFilter>("");
  const [adminClientVersions, setAdminClientVersions] = useState<
    ClientVersion[]
  >([]);
  const [clientVersionsLoading, setClientVersionsLoading] = useState(false);
  const [clientVersionsError, setClientVersionsError] = useState<string | null>(
    null,
  );
  const [clientVersionForm, setClientVersionForm] =
    useState<ClientVersionFormState>(emptyClientVersionForm);
  const [clientVersionSubmitting, setClientVersionSubmitting] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const dashboardRequestRef = useRef(0);
  const atlasRequestRef = useRef(0);
  const atlasLoadMoreInFlightRef = useRef(false);

  useEffect(() => {
    api.setSessionTokens(session?.accessToken, session?.refreshToken);
  }, [api, session?.accessToken, session?.refreshToken]);

  useEffect(() => {
    api.setSessionRefreshHandler((response) => {
      const nextSession = sessionFromLoginResponse(response);
      saveSession(nextSession);
      setSession(nextSession);
    });
    return () => api.setSessionRefreshHandler(undefined);
  }, [api]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  useEffect(() => {
    const handlePopState = () => {
      const nextPage = activePageFromPath(window.location.pathname);
      setActivePage(nextPage);
      if (isAdminPage(nextPage)) {
        setAdminNavOpen(true);
      }
    };
    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

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
    const requestId = dashboardRequestRef.current + 1;
    dashboardRequestRef.current = requestId;
    setLoading(true);
    setError(null);
    try {
      const [categoryResponse, statsResponse, listResponse] = await Promise.all(
        [
          api.listGearCategories(tab),
          api.getGearStats(tab),
          api.listGears(listRequest),
        ],
      );
      if (requestId !== dashboardRequestRef.current) {
        return;
      }
      setCategories(
        categoryResponse.items.length
          ? categoryResponse.items
          : EMPTY_CATEGORIES,
      );
      setStats(statsResponse);
      setGears(listResponse.items);
    } catch (err) {
      if (requestId !== dashboardRequestRef.current) {
        return;
      }
      const message = errorMessage(err);
      setError(message);
      if (message.includes("401")) {
        clearSession();
        setSession(null);
      }
    } finally {
      if (requestId === dashboardRequestRef.current) {
        setLoading(false);
      }
    }
  }, [api, listRequest, session, tab]);

  useEffect(() => {
    void loadDashboard();
  }, [loadDashboard]);

  const loadAtlasSubmissions = useCallback(
    async (
      options: { cursor?: string | null; append?: boolean } = {},
    ): Promise<void> => {
      if (!session || activePage !== "atlasReview") {
        return;
      }
      const append = Boolean(options.append && options.cursor);
      if (append) {
        if (atlasLoadMoreInFlightRef.current) {
          return;
        }
        atlasLoadMoreInFlightRef.current = true;
        setAtlasLoadingMore(true);
      } else {
        atlasRequestRef.current += 1;
        setAtlasLoading(true);
        setAtlasNextCursor(null);
      }
      const requestId = atlasRequestRef.current;
      setAtlasError(null);
      try {
        const response = await api.listAdminGearAtlasSubmissions({
          status: atlasStatus || undefined,
          deleted: atlasDeleted,
          limit: ATLAS_REVIEW_PAGE_SIZE,
          cursor: options.cursor || undefined,
        });
        if (requestId !== atlasRequestRef.current) {
          return;
        }
        setAtlasNextCursor(response.next_cursor ?? null);
        if (append) {
          setAtlasSubmissions((current) =>
            mergeAtlasSubmissionPages(current, response.items),
          );
          setAtlasDetail((current) =>
            current
              ? (response.items.find((item) => item.id === current.id) ??
                current)
              : null,
          );
        } else {
          setAtlasSubmissions(response.items);
          setAtlasDetail((current) =>
            current
              ? (response.items.find((item) => item.id === current.id) ??
                response.items[0] ??
                null)
              : (response.items[0] ?? null),
          );
        }
      } catch (err) {
        if (requestId !== atlasRequestRef.current) {
          return;
        }
        if (!append) {
          setAtlasSubmissions([]);
          setAtlasDetail(null);
          setAtlasNextCursor(null);
        }
        setAtlasError(errorMessage(err));
      } finally {
        if (append) {
          atlasLoadMoreInFlightRef.current = false;
          setAtlasLoadingMore(false);
        } else if (requestId === atlasRequestRef.current) {
          setAtlasLoading(false);
        }
      }
    },
    [activePage, api, atlasDeleted, atlasStatus, session],
  );

  useEffect(() => {
    void loadAtlasSubmissions();
  }, [loadAtlasSubmissions]);

  const loadAdminFeedback = useCallback(async () => {
    if (!session || activePage !== "adminFeedback") {
      return;
    }
    setFeedbackLoading(true);
    setFeedbackError(null);
    try {
      const response = await api.listAdminFeedback({
        status: feedbackStatus || undefined,
        deleted: feedbackDeleted,
        limit: 50,
      });
      setAdminFeedback(response.items);
    } catch (err) {
      setAdminFeedback([]);
      setFeedbackError(errorMessage(err));
    } finally {
      setFeedbackLoading(false);
    }
  }, [activePage, api, feedbackDeleted, feedbackStatus, session]);

  useEffect(() => {
    void loadAdminFeedback();
  }, [loadAdminFeedback]);

  const loadAdminClientVersions = useCallback(async () => {
    if (!session || activePage !== "clientVersions") {
      return;
    }
    setClientVersionsLoading(true);
    setClientVersionsError(null);
    try {
      const response = await api.listAdminClientVersions({
        client_key: clientVersionClientKey,
        status: clientVersionStatus || undefined,
        limit: 50,
      });
      setAdminClientVersions(response.items);
    } catch (err) {
      setAdminClientVersions([]);
      setClientVersionsError(errorMessage(err));
    } finally {
      setClientVersionsLoading(false);
    }
  }, [activePage, api, clientVersionClientKey, clientVersionStatus, session]);

  useEffect(() => {
    void loadAdminClientVersions();
  }, [loadAdminClientVersions]);

  function resetDashboardState() {
    dashboardRequestRef.current += 1;
    setCategory("all");
    setStatus("");
    setQuery("");
    setCategories(EMPTY_CATEGORIES);
    setStats(EMPTY_STATS);
    setGears([]);
    setLoading(false);
    setDetail(null);
    setDetailAtlasItem(null);
    setDetailAtlasSubmission(null);
  }

  function completeLogin(response: WechatLoginResponse) {
    resetDashboardState();
    const nextSession = sessionFromLoginResponse(response);
    saveSession(nextSession);
    api.setSessionTokens(nextSession.accessToken, nextSession.refreshToken);
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
      if (isApiStatus(err, 428)) {
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

  async function handleSendEmailLoginCode() {
    const email = emailLoginForm.email.trim();
    if (!email) {
      setError("请先填写邮箱");
      return;
    }
    setSubmitting(true);
    setError(null);
    setEmailCodeNotice(null);
    try {
      const response = await api.sendEmailLoginCode({ email });
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

  async function handleEmailCodeLogin(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const email = emailLoginForm.email.trim();
    const emailVerificationCode = emailLoginForm.emailVerificationCode.trim();
    if (!email || !emailVerificationCode) {
      setError("请填写邮箱和验证码");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const response = await api.loginWithEmailCode({
        email,
        email_verification_code: emailVerificationCode,
      });
      completeLogin(response);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleSendPasswordResetCode() {
    const email = passwordResetForm.email.trim();
    if (!email) {
      setError("请先填写邮箱");
      return;
    }
    setSubmitting(true);
    setError(null);
    setEmailCodeNotice(null);
    try {
      const response = await api.sendPasswordResetCode({ email });
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

  async function handlePasswordReset(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (passwordResetForm.password !== passwordResetForm.confirmPassword) {
      setError("两次输入的密码不一致");
      return;
    }
    const email = passwordResetForm.email.trim();
    const emailVerificationCode =
      passwordResetForm.emailVerificationCode.trim();
    if (!email || !emailVerificationCode) {
      setError("请填写邮箱和验证码");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const response = await api.resetPassword({
        email,
        email_verification_code: emailVerificationCode,
        password: passwordResetForm.password,
        confirm_password: passwordResetForm.confirmPassword,
      });
      completeLogin(response);
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
    const username = registerForm.username.trim();
    const email = registerForm.email.trim();
    const emailVerificationCode = registerForm.emailVerificationCode.trim();
    setSubmitting(true);
    setError(null);
    try {
      const registerResponse = await api.register({
        username,
        email,
        password: registerForm.password,
        confirm_password: registerForm.confirmPassword,
        email_verification_code: emailVerificationCode,
      });
      completeLogin(registerResponse);
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
    api.setSessionTokens(undefined, undefined);
    setSession(null);
    resetDashboardState();
  }

  function navigateToLogin() {
    setActivePage("gear");
    if (window.location.pathname !== "/") {
      window.history.pushState(null, "", "/");
    }
  }

  function toggleTheme() {
    setTheme((current) => (current === "dark" ? "light" : "dark"));
  }

  function navigateToPage(page: ActivePage) {
    setActivePage(page);
    const nextPath = pathForActivePage(page);
    if (window.location.pathname !== nextPath) {
      window.history.pushState(null, "", nextPath);
    }
  }

  function openCreateForm() {
    setFormMode("create");
    setFormGearId(null);
    setForm(emptyForm);
    setFormAtlasItem(null);
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
      setFormAtlasItem(null);
      if (item.atlas_item_id) {
        try {
          setFormAtlasItem(
            await api.getGearAtlasItem(item.atlas_item_id, "zh-CN"),
          );
        } catch {
          setFormAtlasItem(null);
        }
      }
      setIsFormOpen(true);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  function startNewClientVersion() {
    setClientVersionForm({
      ...emptyClientVersionForm,
      clientKey: clientVersionClientKey,
    });
  }

  function editClientVersion(item: ClientVersion) {
    const sections = clientVersionSections(item);
    setClientVersionForm({
      id: item.id,
      clientKey: item.client_key,
      version: item.version,
      title: item.title,
      featureNotesText: sectionItemsText(sections, "feature"),
      bugFixNotesText: sectionItemsText(sections, "bug_fix"),
      notesText: sectionItemsText(sections, "notes"),
      status: item.status,
    });
  }

  function cancelClientVersionEdit() {
    startNewClientVersion();
  }

  async function saveClientVersion() {
    setClientVersionSubmitting(true);
    setClientVersionsError(null);
    try {
      const request = clientVersionRequestFromForm(clientVersionForm);
      if (clientVersionForm.id) {
        await api.updateAdminClientVersion(clientVersionForm.id, request);
      } else {
        await api.createAdminClientVersion(request);
      }
      startNewClientVersion();
      await loadAdminClientVersions();
    } catch (err) {
      setClientVersionsError(errorMessage(err));
    } finally {
      setClientVersionSubmitting(false);
    }
  }

  async function openDetail(id: string) {
    setSubmitting(true);
    setError(null);
    try {
      const item = await api.getGear(id);
      setDetail(item);
      setDetailAtlasItem(null);
      setDetailAtlasSubmission(null);
      if (item.atlas_item_id) {
        try {
          setDetailAtlasItem(
            await api.getGearAtlasItem(item.atlas_item_id, "zh-CN"),
          );
        } catch {
          setDetailAtlasItem(null);
        }
      }
      try {
        const submissions = await api.listMyGearAtlasSubmissions({
          limit: 100,
        });
        setDetailAtlasSubmission(
          submissions.items.find(
            (submission) => submission.source_user_gear_id === item.id,
          ) ?? null,
        );
      } catch {
        setDetailAtlasSubmission(null);
      }
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
      setFormAtlasItem(null);
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

  async function openAtlasSubmission(id: string) {
    setSubmitting(true);
    setAtlasError(null);
    try {
      setAtlasDetail(await api.getAdminGearAtlasSubmission(id));
    } catch (err) {
      setAtlasError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function approveAtlasSubmission(id: string) {
    setSubmitting(true);
    setAtlasError(null);
    try {
      const updated = await api.approveGearAtlasSubmission(id);
      setAtlasDetail(updated);
      await loadAtlasSubmissions();
    } catch (err) {
      setAtlasError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function updateAtlasSubmission(
    id: string,
    request: UpdateGearAtlasSubmissionRequest,
  ) {
    setSubmitting(true);
    setAtlasError(null);
    try {
      const updated = await api.updateAdminGearAtlasSubmission(id, request);
      setAtlasDetail(updated);
      await loadAtlasSubmissions();
    } catch (err) {
      setAtlasError(errorMessage(err));
      throw err;
    } finally {
      setSubmitting(false);
    }
  }

  async function rejectAtlasSubmission(id: string, reason: string) {
    setSubmitting(true);
    setAtlasError(null);
    try {
      const updated = await api.rejectGearAtlasSubmission(id, {
        reason: reason.trim(),
      });
      setAtlasDetail(updated);
      await loadAtlasSubmissions();
    } catch (err) {
      setAtlasError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function deleteAtlasSubmission(id: string) {
    if (!window.confirm("确认隐藏这条图鉴记录吗？")) {
      return;
    }
    setSubmitting(true);
    setAtlasError(null);
    try {
      await api.deleteAdminGearAtlasSubmission(id);
      setAtlasDetail(null);
      await loadAtlasSubmissions();
    } catch (err) {
      setAtlasError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function restoreAtlasSubmission(id: string) {
    setSubmitting(true);
    setAtlasError(null);
    try {
      const restored = await api.restoreAdminGearAtlasSubmission(id);
      setAtlasDetail(restored);
      await loadAtlasSubmissions();
    } catch (err) {
      setAtlasError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function deleteAdminFeedback(id: string) {
    if (!window.confirm("确认隐藏这条反馈吗？")) {
      return;
    }
    setFeedbackLoading(true);
    setFeedbackError(null);
    try {
      await api.deleteAdminFeedback(id);
      await loadAdminFeedback();
    } catch (err) {
      setFeedbackError(errorMessage(err));
    } finally {
      setFeedbackLoading(false);
    }
  }

  async function restoreAdminFeedback(id: string) {
    setFeedbackLoading(true);
    setFeedbackError(null);
    try {
      await api.restoreAdminFeedback(id);
      await loadAdminFeedback();
    } catch (err) {
      setFeedbackError(errorMessage(err));
    } finally {
      setFeedbackLoading(false);
    }
  }

  async function submitGearToAtlas(id: string) {
    if (!session) {
      setError("登录后可以把个人装备投稿到图鉴审核。");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const submission = await api.createGearAtlasSubmissionFromGear(id);
      setDetailAtlasSubmission(submission);
      setError("已提交图鉴审核");
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  if (!session && activePage !== "gearAtlas") {
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
              ? "使用微信 code 登录，或切换到其他登录方式。"
              : authMode === "password"
                ? "使用账号密码登录；忘记密码时可以用邮箱重新设置。"
                : authMode === "email"
                  ? "收取邮箱验证码，不输入密码也能进入个人装备库。"
                  : authMode === "reset"
                    ? "通过邮箱验证码确认身份，再设置新密码。"
                    : "通过邮箱验证码创建账号，注册成功后会自动进入个人装备库。"}
          </p>
          {authMode !== "register" ? (
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
                className={authMode === "email" ? "active" : ""}
                onClick={() => switchAuthMode("email")}
              >
                邮箱验证码
              </button>
            </div>
          ) : null}

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
                {submitting ? "登录中…" : "登录"}
              </button>
              <div className="auth-secondary-action">
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("email")}
                >
                  用邮箱验证码登录
                </button>
                <span aria-hidden="true"> · </span>
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("reset")}
                >
                  忘记密码
                </button>
                <span aria-hidden="true"> · </span>
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("register")}
                >
                  注册账号
                </button>
              </div>
            </form>
          ) : null}

          {authMode === "email" ? (
            <form className="auth-form" onSubmit={handleEmailCodeLogin}>
              <label htmlFor="email-login-email">邮箱</label>
              <input
                id="email-login-email"
                type="email"
                value={emailLoginForm.email}
                autoComplete="email"
                onChange={(event) =>
                  setEmailLoginForm((current) => ({
                    ...current,
                    email: event.target.value,
                  }))
                }
              />
              <button
                type="button"
                className="secondary-button"
                onClick={() => void handleSendEmailLoginCode()}
                disabled={submitting}
              >
                获取邮箱验证码
              </button>
              {emailCodeNotice ? (
                <p className="helper-text">{emailCodeNotice}</p>
              ) : null}
              <label htmlFor="email-login-code">邮箱验证码</label>
              <input
                id="email-login-code"
                value={emailLoginForm.emailVerificationCode}
                inputMode="numeric"
                onChange={(event) =>
                  setEmailLoginForm((current) => ({
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
                {submitting ? "登录中…" : "邮箱验证码登录"}
              </button>
              <div className="auth-secondary-action">
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("password")}
                >
                  返回账号登录
                </button>
              </div>
            </form>
          ) : null}

          {authMode === "reset" ? (
            <form className="auth-form" onSubmit={handlePasswordReset}>
              <label htmlFor="reset-email">邮箱</label>
              <input
                id="reset-email"
                type="email"
                value={passwordResetForm.email}
                autoComplete="email"
                onChange={(event) =>
                  setPasswordResetForm((current) => ({
                    ...current,
                    email: event.target.value,
                  }))
                }
              />
              <button
                type="button"
                className="secondary-button"
                onClick={() => void handleSendPasswordResetCode()}
                disabled={submitting}
              >
                获取邮箱验证码
              </button>
              {emailCodeNotice ? (
                <p className="helper-text">{emailCodeNotice}</p>
              ) : null}
              <label htmlFor="reset-email-code">邮箱验证码</label>
              <input
                id="reset-email-code"
                value={passwordResetForm.emailVerificationCode}
                inputMode="numeric"
                onChange={(event) =>
                  setPasswordResetForm((current) => ({
                    ...current,
                    emailVerificationCode: event.target.value,
                  }))
                }
              />
              <label htmlFor="reset-password">新密码</label>
              <input
                id="reset-password"
                type="password"
                value={passwordResetForm.password}
                autoComplete="new-password"
                onChange={(event) =>
                  setPasswordResetForm((current) => ({
                    ...current,
                    password: event.target.value,
                  }))
                }
              />
              <label htmlFor="reset-confirm-password">确认新密码</label>
              <input
                id="reset-confirm-password"
                type="password"
                value={passwordResetForm.confirmPassword}
                autoComplete="new-password"
                onChange={(event) =>
                  setPasswordResetForm((current) => ({
                    ...current,
                    confirmPassword: event.target.value,
                  }))
                }
              />
              <button
                type="submit"
                className="primary-button"
                disabled={submitting}
              >
                {submitting ? "提交中…" : "重设密码并登录"}
              </button>
              <div className="auth-secondary-action">
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("password")}
                >
                  返回账号登录
                </button>
              </div>
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
                {submitting ? "注册中…" : "注册并登录"}
              </button>
              <div className="auth-secondary-action">
                <button
                  type="button"
                  className="auth-link-button"
                  onClick={() => switchAuthMode("password")}
                >
                  已有账号？返回登录
                </button>
              </div>
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
          <div className="brand-logo">
            <img
              className="brand-logo-image"
              src="/app-icon.png"
              alt="寻径星野产品图标"
            />
          </div>
          <div className="brand-wordmark" aria-label="寻径星野 StellarTrail">
            <strong className="brand-wordmark-cn">寻径星野</strong>
            <span className="brand-wordmark-en">StellarTrail</span>
          </div>
        </div>
        <nav aria-label="主导航">
          <button
            type="button"
            className={activePage === "gear" ? "active" : ""}
            aria-current={activePage === "gear" ? "page" : undefined}
            onClick={() => navigateToPage("gear")}
          >
            装备库
          </button>
          <button
            type="button"
            className={activePage === "gearAtlas" ? "active" : ""}
            aria-current={activePage === "gearAtlas" ? "page" : undefined}
            onClick={() => navigateToPage("gearAtlas")}
          >
            装备图鉴
          </button>
          <div
            className="nav-group"
            data-active-parent={activePage === "knots" ? "true" : undefined}
          >
            <button
              type="button"
              className="nav-group-trigger"
              aria-expanded={outdoorSkillsOpen}
              aria-controls="outdoor-skills-nav"
              onClick={() => setOutdoorSkillsOpen((open) => !open)}
            >
              户外技能
            </button>
            {outdoorSkillsOpen ? (
              <div className="nav-children" id="outdoor-skills-nav">
                <button
                  type="button"
                  className={
                    activePage === "knots" ? "nav-child active" : "nav-child"
                  }
                  aria-current={activePage === "knots" ? "page" : undefined}
                  onClick={() => navigateToPage("knots")}
                >
                  绳结
                </button>
              </div>
            ) : null}
          </div>
          <span>路线清单 · 待接入</span>
        </nav>
        <nav className="sidebar-admin-nav" aria-label="管理员导航">
          <div
            className="nav-group"
            data-active-parent={
              activePage === "atlasReview" ||
              activePage === "adminFeedback" ||
              activePage === "clientVersions"
                ? "true"
                : undefined
            }
          >
            <button
              type="button"
              className="nav-group-trigger"
              aria-expanded={adminNavOpen}
              aria-controls="admin-nav"
              onClick={() => setAdminNavOpen((open) => !open)}
            >
              管理员后台
            </button>
            {adminNavOpen ? (
              <div className="nav-children" id="admin-nav">
                <button
                  type="button"
                  className={
                    activePage === "atlasReview"
                      ? "nav-child active"
                      : "nav-child"
                  }
                  aria-current={
                    activePage === "atlasReview" ? "page" : undefined
                  }
                  onClick={() => navigateToPage("atlasReview")}
                >
                  装备图鉴审核
                </button>
                <button
                  type="button"
                  className={
                    activePage === "adminFeedback"
                      ? "nav-child active"
                      : "nav-child"
                  }
                  aria-current={
                    activePage === "adminFeedback" ? "page" : undefined
                  }
                  onClick={() => navigateToPage("adminFeedback")}
                >
                  反馈信息
                </button>
                <button
                  type="button"
                  className={
                    activePage === "clientVersions"
                      ? "nav-child active"
                      : "nav-child"
                  }
                  aria-current={
                    activePage === "clientVersions" ? "page" : undefined
                  }
                  onClick={() => navigateToPage("clientVersions")}
                >
                  版本信息
                </button>
              </div>
            ) : null}
          </div>
        </nav>
        <div
          className="sidebar-global-actions"
          role="group"
          aria-label="全局设置"
        >
          <ThemeToggle theme={theme} onToggle={toggleTheme} />
        </div>
        <div className="sidebar-footer">
          <span>{session ? displayUserName(session) : "未登录"}</span>
          {session ? (
            <button className="ghost-button" onClick={handleLogout}>
              退出
            </button>
          ) : (
            <button className="ghost-button" onClick={navigateToLogin}>
              登录
            </button>
          )}
        </div>
      </aside>

      {activePage === "gear" ? (
        <main className="dashboard" id="gear">
          <header className="page-header">
            <div>
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
      ) : activePage === "gearAtlas" ? (
        <main className="dashboard" id="gear-atlas">
          <GearAtlasPage
            api={api}
            session={session}
            initialDetailId={gearAtlasDetailIdFromPath(
              window.location.pathname,
            )}
          />
        </main>
      ) : activePage === "atlasReview" ? (
        <main className="dashboard" id="atlas-review">
          <AtlasReviewPage
            submissions={atlasSubmissions}
            selected={atlasDetail}
            status={atlasStatus}
            deleted={atlasDeleted}
            loading={atlasLoading}
            loadingMore={atlasLoadingMore}
            submitting={submitting}
            error={atlasError}
            hasMore={Boolean(atlasNextCursor)}
            onStatusChange={(nextStatus) => {
              atlasRequestRef.current += 1;
              atlasLoadMoreInFlightRef.current = false;
              setAtlasStatus(nextStatus);
              setAtlasSubmissions([]);
              setAtlasDetail(null);
              setAtlasNextCursor(null);
              setAtlasLoadingMore(false);
            }}
            onDeletedChange={(nextDeleted) => {
              atlasRequestRef.current += 1;
              atlasLoadMoreInFlightRef.current = false;
              setAtlasDeleted(nextDeleted);
              setAtlasSubmissions([]);
              setAtlasDetail(null);
              setAtlasNextCursor(null);
              setAtlasLoadingMore(false);
            }}
            onRefresh={() => void loadAtlasSubmissions()}
            onLoadMore={() => {
              if (!atlasNextCursor) {
                return;
              }
              void loadAtlasSubmissions({
                cursor: atlasNextCursor,
                append: true,
              });
            }}
            onOpen={openAtlasSubmission}
            onUpdate={updateAtlasSubmission}
            onApprove={approveAtlasSubmission}
            onDelete={deleteAtlasSubmission}
            onRestore={restoreAtlasSubmission}
            onReject={rejectAtlasSubmission}
          />
        </main>
      ) : activePage === "adminFeedback" ? (
        <main className="dashboard" id="admin-feedback">
          <AdminFeedbackPage
            items={adminFeedback}
            status={feedbackStatus}
            deleted={feedbackDeleted}
            loading={feedbackLoading}
            error={feedbackError}
            onStatusChange={setFeedbackStatus}
            onDeletedChange={setFeedbackDeleted}
            onDelete={deleteAdminFeedback}
            onRestore={restoreAdminFeedback}
            onRefresh={() => void loadAdminFeedback()}
          />
        </main>
      ) : activePage === "clientVersions" ? (
        <main className="dashboard" id="client-versions">
          <ClientVersionsAdminPage
            items={adminClientVersions}
            clientKey={clientVersionClientKey}
            status={clientVersionStatus}
            form={clientVersionForm}
            loading={clientVersionsLoading}
            submitting={clientVersionSubmitting}
            error={clientVersionsError}
            onClientKeyChange={(nextClientKey) => {
              setClientVersionClientKey(nextClientKey);
              setClientVersionForm((current) =>
                current.id ? current : { ...current, clientKey: nextClientKey },
              );
            }}
            onStatusChange={setClientVersionStatus}
            onFormChange={setClientVersionForm}
            onCreate={startNewClientVersion}
            onEdit={editClientVersion}
            onCancel={cancelClientVersionEdit}
            onSave={() => void saveClientVersion()}
            onRefresh={() => void loadAdminClientVersions()}
          />
        </main>
      ) : (
        <main className="dashboard" id="skills">
          <KnotsPage api={api} />
        </main>
      )}

      {isFormOpen ? (
        <GearFormModal
          form={form}
          atlasItem={formAtlasItem}
          mode={formMode}
          submitting={submitting}
          onClose={() => {
            setIsFormOpen(false);
            setFormAtlasItem(null);
          }}
          onSubmit={submitForm}
          onChange={setForm}
        />
      ) : null}
      {detail ? (
        <GearDetailDrawer
          item={detail}
          atlasItem={detailAtlasItem}
          atlasSubmission={detailAtlasSubmission}
          submitting={submitting}
          onClose={() => {
            setDetail(null);
            setDetailAtlasItem(null);
            setDetailAtlasSubmission(null);
          }}
          onEdit={() => void openEditForm(detail.id)}
          onSubmitToAtlas={() => void submitGearToAtlas(detail.id)}
        />
      ) : null}
    </div>
  );
}

function AtlasReviewPage({
  submissions,
  selected,
  status,
  deleted,
  loading,
  loadingMore,
  submitting,
  error,
  hasMore,
  onStatusChange,
  onDeletedChange,
  onRefresh,
  onLoadMore,
  onOpen,
  onUpdate,
  onApprove,
  onDelete,
  onRestore,
  onReject,
}: {
  submissions: GearAtlasSubmission[];
  selected: GearAtlasSubmission | null;
  status: "" | GearAtlasStatus;
  deleted: DeletedFilter;
  loading: boolean;
  loadingMore: boolean;
  submitting: boolean;
  error: string | null;
  hasMore: boolean;
  onStatusChange(status: "" | GearAtlasStatus): void;
  onDeletedChange(deleted: DeletedFilter): void;
  onRefresh(): void;
  onLoadMore(): void;
  onOpen(id: string): Promise<void> | void;
  onUpdate(
    id: string,
    request: UpdateGearAtlasSubmissionRequest,
  ): Promise<void> | void;
  onApprove(id: string): Promise<void> | void;
  onDelete(id: string): Promise<void> | void;
  onRestore(id: string): Promise<void> | void;
  onReject(id: string, reason: string): Promise<void> | void;
}) {
  const [form, setForm] = useState<AtlasReviewFormState | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [rejectReason, setRejectReason] = useState("");
  const [rejectError, setRejectError] = useState<string | null>(null);
  const rejectReasonRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    setForm(selected ? atlasReviewFormFromSubmission(selected) : null);
    setFormError(null);
    setRejectReason("");
    setRejectError(null);
  }, [selected]);

  const specFields = useMemo(
    () => (form ? getGearAtlasSpecFieldViews(form.category, form.specs) : []),
    [form],
  );

  const isDirty = useMemo(
    () =>
      Boolean(
        selected &&
        form &&
        !atlasReviewPayloadsEqual(
          atlasReviewPayloadFromSubmission(selected),
          atlasReviewPayloadFromForm(form),
        ),
      ),
    [form, selected],
  );

  const handleListScroll = useCallback(
    (event: React.UIEvent<HTMLElement>) => {
      const list = event.currentTarget;
      const distanceToBottom =
        list.scrollHeight - list.scrollTop - list.clientHeight;
      if (distanceToBottom < 96 && hasMore && !loading && !loadingMore) {
        onLoadMore();
      }
    },
    [hasMore, loading, loadingMore, onLoadMore],
  );
  const updateVariant = (index: number, patch: Partial<GearVariant>) => {
    setForm((current) =>
      current
        ? {
            ...current,
            variants: current.variants.map((variant, currentIndex) =>
              currentIndex === index ? { ...variant, ...patch } : variant,
            ),
          }
        : current,
    );
  };
  const addVariant = () => {
    setForm((current) =>
      current
        ? {
            ...current,
            variants: [
              ...current.variants,
              {
                key: "",
                label: "",
                official_price_cents: null,
                official_price_currency: current.officialPriceCurrency,
                weight_g: null,
              },
            ],
          }
        : current,
    );
  };
  const removeVariant = (index: number) => {
    setForm((current) =>
      current
        ? {
            ...current,
            variants: current.variants.filter(
              (_, currentIndex) => currentIndex !== index,
            ),
          }
        : current,
    );
  };
  const updateSpecValue = (key: string, value: string, unit?: string) => {
    setForm((current) =>
      current
        ? {
            ...current,
            specs: {
              ...current.specs,
              [key]: combineSpecValue(value, unit),
            },
          }
        : current,
    );
  };
  const saveSelected = async () => {
    if (!selected || !form) return;
    try {
      setFormError(null);
      await onUpdate(selected.id, atlasReviewPayloadFromForm(form));
    } catch (err) {
      setFormError(errorMessage(err));
    }
  };
  const approveSelected = async () => {
    if (!selected || !form) return;
    try {
      setFormError(null);
      if (isDirty) {
        await onUpdate(selected.id, atlasReviewPayloadFromForm(form));
      }
      await onApprove(selected.id);
    } catch (err) {
      setFormError(errorMessage(err));
    }
  };
  const rejectSelected = () => {
    if (!selected) return;
    const reason = (rejectReasonRef.current?.value ?? rejectReason).trim();
    if (!reason) {
      setRejectError("请填写拒绝原因");
      return;
    }
    setRejectError(null);
    onReject(selected.id, reason);
  };
  const updateRejectReason = (value: string) => {
    setRejectReason(value);
    setRejectError(null);
  };

  return (
    <section className="atlas-review-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Admin Review</p>
          <h1>装备图鉴审核</h1>
          <p className="muted">
            只审核公共装备参数；个人购入价、购买渠道、存放位置、备注和标签不会进入图鉴。
          </p>
        </div>
        <div className="toolbar">
          <select
            aria-label="图鉴投稿状态"
            value={status}
            onChange={(event) =>
              onStatusChange(event.target.value as "" | GearAtlasStatus)
            }
          >
            <option value="pending">待审核</option>
            <option value="approved">已通过</option>
            <option value="rejected">已拒绝</option>
            <option value="">全部状态</option>
          </select>
          <select
            aria-label="图鉴删除状态"
            value={deleted}
            onChange={(event) =>
              onDeletedChange(event.target.value as DeletedFilter)
            }
          >
            <option value="active">未删除</option>
            <option value="deleted">已删除</option>
            <option value="all">全部记录</option>
          </select>
          <button
            type="button"
            className="secondary-button"
            onClick={onRefresh}
            disabled={loading}
          >
            刷新
          </button>
        </div>
      </header>

      {error ? (
        <div className="notice" role="status">
          {error.includes("403") ? "当前账号没有图鉴审核权限。" : error}
        </div>
      ) : null}

      <div className="atlas-review-layout">
        <section
          className="content-card atlas-review-list"
          aria-busy={loading || loadingMore}
          aria-label="图鉴审核投稿列表"
          onScroll={handleListScroll}
        >
          {loading ? <p className="muted">正在加载投稿...</p> : null}
          {!loading && submissions.length === 0 ? (
            <div className="empty-state compact">
              <h2>暂无投稿</h2>
              <p>当前筛选条件下没有图鉴投稿。</p>
            </div>
          ) : null}
          {submissions.map((item) => (
            <button
              type="button"
              key={item.id}
              className={
                selected?.id === item.id
                  ? "atlas-review-row active"
                  : "atlas-review-row"
              }
              onClick={() => void onOpen(item.id)}
            >
              <span className={`status-pill status-atlas-${item.status}`}>
                {atlasStatusLabel(item.status)}
              </span>
              {item.is_deleted ? (
                <span className="status-pill status-feedback-open">已删除</span>
              ) : null}
              <strong>{joinGearName(item)}</strong>
              <small>
                {item.category_label || categoryLabel(item.category)} ·{" "}
                {atlasSourceLabel(item.source_type)}
              </small>
            </button>
          ))}
          {submissions.length > 0 && hasMore ? (
            <button
              type="button"
              className="secondary-button atlas-load-more"
              disabled={loading || loadingMore}
              onClick={onLoadMore}
            >
              {loadingMore ? "继续加载中..." : "加载更多"}
            </button>
          ) : null}
          {loadingMore ? (
            <p className="muted atlas-loading-more" role="status">
              正在加载更多投稿...
            </p>
          ) : null}
        </section>

        <section className="content-card atlas-review-detail">
          {selected ? (
            <>
              <div className="atlas-detail-head">
                <div>
                  <p className="eyebrow">
                    {selected.category_label ||
                      categoryLabel(selected.category)}
                  </p>
                  <h2>{joinGearName(selected)}</h2>
                  <p className="muted">
                    {atlasSourceLabel(selected.source_type)} ·{" "}
                    {formatDate(selected.created_at)}
                  </p>
                </div>
                <span className={`status-pill status-atlas-${selected.status}`}>
                  {atlasStatusLabel(selected.status)}
                </span>
                {selected.is_deleted ? (
                  <span className="status-pill status-feedback-open">
                    已删除
                  </span>
                ) : null}
              </div>

              {form ? (
                <form
                  className="atlas-review-form"
                  onSubmit={(event) => {
                    event.preventDefault();
                    void saveSelected();
                  }}
                >
                  <fieldset className="atlas-review-fieldset">
                    <legend>公共字段</legend>
                    <label>
                      分类
                      <select
                        value={form.category}
                        onChange={(event) => {
                          const category = event.target.value as GearCategory;
                          setForm({
                            ...form,
                            category,
                            specs: normalizeSpecsForCategory(
                              category,
                              form.specs,
                            ),
                          });
                        }}
                      >
                        {CATEGORY_OPTIONS.map((item) => (
                          <option key={item.value} value={item.value}>
                            {item.label}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      名称
                      <input
                        required
                        value={form.name}
                        onChange={(event) =>
                          setForm({ ...form, name: event.target.value })
                        }
                      />
                    </label>
                    <label>
                      品牌
                      <input
                        value={form.brand}
                        onChange={(event) =>
                          setForm({ ...form, brand: event.target.value })
                        }
                      />
                    </label>
                    <label>
                      型号
                      <input
                        value={form.model}
                        onChange={(event) =>
                          setForm({ ...form, model: event.target.value })
                        }
                      />
                    </label>
                    <label>
                      重量（g）
                      <input
                        type="number"
                        min="0"
                        value={form.weightG}
                        onChange={(event) =>
                          setForm({ ...form, weightG: event.target.value })
                        }
                      />
                    </label>
                    <label>
                      官方价格
                      <input
                        type="number"
                        min="0"
                        step="0.01"
                        value={form.officialPrice}
                        onChange={(event) =>
                          setForm({
                            ...form,
                            officialPrice: event.target.value,
                          })
                        }
                      />
                    </label>
                    <label>
                      价格币种
                      <select
                        value={form.officialPriceCurrency}
                        onChange={(event) =>
                          setForm({
                            ...form,
                            officialPriceCurrency: event.target
                              .value as GearCurrency,
                          })
                        }
                      >
                        {CURRENCY_OPTIONS.map((currency) => (
                          <option key={currency} value={currency}>
                            {currency}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label className="full-width">
                      描述
                      <textarea
                        value={form.description}
                        onChange={(event) =>
                          setForm({
                            ...form,
                            description: event.target.value,
                          })
                        }
                      />
                    </label>
                  </fieldset>

                  <dl className="atlas-public-fields atlas-readonly-fields">
                    <div>
                      <dt>来源类型</dt>
                      <dd>{atlasSourceLabel(selected.source_type)}</dd>
                    </div>
                    <div>
                      <dt>来源摘要</dt>
                      <dd>{atlasSourceAuditSummary(selected)}</dd>
                    </div>
                    <div>
                      <dt>来源装备</dt>
                      <dd>{selected.source_user_gear_id || "手动投稿"}</dd>
                    </div>
                    <div>
                      <dt>来源链接</dt>
                      <dd>{selected.source_url || "—"}</dd>
                    </div>
                    <div>
                      <dt>评分摘要</dt>
                      <dd>{atlasRatingSummary(selected)}</dd>
                    </div>
                  </dl>

                  <div className="atlas-specs">
                    <h3>可选尺寸</h3>
                    {form.variants.length ? (
                      <div className="atlas-variant-editor">
                        {form.variants.map((variant, index) => (
                          <div
                            className="atlas-variant-row"
                            key={`${index}-${variant.key}`}
                          >
                            <label>
                              尺寸
                              <input
                                value={variant.label}
                                placeholder="例如 M 75*195"
                                onChange={(event) =>
                                  updateVariant(index, {
                                    label: event.target.value,
                                    key: variantKeyFromLabel(
                                      event.target.value,
                                      index,
                                    ),
                                  })
                                }
                              />
                            </label>
                            <label>
                              分尺寸官方价
                              <input
                                type="number"
                                min="0"
                                step="0.01"
                                placeholder="例如 900.00"
                                value={fromPriceCents(
                                  variant.official_price_cents,
                                )}
                                onChange={(event) =>
                                  updateVariant(index, {
                                    official_price_cents: toPriceCents(
                                      event.target.value,
                                    ),
                                    official_price_currency:
                                      variant.official_price_currency ||
                                      form.officialPriceCurrency,
                                  })
                                }
                              />
                            </label>
                            <label>
                              重量（g）
                              <input
                                type="number"
                                min="0"
                                placeholder="例如 1000"
                                value={variant.weight_g ?? ""}
                                onChange={(event) =>
                                  updateVariant(index, {
                                    weight_g: event.target.value
                                      ? Number(event.target.value)
                                      : null,
                                  })
                                }
                              />
                            </label>
                            <button
                              type="button"
                              className="secondary-button variant-remove-button"
                              onClick={() => removeVariant(index)}
                            >
                              移除
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="muted">未填写可选尺寸。</p>
                    )}
                    <div className="actions inline-actions">
                      <button
                        type="button"
                        className="secondary-button"
                        onClick={addVariant}
                      >
                        添加尺寸
                      </button>
                    </div>
                  </div>

                  <div className="atlas-specs">
                    <h3>分类参数</h3>
                    {specFields.length ? (
                      <div className="atlas-review-spec-grid">
                        {specFields.map((field) => (
                          <div className="atlas-spec-input" key={field.key}>
                            <label htmlFor={`atlas-review-spec-${field.key}`}>
                              {field.label}
                            </label>
                            <div className="atlas-spec-input-row">
                              {field.choiceOnly ? (
                                <span
                                  className={
                                    field.unitLabel
                                      ? "choice-value"
                                      : "choice-value muted"
                                  }
                                >
                                  {field.unitLabel || field.placeholder}
                                </span>
                              ) : (
                                <input
                                  id={`atlas-review-spec-${field.key}`}
                                  type={
                                    field.inputType === "number"
                                      ? "number"
                                      : "text"
                                  }
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
                                    const unitIndex = Number(
                                      event.target.value || 0,
                                    );
                                    const unit = field.units?.[unitIndex] ?? "";
                                    updateSpecValue(
                                      field.key,
                                      field.choiceOnly ? "" : field.valueText,
                                      unit,
                                    );
                                  }}
                                >
                                  {field.unitLabels.map((label, index) => (
                                    <option
                                      key={`${field.key}-${label}`}
                                      value={index}
                                    >
                                      {label || "无单位"}
                                    </option>
                                  ))}
                                </select>
                              ) : null}
                            </div>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="muted">未填写分类参数。</p>
                    )}
                  </div>
                  {formError ? (
                    <div className="notice" role="status">
                      {formError}
                    </div>
                  ) : null}
                </form>
              ) : null}

              {selected.rejection_reason ? (
                <div className="notice">
                  拒绝原因：{selected.rejection_reason}
                </div>
              ) : null}

              {selected.review_changes?.length ? (
                <div className="notice">
                  管理员调整：
                  {selected.review_changes
                    .map(
                      (change) =>
                        `${change.label} 从 ${change.before || "—"} 改为 ${
                          change.after || "—"
                        }`,
                    )
                    .join("；")}
                </div>
              ) : null}

              <label className="atlas-reject-reason">
                拒绝原因
                <textarea
                  key={selected.id}
                  ref={rejectReasonRef}
                  defaultValue=""
                  maxLength={200}
                  placeholder="请说明需要用户修改的字段或原因"
                  onChange={(event) => {
                    updateRejectReason(event.target.value);
                  }}
                  onInput={(event) => {
                    updateRejectReason(event.currentTarget.value);
                  }}
                />
              </label>
              {rejectError ? (
                <div className="notice" role="status">
                  {rejectError}
                </div>
              ) : null}

              <div className="actions">
                <button
                  type="button"
                  className="secondary-button"
                  disabled={submitting || !isDirty}
                  onClick={() => void saveSelected()}
                >
                  {submitting ? "保存中..." : isDirty ? "保存字段" : "已保存"}
                </button>
                <button
                  type="button"
                  className="primary-button"
                  disabled={submitting || selected.status === "approved"}
                  onClick={() => void approveSelected()}
                >
                  {isDirty ? "保存并通过" : "通过"}
                </button>
                <button
                  type="button"
                  className="secondary-button"
                  disabled={submitting || selected.status === "rejected"}
                  onClick={rejectSelected}
                >
                  拒绝
                </button>
                {selected.is_deleted ? (
                  <button
                    type="button"
                    className="secondary-button"
                    disabled={submitting}
                    onClick={() => void onRestore(selected.id)}
                  >
                    恢复
                  </button>
                ) : (
                  <button
                    type="button"
                    className="secondary-button"
                    disabled={submitting}
                    onClick={() => void onDelete(selected.id)}
                  >
                    删除
                  </button>
                )}
              </div>
            </>
          ) : (
            <div className="empty-state compact">
              <h2>选择一条投稿</h2>
              <p>点击左侧列表查看公开字段并完成审核。</p>
            </div>
          )}
        </section>
      </div>
    </section>
  );
}

function AdminFeedbackPage({
  items,
  status,
  deleted,
  loading,
  error,
  onStatusChange,
  onDeletedChange,
  onDelete,
  onRestore,
  onRefresh,
}: {
  items: AdminFeedbackItem[];
  status: FeedbackStatusFilter;
  deleted: DeletedFilter;
  loading: boolean;
  error: string | null;
  onStatusChange(status: FeedbackStatusFilter): void;
  onDeletedChange(deleted: DeletedFilter): void;
  onDelete(id: string): Promise<void> | void;
  onRestore(id: string): Promise<void> | void;
  onRefresh(): void;
}) {
  return (
    <section className="admin-feedback-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Admin Feedback</p>
          <h1>反馈信息</h1>
          <p className="muted">
            查看用户提交的问题、建议、联系方式和客户端环境信息。
          </p>
        </div>
        <div className="toolbar">
          <select
            aria-label="反馈状态"
            value={status}
            onChange={(event) =>
              onStatusChange(event.target.value as FeedbackStatusFilter)
            }
          >
            <option value="open">待处理</option>
            <option value="">全部状态</option>
          </select>
          <select
            aria-label="反馈删除状态"
            value={deleted}
            onChange={(event) =>
              onDeletedChange(event.target.value as DeletedFilter)
            }
          >
            <option value="active">未删除</option>
            <option value="deleted">已删除</option>
            <option value="all">全部记录</option>
          </select>
          <button
            type="button"
            className="secondary-button"
            onClick={onRefresh}
            disabled={loading}
          >
            刷新
          </button>
        </div>
      </header>

      {error ? (
        <div className="notice" role="status">
          {error.includes("403") ? "当前账号没有反馈查看权限。" : error}
        </div>
      ) : null}

      <section className="content-card admin-feedback-list" aria-busy={loading}>
        {loading ? <p className="muted">正在加载反馈...</p> : null}
        {!loading && items.length === 0 ? (
          <div className="empty-state compact">
            <h2>暂无反馈</h2>
            <p>当前筛选条件下还没有用户反馈。</p>
          </div>
        ) : null}
        {items.map((item) => (
          <article className="admin-feedback-card" key={item.id}>
            <div className="admin-feedback-head">
              <div>
                <span className="category-text">
                  {feedbackCategoryLabel(item.category)}
                </span>
                <h2>{item.content}</h2>
                <p className="muted">
                  {feedbackUserName(item.user)} · {formatDate(item.created_at)}
                </p>
              </div>
              <span className="status-pill status-feedback-open">
                {feedbackStatusLabel(item.status)}
              </span>
              {item.is_deleted ? (
                <span className="status-pill status-feedback-open">已删除</span>
              ) : null}
            </div>
            <dl className="admin-feedback-meta">
              <div>
                <dt>联系方式</dt>
                <dd>{item.contact || "—"}</dd>
              </div>
              <div>
                <dt>页面</dt>
                <dd>{item.page || "—"}</dd>
              </div>
              <div>
                <dt>客户端</dt>
                <dd>
                  {[item.client_platform, item.client_version]
                    .filter(Boolean)
                    .join(" · ") || "—"}
                </dd>
              </div>
              <div>
                <dt>设备</dt>
                <dd>{item.device_model || "—"}</dd>
              </div>
              <div>
                <dt>图片</dt>
                <dd>
                  {item.images.length
                    ? item.images
                        .map((image) => image.original_filename)
                        .join("、")
                    : "—"}
                </dd>
              </div>
              <div>
                <dt>用户 ID</dt>
                <dd>{item.user.id}</dd>
              </div>
            </dl>
            <div className="actions">
              {item.is_deleted ? (
                <button
                  type="button"
                  className="secondary-button"
                  disabled={loading}
                  onClick={() => void onRestore(item.id)}
                >
                  恢复
                </button>
              ) : (
                <button
                  type="button"
                  className="secondary-button"
                  disabled={loading}
                  onClick={() => void onDelete(item.id)}
                >
                  删除
                </button>
              )}
            </div>
          </article>
        ))}
      </section>
    </section>
  );
}

function ClientVersionsAdminPage({
  items,
  clientKey,
  status,
  form,
  loading,
  submitting,
  error,
  onClientKeyChange,
  onStatusChange,
  onFormChange,
  onCreate,
  onEdit,
  onCancel,
  onSave,
  onRefresh,
}: {
  items: ClientVersion[];
  clientKey: ClientKey;
  status: ClientVersionStatusFilter;
  form: ClientVersionFormState;
  loading: boolean;
  submitting: boolean;
  error: string | null;
  onClientKeyChange(clientKey: ClientKey): void;
  onStatusChange(status: ClientVersionStatusFilter): void;
  onFormChange(form: ClientVersionFormState): void;
  onCreate(): void;
  onEdit(item: ClientVersion): void;
  onCancel(): void;
  onSave(): void;
  onRefresh(): void;
}) {
  return (
    <section className="client-versions-page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Admin Releases</p>
          <h1>版本信息</h1>
          <p className="muted">
            维护各客户端对用户公开展示的版本号和更新说明。
          </p>
        </div>
        <div className="toolbar">
          <select
            aria-label="客户端筛选"
            value={clientKey}
            onChange={(event) =>
              onClientKeyChange(event.target.value as ClientKey)
            }
          >
            {CLIENT_VERSION_CLIENT_OPTIONS.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
          <select
            aria-label="版本状态筛选"
            value={status}
            onChange={(event) =>
              onStatusChange(event.target.value as ClientVersionStatusFilter)
            }
          >
            <option value="">全部状态</option>
            {CLIENT_VERSION_STATUS_OPTIONS.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="secondary-button"
            onClick={onRefresh}
            disabled={loading}
          >
            刷新
          </button>
          <button type="button" className="primary-button" onClick={onCreate}>
            新增版本
          </button>
        </div>
      </header>

      {error ? (
        <div className="notice" role="status">
          {error.includes("403") ? "当前账号没有版本维护权限。" : error}
        </div>
      ) : null}

      <div className="client-versions-layout">
        <section className="content-card client-version-form-card">
          <h2>{form.id ? "编辑版本" : "新增版本"}</h2>
          <div className="client-version-form">
            <label>
              客户端
              <select
                value={form.clientKey}
                onChange={(event) =>
                  onFormChange({
                    ...form,
                    clientKey: event.target.value as ClientKey,
                  })
                }
              >
                {CLIENT_VERSION_CLIENT_OPTIONS.map((item) => (
                  <option key={item.id} value={item.id}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>
            <label>
              版本号
              <input
                value={form.version}
                placeholder="例如 0.1.0"
                onChange={(event) =>
                  onFormChange({ ...form, version: event.target.value })
                }
              />
            </label>
            <label>
              状态
              <select
                value={form.status}
                onChange={(event) =>
                  onFormChange({
                    ...form,
                    status: event.target.value as ClientVersionStatus,
                  })
                }
              >
                {CLIENT_VERSION_STATUS_OPTIONS.map((item) => (
                  <option key={item.id} value={item.id}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="full-width">
              标题
              <input
                value={form.title}
                placeholder="例如 0.1.0 初始版本"
                onChange={(event) =>
                  onFormChange({ ...form, title: event.target.value })
                }
              />
            </label>
            <label className="full-width">
              Feature
              <textarea
                value={form.featureNotesText}
                placeholder="每行一条新增功能"
                onChange={(event) =>
                  onFormChange({
                    ...form,
                    featureNotesText: event.target.value,
                  })
                }
              />
            </label>
            <label className="full-width">
              BugFix
              <textarea
                value={form.bugFixNotesText}
                placeholder="每行一条修复内容"
                onChange={(event) =>
                  onFormChange({
                    ...form,
                    bugFixNotesText: event.target.value,
                  })
                }
              />
            </label>
            <label className="full-width">
              Notes（可选）
              <textarea
                value={form.notesText}
                placeholder="每行一条兼容说明、已知限制或注意事项"
                onChange={(event) =>
                  onFormChange({
                    ...form,
                    notesText: event.target.value,
                  })
                }
              />
            </label>
          </div>
          <div className="actions">
            <button
              type="button"
              className="primary-button"
              onClick={onSave}
              disabled={submitting}
            >
              {submitting ? "保存中..." : form.id ? "保存版本" : "创建版本"}
            </button>
            {form.id ? (
              <button
                type="button"
                className="secondary-button"
                onClick={onCancel}
                disabled={submitting}
              >
                取消编辑
              </button>
            ) : null}
          </div>
        </section>

        <section
          className="content-card client-version-list"
          aria-busy={loading}
        >
          {loading ? <p className="muted">正在加载版本...</p> : null}
          {!loading && items.length === 0 ? (
            <div className="empty-state compact">
              <h2>暂无版本</h2>
              <p>当前筛选条件下还没有客户端版本。</p>
            </div>
          ) : null}
          {items.map((item) => (
            <article className="client-version-card" key={item.id}>
              <div className="client-version-card-head">
                <div>
                  <span className="category-text">
                    {clientKeyLabel(item.client_key)}
                  </span>
                  <h2>{item.title}</h2>
                  <p className="muted">
                    {item.version} ·{" "}
                    {item.published_at
                      ? `发布于 ${formatDate(item.published_at)}`
                      : "未发布"}
                  </p>
                </div>
                <span
                  className={`status-pill status-client-version-${item.status}`}
                >
                  {clientVersionStatusLabel(item.status)}
                </span>
              </div>
              <div className="client-version-notes">
                {clientVersionSections(item).map((section) => (
                  <section
                    className="client-version-note-section"
                    key={section.key}
                  >
                    <h3>{section.title}</h3>
                    <ol>
                      {section.items.map((note) => (
                        <li key={note}>{note}</li>
                      ))}
                    </ol>
                  </section>
                ))}
              </div>
              <div className="actions">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => onEdit(item)}
                >
                  编辑
                </button>
              </div>
            </article>
          ))}
        </section>
      </div>
    </section>
  );
}

function atlasReviewFormFromSubmission(
  item: GearAtlasSubmission,
): AtlasReviewFormState {
  return {
    category: item.category,
    name: item.name,
    brand: item.brand ?? "",
    model: item.model ?? "",
    description: item.description ?? "",
    weightG: item.weight_g?.toString() ?? "",
    officialPrice: fromPriceCents(item.official_price_cents),
    officialPriceCurrency:
      (item.official_price_currency as GearCurrency | null | undefined) ??
      "CNY",
    variants: normalizeVariants(item.variants ?? []),
    specs: item.specs ?? {},
  };
}

function atlasReviewPayloadFromForm(
  form: AtlasReviewFormState,
): UpdateGearAtlasSubmissionRequest {
  const name = form.name.trim();
  if (!name) {
    throw new Error("请填写装备名称");
  }
  return {
    category: form.category,
    name,
    brand: optionalAtlasText(form.brand),
    model: optionalAtlasText(form.model),
    description: optionalAtlasText(form.description),
    weight_g: optionalAtlasWholeNumber(form.weightG, "重量必须是非负整数"),
    official_price_cents: optionalAtlasPriceCents(form.officialPrice),
    official_price_currency: form.officialPrice.trim()
      ? form.officialPriceCurrency
      : null,
    variants: normalizeVariants(form.variants),
    specs: normalizeSpecsForCategory(form.category, form.specs),
  };
}

function atlasReviewPayloadFromSubmission(
  item: GearAtlasSubmission,
): UpdateGearAtlasSubmissionRequest {
  return {
    category: item.category,
    name: item.name,
    brand: item.brand ?? null,
    model: item.model ?? null,
    description: item.description ?? null,
    weight_g: item.weight_g ?? null,
    official_price_cents: item.official_price_cents ?? null,
    official_price_currency: item.official_price_currency ?? null,
    variants: normalizeVariants(item.variants ?? []),
    specs: normalizeSpecsForCategory(item.category, item.specs ?? {}),
  };
}

function atlasReviewPayloadsEqual(
  left: UpdateGearAtlasSubmissionRequest,
  right: UpdateGearAtlasSubmissionRequest,
): boolean {
  return (
    JSON.stringify(normalizeAtlasReviewPayload(left)) ===
    JSON.stringify(normalizeAtlasReviewPayload(right))
  );
}

function normalizeAtlasReviewPayload(
  payload: UpdateGearAtlasSubmissionRequest,
): UpdateGearAtlasSubmissionRequest {
  const specs = Object.fromEntries(
    Object.entries(payload.specs ?? {}).sort(([left], [right]) =>
      left.localeCompare(right),
    ),
  );
  return {
    ...payload,
    brand: payload.brand || null,
    model: payload.model || null,
    description: payload.description || null,
    official_price_currency:
      payload.official_price_cents !== undefined &&
      payload.official_price_cents !== null
        ? payload.official_price_currency || "CNY"
        : null,
    variants: normalizeVariants(payload.variants ?? []),
    specs,
  };
}

function optionalAtlasText(value: string): string | null {
  const text = value.trim();
  return text || null;
}

function optionalAtlasWholeNumber(
  value: string,
  message: string,
): number | null {
  const text = value.trim();
  if (!text) return null;
  const parsed = Number(text);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(message);
  }
  return parsed;
}

function optionalAtlasPriceCents(value: string): number | null {
  const text = value.trim();
  if (!text) return null;
  const parsed = Number(text);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error("官方价格必须是非负数字");
  }
  return Math.round(parsed * 100);
}

function mergeAtlasSubmissionPages(
  current: GearAtlasSubmission[],
  next: GearAtlasSubmission[],
): GearAtlasSubmission[] {
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
  atlasItem,
  mode,
  submitting,
  onClose,
  onSubmit,
  onChange,
}: {
  form: GearFormState;
  atlasItem: GearAtlasPublicItem | null;
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
  const atlasVariants = atlasItem?.variants ?? [];
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
            我的尺寸
            <input
              placeholder="例如 M 75*195"
              value={form.selectedVariantLabel}
              onChange={(event) =>
                onChange({
                  ...form,
                  selectedVariantKey: "",
                  selectedVariantLabel: event.target.value,
                })
              }
            />
          </label>
          {atlasVariants.length ? (
            <label>
              图鉴尺寸
              <select
                value={form.selectedVariantKey}
                onChange={(event) => {
                  const variant = atlasVariants.find(
                    (item) => item.key === event.target.value,
                  );
                  onChange({
                    ...form,
                    atlasItemId: atlasItem?.id ?? form.atlasItemId,
                    selectedVariantKey: variant?.key ?? "",
                    selectedVariantLabel:
                      variant?.label ?? form.selectedVariantLabel,
                  });
                }}
              >
                <option value="">保留手填尺寸</option>
                {atlasVariants.map((variant) => (
                  <option key={variant.key} value={variant.key}>
                    {variant.label}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
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
  atlasItem,
  atlasSubmission,
  submitting,
  onClose,
  onEdit,
  onSubmitToAtlas,
}: {
  item: GearItem;
  atlasItem: GearAtlasPublicItem | null;
  atlasSubmission: GearAtlasSubmission | null;
  submitting: boolean;
  onClose(): void;
  onEdit(): void;
  onSubmitToAtlas(): Promise<void> | void;
}) {
  const specs = item.specs ?? {};
  const variants = atlasItem?.variants ?? [];
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
          <dt>容量</dt>
          <dd>{specs.capacity || "—"}</dd>
        </div>
        <div>
          <dt>我的尺寸</dt>
          <dd>{item.selected_variant_label || "—"}</dd>
        </div>
        <div>
          <dt>可选尺寸</dt>
          <dd>
            {item.atlas_item_id ? (
              variants.length ? (
                variants
                  .map((variant) => variantSummary(variant, formatAtlasPrice))
                  .join("、")
              ) : (
                "—"
              )
            ) : (
              <span className="muted">
                关联或投稿到图鉴后可查看该装备可选尺寸
              </span>
            )}
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
      {atlasSubmission ? (
        <div className="notice atlas-submission-status">
          <strong>图鉴投稿：{atlasStatusLabel(atlasSubmission.status)}</strong>
          <span>{atlasSubmissionHint(atlasSubmission)}</span>
        </div>
      ) : null}
      <div className="actions detail-actions">
        <button
          className="secondary-button"
          onClick={onSubmitToAtlas}
          disabled={
            submitting ||
            atlasSubmission?.status === "pending" ||
            atlasSubmission?.status === "approved"
          }
        >
          {submitting
            ? "提交中..."
            : atlasSubmission?.status === "rejected"
              ? "重新投稿到图鉴"
              : atlasSubmission
                ? "已投稿"
                : "投稿到图鉴"}
        </button>
        <button className="primary-button" onClick={onEdit}>
          编辑装备
        </button>
      </div>
    </aside>
  );
}

function formToPayload(form: GearFormState): CreateGearRequest {
  return {
    category: form.category,
    name: form.name.trim(),
    brand: optional(form.brand),
    model: optional(form.model),
    description: optional(form.description),
    weight_g: optionalNumber(form.weightG),
    purchase_date: optional(form.purchaseDate),
    purchase_price_cents: toPriceCents(form.purchasePrice),
    purchase_location: optional(form.purchaseLocation),
    status: form.status,
    storage_location: optional(form.storageLocation),
    atlas_item_id: optional(form.atlasItemId),
    selected_variant_key: optional(form.selectedVariantKey),
    selected_variant_label: optional(form.selectedVariantLabel),
    specs: specsFromForm(form),
    tags: form.tags
      .split(/[,，]/)
      .map((tag) => tag.trim())
      .filter(Boolean),
    share_enabled: false,
    notes: optional(form.notes),
  };
}

function formFromGear(item: GearItem): GearFormState {
  const specs = item.specs ?? {};
  return {
    category: item.category,
    name: item.name,
    brand: item.brand ?? "",
    model: item.model ?? "",
    color: specs.color ?? "",
    material: specs.material ?? "",
    capacity: specs.capacity ?? "",
    atlasItemId: item.atlas_item_id ?? "",
    selectedVariantKey: item.selected_variant_key ?? "",
    selectedVariantLabel: item.selected_variant_label ?? "",
    description: item.description ?? "",
    weightG:
      item.weight_g === null || item.weight_g === undefined
        ? ""
        : String(item.weight_g),
    warmthIndex: specs.warmth_index ?? "",
    waterproofIndex: specs.waterproof_index ?? "",
    purchaseDate: item.purchase_date ?? "",
    purchasePrice: fromPriceCents(item.purchase_price_cents),
    expiryOrWarrantyDate: specs.expiry_or_warranty_date ?? "",
    purchaseLocation: item.purchase_location ?? "",
    status: item.status,
    storageLocation: item.storage_location ?? "",
    tags: item.tags.join("，"),
    notes: item.notes ?? "",
  };
}

function specsFromForm(form: GearFormState): Record<string, string> {
  const specs: Record<string, string> = {};
  setSpec(specs, "color", form.color);
  setSpec(specs, "material", form.material);
  setSpec(specs, "capacity", form.capacity);
  setSpec(specs, "warmth_index", form.warmthIndex);
  setSpec(specs, "waterproof_index", form.waterproofIndex);
  setSpec(specs, "expiry_or_warranty_date", form.expiryOrWarrantyDate);
  return specs;
}

function setSpec(specs: Record<string, string>, key: string, value: string) {
  const trimmed = value.trim();
  if (trimmed) {
    specs[key] = trimmed;
  }
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
  if (mode === "email") {
    return "邮箱验证码登录";
  }
  if (mode === "reset") {
    return "找回密码";
  }
  if (mode === "register") {
    return "注册账号";
  }
  return "本地开发登录";
}

function sessionFromLoginResponse(response: WechatLoginResponse): WebSession {
  return {
    accessToken: response.access_token,
    expiresAt: response.expires_at,
    refreshToken: response.refresh_token,
    refreshExpiresAt: response.refresh_expires_at,
    user: response.user,
  };
}

function displayUserName(session: WebSession): string {
  return (
    session.user.nickname ??
    session.user.username ??
    session.user.email ??
    "本地用户"
  );
}

function atlasStatusLabel(status: GearAtlasStatus): string {
  if (status === "approved") {
    return "已通过";
  }
  if (status === "rejected") {
    return "已拒绝";
  }
  return "待审核";
}

function atlasSourceLabel(sourceType: GearAtlasSubmission["source_type"]) {
  if (sourceType === "user_gear") {
    return "个人装备生成";
  }
  if (sourceType === "external_import") {
    return "外部导入";
  }
  return "手动投稿";
}

function atlasSubmissionHint(submission: GearAtlasSubmission): string {
  if (submission.status === "rejected") {
    return submission.rejection_reason
      ? `拒绝原因：${submission.rejection_reason}`
      : "审核未通过，暂未填写原因。";
  }
  if (submission.status === "approved" && submission.review_changes?.length) {
    return `管理员调整：${submission.review_changes
      .map(
        (change) =>
          `${change.label} 从 ${change.before || "—"} 改为 ${
            change.after || "—"
          }`,
      )
      .join("；")}`;
  }
  if (submission.status === "approved") {
    return "已收录到装备图鉴。";
  }
  return "审核中，结果会在这里显示。";
}

function atlasRatingSummary(item: GearAtlasSubmission): string {
  if (
    item.source_rating_score === undefined ||
    item.source_rating_score === null
  ) {
    return "—";
  }
  const count =
    item.source_rating_count !== undefined && item.source_rating_count !== null
      ? ` / ${item.source_rating_count} 条`
      : "";
  return `${item.source_rating_score.toFixed(1)} 分${count}`;
}

function atlasSourceAuditSummary(item: GearAtlasSubmission): string {
  const source = item.source_name || atlasSourceLabel(item.source_type);
  const rating = atlasRatingSummary(item);
  return rating === "—" ? source : `${source} · ${rating}`;
}

function formatAtlasPrice(
  cents?: number | null,
  currency?: string | null,
): string {
  if (cents === undefined || cents === null) {
    return "—";
  }
  const code = currency || "CNY";
  if (code === "CNY") {
    return formatCurrency(cents);
  }
  if (code === "JPY") {
    return `${code} ${cents}`;
  }
  return `${code} ${(cents / 100).toFixed(2)}`;
}

function feedbackCategoryLabel(category: string): string {
  const labels: Record<string, string> = {
    bug: "问题反馈",
    suggestion: "功能建议",
    content: "内容反馈",
    account: "账号问题",
    other: "其他反馈",
  };
  return labels[category] ?? category;
}

function feedbackStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    open: "待处理",
    triaged: "已分流",
    resolved: "已处理",
    closed: "已关闭",
  };
  return labels[status] ?? status;
}

function clientKeyLabel(clientKey: ClientKey): string {
  return (
    CLIENT_VERSION_CLIENT_OPTIONS.find((item) => item.id === clientKey)
      ?.label ?? clientKey
  );
}

function clientVersionStatusLabel(status: ClientVersionStatus): string {
  if (status === "published") {
    return "已发布";
  }
  return "草稿";
}

function clientVersionRequestFromForm(
  form: ClientVersionFormState,
): ClientVersionRequest {
  const featureItems = splitReleaseNoteText(form.featureNotesText);
  const bugFixItems = splitReleaseNoteText(form.bugFixNotesText);
  const notesItems = splitReleaseNoteText(form.notesText);
  const releaseNoteSections: ClientVersionReleaseNoteSection[] = [
    { key: "feature", title: "Feature", items: featureItems },
    { key: "bug_fix", title: "BugFix", items: bugFixItems },
  ];
  if (notesItems.length) {
    releaseNoteSections.push({
      key: "notes",
      title: "Notes",
      items: notesItems,
    });
  }
  return {
    client_key: form.clientKey,
    version: form.version.trim(),
    title: form.title.trim(),
    release_notes: [...featureItems, ...bugFixItems, ...notesItems],
    release_note_sections: releaseNoteSections,
    status: form.status,
  };
}

function splitReleaseNoteText(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function clientVersionSections(
  item: ClientVersion,
): ClientVersionReleaseNoteSection[] {
  const sections = item.release_note_sections?.filter(
    (section) => section.items.length > 0,
  );
  if (sections?.length) {
    return sections;
  }
  if (item.release_notes.length) {
    return [{ key: "feature", title: "Feature", items: item.release_notes }];
  }
  return [];
}

function sectionItemsText(
  sections: ClientVersionReleaseNoteSection[],
  key: ClientVersionReleaseNoteSection["key"],
): string {
  return (
    sections.find((section) => section.key === key)?.items.join("\n") ?? ""
  );
}

function feedbackUserName(user: AdminFeedbackItem["user"]): string {
  return (
    user.nickname ??
    user.username ??
    user.email ??
    `用户 ${user.id.slice(0, 8)}`
  );
}

function activePageFromPath(pathname: string): ActivePage {
  if (pathname === "/gear-atlas" || pathname.startsWith("/gear-atlas/")) {
    return "gearAtlas";
  }
  if (
    pathname === "/admin/feedback" ||
    pathname.startsWith("/admin/feedback/")
  ) {
    return "adminFeedback";
  }
  if (
    pathname === "/admin/client-versions" ||
    pathname.startsWith("/admin/client-versions/")
  ) {
    return "clientVersions";
  }
  if (pathname === "/admin" || pathname.startsWith("/admin/")) {
    return "atlasReview";
  }
  if (pathname === "/knots" || pathname.startsWith("/skills/knots")) {
    return "knots";
  }
  return "gear";
}

function pathForActivePage(page: ActivePage): string {
  if (page === "gearAtlas") {
    return "/gear-atlas";
  }
  if (page === "adminFeedback") {
    return "/admin/feedback";
  }
  if (page === "clientVersions") {
    return "/admin/client-versions";
  }
  if (page === "atlasReview") {
    return "/admin";
  }
  if (page === "knots") {
    return "/skills/knots";
  }
  return "/";
}

function gearAtlasDetailIdFromPath(pathname: string): string | null {
  if (!pathname.startsWith("/gear-atlas/")) {
    return null;
  }
  const id = pathname.slice("/gear-atlas/".length);
  return id ? decodeURIComponent(id) : null;
}

function isAdminPage(page: ActivePage): boolean {
  return (
    page === "atlasReview" ||
    page === "adminFeedback" ||
    page === "clientVersions"
  );
}

function loadThemePreference(): ThemeMode {
  return localStorage.getItem(THEME_STORAGE_KEY) === "dark" ? "dark" : "light";
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : "请求失败，请稍后重试";
}

function isApiStatus(err: unknown, status: number): boolean {
  return err instanceof StellarTrailApiError && err.status === status;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
