import type {
  GearCategory,
  GearSort,
  GearStatus,
} from "@stellartrail/shared-types";

export const CATEGORY_OPTIONS: Array<{ value: GearCategory; label: string }> = [
  { value: "backpack_system", label: "背负系统" },
  { value: "sleep_system", label: "睡眠系统" },
  { value: "kitchen_system", label: "餐厨系统" },
  { value: "walking_system", label: "行走系统" },
  { value: "clothing_system", label: "衣物系统" },
  { value: "lighting_system", label: "照明系统" },
  { value: "first_aid_system", label: "急救系统" },
  { value: "electronics_system", label: "电子系统" },
  { value: "technical_gear", label: "技术装备" },
  { value: "other_gear", label: "其它装备" },
  { value: "consumable", label: "消耗品" },
];

export const STATUS_OPTIONS: Array<{ value: GearStatus; label: string }> = [
  { value: "available", label: "可用" },
  { value: "in_use", label: "使用中" },
  { value: "maintenance", label: "保养中" },
  { value: "damaged", label: "损坏" },
  { value: "lost", label: "遗失" },
  { value: "retired", label: "退役" },
  { value: "sold", label: "已售出" },
  { value: "idle", label: "闲置" },
];

export const SORT_OPTIONS: Array<{ value: GearSort; label: string }> = [
  { value: "created_at_desc", label: "添加时间由新到旧" },
  { value: "created_at_asc", label: "添加时间由旧到新" },
  { value: "purchase_date_desc", label: "购买日期由新到旧" },
  { value: "name_asc", label: "装备名称 A-Z" },
  { value: "weight_desc", label: "重量由高到低" },
  { value: "price_desc", label: "价格由高到低" },
];

export function categoryLabel(value: GearCategory): string {
  return CATEGORY_OPTIONS.find((item) => item.value === value)?.label ?? value;
}

export function statusLabel(value: GearStatus): string {
  return STATUS_OPTIONS.find((item) => item.value === value)?.label ?? value;
}
