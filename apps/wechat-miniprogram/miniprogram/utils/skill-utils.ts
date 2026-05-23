export type SkillLocale = "zh-CN" | "en";

export type SkillDifficulty =
  | "leisure"
  | "beginner"
  | "intermediate"
  | "advanced"
  | "technical";

export interface SkillCategorySummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  item_count: number;
  href: string;
}

export interface ListSkillsResponse {
  items: SkillCategorySummary[];
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
  size_bytes: number;
  attribution?: string | null;
  license_note?: string | null;
}

export interface KnotSummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  categories: KnotTaxonomyItem[];
  types: KnotTaxonomyItem[];
  media: KnotMediaAsset[];
  href: string;
}

export interface KnotListResponse {
  locale: SkillLocale;
  items: KnotSummary[];
  page: {
    limit: number;
    offset: number;
    next_offset?: number | null;
  };
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

export type FavoriteSkillCategory = "all" | "knots";

export interface FavoriteSkillFilterOption {
  id: FavoriteSkillCategory;
  title: string;
  count: number;
}

export interface FavoriteKnotItem {
  skill_category: "knots";
  favorited_at: string;
  knot: KnotSummary;
}

export interface ListFavoriteSkillsResponse {
  locale: SkillLocale;
  filters: FavoriteSkillFilterOption[];
  items: FavoriteKnotItem[];
  page: {
    limit: number;
    offset: number;
    next_offset?: number | null;
  };
}

export interface FavoriteKnotStatusResponse {
  skill_category: "knots";
  knot_id: string;
  is_favorited: boolean;
  favorited_at?: string | null;
}

export interface ListFavoriteSkillsRequest {
  skill_category?: FavoriteSkillCategory;
  offset?: number;
  limit?: number;
}

export interface SkillCard {
  id: string;
  title: string;
  categoryText: string;
  summary: string;
}

export function mapSkillCard(
  item: SkillCategorySummary | KnotSummary | KnotDetail,
): SkillCard {
  const isKnot = "media" in item;
  return {
    id: item.id,
    title: item.title,
    categoryText: isKnot ? (item.categories[0]?.title ?? "绳结") : "户外技能",
    summary: item.summary,
  };
}
