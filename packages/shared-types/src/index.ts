export interface HealthResponse {
  status: "ok";
}

export interface MetaResponse {
  name: "StellarTrail";
  env: string;
  database_kind: "sqlite" | "postgres" | "mysql";
}

export type ClientKey =
  | "wechat_miniprogram"
  | "web"
  | "android"
  | "ios"
  | "macos";

export type ClientVersionStatus = "draft" | "published";

export interface ClientVersion {
  id: string;
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes: string[];
  status: ClientVersionStatus;
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListClientVersionsRequest {
  client_key?: ClientKey;
  status?: ClientVersionStatus;
  limit?: number;
  cursor?: string;
}

export interface ListClientVersionsResponse {
  items: ClientVersion[];
  next_cursor?: string | null;
}

export interface ClientVersionRequest {
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes: string[];
  status: ClientVersionStatus;
}

export type RoadmapStatus =
  | "planned"
  | "designing"
  | "building"
  | "preview"
  | "shipped";

export type RoadmapCategory =
  | "gear"
  | "skills"
  | "routes"
  | "offline"
  | "safety"
  | "community";

export interface RoadmapItem {
  id: string;
  client_key: ClientKey;
  title: string;
  summary: string;
  details?: string | null;
  category: RoadmapCategory | string;
  status: RoadmapStatus | string;
  priority: number;
  sort_order: number;
  is_published: boolean;
  vote_count: number;
  subscription_count: number;
  is_voted: boolean;
  is_subscribed: boolean;
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListRoadmapRequest {
  client_key?: ClientKey;
  status?: RoadmapStatus;
  limit?: number;
  cursor?: string;
}

export interface ListRoadmapResponse {
  items: RoadmapItem[];
  next_cursor?: string | null;
}

export interface RoadmapItemRequest {
  client_key: ClientKey;
  title: string;
  summary: string;
  details?: string | null;
  category: RoadmapCategory;
  status: RoadmapStatus;
  priority: number;
  sort_order: number;
  is_published: boolean;
}

export type RoadmapInteractionStatusResponse = RoadmapItem;

export interface ContentListResponse<T> {
  items: T[];
}

export type AppLocale = "zh-CN" | "en";
export type SkillLocale = AppLocale;

export interface SkillCategorySummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  item_count: number;
  href: string;
}

export interface SkillCategoriesResponse {
  items: SkillCategorySummary[];
}

export interface PageInfo {
  limit: number;
  offset: number;
  next_offset?: number | null;
}

export interface KnotTaxonomyItem {
  id: string;
  slug: string;
  title: string;
}

export type KnotMediaAssetId =
  | "thumbnail"
  | "preview"
  | "draw_gif"
  | "turntable_gif"
  | "draw_mp4"
  | "turntable_mp4";

export type KnotMediaType = KnotMediaAssetId;

export interface KnotMediaAsset {
  id: KnotMediaAssetId | string;
  media_type: KnotMediaType | string;
  url: string;
  mime_type: string;
  width?: number | null;
  height?: number | null;
  size_bytes: number;
  attribution?: string | null;
  license_note?: string | null;
}

export interface KnotMediaUploadResponse {
  status: "uploaded";
  knot_id: string;
  media: KnotMediaAsset;
}

export interface KnotSummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  aliases: string[];
  categories: KnotTaxonomyItem[];
  types: KnotTaxonomyItem[];
  media: KnotMediaAsset[];
  href: string;
}

export interface KnotFilterOption {
  id: string;
  slug?: string | null;
  title: string;
  count: number;
}

export interface KnotFiltersResponse {
  locale: SkillLocale;
  categories: KnotFilterOption[];
}

export interface KnotListResponse {
  locale: SkillLocale;
  items: KnotSummary[];
  page: PageInfo;
}

export interface KnotOfflineManifestResponse {
  locale: SkillLocale;
  item_count: number;
  media_count: number;
  estimated_bytes: number;
  items: KnotDetail[];
}

export interface KnotDetail extends Omit<KnotSummary, "href"> {
  href?: string;
  description?: string | null;
  steps: string[];
  locale: SkillLocale;
}

export interface ListKnotsRequest {
  offset?: number;
  limit?: number;
  category?: string;
  q?: string;
}

export interface KnotDisclaimerResponse {
  key: string;
  version: string;
  title: string;
  content: string;
  accepted: boolean;
  accepted_at?: string | null;
}

export interface AcceptKnotDisclaimerRequest {
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
}

export type ApiUsageMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ApiUsageListRequest {
  from?: string;
  to?: string;
  user_id?: string;
  method?: ApiUsageMethod;
  route?: string;
  limit?: number;
  offset?: number;
}

export interface ApiUsageSummary {
  bucket_date: string;
  user_id?: string | null;
  method: ApiUsageMethod | string;
  route_pattern: string;
  status_code: number;
  call_count: number;
  first_called_at: string;
  last_called_at: string;
}

export interface ApiUsageListResponse {
  items: ApiUsageSummary[];
  page: PageInfo;
}

export type AdminRole = "admin" | "super_admin";

export interface AdminUserSelector {
  username?: string | null;
  user_id?: string | null;
}

export interface AdminRoleResponse {
  user_id: string;
  role: AdminRole;
}

export type DeletedFilter = "active" | "deleted" | "all";

export interface UploadImageInfo {
  id: string;
  purpose: string;
  original_filename: string;
  image_type: string;
  content_type: string;
  size_bytes: number;
  sha256: string;
  download_url: string;
  is_deleted: boolean;
  created_at: string;
}

export interface AdminFeedbackUser {
  id: string;
  username?: string | null;
  email?: string | null;
  nickname?: string | null;
  avatar_url?: string | null;
}

export interface AdminFeedbackItem {
  id: string;
  user: AdminFeedbackUser;
  category: string;
  content: string;
  contact?: string | null;
  page?: string | null;
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
  status: string;
  images: UploadImageInfo[];
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface ListAdminFeedbackRequest {
  status?: string;
  deleted?: DeletedFilter;
  limit?: number;
  cursor?: string;
}

export interface ListAdminFeedbackResponse {
  items: AdminFeedbackItem[];
  next_cursor?: string | null;
}

export interface GearTemplateCategory {
  id: string;
  name: string;
  items: string[];
}

export interface GearTemplate {
  id: string;
  title: string;
  categories: GearTemplateCategory[];
}

export type GearCategory =
  | "backpack_system"
  | "sleep_system"
  | "kitchen_system"
  | "walking_system"
  | "clothing_system"
  | "lighting_system"
  | "first_aid_system"
  | "electronics_system"
  | "technical_gear"
  | "other_gear"
  | "consumable";

export type GearStatus =
  | "available"
  | "in_use"
  | "maintenance"
  | "damaged"
  | "lost"
  | "retired"
  | "sold"
  | "idle";

export type GearShareStatus =
  | "not_shared"
  | "pending"
  | "approved"
  | "rejected"
  | "withdrawn";

export type GearTab = "available" | "history";

export type GearSort =
  | "created_at_desc"
  | "created_at_asc"
  | "purchase_date_desc"
  | "name_asc"
  | "weight_desc"
  | "price_desc";

export type GearCurrency = "CNY" | "USD" | "EUR" | "JPY" | "HKD";
export type GearSpecs = Record<string, string>;
export interface GearVariant {
  key: string;
  label: string;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  weight_g?: number | null;
}
export type GearTagColor =
  | "teal"
  | "blue"
  | "violet"
  | "rose"
  | "orange"
  | "amber"
  | "green"
  | "slate";
export type GearTagColorMap = Record<string, GearTagColor | string>;

export interface GearItem {
  id: string;
  user_id: string;
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_location?: string | null;
  status: GearStatus;
  storage_location?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity: number;
  specs?: GearSpecs | null;
  tags: string[];
  tag_colors?: GearTagColorMap | null;
  share_enabled: boolean;
  share_status: GearShareStatus;
  notes?: string | null;
  archived_at?: string | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface GearSummary {
  id: string;
  category: GearCategory;
  category_label: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  status: GearStatus;
  status_label: string;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity: number;
  specs?: GearSpecs | null;
  tags: string[];
  tag_colors?: GearTagColorMap | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateGearRequest {
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_location?: string | null;
  status?: GearStatus | null;
  storage_location?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity?: number | null;
  specs?: GearSpecs | null;
  tags?: string[];
  tag_colors?: GearTagColorMap | null;
  share_enabled?: boolean;
  notes?: string | null;
}

export type UpdateGearRequest = Partial<CreateGearRequest>;

export interface GearCategoryCount {
  category: GearCategory;
  label: string;
  count: number;
}

export interface GearStatusCount {
  status: GearStatus;
  label: string;
  count: number;
}

export interface GearStatsResponse {
  current_count: number;
  archived_count: number;
  total_value_cents: number;
  total_weight_g: number;
  by_category: GearCategoryCount[];
  by_status: GearStatusCount[];
}

export interface GearCategoryFilter {
  id: "all" | GearCategory;
  label: string;
  count: number;
}

export interface GearCategoriesResponse {
  items: GearCategoryFilter[];
}

export interface GearSpecKeyRankingsResponse {
  keys: string[];
}

export interface GearTagSuggestion {
  tag: string;
  color?: GearTagColor | string | null;
}

export interface GearTagSuggestionsResponse {
  items: GearTagSuggestion[];
}

export interface ListGearsRequest {
  tab?: GearTab;
  category?: GearCategory;
  status?: GearStatus;
  deleted?: DeletedFilter;
  q?: string;
  sort?: GearSort;
  limit?: number;
  cursor?: string;
}

export interface ListGearsResponse {
  items: GearSummary[];
  next_cursor?: string | null;
}

export interface GearOverviewRequest {
  tab?: GearTab;
  limit?: number;
  sort?: GearSort;
}

export interface GearOverviewResponse {
  categories: GearCategoriesResponse;
  stats: GearStatsResponse;
  list: ListGearsResponse;
}

export interface GearPackingListStats {
  item_count: number;
  packed_count: number;
  total_weight_g: number;
}

export interface GearPackingListSummary extends GearPackingListStats {
  id: string;
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListGearPackingListsRequest {
  limit?: number;
  cursor?: string;
}

export interface ListGearPackingListsResponse {
  items: GearPackingListSummary[];
  next_cursor?: string | null;
}

export interface CreateGearPackingListRequest {
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
}

export type UpdateGearPackingListRequest = CreateGearPackingListRequest;

export interface AddGearPackingItemsRequest {
  gear_ids: string[];
}

export interface UpdateGearPackingItemRequest {
  packed?: boolean;
  planned_quantity?: number | null;
  packed_quantity?: number | null;
}

export type GearPackingUnavailableReason = "archived" | "deleted" | string;

export interface GearPackingListItem {
  id: string;
  gear_id: string;
  planned_quantity: number;
  packed_quantity: number;
  packed: boolean;
  unavailable: boolean;
  unavailable_reason?: GearPackingUnavailableReason | null;
  gear: GearSummary;
  created_at: string;
  updated_at: string;
}

export interface GearPackingListDetail {
  id: string;
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
  stats: GearPackingListStats;
  items: GearPackingListItem[];
  created_at: string;
  updated_at: string;
}

export type GearAtlasStatus = "pending" | "approved" | "rejected";

export type GearAtlasSourceType = "manual" | "user_gear" | "external_import";

export type GearAtlasSort =
  | "approved_at_desc"
  | "name_asc"
  | "weight_desc"
  | "official_price_desc";

export interface GearAtlasPublicItem {
  id: string;
  category: GearCategory;
  category_label: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  variants?: GearVariant[] | null;
  specs?: GearSpecs | null;
  approved_at?: string | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface GearAtlasReviewChange {
  field: string;
  label: string;
  before?: string | null;
  after?: string | null;
}

export interface GearAtlasSubmission extends GearAtlasPublicItem {
  source_type: GearAtlasSourceType;
  source_user_gear_id?: string | null;
  source_name?: string | null;
  source_url?: string | null;
  source_rating_score?: number | null;
  source_rating_count?: number | null;
  status: GearAtlasStatus;
  rejection_reason?: string | null;
  review_changes?: GearAtlasReviewChange[] | null;
  reviewed_at?: string | null;
}

export interface CreateGearAtlasSubmissionRequest {
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  variants?: GearVariant[] | null;
  specs?: GearSpecs | null;
}

export type UpdateGearAtlasSubmissionRequest = CreateGearAtlasSubmissionRequest;

export interface ListGearAtlasRequest {
  category?: GearCategory;
  q?: string;
  sort?: GearAtlasSort;
  limit?: number;
  cursor?: string;
}

export interface ListGearAtlasResponse {
  items: GearAtlasPublicItem[];
  next_cursor?: string | null;
}

export interface ListGearAtlasSubmissionsRequest {
  status?: GearAtlasStatus;
  category?: GearCategory;
  deleted?: DeletedFilter;
  q?: string;
  limit?: number;
  cursor?: string;
}

export interface ListGearAtlasSubmissionsResponse {
  items: GearAtlasSubmission[];
  next_cursor?: string | null;
}

export interface RejectGearAtlasSubmissionRequest {
  reason: string;
}

export interface WechatLoginRequest {
  code: string;
  profile?: {
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export interface EmailVerificationCodeRequest {
  email: string;
}

export interface EmailVerificationCodeResponse {
  email: string;
  expires_at: string;
  debug_code?: string;
}

export interface EmailLoginCodeRequest {
  email: string;
}

export interface EmailLoginRequest {
  email: string;
  email_verification_code: string;
}

export interface PasswordResetCodeRequest {
  email: string;
}

export interface PasswordResetRequest {
  email: string;
  email_verification_code: string;
  password: string;
  confirm_password: string;
}

export interface BindEmailCodeRequest {
  email: string;
}

export interface BindEmailRequest {
  email: string;
  email_verification_code: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  confirm_password: string;
  email_verification_code: string;
}

export interface CaptchaChallengeRequest {
  account: string;
}

export interface CaptchaChallengeResponse {
  captcha_ticket: string;
  captcha_type: "image";
  image_svg: string;
  expires_at: string;
  debug_answer?: string;
}

export interface PasswordLoginRequest {
  account: string;
  password: string;
  captcha_ticket?: string | null;
  captcha_answer?: string | null;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface AuthUserResponse {
  id: string;
  username?: string | null;
  email?: string | null;
  nickname?: string | null;
  avatar_url?: string | null;
}

export interface WechatLoginResponse {
  access_token: string;
  expires_at: string;
  refresh_token: string;
  refresh_expires_at: string;
  user: AuthUserResponse;
}

export type LoginResponse = WechatLoginResponse;

export interface BindEmailResponse {
  user: AuthUserResponse;
}

export interface ProfileUserResponse {
  user: AuthUserResponse;
}

export interface ImportGearsRequest {
  dry_run?: boolean;
  items: CreateGearRequest[];
}

export interface ImportGearError {
  row: number;
  field: string;
  message: string;
}

export interface ImportGearsResponse {
  created_count: number;
  updated_count: number;
  failed_count: number;
  errors: ImportGearError[];
}
