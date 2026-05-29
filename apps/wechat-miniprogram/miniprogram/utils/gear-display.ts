import type {
  GearCategory,
  GearCategoryFilter,
  GearAtlasPublicItem,
  GearCurrency,
  GearSort,
  GearStatus,
  OptionItem,
} from "./gear-utils";

export type {
  GearCategory,
  GearCategoryFilter,
  GearAtlasPublicItem,
  GearCurrency,
  GearSort,
  GearStatsResponse,
  GearStatus,
  GearSummary,
} from "./gear-utils";

export const GEAR_CATEGORY_OPTIONS: Array<OptionItem<GearCategory>> = [
  { value: "backpack_system", label: "背负系统", hint: "背包、外挂、收纳" },
  { value: "sleep_system", label: "睡眠系统", hint: "帐篷、睡袋、防潮垫" },
  { value: "kitchen_system", label: "餐厨系统", hint: "炉具、锅具、餐具" },
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

const GEAR_CURRENCY_OPTIONS: Array<OptionItem<GearCurrency>> = [
  { value: "CNY", label: "¥ CNY" },
  { value: "USD", label: "USD" },
  { value: "EUR", label: "EUR" },
  { value: "JPY", label: "JPY" },
  { value: "HKD", label: "HKD" },
];

export function formatGearWeight(value?: number | null): string {
  if (value === undefined || value === null) {
    return "未记录";
  }
  if (value >= 1000) {
    return `${trimTrailingZeros((value / 1000).toFixed(2))} kg`;
  }
  return `${value} g`;
}

export function formatGearQuantity(value?: number | null): string {
  const quantity = Math.max(1, Math.trunc(value ?? 1));
  return `x${quantity}`;
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

export function getGearCategoryLabel(value: GearCategory): string {
  return findLabel(GEAR_CATEGORY_OPTIONS, value);
}

export function getGearStatusLabel(value: GearStatus): string {
  return findLabel(GEAR_STATUS_OPTIONS, value);
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

export function categoryFilterItems(
  apiItems: GearCategoryFilter[],
): GearCategoryFilter[] {
  if (apiItems.length > 0) {
    return apiItems;
  }
  return [{ id: "all", label: "全部装备", count: 0 }];
}

function findLabel<T extends string>(
  options: Array<OptionItem<T>>,
  value: T,
): string {
  return options.find((item) => item.value === value)?.label ?? value;
}

function normalizeCurrency(value?: string | null): GearCurrency {
  const normalized = value?.trim().toUpperCase();
  const option = GEAR_CURRENCY_OPTIONS.find(
    (item) => item.value === normalized,
  );
  return option?.value ?? "CNY";
}

function trimTrailingZeros(value: string): string {
  return value.replace(/\.0+$/, "").replace(/(\.\d*?)0+$/, "$1");
}
