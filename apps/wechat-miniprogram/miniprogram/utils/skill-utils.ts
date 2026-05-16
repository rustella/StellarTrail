export type SkillLocale = "zh-CN" | "en";

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
  page: {
    limit: number;
    offset: number;
    next_offset?: number | null;
  };
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

export interface SkillCard {
  id: string;
  title: string;
  categoryText: string;
  difficultyText: string;
  difficultyTone: string;
  summary: string;
}

export function mapSkillCard(item: SkillCategorySummary): SkillCard {
  return {
    id: item.id,
    title: item.title,
    categoryText: "户外技能",
    difficultyText: `${item.item_count} 个内容`,
    difficultyTone: item.item_count > 0 ? "success" : "warning",
    summary: item.summary,
  };
}
