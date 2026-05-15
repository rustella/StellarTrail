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

export interface WechatLoginResponse {
  access_token: string;
  expires_at: string;
  user: {
    id: string;
    nickname?: string | null;
    avatar_url?: string | null;
  };
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
