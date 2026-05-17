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
  category_label?: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  status: GearStatus;
  status_label?: string;
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
  color: string;
  material: string;
  capacity: string;
  size: string;
  description: string;
  weightText: string;
  warmthIndex: string;
  waterproofIndex: string;
  purchaseDate: string;
  purchasePriceText: string;
  expiryOrWarrantyDate: string;
  purchaseLocation: string;
  status: GearStatus;
  storageLocation: string;
  tagsText: string;
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

export const GEAR_TAB_OPTIONS: Array<OptionItem<GearTab>> = [
  { value: "available", label: "可用装备" },
  { value: "history", label: "历史装备" },
];

export function createDefaultGearFormData(): GearFormData {
  return {
    category: "backpack_system",
    name: "",
    brand: "",
    model: "",
    color: "",
    material: "",
    capacity: "",
    size: "",
    description: "",
    weightText: "",
    warmthIndex: "",
    waterproofIndex: "",
    purchaseDate: "",
    purchasePriceText: "",
    expiryOrWarrantyDate: "",
    purchaseLocation: "",
    status: "available",
    storageLocation: "",
    tagsText: "",
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
    color: item.color ?? "",
    material: item.material ?? "",
    capacity: item.capacity ?? "",
    size: item.size ?? "",
    description: item.description ?? "",
    weightText:
      item.weight_g === undefined || item.weight_g === null
        ? ""
        : trimTrailingZeros(String(item.weight_g / 1000)),
    warmthIndex: item.warmth_index ?? "",
    waterproofIndex: item.waterproof_index ?? "",
    purchaseDate: item.purchase_date ?? "",
    purchasePriceText:
      item.purchase_price_cents === undefined ||
      item.purchase_price_cents === null
        ? ""
        : trimTrailingZeros(String(item.purchase_price_cents / 100)),
    expiryOrWarrantyDate: item.expiry_or_warranty_date ?? "",
    purchaseLocation: item.purchase_location ?? "",
    status: item.status,
    storageLocation: item.storage_location ?? "",
    tagsText: item.tags.join("，"),
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
    color: nullableText(form.color),
    material: nullableText(form.material),
    capacity: nullableText(form.capacity),
    size: nullableText(form.size),
    description: nullableText(form.description),
    weight_g: weightKgToGrams(form.weightText),
    warmth_index: nullableText(form.warmthIndex),
    waterproof_index: nullableText(form.waterproofIndex),
    purchase_date: nullableText(form.purchaseDate),
    purchase_price_cents: yuanToCents(form.purchasePriceText),
    expiry_or_warranty_date: nullableText(form.expiryOrWarrantyDate),
    purchase_location: nullableText(form.purchaseLocation),
    status: form.status ?? "available",
    storage_location: nullableText(form.storageLocation),
    tags: parseTagsInput(form.tagsText ?? ""),
    share_enabled: Boolean(form.shareEnabled),
    notes: nullableText(form.notes),
  };
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

export function formatGearWeight(value?: number | null): string {
  if (value === undefined || value === null) {
    return "未记录";
  }
  if (value >= 1000) {
    return `${trimTrailingZeros((value / 1000).toFixed(2))} kg`;
  }
  return `${value} g`;
}

export function formatGearPrice(value?: number | null): string {
  if (value === undefined || value === null) {
    return "未记录";
  }
  const yuan = value / 100;
  return value % 100 === 0 ? `¥${Math.trunc(yuan)}` : `¥${yuan.toFixed(2)}`;
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

function findLabel<T extends string>(
  options: Array<OptionItem<T>>,
  value: T,
): string {
  return options.find((item) => item.value === value)?.label ?? value;
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

function weightKgToGrams(value?: string): number | null {
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
  return Math.round(numberValue * 1000);
}

function yuanToCents(value?: string): number | null {
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
  return Math.round(numberValue * 100);
}

function trimTrailingZeros(value: string): string {
  return value.replace(/\.0+$/, "").replace(/(\.\d*?)0+$/, "$1");
}
