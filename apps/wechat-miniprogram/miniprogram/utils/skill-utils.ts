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

const SKILL_DIFFICULTY_LABELS: Record<string, string> = {
  leisure: "入门",
  beginner: "新手",
  intermediate: "进阶",
  advanced: "高阶",
  technical: "技术",
};

export function getSkillDifficultyLabel(value?: string | null): string {
  if (!value) {
    return "未分级";
  }
  return SKILL_DIFFICULTY_LABELS[value] ?? value;
}

export function getSkillDifficultyTone(value?: string | null): string {
  if (value === "leisure" || value === "beginner" || !value) {
    return "success";
  }
  if (value === "intermediate") {
    return "warning";
  }
  return "danger";
}

export function mapSkillCard(item: KnotSummary): SkillCard {
  return {
    id: item.id,
    title: item.title,
    categoryText: item.categories[0]?.title ?? "绳结",
    difficultyText: getSkillDifficultyLabel(item.difficulty),
    difficultyTone: getSkillDifficultyTone(item.difficulty),
    summary: item.summary,
  };
}
