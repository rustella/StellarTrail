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

export type AppLocale = "zh-CN" | "en";

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
  next_offset: number | null;
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
  locale: AppLocale;
  items: KnotSummary[];
  page: PageInfo;
}

export interface KnotDetail {
  id: string;
  slug: string;
  title: string;
  summary: string;
  description?: string | null;
  steps: string[];
  difficulty?: string | null;
  categories: KnotTaxonomyItem[];
  types: KnotTaxonomyItem[];
  media: KnotMediaAsset[];
  locale: AppLocale;
}
