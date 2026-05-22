import type {
  GearCategory,
  GearSpecs,
  GearVariant,
} from "@stellartrail/shared-types";

export interface GearSpecField {
  key: string;
  label: string;
  placeholder: string;
  inputType?: "text" | "number";
  units?: string[];
  unitLabels?: string[];
  choiceOnly?: boolean;
}

export interface GearSpecFieldView extends GearSpecField {
  valueText: string;
  unitIndex: number;
  unitLabel: string;
  unitLabels: string[];
}

const CAPACITY_UNITS = ["L", "ml", "fl oz"];
const CONSUMABLE_CONTENT_UNITS = ["g", "ml", "kg", "L", "oz"];
const LENGTH_UNITS = ["cm", "m", "mm", "in"];
const BACK_LENGTH_UNITS = ["cm", "in"];
const LOAD_UNITS = ["kg", "g", "lb"];
const TEMPERATURE_UNITS = ["℃", "℉"];
const TIME_UNITS = ["h", "min"];
const DISTANCE_UNITS = ["m", "km"];
const COMMON_WATERPROOF_UNITS = [
  "",
  "IPX4",
  "IPX5",
  "IPX6",
  "IPX7",
  "IPX8",
  "mm",
];

export const GEAR_ATLAS_SPEC_FIELDS: Record<GearCategory, GearSpecField[]> = {
  backpack_system: [
    spec("capacity", "容量", "例如 45", "number", CAPACITY_UNITS),
    spec("recommended_load", "推荐负重", "例如 12", "number", LOAD_UNITS),
    spec("back_length", "背长", "例如 48", "number", BACK_LENGTH_UNITS),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 防泼水",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
  ],
  sleep_system: [
    spec("type", "类型", "例如 睡袋 / 帐篷"),
    spec("people_count", "适用人数", "例如 2", "number", ["人"]),
    spec("temperature_or_r_value", "温标/R 值", "例如 -5 或 4.2", "text", [
      "℃",
      "℉",
      "R",
    ]),
    spec("filling", "填充物", "例如 800FP 羽绒"),
    spec("fill_weight", "填充重量", "例如 700", "number", ["g", "kg", "oz"]),
    spec("material", "材质", "例如 15D 尼龙"),
    spec("packed_size", "收纳尺寸", "例如 18 x 30", "text", LENGTH_UNITS),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 外帐 3000",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
  ],
  kitchen_system: [
    spec("fuel_type", "燃料类型", "例如 气罐"),
    spec("capacity", "容量", "例如 1.2", "number", CAPACITY_UNITS),
    spec("power", "功率", "例如 2600", "number", ["W"]),
    spec("people_count", "适用人数", "例如 2", "number", ["人"]),
    spec("packed_size", "收纳尺寸", "例如 12 x 8", "text", LENGTH_UNITS),
  ],
  walking_system: [
    spec("terrain", "适用地形", "例如 山地 / 泥地"),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 GTX",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
    spec("material", "材质", "例如 铝合金"),
    spec("support", "缓震/支撑", "例如 中等支撑"),
  ],
  clothing_system: [
    spec("layer", "适用层级", "例如 中间层"),
    spec("warmth_rating", "保暖等级", "例如 200", "text", ["g/m²"]),
    spec("waterproof_rating", "防水指数", "例如 20000", "number", ["mm"]),
    spec("breathability_rating", "透湿指数", "例如 15000", "number", [
      "g/m²/24h",
    ]),
    spec("season", "适用季节", "例如 三季"),
  ],
  lighting_system: [
    spec("max_brightness", "最大亮度", "例如 450", "number", ["lm"]),
    spec("runtime", "续航时间", "例如 8", "number", TIME_UNITS),
    spec("battery_type", "电池类型", "例如 18650"),
    spec("charging_port", "充电接口", "例如 USB-C"),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 IPX4",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
    spec("beam_distance", "照射距离", "例如 120", "number", DISTANCE_UNITS),
  ],
  first_aid_system: [
    spec("kit_size", "套装规格", "例如 轻量 12 件"),
    spec("expiry_date", "有效期", "例如 2027-05"),
    spec("people_count", "适用人数", "例如 2", "number", ["人"]),
    spec("days", "适用天数", "例如 3", "number", ["天"]),
    spec("waterproof_packaging", "防水包装", "例如 自封袋"),
  ],
  electronics_system: [
    spec("battery_capacity", "电池容量", "例如 20000", "number", ["mAh", "Wh"]),
    spec("rated_energy", "额定能量", "例如 74", "number", ["Wh"]),
    spec("output_power", "输出功率", "例如 65", "number", ["W"]),
    spec("ports", "接口类型", "例如 USB-C x2"),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 IPX4",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
    spec(
      "working_temperature",
      "工作温度",
      "例如 -10 - 45",
      "text",
      TEMPERATURE_UNITS,
    ),
  ],
  technical_gear: [
    spec("certification", "认证标准", "例如 CE / UIAA"),
    spec("strength", "承重/强度", "例如 22", "number", ["kN", "kg"]),
    spec("specification", "规格", "例如 HMS"),
    spec("length", "长度", "例如 60", "number", LENGTH_UNITS),
    spec("material", "材质", "例如 尼龙"),
    spec("retirement_date", "报废期限", "例如 2030-05"),
  ],
  other_gear: [
    spec("use_case", "用途", "例如 营地收纳"),
    spec("specification", "规格", "例如 大号"),
    spec("capacity", "容量", "例如 10", "number", CAPACITY_UNITS),
    spec(
      "waterproof_rating",
      "防水等级",
      "例如 防泼水",
      "text",
      COMMON_WATERPROOF_UNITS,
    ),
    spec("accessories", "附件", "例如 收纳袋"),
  ],
  consumable: [
    spec("type", "类型", "例如 气罐 / 食品"),
    spec(
      "net_content",
      "净含量",
      "例如 230",
      "number",
      CONSUMABLE_CONTENT_UNITS,
    ),
    spec("quantity", "数量", "例如 2", "number", ["件", "个", "包"]),
    spec("expiry_date", "有效期", "例如 2027-05"),
    spec("storage_condition", "储存条件", "例如 阴凉干燥"),
    spec("restock_threshold", "补货阈值", "例如 1", "number", [
      "件",
      "个",
      "包",
    ]),
  ],
};

export function getGearAtlasSpecFieldViews(
  category: GearCategory,
  specs: GearSpecs = {},
): GearSpecFieldView[] {
  return (GEAR_ATLAS_SPEC_FIELDS[category] ?? []).map((field) => {
    const parsed = splitSpecValue(specs[field.key] ?? "", field.units);
    return {
      ...field,
      inputType: field.inputType ?? "text",
      valueText: parsed.valueText,
      unitIndex: parsed.unitIndex,
      unitLabel: field.units?.[parsed.unitIndex] ?? "",
      unitLabels: field.unitLabels ?? field.units ?? [],
    };
  });
}

export function combineSpecValue(value?: string, unit?: string): string {
  const text = value?.trim() ?? "";
  const unitText = unit?.trim() ?? "";
  if (!text) return unitText;
  if (!unitText) return text;
  return `${text} ${unitText}`;
}

export function normalizeSpecsForCategory(
  category: GearCategory,
  specs: GearSpecs,
): GearSpecs {
  const fields = new Set(
    (GEAR_ATLAS_SPEC_FIELDS[category] ?? []).map((field) => field.key),
  );
  const normalized: GearSpecs = {};
  Object.entries(specs).forEach(([key, value]) => {
    if (!fields.has(key)) return;
    const text = value.trim();
    if (text) normalized[key] = text;
  });
  return normalized;
}

export function specLabel(category: GearCategory, key: string): string {
  const categoryField = GEAR_ATLAS_SPEC_FIELDS[category]?.find(
    (field) => field.key === key,
  );
  if (categoryField) return categoryField.label;

  for (const fields of Object.values(GEAR_ATLAS_SPEC_FIELDS)) {
    const field = fields.find((item) => item.key === key);
    if (field) return field.label;
  }

  return EXTRA_SPEC_LABELS[key] ?? key;
}

const EXTRA_SPEC_LABELS: Record<string, string> = {
  fill_weight: "填充重量",
  material: "材质",
};

export function normalizeVariants(
  values?: GearVariant[] | null,
): GearVariant[] {
  const normalized: GearVariant[] = [];
  (values ?? []).forEach((variant, index) => {
    const label = variant.label?.trim();
    if (!label) return;
    const key = variant.key?.trim() || variantKeyFromLabel(label, index);
    if (normalized.some((existing) => existing.key === key)) return;
    normalized.push({
      key,
      label,
      official_price_cents: variant.official_price_cents ?? null,
      official_price_currency: variant.official_price_currency ?? null,
      weight_g: variant.weight_g ?? null,
    });
  });
  return normalized;
}

export function variantKeyFromLabel(label: string, index = 0): string {
  const key = label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80);
  return key || `variant-${index}`;
}

export function variantSummary(
  variant: GearVariant,
  formatPrice: (cents?: number | null, currency?: string | null) => string,
): string {
  const details = [
    formatPrice(variant.official_price_cents, variant.official_price_currency),
    variant.weight_g ? `${variant.weight_g} g` : "",
  ].filter((value) => value && value !== "—");
  return details.length
    ? `${variant.label} · ${details.join(" · ")}`
    : variant.label;
}

function spec(
  key: string,
  label: string,
  placeholder: string,
  inputType: "text" | "number" = "text",
  units?: string[],
  extra: Partial<GearSpecField> = {},
): GearSpecField {
  return { key, label, placeholder, inputType, units, ...extra };
}

function splitSpecValue(
  value: string,
  units?: string[],
): { valueText: string; unitIndex: number } {
  const text = value.trim();
  if (!units?.length || !text) {
    return { valueText: text, unitIndex: 0 };
  }
  const matchedIndex = units.findIndex(
    (unit) => unit && (text === unit || text.endsWith(` ${unit}`)),
  );
  if (matchedIndex < 0) {
    return { valueText: text, unitIndex: 0 };
  }
  const unit = units[matchedIndex];
  const valueText = text === unit ? "" : text.slice(0, -unit.length).trim();
  return { valueText, unitIndex: matchedIndex };
}
