export type SkillCategory =
  | "knot"
  | "camping"
  | "first_aid"
  | "packing"
  | "navigation"
  | "weather";

export type SkillDifficulty =
  | "leisure"
  | "beginner"
  | "intermediate"
  | "advanced"
  | "technical";

export interface SkillContent {
  id: string;
  title: string;
  category: SkillCategory;
  difficulty_level: SkillDifficulty;
  summary: string;
  related_gear_categories: string[];
  body_markdown: string;
}

export interface ListSkillsResponse {
  items: SkillContent[];
}

export interface SkillCard {
  id: string;
  title: string;
  categoryText: string;
  difficultyText: string;
  difficultyTone: string;
  summary: string;
}

const SKILL_CATEGORY_LABELS: Record<SkillCategory, string> = {
  knot: "绳结",
  camping: "扎营",
  first_aid: "急救",
  packing: "打包",
  navigation: "导航技能",
  weather: "天气",
};

const SKILL_DIFFICULTY_LABELS: Record<SkillDifficulty, string> = {
  leisure: "入门",
  beginner: "新手",
  intermediate: "进阶",
  advanced: "高阶",
  technical: "技术",
};

export function getSkillCategoryLabel(value: string): string {
  return SKILL_CATEGORY_LABELS[value as SkillCategory] ?? value;
}

export function getSkillDifficultyLabel(value: string): string {
  return SKILL_DIFFICULTY_LABELS[value as SkillDifficulty] ?? value;
}

export function getSkillDifficultyTone(value: string): string {
  if (value === "leisure" || value === "beginner") {
    return "success";
  }
  if (value === "intermediate") {
    return "warning";
  }
  return "danger";
}

export function mapSkillCard(item: SkillContent): SkillCard {
  return {
    id: item.id,
    title: item.title,
    categoryText: getSkillCategoryLabel(item.category),
    difficultyText: getSkillDifficultyLabel(item.difficulty_level),
    difficultyTone: getSkillDifficultyTone(item.difficulty_level),
    summary: item.summary,
  };
}
