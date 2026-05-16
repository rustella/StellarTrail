export interface HealthResponse {
  status: "ok";
}

export interface MetaResponse {
  name: "StellarTrail";
  env: string;
  database_kind: "sqlite" | "postgres" | "mysql";
}

export type DifficultyLevel =
  | "leisure"
  | "beginner"
  | "intermediate"
  | "advanced"
  | "technical";

export interface ContentListResponse<T> {
  items: T[];
}

export interface MountainContent {
  id: string;
  name: string;
  aliases: string[];
  province: string;
  city?: string | null;
  area?: string | null;
  elevation_m?: number | null;
  lat?: number | null;
  lng?: number | null;
  summary: string;
  best_seasons: string[];
  difficulty_level: DifficultyLevel;
  status: string;
}

export type RoutePointType =
  | "start"
  | "end"
  | "camp"
  | "water"
  | "supply"
  | "danger"
  | "viewpoint"
  | "exit";

export interface RoutePoint {
  type: RoutePointType;
  name: string;
  description?: string | null;
  sort_order: number;
}

export interface RouteGearSuggestion {
  gear_category: string;
  gear_name: string;
  required_level: string;
  reason?: string | null;
}

export interface RouteSkillLink {
  skill_id: string;
  reason?: string | null;
}

export interface RouteContent extends RouteSummary {
  mountain_id?: string | null;
  city?: string | null;
  route_type: string;
  descent_m?: number | null;
  transport_info?: string | null;
  permit_info?: string | null;
  risk_summary?: string | null;
  status: string;
  points: RoutePoint[];
  gear_suggestions: RouteGearSuggestion[];
  skill_links: RouteSkillLink[];
}

export type SkillCategory =
  | "knot"
  | "camping"
  | "first_aid"
  | "packing"
  | "navigation"
  | "weather";

export interface SkillContent {
  id: string;
  title: string;
  category: SkillCategory;
  difficulty_level: DifficultyLevel;
  summary: string;
  related_gear_categories: string[];
  body_markdown: string;
}

export type SkillLocale = "zh-CN" | "en";

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

export interface KnotMediaAsset {
  id: string;
  media_type: string;
  url: string;
  mime_type: string;
  width?: number | null;
  height?: number | null;
  attribution?: string | null;
  license_note?: string | null;
}

export interface KnotSummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  difficulty?: string | null;
  categories: KnotTaxonomyItem[];
  types: KnotTaxonomyItem[];
  media: KnotMediaAsset[];
  href: string;
}

export interface KnotListResponse {
  locale: SkillLocale;
  items: KnotSummary[];
  page: PageInfo;
}

export interface KnotDetail extends KnotSummary {
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

export interface RouteSummary {
  id: string;
  title: string;
  province: string;
  difficulty_level: DifficultyLevel;
  distance_m?: number;
  ascent_m?: number;
  duration_min?: number;
  best_seasons: string[];
  summary: string;
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

export interface GearItem {
  id: string;
  user_id: string;
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  color?: string | null;
  material?: string | null;
  capacity?: string | null;
  size?: string | null;
  description?: string | null;
  weight_g?: number | null;
  warmth_index?: string | null;
  waterproof_index?: string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  expiry_or_warranty_date?: string | null;
  purchase_location?: string | null;
  status: GearStatus;
  storage_location?: string | null;
  tags: string[];
  share_enabled: boolean;
  share_status: GearShareStatus;
  notes?: string | null;
  archived_at?: string | null;
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
  purchase_price_cents?: number | null;
  purchase_date?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateGearRequest {
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  color?: string | null;
  material?: string | null;
  capacity?: string | null;
  size?: string | null;
  description?: string | null;
  weight_g?: number | null;
  warmth_index?: string | null;
  waterproof_index?: string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  expiry_or_warranty_date?: string | null;
  purchase_location?: string | null;
  status?: GearStatus | null;
  storage_location?: string | null;
  tags?: string[];
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

export interface ListGearsRequest {
  tab?: GearTab;
  category?: GearCategory;
  status?: GearStatus;
  q?: string;
  sort?: GearSort;
  limit?: number;
  cursor?: string;
}

export interface ListGearsResponse {
  items: GearSummary[];
  next_cursor?: string | null;
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

export interface WechatLoginResponse {
  access_token: string;
  expires_at: string;
  user: {
    id: string;
    username?: string | null;
    email?: string | null;
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export type LoginResponse = WechatLoginResponse;

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
