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
export type GearWeightUnit = "kg" | "g" | "lb" | "oz";
export type GearTagColor =
  | "teal"
  | "blue"
  | "violet"
  | "rose"
  | "orange"
  | "amber"
  | "green"
  | "slate";
export type GearSpecs = Record<string, string>;
export type GearTagColorMap = Record<string, GearTagColor | string>;

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

export interface GearTagView {
  name: string;
  color: GearTagColor;
  colorClass: string;
}

export interface GearTagSuggestion {
  tag: string;
  color?: GearTagColor | string | null;
}

export interface GearTagSuggestionView {
  name: string;
  color: GearTagColor;
  colorClass: string;
}

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
  specs?: GearSpecs | null;
  tags: string[];
  tag_colors?: GearTagColorMap | null;
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
  category_label?: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  status: GearStatus;
  status_label?: string;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  specs?: GearSpecs | null;
  tags?: string[];
  tag_colors?: GearTagColorMap | null;
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
  specs?: GearSpecs | null;
  tags?: string[];
  tag_colors?: GearTagColorMap | null;
  share_enabled?: boolean;
  notes?: string | null;
}

export type UpdateGearRequest = Partial<CreateGearRequest>;

export interface GearStatsResponse {
  current_count: number;
  archived_count: number;
  total_value_cents: number;
  total_weight_g: number;
  by_category: Array<{ category: GearCategory; label: string; count: number }>;
  by_status: Array<{ status: GearStatus; label: string; count: number }>;
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

export interface GearTagSuggestionsResponse {
  items: GearTagSuggestion[];
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

export interface ListGearTemplatesResponse {
  items: GearTemplate[];
}

export interface WechatLoginResponse {
  access_token: string;
  expires_at: string;
  refresh_token: string;
  refresh_expires_at: string;
  user: {
    id: string;
    username?: string | null;
    email?: string | null;
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export interface OptionItem<T extends string = string> {
  value: T;
  label: string;
  hint?: string;
}

export interface GearFormData {
  category: GearCategory;
  name: string;
  brand: string;
  model: string;
  description: string;
  weightText: string;
  weightUnit: GearWeightUnit;
  purchaseDate: string;
  officialPriceText: string;
  officialPriceCurrency: GearCurrency;
  purchasePriceText: string;
  purchasePriceCurrency: GearCurrency;
  purchaseLocation: string;
  status: GearStatus;
  storageLocation: string;
  specs: GearSpecs;
  tags: GearTagView[];
  shareEnabled: boolean;
  notes: string;
}

export const GEAR_CATEGORY_OPTIONS: Array<OptionItem<GearCategory>> = [
  { value: "backpack_system", label: "背负系统", hint: "背包、外挂、收纳" },
  { value: "sleep_system", label: "睡眠系统", hint: "帐篷、睡袋、防潮垫" },
  { value: "kitchen_system", label: "炊具系统", hint: "炉具、锅具、餐具" },
  { value: "walking_system", label: "行走系统", hint: "登山杖、鞋袜、护具" },
  { value: "clothing_system", label: "服装系统", hint: "冲锋衣、保暖、换洗" },
  { value: "lighting_system", label: "照明系统", hint: "头灯、营灯、电池" },
  { value: "first_aid_system", label: "急救系统", hint: "药包、绷带、应急毯" },
  { value: "electronics_system", label: "电子系统", hint: "电源、导航、通信" },
  { value: "technical_gear", label: "技术装备", hint: "冰雪、攀登、安全" },
  { value: "other_gear", label: "其他装备", hint: "杂项与个性化装备" },
  { value: "consumable", label: "消耗品", hint: "气罐、食品、一次性用品" },
];

export const GEAR_STATUS_OPTIONS: Array<OptionItem<GearStatus>> = [
  { value: "available", label: "可用" },
  { value: "in_use", label: "使用中" },
  { value: "maintenance", label: "保养中" },
  { value: "damaged", label: "损坏" },
  { value: "lost", label: "丢失" },
  { value: "retired", label: "退役" },
  { value: "sold", label: "已售出" },
  { value: "idle", label: "闲置" },
];

export const GEAR_STATUS_FILTER_OPTIONS: Array<OptionItem<"" | GearStatus>> = [
  { value: "", label: "全部状态" },
  ...GEAR_STATUS_OPTIONS,
];

export const GEAR_SORT_OPTIONS: Array<OptionItem<GearSort>> = [
  { value: "created_at_desc", label: "最近添加" },
  { value: "created_at_asc", label: "最早添加" },
  { value: "purchase_date_desc", label: "最近购买" },
  { value: "name_asc", label: "名称 A-Z" },
  { value: "weight_desc", label: "重量优先" },
  { value: "price_desc", label: "价格优先" },
];

export const GEAR_CURRENCY_OPTIONS: Array<OptionItem<GearCurrency>> = [
  { value: "CNY", label: "¥ CNY" },
  { value: "USD", label: "USD" },
  { value: "EUR", label: "EUR" },
  { value: "JPY", label: "JPY" },
  { value: "HKD", label: "HKD" },
];

export const GEAR_WEIGHT_UNIT_OPTIONS: Array<OptionItem<GearWeightUnit>> = [
  { value: "kg", label: "kg" },
  { value: "g", label: "g" },
  { value: "lb", label: "lb" },
  { value: "oz", label: "oz" },
];

export const GEAR_TAG_COLOR_OPTIONS: Array<OptionItem<GearTagColor>> = [
  { value: "teal", label: "青绿" },
  { value: "blue", label: "蓝" },
  { value: "violet", label: "紫" },
  { value: "rose", label: "粉" },
  { value: "orange", label: "橙" },
  { value: "amber", label: "黄" },
  { value: "green", label: "绿" },
  { value: "slate", label: "灰" },
];

export const PURCHASE_LOCATION_OPTIONS = [
  "京东",
  "淘宝",
  "天猫",
  "拼多多",
  "亚马逊",
  "闲鱼",
  "迪卡侬",
  "三夫户外",
  "REI",
  "Backcountry",
  "Moosejaw",
  "Campsaver",
  "品牌官网",
  "品牌门店",
  "线下户外店",
  "朋友赠送",
  "其他",
];

export const GEAR_TAB_OPTIONS: Array<OptionItem<GearTab>> = [
  { value: "available", label: "可用装备" },
  { value: "history", label: "历史装备" },
];

const CAPACITY_UNITS = ["L", "ml", "fl oz"];
const CONSUMABLE_CONTENT_UNITS = ["g", "ml", "kg", "L", "oz"];
const LENGTH_UNITS = ["cm", "m", "mm", "in"];
const BACK_LENGTH_UNITS = ["cm", "in"];
const BACKPACK_SIZE_UNITS = ["", "XS", "S", "M", "L", "XL", "XXL", "均码"];
const BACKPACK_SIZE_UNIT_LABELS = [
  "选择尺码",
  "XS",
  "S",
  "M",
  "L",
  "XL",
  "XXL",
  "均码",
];
const SHOE_SIZE_OR_LENGTH_UNITS = ["cm", "EU", "US", "UK", "in"];
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

export const GEAR_SPEC_FIELDS: Record<GearCategory, GearSpecField[]> = {
  backpack_system: [
    spec("capacity", "容量", "例如 45", "number", CAPACITY_UNITS),
    spec("recommended_load", "推荐负重", "例如 12", "number", LOAD_UNITS),
    spec("back_length", "背长", "例如 48", "number", BACK_LENGTH_UNITS),
    spec(
      "backpack_size",
      "尺码",
      "选择 XS / S / M / L",
      "text",
      BACKPACK_SIZE_UNITS,
      {
        choiceOnly: true,
        unitLabels: BACKPACK_SIZE_UNIT_LABELS,
      },
    ),
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
    spec(
      "size_or_length",
      "尺码/长度",
      "例如 42 或 120",
      "text",
      SHOE_SIZE_OR_LENGTH_UNITS,
    ),
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
    spec("size", "尺码", "例如 M"),
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

export function createDefaultGearFormData(): GearFormData {
  return {
    category: "backpack_system",
    name: "",
    brand: "",
    model: "",
    description: "",
    weightText: "",
    weightUnit: "kg",
    purchaseDate: "",
    officialPriceText: "",
    officialPriceCurrency: "CNY",
    purchasePriceText: "",
    purchasePriceCurrency: "CNY",
    purchaseLocation: "",
    status: "available",
    storageLocation: "",
    specs: {},
    tags: [],
    shareEnabled: false,
    notes: "",
  };
}

export function gearToFormData(item: GearItem): GearFormData {
  return {
    category: item.category,
    name: item.name,
    brand: item.brand ?? "",
    model: item.model ?? "",
    description: item.description ?? "",
    weightText:
      item.weight_g === undefined || item.weight_g === null
        ? ""
        : trimTrailingZeros(String(item.weight_g / 1000)),
    weightUnit: "kg",
    purchaseDate: item.purchase_date ?? "",
    officialPriceText: priceFromMinorUnits(
      item.official_price_cents,
      item.official_price_currency,
    ),
    officialPriceCurrency: normalizeCurrency(item.official_price_currency),
    purchasePriceText: priceFromMinorUnits(
      item.purchase_price_cents,
      item.purchase_price_currency,
    ),
    purchasePriceCurrency: normalizeCurrency(item.purchase_price_currency),
    purchaseLocation: item.purchase_location ?? "",
    status: item.status,
    storageLocation: item.storage_location ?? "",
    specs: { ...(item.specs ?? {}) },
    tags: createGearTagViews(item.tags ?? [], item.tag_colors ?? {}),
    shareEnabled: item.share_enabled,
    notes: item.notes ?? "",
  };
}

export function buildGearPayload(
  form: Partial<GearFormData> & Pick<GearFormData, "category" | "name">,
): CreateGearRequest {
  const name = requiredText(form.name, "装备名称不能为空");
  if (!form.category) {
    throw new Error("请选择装备分类");
  }

  return {
    category: form.category,
    name,
    brand: nullableText(form.brand),
    model: nullableText(form.model),
    description: nullableText(form.description),
    weight_g: weightToGrams(form.weightText, form.weightUnit),
    purchase_date: nullableText(form.purchaseDate),
    official_price_cents: priceToMinorUnits(
      form.officialPriceText,
      form.officialPriceCurrency,
    ),
    official_price_currency: currencyForPrice(
      form.officialPriceText,
      form.officialPriceCurrency,
    ),
    purchase_price_cents: priceToMinorUnits(
      form.purchasePriceText,
      form.purchasePriceCurrency,
    ),
    purchase_price_currency: currencyForPrice(
      form.purchasePriceText,
      form.purchasePriceCurrency,
    ),
    purchase_location: nullableText(form.purchaseLocation),
    status: form.status ?? "available",
    storage_location: nullableText(form.storageLocation),
    specs: normalizeSpecsForCategory(form.category, form.specs ?? {}),
    tags: normalizeGearTagViews(form.tags ?? []).map((tag) => tag.name),
    tag_colors: tagColorPayload(form.tags ?? []),
    share_enabled: Boolean(form.shareEnabled),
    notes: nullableText(form.notes),
  };
}

export function createGearTagViews(
  tags: string[],
  colors: GearTagColorMap = {},
): GearTagView[] {
  const seen = new Set<string>();
  const views: GearTagView[] = [];
  tags.forEach((raw) => {
    const name = raw.trim();
    if (!name || seen.has(name)) {
      return;
    }
    seen.add(name);
    const color =
      normalizeGearTagColor(colors[name]) ?? fallbackGearTagColor(name);
    views.push({
      name,
      color,
      colorClass: gearTagColorClass(color),
    });
  });
  return views.slice(0, 20);
}

export function createGearTagSuggestionViews(
  items: GearTagSuggestion[],
): GearTagSuggestionView[] {
  return items
    .map((item) => {
      const name = item.tag.trim();
      if (!name) {
        return null;
      }
      const color =
        normalizeGearTagColor(item.color) ?? fallbackGearTagColor(name);
      return {
        name,
        color,
        colorClass: gearTagColorClass(color),
      };
    })
    .filter((item): item is GearTagSuggestionView => Boolean(item));
}

export function addGearTagViews(
  current: GearTagView[],
  input: string,
  color?: GearTagColor | null,
): GearTagView[] {
  const existing = normalizeGearTagViews(current);
  const seen = new Set(existing.map((tag) => tag.name));
  parseTagsInput(input).forEach((name) => {
    if (seen.has(name) || existing.length >= 20) {
      return;
    }
    const normalizedColor = color ?? randomGearTagColor();
    existing.push({
      name,
      color: normalizedColor,
      colorClass: gearTagColorClass(normalizedColor),
    });
    seen.add(name);
  });
  return existing;
}

export function normalizeGearTagViews(tags: GearTagView[]): GearTagView[] {
  const seen = new Set<string>();
  const normalized: GearTagView[] = [];
  tags.forEach((tag) => {
    const name = tag.name.trim();
    if (!name || seen.has(name)) {
      return;
    }
    const color =
      normalizeGearTagColor(tag.color) ?? fallbackGearTagColor(name);
    normalized.push({
      name,
      color,
      colorClass: gearTagColorClass(color),
    });
    seen.add(name);
  });
  return normalized.slice(0, 20);
}

export function normalizeGearTagColor(
  value?: GearTagColor | string | null,
): GearTagColor | null {
  if (!value) {
    return null;
  }
  const normalized = value.trim() as GearTagColor;
  return GEAR_TAG_COLOR_OPTIONS.some((item) => item.value === normalized)
    ? normalized
    : null;
}

export function randomGearTagColor(): GearTagColor {
  const index = Math.floor(Math.random() * GEAR_TAG_COLOR_OPTIONS.length);
  return GEAR_TAG_COLOR_OPTIONS[index]?.value ?? "teal";
}

export function fallbackGearTagColor(tag: string): GearTagColor {
  const options = GEAR_TAG_COLOR_OPTIONS.map((item) => item.value);
  let hash = 0;
  for (const char of tag) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return options[hash % options.length] ?? "teal";
}

export function gearTagColorClass(color: GearTagColor): string {
  return `tag-color-${color}`;
}

export function parseTagsInput(input: string): string[] {
  const seen = new Set<string>();
  const tags: string[] = [];
  input
    .split(/[,，;；\n]/)
    .map((item) => item.trim())
    .filter(Boolean)
    .forEach((item) => {
      if (!seen.has(item)) {
        seen.add(item);
        tags.push(item);
      }
    });
  return tags;
}

function tagColorPayload(tags: GearTagView[]): GearTagColorMap | null {
  const colors: GearTagColorMap = {};
  normalizeGearTagViews(tags).forEach((tag) => {
    colors[tag.name] = tag.color;
  });
  return Object.keys(colors).length ? colors : null;
}

export function formatGearWeight(value?: number | null): string {
  if (value === undefined || value === null) {
    return "未记录";
  }
  if (value >= 1000) {
    return `${trimTrailingZeros((value / 1000).toFixed(2))} kg`;
  }
  return `${value} g`;
}

export function formatGearPrice(
  value?: number | null,
  currency?: string | null,
): string {
  if (value === undefined || value === null) {
    return "未记录";
  }
  const normalized = normalizeCurrency(currency);
  const amount =
    normalized === "JPY"
      ? String(value)
      : value % 100 === 0
        ? String(Math.trunc(value / 100))
        : (value / 100).toFixed(2);
  return normalized === "CNY" ? `¥${amount}` : `${normalized} ${amount}`;
}

export function formatDateText(value?: string | null): string {
  if (!value) {
    return "未记录";
  }
  return value.slice(0, 10);
}

export function getGearCategoryLabel(value: GearCategory): string {
  return findLabel(GEAR_CATEGORY_OPTIONS, value);
}

export function getGearStatusLabel(value: GearStatus): string {
  return findLabel(GEAR_STATUS_OPTIONS, value);
}

export function getGearShareStatusLabel(value: GearShareStatus): string {
  const labels: Record<GearShareStatus, string> = {
    not_shared: "未共享",
    pending: "审核中",
    approved: "已共享",
    rejected: "已拒绝",
    withdrawn: "已撤回",
  };
  return labels[value] ?? value;
}

export function getStatusTone(value: GearStatus): string {
  if (value === "available") {
    return "success";
  }
  if (value === "in_use" || value === "maintenance" || value === "idle") {
    return "warning";
  }
  return "muted";
}

export function optionIndex<T extends string>(
  options: Array<OptionItem<T>>,
  value: T,
): number {
  const index = options.findIndex((item) => item.value === value);
  return index >= 0 ? index : 0;
}

export function categoryFilterItems(
  apiItems: GearCategoryFilter[],
): GearCategoryFilter[] {
  if (apiItems.length > 0) {
    return apiItems;
  }
  return [{ id: "all", label: "全部装备", count: 0 }];
}

export function valueOrUnset(value?: string | number | null): string {
  if (value === undefined || value === null || value === "") {
    return "未记录";
  }
  return String(value);
}

export function getGearSpecFieldViews(
  category: GearCategory,
  specs: GearSpecs = {},
  rankedKeys: string[] = [],
): GearSpecFieldView[] {
  return rankSpecFields(GEAR_SPEC_FIELDS[category] ?? [], rankedKeys).map(
    (field) => {
      const parsed = splitSpecValue(specs[field.key] ?? "", field.units);
      return {
        ...field,
        inputType: field.inputType ?? "text",
        valueText: parsed.valueText,
        unitIndex: parsed.unitIndex,
        unitLabel: field.units?.[parsed.unitIndex] ?? "",
        unitLabels: field.unitLabels ?? field.units ?? [],
      };
    },
  );
}

export function combineSpecValue(value?: string, unit?: string): string {
  const text = value?.trim() ?? "";
  const unitText = unit?.trim() ?? "";
  if (!text) {
    return unitText;
  }
  if (!unitText) {
    return text;
  }
  return `${text} ${unitText}`;
}

export function normalizeSpecsForCategory(
  category: GearCategory,
  specs: GearSpecs,
): GearSpecs {
  const fields = new Set(
    (GEAR_SPEC_FIELDS[category] ?? []).map((field) => field.key),
  );
  const normalized: GearSpecs = {};
  Object.entries(specs).forEach(([key, value]) => {
    if (!fields.has(key)) {
      return;
    }
    const text = value.trim();
    if (text) {
      normalized[key] = text;
    }
  });
  return normalized;
}

function findLabel<T extends string>(
  options: Array<OptionItem<T>>,
  value: T,
): string {
  return options.find((item) => item.value === value)?.label ?? value;
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

function rankSpecFields(
  fields: GearSpecField[],
  rankedKeys: string[],
): GearSpecField[] {
  if (rankedKeys.length === 0) {
    return fields;
  }
  const fieldsByKey = new Map(fields.map((field) => [field.key, field]));
  const used = new Set<string>();
  const rankedFields: GearSpecField[] = [];
  rankedKeys.forEach((key) => {
    if (used.has(key)) {
      return;
    }
    const field = fieldsByKey.get(key);
    if (!field) {
      return;
    }
    used.add(key);
    rankedFields.push(field);
  });
  fields.forEach((field) => {
    if (!used.has(field.key)) {
      rankedFields.push(field);
    }
  });
  return rankedFields;
}

function nullableText(value?: string): string | null {
  const text = value?.trim() ?? "";
  return text.length > 0 ? text : null;
}

function requiredText(value: string | undefined, message: string): string {
  const text = value?.trim() ?? "";
  if (!text) {
    throw new Error(message);
  }
  return text;
}

function weightToGrams(value?: string, unit?: GearWeightUnit): number | null {
  const text = value?.trim() ?? "";
  if (!text) {
    return null;
  }
  const numberValue = Number(text);
  if (Number.isNaN(numberValue)) {
    throw new Error("重量必须是数字");
  }
  if (numberValue < 0) {
    throw new Error("重量不能为负数");
  }
  switch (unit ?? "kg") {
    case "g":
      return Math.round(numberValue);
    case "lb":
      return Math.round(numberValue * 453.59237);
    case "oz":
      return Math.round(numberValue * 28.349523125);
    case "kg":
    default:
      return Math.round(numberValue * 1000);
  }
}

function priceToMinorUnits(
  value?: string,
  currency?: string | null,
): number | null {
  const text = value?.trim() ?? "";
  if (!text) {
    return null;
  }
  const numberValue = Number(text);
  if (Number.isNaN(numberValue)) {
    throw new Error("价格必须是数字");
  }
  if (numberValue < 0) {
    throw new Error("价格不能为负数");
  }
  return normalizeCurrency(currency) === "JPY"
    ? Math.round(numberValue)
    : Math.round(numberValue * 100);
}

function priceFromMinorUnits(
  value?: number | null,
  currency?: string | null,
): string {
  if (value === undefined || value === null) {
    return "";
  }
  if (normalizeCurrency(currency) === "JPY") {
    return String(value);
  }
  return trimTrailingZeros(String(value / 100));
}

function currencyForPrice(
  value?: string,
  currency?: string | null,
): GearCurrency | null {
  return value?.trim() ? normalizeCurrency(currency) : null;
}

function normalizeCurrency(value?: string | null): GearCurrency {
  const normalized = value?.trim().toUpperCase();
  const option = GEAR_CURRENCY_OPTIONS.find(
    (item) => item.value === normalized,
  );
  return option?.value ?? "CNY";
}

function splitSpecValue(
  value: string,
  units?: string[],
): { valueText: string; unitIndex: number } {
  const text = value.trim();
  if (!units || units.length === 0) {
    return { valueText: text, unitIndex: 0 };
  }
  const nonEmptyUnits = units
    .filter(Boolean)
    .sort((a, b) => b.length - a.length);
  const matched = nonEmptyUnits.find(
    (unit) => text === unit || text.endsWith(` ${unit}`),
  );
  if (!matched) {
    return { valueText: text, unitIndex: 0 };
  }
  return {
    valueText: text === matched ? "" : text.slice(0, -matched.length).trim(),
    unitIndex: units.indexOf(matched),
  };
}

function trimTrailingZeros(value: string): string {
  return value.replace(/\.0+$/, "").replace(/(\.\d*?)0+$/, "$1");
}
